use std::collections::HashSet;

use log::{debug, info, warn};
use screeps::{
    find, Creep, HasPosition, HasStore, ResourceType, ReturnCode, Room, RoomObjectProperties, SharedCreepProperties, Source,
};

use crate::{pathing, spawning, architect};

pub fn start_loop() {
    debug!("top of loop");
}

pub fn prioritize_actions() {
    let creeps = screeps::game::creeps::values();

    let mut rooms = screeps::game::spawns::values()
        .iter()
        .map(|spawn| spawn.room().unwrap())
        .collect::<Vec<Room>>();

    rooms.dedup_by_key(|room| room.name());

    let construction = rooms
        .iter()
        .flat_map(|room| room.find(MY_CONSTRUCTION_SITES))
        .collect::<Vec<ConstructionSite>>();

    let mut repairs = rooms
        .iter()
        .flat_map(|room| room.find(STRUCTURES))
        .filter(|structure| {
            structure
                .as_attackable()
                .map(|os| os.hits_max() > os.hits() + 99)
                .unwrap_or(false)
        })
        .filter(|structure| structure.as_owned().map(|os| os.my()).unwrap_or(false))
        .collect::<Vec<Structure>>();
    repairs.sort_by_key(|structure| {
        structure
            .as_attackable()
            .map(|os| os.hits())
            .unwrap_or(99999)
    });

    let sources = rooms
        .iter()
        .flat_map(|room| get_sources_in_room(room))
        .collect::<Vec<Source>>();

    let mut dropped_resources = rooms
        .iter()
        .flat_map(|room| room.look_for_at_area(look::RESOURCES, 0..50, 0..50))
        .collect::<Vec<Resource>>();
    dropped_resources.sort_by_key(|resource| resource.amount());

    for (idx, creep) in screeps::game::creeps::values()
        .iter()
        .filter(|&creep| spawning::get_role(creep) == "harvester" && creep.my())
        .enumerate()
    {
        let target = &sources[idx % sources.len()];
        match creep.harvest(target) {
            ReturnCode::Ok | ReturnCode::Busy | ReturnCode::Tired | ReturnCode::Full => {
                continue;
            }
            ReturnCode::NotOwner => {
                warn!("{:?} is not the owner of {:?}", creep.name(), target.pos());
            }
            ReturnCode::NoPath => {
                pathing::set_waypoint(&creep, &target.pos());
                info!(
                    "{:?} nopath to {:?}, retrying",
                    &creep.name(),
                    &target.pos()
                );
            }
            ReturnCode::InvalidArgs
            | ReturnCode::NameExists
            | ReturnCode::NotFound
            | ReturnCode::InvalidTarget
            | ReturnCode::NotEnough => warn!("{} Invalid args given to harvester", creep.name()),
            ReturnCode::NotInRange => {
                pathing::set_waypoint(&creep, &target.pos());
                info!("{:?} headed to {:?}", &creep.name(), &target.pos());
            }
            ReturnCode::NoBodypart => warn!(
                "{:?} doesn't have the required body parts for his job",
                &creep.name()
            ),
            ReturnCode::RclNotEnough | ReturnCode::GclNotEnough => warn!("RCL/GCL"),
        }
    }

    for (idx, creep) in screeps::game::creeps::values()
        .iter()
        .filter(|&creep| spawning::get_role(creep) == "hauler" && creep.my())
        .enumerate()
    {
        let groundscore = dropped_resources
            .iter()
            .filter(|&resource| resource.room() == creep.room())
            .last()
            .expect("Nothing to pick up.");
        let target = &creep
            .room()
            .expect("No controller in room")
            .controller()
            .unwrap();

        match creep.pos().in_range_to(target, 4) {
            true => match creep.store_used_capacity(Some(ResourceType::Energy)) {
                0 => {
                    pathing::set_waypoint(&creep, &groundscore.pos());
                    debug!("{:?} headed to {:?}", &creep.name(), &groundscore.pos());
                }
                _ => match creep.drop(
                    ResourceType::Energy,
                    Some(creep.store_used_capacity(Some(ResourceType::Energy))),
                ) {
                    ReturnCode::NotOwner => {
                        warn!("{:?} is not the owner of {:?}", creep.name(), target.pos());
                    }
                    ReturnCode::NoPath => {
                        pathing::set_waypoint(&creep, &target.pos());
                        info!(
                            "{:?} nopath to {:?}, retrying",
                            &creep.name(),
                            &target.pos()
                        );
                    }
                    ReturnCode::Ok | ReturnCode::Full | ReturnCode::Tired | ReturnCode::Busy => {
                        continue;
                    }
                    ReturnCode::NotEnough => {
                        pathing::set_waypoint(&creep, &groundscore.pos());
                        info!("{:?} headed to {:?}", &creep.name(), &groundscore.pos());
                    }
                    ReturnCode::NameExists
                    | ReturnCode::NotFound
                    | ReturnCode::InvalidTarget
                    | ReturnCode::InvalidArgs => {
                        warn!("{} received invalid command", &creep.name())
                    }
                    ReturnCode::NotInRange => {
                        pathing::set_waypoint(&creep, &target.pos());
                        info!("{:?} headed to {:?}", &creep.name(), &target.pos());
                    }
                    ReturnCode::NoBodypart => warn!(
                        "{:?} doesn't have the required body parts for his job",
                        &creep.name()
                    ),
                    ReturnCode::RclNotEnough | ReturnCode::GclNotEnough => warn!("RCL/GCL"),
                },
            },
            false => {
                match creep.store_free_capacity(Some(ResourceType::Energy)) {
                    // full branch
                    0..=15 => {
                        pathing::set_waypoint(&creep, &target.pos());
                        info!("{:?} headed to {:?}", &creep.name(), &target.pos());
                    }
                    _ => match creep.pickup(groundscore) {
                        ReturnCode::NotOwner => {
                            warn!(
                                "{:?} is not the owner of {:?}",
                                creep.name(),
                                groundscore.pos()
                            );
                        }
                        ReturnCode::NoPath => {
                            pathing::set_waypoint(&creep, &groundscore.pos());
                            info!(
                                "{:?} nopath to {:?}, retrying",
                                &creep.name(),
                                &groundscore.pos()
                            );
                        }
                        ReturnCode::Ok | ReturnCode::Tired | ReturnCode::Busy => {
                            continue;
                        }
                        ReturnCode::NameExists
                        | ReturnCode::NotFound
                        | ReturnCode::InvalidTarget
                        | ReturnCode::InvalidArgs => {
                            warn!("{} received invalid command", &creep.name())
                        }
                        ReturnCode::Full => {
                            pathing::set_waypoint(&creep, &target.pos());
                            info!("{:?} headed to {:?}", &creep.name(), &target.pos());
                        }
                        ReturnCode::NotEnough | ReturnCode::NotInRange => {
                            pathing::set_waypoint(&creep, &groundscore.pos());
                            info!("{:?} headed to {:?}", &creep.name(), &groundscore.pos());
                        }
                        ReturnCode::NoBodypart => warn!(
                            "{:?} doesn't have the required body parts for his job",
                            &creep.name()
                        ),
                        ReturnCode::RclNotEnough | ReturnCode::GclNotEnough => warn!("RCL/GCL"),
                    },
                };
            }
        };
    }

    for (idx, creep) in screeps::game::creeps::values()
        .iter()
        .filter(|&creep| spawning::get_role(creep) == "builder" && creep.my())
        .enumerate()
    {
        let pickup = &creep
            .pos()
            .find_closest_by_range(find::DROPPED_RESOURCES)
            .expect("No sources on map");

        match construction.len() {
            // no construction projects
            0 => {
                if repairs.len() == 0 {
                    continue;
                }
                let repair = repairs.last().unwrap();

                match creep.store_used_capacity(Some(ResourceType::Energy)) {
                    // empty branch
                    0 => match creep.pickup(pickup) {
                        ReturnCode::NotOwner => {
                            warn!("{:?} is not the owner of {:?}", creep.name(), pickup.pos());
                        }
                        ReturnCode::NoPath => {
                            pathing::set_waypoint(&creep, &pickup.pos());
                            info!(
                                "{:?} nopath to {:?}, retrying",
                                &creep.name(),
                                &pickup.pos()
                            );
                        }
                        ReturnCode::Ok | ReturnCode::Tired | ReturnCode::Busy => {
                            continue;
                        }
                        ReturnCode::NameExists
                        | ReturnCode::NotFound
                        | ReturnCode::InvalidTarget
                        | ReturnCode::InvalidArgs => {
                            warn!("{} received invalid command", &creep.name())
                        }
                        ReturnCode::Full => {
                            pathing::set_waypoint(&creep, &repair.pos());
                            info!("{:?} headed to {:?}", &creep.name(), &repair.pos());
                        }
                        ReturnCode::NotEnough | ReturnCode::NotInRange => {
                            pathing::set_waypoint(&creep, &pickup.pos());
                            info!("{:?} headed to {:?}", &creep.name(), &pickup.pos());
                        }
                        ReturnCode::NoBodypart => warn!(
                            "{:?} doesn't have the required body parts for his job",
                            &creep.name()
                        ),
                        ReturnCode::RclNotEnough | ReturnCode::GclNotEnough => warn!("RCL/GCL"),
                    },
                    _ => match creep.repair(repair) {
                        ReturnCode::Ok => {
                            info!("{:?} repairing structure {:?}", creep.name(), repair.pos())
                        }
                        ReturnCode::Busy | ReturnCode::Full | ReturnCode::Tired => warn!(
                            "{:?} can't repair structure {:?}",
                            creep.name(),
                            repair.pos()
                        ),
                        ReturnCode::NotOwner => {
                            warn!("{:?} is not the owner of {:?}", creep.name(), repair.pos());
                        }
                        ReturnCode::NoPath => {
                            pathing::set_waypoint(&creep, &repair.pos());
                            info!(
                                "{:?} nopath to {:?}, retrying",
                                &creep.name(),
                                &repair.pos()
                            );
                        }
                        ReturnCode::InvalidTarget
                        | ReturnCode::NotFound
                        | ReturnCode::InvalidArgs
                        | ReturnCode::NameExists => warn!(
                            "{:?} cannot target {:?}; invalid",
                            creep.name(),
                            repair.pos()
                        ),
                        ReturnCode::NotEnough => {}
                        ReturnCode::NotInRange => {
                            pathing::set_waypoint(&creep, &repair.pos());
                            info!(
                                "{:?} nopath to {:?}, retrying",
                                &creep.name(),
                                &repair.pos()
                            );
                        }
                        ReturnCode::NoBodypart => {
                            creep.drop(
                                ResourceType::Energy,
                                Some(creep.store_used_capacity(Some(ResourceType::Energy))),
                            );
                        }
                        ReturnCode::RclNotEnough | ReturnCode::GclNotEnough => warn!("RCL/GCL"),
                    },
                }
            }
            _ => {
                if construction.len() == 0 {
                    continue;
                }
                let target = construction.last().unwrap();
                match creep.store_used_capacity(Some(ResourceType::Energy)) {
                    // empty branch
                    0 => match creep.pickup(pickup) {
                        ReturnCode::NotOwner => {
                            warn!("{:?} is not the owner of {:?}", creep.name(), pickup.pos());
                        }
                        ReturnCode::NoPath => {
                            pathing::set_waypoint(&creep, &pickup.pos());
                            info!(
                                "{:?} nopath to {:?}, retrying",
                                &creep.name(),
                                &pickup.pos()
                            );
                        }
                        ReturnCode::Ok | ReturnCode::Tired | ReturnCode::Busy => {
                            continue;
                        }
                        ReturnCode::NameExists
                        | ReturnCode::NotFound
                        | ReturnCode::InvalidTarget
                        | ReturnCode::InvalidArgs => {
                            warn!("{} received invalid command", &creep.name())
                        }
                        ReturnCode::Full => {
                            pathing::set_waypoint(&creep, &target.pos());
                            info!("{:?} headed to {:?}", &creep.name(), &target.pos());
                        }
                        ReturnCode::NotEnough | ReturnCode::NotInRange => {
                            pathing::set_waypoint(&creep, &pickup.pos());
                            info!("{:?} headed to {:?}", &creep.name(), &pickup.pos());
                        }
                        ReturnCode::NoBodypart => warn!(
                            "{:?} doesn't have the required body parts for his job",
                            &creep.name()
                        ),
                        ReturnCode::RclNotEnough | ReturnCode::GclNotEnough => warn!("RCL/GCL"),
                    },
                    // full branch
                    _ => match creep.build(target) {
                        ReturnCode::Ok
                        | ReturnCode::Busy
                        | ReturnCode::Full
                        | ReturnCode::Tired => {}
                        ReturnCode::NotOwner => {
                            warn!("{:?} is not the owner of {:?}", creep.name(), target.pos());
                        }
                        ReturnCode::NoPath => {
                            pathing::set_waypoint(&creep, &target.pos());
                            info!(
                                "{:?} nopath to {:?}, retrying",
                                &creep.name(),
                                &target.pos()
                            );
                        }
                        ReturnCode::InvalidTarget
                        | ReturnCode::NotFound
                        | ReturnCode::InvalidArgs
                        | ReturnCode::NameExists => warn!(
                            "{:?} cannot target {:?}; invalid",
                            creep.name(),
                            target.pos()
                        ),
                        ReturnCode::NotEnough => {}
                        ReturnCode::NotInRange => {
                            pathing::set_waypoint(&creep, &target.pos());
                            info!(
                                "{:?} nopath to {:?}, retrying",
                                &creep.name(),
                                &target.pos()
                            );
                        }
                        ReturnCode::NoBodypart => {
                            creep.drop(
                                ResourceType::Energy,
                                Some(creep.store_used_capacity(Some(ResourceType::Energy))),
                            );
                        }
                        ReturnCode::RclNotEnough | ReturnCode::GclNotEnough => warn!("RCL/GCL"),
                    },
                }
            }
        }
    }

    for (idx, creep) in screeps::game::creeps::values()
        .iter()
        .filter(|&creep| spawning::get_role(creep) == "average" && creep.my())
        .enumerate()
    {
        let pickup = &creep
            .pos()
            .find_closest_by_range(find::DROPPED_RESOURCES)
            .expect("No sources on map");
        let target = &creep
            .room()
            .expect("No controller in room")
            .controller()
            .unwrap();

        match creep.store_used_capacity(Some(ResourceType::Energy)) {
            // empty branch
            0 => match creep.pickup(pickup) {
                ReturnCode::NotOwner => {
                    warn!("{:?} is not the owner of {:?}", creep.name(), pickup.pos());
                }
                ReturnCode::NoPath => {
                    pathing::set_waypoint(&creep, &pickup.pos());
                    info!(
                        "{:?} nopath to {:?}, retrying",
                        &creep.name(),
                        &pickup.pos()
                    );
                }
                ReturnCode::Ok | ReturnCode::Tired | ReturnCode::Busy => {
                    continue;
                }
                ReturnCode::NameExists
                | ReturnCode::NotFound
                | ReturnCode::InvalidTarget
                | ReturnCode::InvalidArgs => warn!("{} received invalid command", &creep.name()),
                ReturnCode::Full => {
                    pathing::set_waypoint(&creep, &target.pos());
                    info!("{:?} headed to {:?}", &creep.name(), &target.pos());
                }
                ReturnCode::NotEnough | ReturnCode::NotInRange => {
                    pathing::set_waypoint(&creep, &pickup.pos());
                    info!("{:?} headed to {:?}", &creep.name(), &pickup.pos());
                }
                ReturnCode::NoBodypart => warn!(
                    "{:?} doesn't have the required body parts for his job",
                    &creep.name()
                ),
                ReturnCode::RclNotEnough | ReturnCode::GclNotEnough => warn!("RCL/GCL"),
            },
            // full branch
            _ => match creep.upgrade_controller(target) {
                ReturnCode::Ok | ReturnCode::Busy | ReturnCode::Full | ReturnCode::Tired => {}
                ReturnCode::NotOwner => {
                    warn!("{:?} is not the owner of {:?}", creep.name(), target.pos());
                }
                ReturnCode::NoPath => {
                    pathing::set_waypoint(&creep, &target.pos());
                    info!(
                        "{:?} nopath to {:?}, retrying",
                        &creep.name(),
                        &target.pos()
                    );
                }
                ReturnCode::InvalidTarget
                | ReturnCode::NotFound
                | ReturnCode::InvalidArgs
                | ReturnCode::NameExists => warn!(
                    "{:?} cannot target {:?}; invalid",
                    creep.name(),
                    target.pos()
                ),
                ReturnCode::NotEnough => {}
                ReturnCode::NotInRange => {
                    pathing::set_waypoint(&creep, &target.pos());
                    info!(
                        "{:?} nopath to {:?}, retrying",
                        &creep.name(),
                        &target.pos()
                    );
                }
                ReturnCode::NoBodypart => {
                    creep.drop(
                        ResourceType::Energy,
                        Some(creep.store_used_capacity(Some(ResourceType::Energy))),
                    );
                }
                ReturnCode::RclNotEnough | ReturnCode::GclNotEnough => warn!("RCL/GCL"),
            },
        }
    }
}

pub fn get_sources_here(creep: &Creep) -> Vec<Source> {
    let mut sources = creep
        .room()
        .expect("room is not visible to you")
        .find(find::SOURCES);
    sources.sort_unstable_by_key(|s| {
        s.pos().get_range_to(creep);
    });
    sources
}

pub fn get_sources_in_room(room: &Room) -> Vec<Source> {
    room.find(find::SOURCES)
}

pub fn end_loop() {}

pub fn manage_memory() {
    let time = screeps::game::time();

    if time % 32 == 3 {
        info!("running memory cleanup");
        cleanup_memory().expect("expected Memory.creeps format to be a regular memory object");
    }
}

fn cleanup_memory() -> Result<(), Box<dyn std::error::Error>> {
    let alive_creeps: HashSet<String> = screeps::game::creeps::keys().into_iter().collect();

    let screeps_memory = match screeps::memory::root().dict("creeps")? {
        Some(v) => v,
        None => {
            warn!("not cleaning game creep memory: no Memory.creeps dict");
            return Ok(());
        }
    };

    for mem_name in screeps_memory.keys() {
        if !alive_creeps.contains(&mem_name) {
            debug!("cleaning up creep memory of dead creep {}", mem_name);
            screeps_memory.del(&mem_name);
        }
    }

    Ok(())
}
