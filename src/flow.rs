use std::collections::HashSet;

use log::{debug, info, warn};
use screeps::{
    find, Creep, HasPosition, HasStore, ResourceType, ReturnCode, Room, RoomObjectProperties,
    SharedCreepProperties, Source,
};

use crate::{architect, pathing, spawning};

pub fn start_loop() {
    debug!("top of loop");
}

pub fn prioritize_actions() {
    for (idx, creep) in spawning::get_by_role(&screeps::game::creeps::values(), "harvester")
        .iter()
        .enumerate()
    {
        let sources = architect::get_sources();
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

    for creep in spawning::get_by_role(&screeps::game::creeps::values(), "hauler") {
        let mut groundscores = architect::get_groundscores();
        let mut containers = architect::get_unfull_containers();
        let mut controllers = architect::get_my_controllers();

        containers.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));
        controllers.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));

        let groundscore = groundscores.first();
        let container = containers.first();
        let controller = controllers.first();

        match creep.store_free_capacity(Some(ResourceType::Energy)) {
            // full carry
            0..=20 => match container {
                // container available
                Some(c) => match c.as_transferable() {
                    // valid container
                    Some(c) => match creep.transfer_all(c, ResourceType::Energy) {
                        ReturnCode::Ok => {
                            debug!("{:?} transferred resources {:?}.", creep.name(), c.pos())
                        }
                        ReturnCode::NoPath => {
                            warn!("{:?} ran into an todo branch. NoPath", creep.name())
                        }
                        ReturnCode::Full => {
                            warn!("{:?} ran into an todo branch. Full.", creep.name())
                        }
                        ReturnCode::NotEnough => {
                            debug!("{:?} finished transferring.", creep.name())
                        }
                        ReturnCode::NotInRange => {
                            pathing::set_waypoint(&creep, &c.pos());
                            debug!("{:?} headed to {:?}; dropoff", creep.name(), &c.pos());
                        }
                        _ => warn!("{:?} ran into an invalid branch.", creep.name()),
                    },
                    // invalid container
                    None => warn!("{:?} ran into an invalid branch.", creep.name()),
                },
                // no container
                None => match controller {
                    // valid controller
                    Some(c) => match creep.upgrade_controller(&c) {
                        ReturnCode::Ok => {
                            debug!("{:?} upgraded controller {:?}.", creep.name(), c.pos())
                        }
                        ReturnCode::NoPath => {
                            warn!("{:?} ran into an todo branch. NoPath", creep.name())
                        }
                        ReturnCode::Full => {
                            warn!("{:?} ran into an todo branch. Full.", creep.name())
                        }
                        ReturnCode::NotEnough => {
                            debug!("{:?} finished transferring.", creep.name())
                        }
                        ReturnCode::NotInRange => {
                            pathing::set_waypoint(&creep, &c.pos());
                            debug!("{:?} headed to {:?}; dropoff", creep.name(), &c.pos());
                        }
                        ReturnCode::NoBodypart => {
                            creep.drop(
                                ResourceType::Energy,
                                Some(creep.store_used_capacity(Some(ResourceType::Energy))),
                            );
                        }
                        _ => warn!("{:?} ran into an invalid branch.", creep.name()),
                    },
                    // invalid controller
                    None => warn!("{:?} ran into an invalid branch.", creep.name()),
                },
            },
            // not full carry
            _ => match groundscore {
                Some(c) => match creep.pickup(&c) {
                    ReturnCode::Ok => debug!("{:?} picked up resources.", creep.name()),
                    ReturnCode::NotInRange => {
                        pathing::set_waypoint(&creep, &c.pos());
                        debug!("{:?} headed to {:?}; pickup", creep.name(), &c.pos());
                    }
                    _ => warn!(
                        "{:?} ran into an invalid branch. {:?}",
                        creep.name(),
                        creep.pickup(&c)
                    ),
                },
                None => warn!("{:?} has nothing to do.", creep.name()),
            },
        }
    }

    for creep in spawning::get_by_role(&screeps::game::creeps::values(), "builder") {
        let mut constructions = architect::get_my_buildables();
        let mut repairs = architect::get_my_repairables();
        let mut groundscores = architect::get_groundscores();
        let mut containers = architect::get_full_containers();

        containers.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));
        constructions.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));
        repairs.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));

        let groundscore = groundscores.first();
        let container = containers.first();
        let repair = repairs.first();
        let construction = constructions.first();

        match creep.store_used_capacity(Some(ResourceType::Energy)) {
            // nothing carried
            0 => match groundscore {
                // there is ground loot
                Some(c) => match creep.pickup(&c) {
                    ReturnCode::Ok => debug!("{:?} picked up resources.", creep.name()),
                    ReturnCode::NotInRange => {
                        pathing::set_waypoint(&creep, &c.pos());
                        debug!("{:?} headed to {:?}; pickup", creep.name(), &c.pos());
                    }
                    _ => warn!(
                        "{:?} ran into an invalid branch. {:?}",
                        creep.name(),
                        creep.pickup(&c)
                    ),
                },
                // no ground loot
                None => match container {
                    // there is a container
                    Some(c) => match c.as_withdrawable() {
                        Some(c) => match creep.withdraw_amount(
                            c,
                            ResourceType::Energy,
                            creep.store_capacity(Some(ResourceType::Energy))
                                - creep.store_used_capacity(Some(ResourceType::Energy)),
                        ) {
                            ReturnCode::Ok => debug!("{:?} picked up resources.", creep.name()),
                            ReturnCode::NoPath => {
                                info!("{:?} nopathed to {:?}", creep.name(), &c.pos())
                            }
                            ReturnCode::NotEnough => {
                                creep.withdraw_all(c, ResourceType::Energy);
                            }
                            ReturnCode::NotInRange => {
                                pathing::set_waypoint(&creep, &c.pos());
                                debug!("{:?} headed to {:?}; pickup", creep.name(), &c.pos());
                            }
                            _ => warn!(
                                "{:?} ran into an invalid branch. Withdrawing from container",
                                creep.name()
                            ),
                        },
                        None => warn!(
                            "{:?} ran into an invalid branch. Container unwithdrawable",
                            creep.name()
                        ),
                    },

                    // no container
                    None => warn!("{:?} has nothing to do", creep.name()),
                },
            },
            // we have resources
            _ => match construction {
                // there are construction projects
                Some(c) => match creep.build(&c) {
                    ReturnCode::Ok => debug!("{:?} built {:?}.", creep.name(), c.structure_type()),
                    ReturnCode::NotInRange => {
                        pathing::set_waypoint(&creep, &c.pos());
                        info!("{:?} headed to {:?}, building", &creep.name(), &c.pos());
                    }
                    _ => warn!(
                        "{:?} ran into an invalid branch. {:?}",
                        creep.name(),
                        creep.build(&c)
                    ),
                },
                // no construction projects
                None => match repair {
                    // there are repair projects
                    Some(c) => match creep.repair(c) {
                        ReturnCode::Ok => debug!("{:?} repaired {:?}.", creep.name(), c.pos()),
                        ReturnCode::NotInRange => {
                            pathing::set_waypoint(&creep, &c.pos());
                            info!("{:?} headed to {:?}, repairing", &creep.name(), &c.pos());
                        }
                        _ => warn!(
                            "{:?} ran into an invalid branch. {:?}",
                            creep.name(),
                            creep.repair(c)
                        ),
                    },
                    // no repair projects
                    None => warn!("{:?} has nothing to do.", creep.name()),
                },
            },
        }
    }

    for creep in spawning::get_by_role(&screeps::game::creeps::values(), "average") {
        let mut groundscores = architect::get_groundscores();
        let mut containers = architect::get_full_containers();
        let mut controllers = architect::get_my_controllers();

        containers.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));
        controllers.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));

        let groundscore = groundscores.first();
        let container = containers.first();
        let controller = controllers.first();

        match creep.store_used_capacity(Some(ResourceType::Energy)) {
            0 => match container {
                // there is a container
                Some(c) => match c.as_withdrawable() {
                    Some(c) => match creep.withdraw_amount(
                        c,
                        ResourceType::Energy,
                        creep.store_capacity(Some(ResourceType::Energy))
                            - creep.store_used_capacity(Some(ResourceType::Energy)),
                    ) {
                        ReturnCode::Ok => debug!("{:?} picked up resources.", creep.name()),
                        ReturnCode::NotInRange => {
                            pathing::set_waypoint(&creep, &c.pos());
                            debug!("{:?} headed to {:?}; pickup", creep.name(), &c.pos());
                        }
                        _ => warn!(
                            "{:?} ran into an invalid branch. Withdrawing from container",
                            creep.name()
                        ),
                    },
                    None => warn!(
                        "{:?} ran into an invalid branch. Withdrawing from container",
                        creep.name()
                    ),
                },

                // no container
                None => match groundscore {
                    // there is ground loot
                    Some(c) => match creep.pickup(&c) {
                        ReturnCode::Ok => debug!("{:?} picked up resources.", creep.name()),
                        ReturnCode::NotInRange => {
                            pathing::set_waypoint(&creep, &c.pos());
                            debug!("{:?} headed to {:?}; pickup", creep.name(), &c.pos());
                        }
                        _ => warn!(
                            "{:?} ran into an invalid branch. {:?}",
                            creep.name(),
                            creep.pickup(&c)
                        ),
                    },
                    // no ground loot
                    None => warn!("{:?} has nothing to do.", creep.name()),
                },
            },
            _ => match controller {
                Some(c) => match creep.upgrade_controller(&c) {
                    ReturnCode::Ok => debug!("{:?} upgraded controller.", creep.name()),
                    ReturnCode::NotInRange => {
                        pathing::set_waypoint(&creep, &c.pos());
                        debug!("{:?} headed to {:?}; controller", creep.name(), &c.pos());
                    }
                    _ => warn!(
                        "{:?} ran into an invalid branch. No controller.",
                        creep.name()
                    ),
                },
                None => warn!(
                    "{:?} ran into an invalid branch. No controller.",
                    creep.name()
                ),
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
