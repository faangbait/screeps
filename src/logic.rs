use std::collections::HashMap;

use crate::filters::{
    get_groundscores, get_hostility, get_my_buildables, get_my_controllers, get_my_creeps,
    get_my_ramparts, get_my_repairables, get_my_rooms, get_my_sources, get_my_storages,
    get_my_structures, get_my_towers,
};
use crate::jobs::{JobProperties, JobType};
use crate::structures::OptimalHarvest;
use crate::{source, spawning};
use log::{info, warn};
use screeps::game::cpu;
use screeps::{
    Attackable, Creep, HasId, HasPosition, HasStore, Position, RoomObjectProperties,
    SharedCreepProperties, Step, StructureProperties,
};
/// Identifies the best harvesters to pair with available sources
fn select_harvesters(creeps: Vec<screeps::Creep>, sources: Vec<screeps::Source>) -> Vec<Creep> {
    // Holds a vector of positions near the source
    let source_slots = sources
        .iter()
        .map(|s| {
            (
                s,
                s.pos()
                    .neighbors()
                    .iter()
                    .filter(|&pos| pos.move_cost().is_some())
                    .map(|&pos| pos)
                    .collect::<Vec<Position>>(),
            )
        })
        .collect::<Vec<(&screeps::Source, Vec<Position>)>>();

    let mut harvesters = creeps
        .iter()
        .filter(|&c| {
            c.has_parts_for_job(JobType::Harvest)
                && !c.has_parts_for_job(JobType::Claim)
                && !c.has_parts_for_job(JobType::Attack)
                && !c.has_parts_for_job(JobType::AttackR)
                && c.ticks_to_live().unwrap_or(0) > 15
                && c.store_capacity(Some(screeps::ResourceType::Energy)) == 0
        })
        .collect::<Vec<&Creep>>();

    harvesters.sort_unstable_by_key(|&h| -1 * h.get_active_bodyparts(screeps::Part::Work) as i32);

    harvesters.truncate(2.max(source_slots.len()));

    let matrix = harvesters
        .iter()
        .map(|&h| sources.iter().map(move |s| s.pos().get_range_to(h)))
        .flatten()
        .collect::<Vec<u32>>();

    let height = harvesters.len();
    let width = sources.len();

    let assignments = hungarian::minimize(&matrix, height, width);
    assignments
        .iter()
        .filter_map(|&a| a)
        .enumerate()
        .for_each(|(i, j)| {
            let h = harvesters[j];
            let s = &sources[i];
            info!("{:?} assigned to {:?}", h.name(), &s.pos());
            // let bp = h.count_bp_vec(vec![screeps::Part::Work])[0] as u32;

            match h.harvest(s) {
                screeps::ReturnCode::Ok => {}
                screeps::ReturnCode::NotInRange => {
                    h.move_to(s);
                }
                _ => info!("Problem harvesting"),
            };
        });

    harvesters
        .iter()
        .map(|&c| c.to_owned())
        .collect::<Vec<Creep>>()
}

pub fn select_wd_haulers(
    creeps: Vec<screeps::Creep>,
    structures: Vec<screeps::Structure>,
) -> Vec<Creep> {
    let withdrawal_targets = structures
        .iter()
        .filter(|&st| match st.structure_type() {
            screeps::StructureType::Container => true,
            _ => false,
        })
        .collect::<Vec<&screeps::Structure>>();

    if withdrawal_targets.is_empty() {
        return vec![];
    }

    let mut haulers = creeps
        .iter()
        .filter(|&c| {
            c.has_parts_for_job(JobType::Withdraw)
                && !c.has_parts_for_job(JobType::Claim)
                && !c.has_parts_for_job(JobType::Attack)
                && !c.has_parts_for_job(JobType::AttackR)
                && c.ticks_to_live().unwrap_or(0) > 15
                && c.store_total() == 0
        })
        .collect::<Vec<&Creep>>();

    haulers.sort_unstable_by_key(|&u| {
        -1 * u.get_active_bodyparts(screeps::Part::Carry) as i32

        // if let Some(dist) = structures.iter().map(|c| c.pos().get_range_to(u)).min() {
        //     1000 - (((u.get_active_bodyparts(screeps::Part::Carry)* 100)) - dist)
        // } else {
        //     1000 - ((u.get_active_bodyparts(screeps::Part::Carry) * 100))
        // }
    });

    haulers.truncate(2.max(creeps.len() / 4));

    let mut assigned_haulers = vec![];
    let mut assigned_targets = HashMap::<u128, u32>::new();

    while haulers.len() > 0 && assigned_haulers.len() < withdrawal_targets.len() {
        let height = withdrawal_targets.len();
        let width = haulers.len();

        // Builds matrix
        let matrix =
            withdrawal_targets
                .iter()
                .map(|&t| {
                    haulers
                        .iter()
                        .map(|&c| {
                            t.as_has_store().unwrap().store_types().iter().fold(
                                255,
                                |acc: u8, cur| {
                                    acc.checked_sub(255.min(
                                        t.as_has_store().unwrap().store_used_capacity(Some(*cur)),
                                    ) as u8)
                                        .unwrap_or(0) as u8
                                },
                            ) + t.pos().get_range_to(&c.pos()) as u8
                        })
                        .collect::<Vec<u8>>()
                })
                .flatten()
                .collect::<Vec<u8>>();

        let assignments = hungarian::minimize(&matrix, height, width);
        if assignments.len() == 0 || assignments.iter().all(|a| a.is_none()) {
            break;
        };

        assignments
            .iter()
            .filter_map(|&a| a)
            .enumerate()
            .for_each(|(i, j)| {
                let h = haulers[j];
                let s = &withdrawal_targets[i];
                // info!("{:?} assigned to {:?}", h.name(), &s.pos());
                let bp = h.count_bp_vec(vec![screeps::Part::Carry])[0] as u32;

                match s.as_has_store() {
                    Some(st) => {
                        for res in st.store_types() {
                            if h.store_free_capacity(Some(res)) > 0 {
                                match h.withdraw_amount(
                                    s.as_withdrawable().unwrap(),
                                    res,
                                    h.store_free_capacity(Some(res)) as u32,
                                ) {
                                    screeps::ReturnCode::Ok => {
                                        assigned_targets.insert(
                                            s.untyped_id().to_u128(),
                                            bp + *assigned_targets
                                                .get(&s.untyped_id().to_u128())
                                                .unwrap_or(&0),
                                        );
                                        break;
                                    }
                                    screeps::ReturnCode::NotEnough => {
                                        h.withdraw_all(s.as_withdrawable().unwrap(), res);
                                        assigned_targets.insert(
                                            s.untyped_id().to_u128(),
                                            bp + *assigned_targets
                                                .get(&s.untyped_id().to_u128())
                                                .unwrap_or(&0),
                                        );
                                        break;
                                    }
                                    screeps::ReturnCode::NotInRange => {
                                        assigned_haulers.push(h);

                                        h.move_to(&s.pos());
                                        assigned_targets.insert(
                                            s.untyped_id().to_u128(),
                                            bp + *assigned_targets
                                                .get(&s.untyped_id().to_u128())
                                                .unwrap_or(&0),
                                        );
                                        break;
                                    }
                                    screeps::ReturnCode::NotOwner
                                    | screeps::ReturnCode::NoPath
                                    | screeps::ReturnCode::NameExists
                                    | screeps::ReturnCode::Busy
                                    | screeps::ReturnCode::NotFound
                                    | screeps::ReturnCode::Full
                                    | screeps::ReturnCode::InvalidTarget
                                    | screeps::ReturnCode::InvalidArgs
                                    | screeps::ReturnCode::Tired
                                    | screeps::ReturnCode::NoBodypart
                                    | screeps::ReturnCode::RclNotEnough
                                    | screeps::ReturnCode::GclNotEnough => {
                                        info!("Weird return from wd_hauler");
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        info!("Nothing left anymore...");
                    }
                }
            });
        haulers.retain(|&c| !assigned_haulers.contains(&c));
    }
    info!(
        "LOGI WD: {:?} | H: {:?} | I: {:?} | CPU {:.2}",
        withdrawal_targets.len(),
        assigned_haulers.len(),
        assigned_targets.keys().fold(0, |acc, key| {
            acc + assigned_targets.get_key_value(key).unwrap().1
        }) * 50,
        cpu::get_used(),
    );

    assigned_haulers
        .iter()
        .map(|&c| c.to_owned())
        .collect::<Vec<Creep>>()
}

pub fn select_tf_haulers(
    creeps: Vec<screeps::Creep>,
    structures: Vec<screeps::Structure>,
) -> Vec<Creep> {
    let deposit_targets = structures
        .iter()
        .filter(|st| match st.structure_type() {
            screeps::StructureType::Spawn => true,
            screeps::StructureType::Extension => true,
            screeps::StructureType::Link => true,
            screeps::StructureType::Storage => true,
            screeps::StructureType::Tower => true,
            screeps::StructureType::Lab => true,
            screeps::StructureType::Factory => true,
            _ => false,
        })
        .filter(|&st| match st.structure_type() {
            screeps::StructureType::Storage => true,
            _ => match st.as_has_store() {
                Some(o) => {
                    if o.store_types().iter().any(|res| {
                        if *res == screeps::ResourceType::Energy {
                            o.energy() >= o.store_capacity(Some(screeps::ResourceType::Energy))
                        } else {
                            o.store_used_capacity(Some(*res)) >= o.store_capacity(Some(*res))
                        }
                    }) {
                        false
                    } else {
                        true
                    }
                }
                None => match st.as_has_energy_for_spawn() {
                    Some(o) => true,
                    None => false,
                },
            },
        })
        .collect::<Vec<&screeps::Structure>>();

    let mut haulers = creeps
        .iter()
        .filter(|&c| {
            c.has_parts_for_job(JobType::Transfer)
                && !c.has_parts_for_job(JobType::Claim)
                && !c.has_parts_for_job(JobType::Attack)
                && !c.has_parts_for_job(JobType::AttackR)
                && c.ticks_to_live().unwrap_or(0) > 15
                && c.store_total() > 0
        })
        .collect::<Vec<&Creep>>();

    // TODO this will panic
    haulers.iter().for_each(|&h| {
        if let Some(nearby) = h
            .pos()
            .find_in_range(screeps::find::MY_CREEPS, 1)
            .iter()
            .filter(|&c| {
                c.pos()
                    .get_range_to(&c.room().unwrap().controller().unwrap())
                    < h.pos()
                        .get_range_to(&h.room().unwrap().controller().unwrap())
            })
            .max_by_key(|&c| c.get_active_bodyparts(screeps::Part::Carry))
        {
            if let Some(ty) = h
                .store_types()
                .iter()
                .max_by_key(|&res| h.store_used_capacity(Some(*res)))
            {
                h.transfer_amount(nearby, *ty, nearby.store_free_capacity(Some(*ty)) as u32);
            }
        };
    });

    if deposit_targets.is_empty() {
        return vec![];
    }

    haulers.sort_unstable_by_key(|&u| {
        // if let Some(dist) = structures.iter().map(|c| c.pos().get_range_to(u)).min() {
        //     1000 - (((u.get_active_bodyparts(screeps::Part::Carry)* 100)) - dist)
        // } else {
        //     1000 - ((u.get_active_bodyparts(screeps::Part::Carry) * 100))
        // }
        -1 * u.get_active_bodyparts(screeps::Part::Carry) as i32
    });

    haulers.truncate(4.max(creeps.len() / 4));

    let mut assigned_haulers = vec![];

    let mut assigned_targets = HashMap::<u128, u32>::new();

    while haulers.len() > 0 && assigned_haulers.len() < deposit_targets.len() {
        let height = deposit_targets.len();
        let width = haulers.len();

        // Builds matrix
        let matrix = deposit_targets
            .iter()
            .map(|&t| {
                haulers
                    .iter()
                    .map(|&c| {
                        if c.store_types().iter().any(|res| {
                            if *res == screeps::ResourceType::Energy {
                                t.as_has_store().unwrap().energy()
                                    < t.as_has_store()
                                        .unwrap()
                                        .store_capacity(Some(screeps::ResourceType::Energy))
                            } else {
                                t.as_has_store().unwrap().store_free_capacity(Some(*res)) > 0
                            }
                        }) {
                            (c.pos().get_range_to(t) / 20) as u8
                        } else {
                            255
                        }
                    })
                    .collect::<Vec<u8>>()
            })
            .flatten()
            .collect::<Vec<u8>>();
        // info!("{:?}", matrix);

        let assignments = hungarian::minimize(&matrix, height, width);
        if assignments.len() == 0 || assignments.iter().all(|a| a.is_none()) {
            break;
        };

        assignments
            .iter()
            .filter_map(|&a| a)
            .enumerate()
            .for_each(|(i, j)| {
                let h = haulers[j];
                let s = &deposit_targets[i];
                // info!("{:?} assigned to {:?}", h.name(), &s.pos());

                let bp = h.count_bp_vec(vec![screeps::Part::Carry])[0] as u32;

                match s.as_has_store() {
                    Some(st) => {
                        for res in h.store_types() {
                            if st.store_free_capacity(Some(res)) > 0 {
                                match h.transfer_amount(
                                    s.as_transferable().unwrap(),
                                    res,
                                    h.store_used_capacity(Some(res)) as u32,
                                ) {
                                    screeps::ReturnCode::Ok => {
                                        // assigned_haulers.push(h);
                                        assigned_targets.insert(
                                            s.untyped_id().to_u128(),
                                            bp + *assigned_targets
                                                .get(&s.untyped_id().to_u128())
                                                .unwrap_or(&0),
                                        );
                                        break;
                                    }
                                    screeps::ReturnCode::NotEnough => {
                                        match h.transfer_all(s.as_transferable().unwrap(), res) {
                                            screeps::ReturnCode::Ok => {
                                                // assigned_haulers.push(h);
                                                assigned_targets.insert(
                                                    s.untyped_id().to_u128(),
                                                    bp + *assigned_targets
                                                        .get(&s.untyped_id().to_u128())
                                                        .unwrap_or(&0),
                                                );
                                            }
                                            _ => {
                                                info!("Unhandled return -> Not Enough");
                                            }
                                        };
                                        break;
                                    }
                                    screeps::ReturnCode::Full => {
                                        match h.transfer_amount(
                                            s.as_transferable().unwrap(),
                                            res,
                                            st.store_free_capacity(Some(res)) as u32,
                                        ) {
                                            screeps::ReturnCode::Ok => {
                                                // assigned_haulers.push(h);
                                                assigned_targets.insert(
                                                    s.untyped_id().to_u128(),
                                                    bp + *assigned_targets
                                                        .get(&s.untyped_id().to_u128())
                                                        .unwrap_or(&0),
                                                );
                                            }
                                            _ => {
                                                info!("Unhandled return -> Full");
                                            }
                                        }
                                        break;
                                    }
                                    screeps::ReturnCode::NotInRange => {
                                        assigned_haulers.push(h);
                                        h.move_to(&s.pos());
                                        assigned_targets.insert(
                                            s.untyped_id().to_u128(),
                                            bp + *assigned_targets
                                                .get(&s.untyped_id().to_u128())
                                                .unwrap_or(&0),
                                        );
                                        break;
                                    }
                                    screeps::ReturnCode::InvalidArgs => {
                                        info!("Invalid args");
                                    }
                                    screeps::ReturnCode::NotOwner
                                    | screeps::ReturnCode::NoPath
                                    | screeps::ReturnCode::NameExists
                                    | screeps::ReturnCode::Busy
                                    | screeps::ReturnCode::NotFound
                                    | screeps::ReturnCode::InvalidTarget
                                    | screeps::ReturnCode::Tired
                                    | screeps::ReturnCode::NoBodypart
                                    | screeps::ReturnCode::RclNotEnough
                                    | screeps::ReturnCode::GclNotEnough => {
                                        info!("Weird return from tf_hauler")
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        info!("Nothing left anymore...");
                    }
                }
            });
        haulers.retain(|&c| !assigned_haulers.contains(&c));
    }

    info!(
        "LOGI DEP: {:?} | H: {:?} | I: {:?} | CPU {:.2}",
        deposit_targets.len(),
        assigned_haulers.len(),
        assigned_targets.keys().fold(0, |acc, key| {
            acc + assigned_targets.get_key_value(key).unwrap().1
        }) * 50,
        cpu::get_used(),
    );

    assigned_haulers
        .iter()
        .map(|&c| c.to_owned())
        .collect::<Vec<Creep>>()
}

pub fn select_builders(
    creeps: Vec<screeps::Creep>,
    mut construction: Vec<screeps::ConstructionSite>,
) -> Vec<Creep> {
    if construction.is_empty() {
        return vec![];
    }

    construction.sort_unstable_by_key(|st| match st.structure_type() {
        screeps::StructureType::Extension => 0,
        screeps::StructureType::Spawn => 1,
        screeps::StructureType::Road => 2,
        screeps::StructureType::Container => 3,
        screeps::StructureType::Wall => 8,
        screeps::StructureType::Rampart => 9,
        screeps::StructureType::Link => 1,
        screeps::StructureType::Storage => 4,
        screeps::StructureType::Tower => 5,
        screeps::StructureType::Observer => 6,
        screeps::StructureType::PowerSpawn => 7,
        screeps::StructureType::Extractor => 7,
        screeps::StructureType::Lab => 7,
        screeps::StructureType::Terminal => 7,
        screeps::StructureType::Nuker => 7,
        screeps::StructureType::Factory => 7,
        _ => 255,
    });

    let mut builders = creeps
        .iter()
        .filter(|&c| {
            c.has_parts_for_job(JobType::Build)
                && !c.has_parts_for_job(JobType::Claim)
                && !c.has_parts_for_job(JobType::Attack)
                && !c.has_parts_for_job(JobType::AttackR)
                && c.store_used_capacity(Some(screeps::ResourceType::Energy)) > 0
        })
        .collect::<Vec<&Creep>>();

    let controllers = get_my_controllers();

    builders.sort_unstable_by_key(|&u| {
        // if let Some(dist) = construction.iter().map(|c| c.pos().get_range_to(u)).min() {
        //     1000 - ((u.get_active_bodyparts(screeps::Part::Work) * 100) - dist)
        // } else {
        //     1000 - (u.get_active_bodyparts(screeps::Part::Work) * 100)
        // }
        -1 * u.get_active_bodyparts(screeps::Part::Work) as i32
    });

    builders.truncate(1.max(creeps.len() / 3));

    let mut assigned_builders = vec![];

    // if let Some(target_site) = construction.iter()
    //     .min_by_key(|&site| {
    //         if let Some(nearest) = controllers.iter().min_by_key(|&c| site.pos().get_range_to(c)) {
    //             site.pos().get_range_to(nearest)
    //         } else { u32::MAX }
    //     })
    builders.iter().for_each(|&c| {
        if let Some(target_site) = construction.get(0) {
            match c.build(&target_site) {
                screeps::ReturnCode::Ok => {
                    assigned_builders.push(c);
                }
                screeps::ReturnCode::NotInRange => {
                    c.move_to(&target_site.pos());
                    assigned_builders.push(c);
                }
                _ => info!("Error in building"),
            }
        }
    });

    info!(
        "BUILDS: {:?} | H: {:?} | CPU: {:.2}",
        construction.len(),
        assigned_builders.len(),
        cpu::get_used(),
    );

    assigned_builders
        .iter()
        .map(|&c| c.to_owned())
        .collect::<Vec<Creep>>()
}

pub fn select_repairers(
    creeps: Vec<screeps::Creep>,
    structures: Vec<screeps::Structure>,
) -> Vec<Creep> {
    if structures.is_empty() {
        return vec![];
    }

    let mut repairers = creeps
        .iter()
        .filter(|&c| {
            c.has_parts_for_job(JobType::Repair)
                && !c.has_parts_for_job(JobType::Claim)
                && !c.has_parts_for_job(JobType::Attack)
                && !c.has_parts_for_job(JobType::AttackR)
                && c.store_used_capacity(Some(screeps::ResourceType::Energy)) > 0
        })
        .collect::<Vec<&Creep>>();

    repairers.sort_unstable_by_key(|&u| {
        // if let Some(dist) = structures.iter().map(|c| c.pos().get_range_to(u)).min() {
        //     1000 - (((u.get_active_bodyparts(screeps::Part::Carry)* 10) + u.get_active_bodyparts(screeps::Part::Work) * 100) - dist)
        // } else {
        //     1000 - ((u.get_active_bodyparts(screeps::Part::Carry) * 10) + (u.get_active_bodyparts(screeps::Part::Work) * 100))
        // }
        -1 * (u.get_active_bodyparts(screeps::Part::Carry)
            + u.get_active_bodyparts(screeps::Part::Work)
            + u.get_active_bodyparts(screeps::Part::Move)) as i32
    });

    repairers.truncate(1.max(creeps.len() / 4));

    let mut assigned_repairers = vec![];

    repairers.iter().for_each(|&c| {
        if let Some(target_site) = structures
            .iter()
            .min_by_key(|&site| site.pos().get_range_to(c))
        {
            match c.repair(target_site) {
                screeps::ReturnCode::Ok => {
                    assigned_repairers.push(c);
                }
                screeps::ReturnCode::NotInRange => {
                    c.move_to(&target_site.pos());
                    assigned_repairers.push(&c);
                }
                _ => info!("Error in repairing"),
            }
        }
    });

    info!(
        "REPAIRS: {:?} | H: {:?} | | CPU: {:.2}",
        structures.len(),
        assigned_repairers.len(),
        cpu::get_used(),
    );

    assigned_repairers
        .iter()
        .map(|&c| c.to_owned())
        .collect::<Vec<Creep>>()
}
pub fn select_upgraders(
    creeps: Vec<screeps::Creep>,
    controllers: Vec<screeps::StructureController>,
) -> Vec<Creep> {
    let mut upgraders = creeps
        .iter()
        .filter(|&c| {
            c.has_parts_for_job(JobType::Upgrade)
                && !c.has_parts_for_job(JobType::Claim)
                && !c.has_parts_for_job(JobType::Attack)
                && !c.has_parts_for_job(JobType::AttackR)
                && c.store_used_capacity(Some(screeps::ResourceType::Energy)) > 0
        })
        .collect::<Vec<&Creep>>();

    upgraders.sort_unstable_by_key(|&u| {
        // if let Some(nearest) = controllers.iter().min_by_key(|&c| u.pos().get_range_to(c)) {
        //     u.pos().get_range_to(nearest)
        // } else { u32::MAX }
        -1 * u.get_active_bodyparts(screeps::Part::Carry) as i32
    });

    upgraders.truncate(1.max(creeps.len() / 10));

    let mut assigned_upgraders = vec![];

    if let Some(target_site) = controllers.iter().min_by_key(|&cont| cont.level()) {
        upgraders
            .iter()
            .for_each(|&c| match c.upgrade_controller(target_site) {
                screeps::ReturnCode::Ok => {
                    assigned_upgraders.push(c);
                }
                screeps::ReturnCode::NotInRange => {
                    c.move_to(&target_site.pos());
                    assigned_upgraders.push(c);
                }
                _ => info!("Error in upgrading"),
            })
    };
    info!(
        "UPGRADES: {:?} | H: {:?} | CPU: {:.2}",
        controllers.len(),
        assigned_upgraders.len(),
        cpu::get_used(),
    );

    assigned_upgraders
        .iter()
        .map(|&c| c.to_owned())
        .collect::<Vec<Creep>>()
}
pub fn select_gatherers(
    creeps: Vec<screeps::Creep>,
    groundscores: Vec<screeps::Resource>,
) -> Vec<Creep> {
    if groundscores.is_empty() {
        return vec![];
    }

    let mut gatherers = creeps
        .iter()
        .filter(|&c| {
            (c.has_parts_for_job(JobType::Build)
                || c.has_parts_for_job(JobType::Repair)
                || c.has_parts_for_job(JobType::Upgrade)
                || c.has_parts_for_job(JobType::Transfer))
                && !c.has_parts_for_job(JobType::Claim)
                && !c.has_parts_for_job(JobType::Attack)
                && !c.has_parts_for_job(JobType::AttackR)
                && c.store_used_capacity(Some(screeps::ResourceType::Energy)) == 0
        })
        .collect::<Vec<&Creep>>();

    gatherers.sort_unstable_by_key(|&u| {
        // if let Some(dist) = groundscores.iter().map(|c| c.pos().get_range_to(u)).min() {
        //     1000 - (((u.get_active_bodyparts(screeps::Part::Move)* 100)) - dist)
        // } else {
        //     1000 - ((u.get_active_bodyparts(screeps::Part::Move) * 100))
        // }
        -1 * u.get_active_bodyparts(screeps::Part::Move) as i32
    });

    // gatherers.truncate(5.max(creeps.len() / 5));

    let mut assigned_gatherers = vec![];

    while gatherers.len() > 0 && assigned_gatherers.len() < groundscores.len() {
        let height = groundscores.len();
        let width = gatherers.len();

        let matrix = groundscores
            .iter()
            .map(|t| {
                gatherers
                    .iter()
                    .map(|&c| {
                        // c.pos().get_range_to(&t.pos()) as u8
                        t.pos()
                            .get_range_to(&t.room().unwrap().controller().unwrap())
                            as u8
                            + (c.store_capacity(Some(t.resource_type()))
                                .checked_sub(t.amount())
                                .unwrap_or(0)) as u8
                        // TODO Will panic
                    })
                    .collect::<Vec<u8>>()
            })
            .flatten()
            .collect::<Vec<u8>>();

        let assignments = hungarian::minimize(&matrix, height, width);
        if assignments.len() == 0 || assignments.iter().all(|a| a.is_none()) {
            break;
        };

        assignments
            .iter()
            .filter_map(|&a| a)
            .enumerate()
            .for_each(|(i, j)| {
                let h = gatherers[j];
                let s = &groundscores[i];

                match h.pickup(s) {
                    screeps::ReturnCode::Ok => {}
                    screeps::ReturnCode::NotInRange => {
                        assigned_gatherers.push(h);
                        h.move_to(&s.pos());
                    }
                    _ => {
                        info!("Unexpected return from pickup");
                    }
                }
            });
        gatherers.retain(|&c| !assigned_gatherers.contains(&c));
    }

    info!(
        "GATHERS: {:?} | H: {:?} | CPU: {:.2}",
        groundscores.len(),
        assigned_gatherers.len(),
        cpu::get_used(),
    );

    assigned_gatherers
        .iter()
        .map(|&c| c.to_owned())
        .collect::<Vec<Creep>>()
}
pub fn select_defenders(creeps: Vec<screeps::Creep>, room: &screeps::Room) -> Vec<screeps::Creep> {
    let (hc, hpc, hsp, hst, hcs) = get_hostility(&room);
    if hc.len() == 0 && hpc.len() == 0 && hsp.len() == 0 && hst.len() == 0 && hcs.len() == 0 {
        return vec![];
    };

    let mut defenders = creeps
        .iter()
        .filter(|&c| c.has_parts_for_job(JobType::DefendR))
        .collect::<Vec<&Creep>>();

    let mut healers = creeps
        .iter()
        .filter(|&c| c.has_parts_for_job(JobType::DefendR))
        .collect::<Vec<&Creep>>();

    let mut brawlers = creeps
        .iter()
        .filter(|&c| c.has_parts_for_job(JobType::DefendR))
        .collect::<Vec<&Creep>>();

    let mut cannons = creeps
        .iter()
        .filter(|&c| c.has_parts_for_job(JobType::DefendR))
        .collect::<Vec<&Creep>>();

    // let (mut defenders, mut free): (Vec<&Creep>,Vec<&Creep>) = creeps.iter().partition(|&c| {
    //     c.has_parts_for_job(JobType::DefendR)
    // });

    // let (mut healers, mut free): (Vec<&Creep>,Vec<&Creep>) = free.iter().partition(|&c| {
    //     c.has_parts_for_job(JobType::Heal)
    // });

    // let (mut brawlers, mut free): (Vec<&Creep>,Vec<&Creep>) = free.iter().partition(|&c| {
    //     c.has_parts_for_job(JobType::Defend) || c.has_parts_for_job(JobType::Attack)
    // });

    // let (mut cannons, mut free): (Vec<&Creep>,Vec<&Creep>) = free.iter().partition(|&c| {
    //     c.has_parts_for_job(JobType::AttackR)
    // });

    let hurt = creeps
        .iter()
        .filter(|&c| c.hits_max() > c.hits())
        .collect::<Vec<&Creep>>();

    let mut towers = get_my_towers();
    let mut ramparts = get_my_ramparts();
    ramparts.retain(|r| r.room().unwrap() == *room);

    let mut assigned_defenders = vec![];
    let mut assigned_towers = vec![];
    let mut assigned_healers = vec![];
    // let mut assigned_brawlers = vec![];
    // let mut assigned_cannons = vec![];

    // while towers.len() > 0 && assigned_towers.len() < towers.len() {
    if let Some(hurtc) = creeps.iter().max_by_key(|&c| c.hits_max() - c.hits()) {
        let badly = hurtc.hits_max() - hurtc.hits();
        towers.iter().for_each(|t| {
            t.heal(hurtc);
            assigned_towers.push(t)
        });
    } else {
        towers.iter().for_each(|t| {
            match t.pos().find_closest_by_range(screeps::find::HOSTILE_CREEPS) {
                Some(e) => {
                    t.attack(&e);
                    assigned_towers.push(t);
                }
                None => {}
            }
        })
    }
    // }

    while defenders.len() > 0 && assigned_defenders.len() < ramparts.len() {
        let height = ramparts.len();
        let width = defenders.len();

        let d_matrix = ramparts
            .iter()
            .map(|r| {
                defenders
                    .iter()
                    .filter(|&c| c.room().unwrap() == *room)
                    .map(|&c| {
                        if let Some(enemy) = hc.first() {
                            r.pos()
                                .get_range_to(&c.pos().midpoint_between(&enemy.pos()))
                                as u8
                        } else {
                            255
                        }
                    })
                    .collect::<Vec<u8>>()
            })
            .flatten()
            .collect::<Vec<u8>>();

        if d_matrix.iter().all(|v| *v == 0 || *v > 200) {
            break;
        };
        let assignments = hungarian::minimize(&d_matrix, height, width);
        if assignments.len() == 0 || assignments.iter().all(|a| a.is_none()) {
            break;
        };

        assignments
            .iter()
            .filter_map(|&a| a)
            .enumerate()
            .for_each(|(i, j)| {
                let h = defenders[j];
                let s = &ramparts[i];

                match h.pos().get_range_to(&s.pos()) {
                    0 => match h.pos().find_closest_by_range(screeps::find::HOSTILE_CREEPS) {
                        Some(hc) => match h.ranged_attack(&hc) {
                            screeps::ReturnCode::Ok => {
                                assigned_defenders.push(h);
                            }
                            _ => {}
                        },
                        None => {}
                    },
                    _ => {
                        h.move_to(&s.pos());
                    }
                }
            });
        defenders.retain(|&c| !assigned_defenders.contains(&c));
    }

    while healers.len() > 0 && assigned_healers.len() < hurt.len() {
        let width = healers.len();
        let height = hurt.len();

        let h_matrix = hurt
            .iter()
            .map(|h| {
                healers
                    .iter()
                    .filter(|&c| c.room().unwrap() == *room)
                    .map(|&c| c.pos().get_range_to(&h.pos()) as u8)
                    .collect::<Vec<u8>>()
            })
            .flatten()
            .collect::<Vec<u8>>();

        let assignments = hungarian::minimize(&h_matrix, height, width);
        if assignments.len() == 0 || assignments.iter().all(|a| a.is_none()) {
            break;
        };

        assignments
            .iter()
            .filter_map(|&a| a)
            .enumerate()
            .for_each(|(i, j)| {
                let h = healers[j];
                let s = &hurt[i];

                match h.heal(*s) {
                    screeps::ReturnCode::Ok => {
                        assigned_healers.push(h);
                        info!("Healing {:?}", s.name());
                    }
                    screeps::ReturnCode::NotInRange => match h.ranged_heal(*s) {
                        screeps::ReturnCode::Ok => {
                            assigned_healers.push(h);
                        }
                        screeps::ReturnCode::NotInRange => {
                            assigned_healers.push(h);
                            h.move_to(&s.pos());
                        }
                        _ => {}
                    },
                    _ => {}
                }
            });
        healers.retain(|&c| !assigned_healers.contains(&c));
    }

    assigned_defenders.extend(assigned_healers);
    assigned_defenders
        .iter()
        .map(|&c| c.to_owned())
        .collect::<Vec<Creep>>()
}
pub fn move_away(creeps: Vec<screeps::Creep>) {
    creeps.iter().for_each(|c| {
        c.move_to_xy(18, 6);
    });
}

pub fn prioritize(mut creeps: Vec<screeps::Creep>) {
    let total = creeps.len();

    let movers = creeps
        .to_vec()
        .iter()
        .filter(|&c| {
            if let Some(mut dest) = c
                .memory()
                .path_arr::<Position>("_move.astar.path")
                .unwrap_or(None)
            {
                if dest.len() > 0 {
                    let step = dest.remove(0);
                    c.move_to(&step);
                    c.memory().path_del("_move.astar");
                    c.memory().path_set(
                        "_move.astar.path",
                        dest.iter().map(|p| p.packed_repr()).collect::<Vec<i32>>(),
                    );
                    return true;
                } else {
                    false
                }
            } else {
                false
            }
        })
        .map(|c| c.to_owned())
        .collect::<Vec<Creep>>();

    creeps.retain(|c| !movers.contains(c));

    let defenders = get_my_rooms()
        .iter()
        .flat_map(|room| select_defenders(creeps.to_vec(), room))
        .collect::<Vec<Creep>>();
    creeps.retain(|c| !defenders.contains(c));

    let mut upgraders = vec![];
    let mut builders = vec![];
    let mut repairers = vec![];

    let harvesters = select_harvesters(creeps.to_vec(), get_my_sources());
    creeps.retain(|c| !harvesters.contains(c));

    if total > 12 {
        upgraders = select_upgraders(creeps.to_vec(), get_my_controllers());
        creeps.retain(|c| !upgraders.contains(c));

        builders = select_builders(creeps.to_vec(), get_my_buildables());
        creeps.retain(|c| !builders.contains(c));

        repairers = select_repairers(creeps.to_vec(), get_my_repairables());
        creeps.retain(|c| !repairers.contains(c));
    }

    let tf_haulers = select_tf_haulers(creeps.to_vec(), get_my_structures());
    creeps.retain(|c| !tf_haulers.contains(c));

    let wd_haulers = select_wd_haulers(creeps.to_vec(), get_my_structures());
    creeps.retain(|c| !wd_haulers.contains(c));

    let gatherers = select_gatherers(creeps.to_vec(), get_groundscores());
    creeps.retain(|c| !gatherers.contains(c));

    // move_away(creeps.to_vec());

    if screeps::game::time() % 15 == 1 || total < 6 {
        spawning::init(
            creeps.to_vec(),
            harvesters.to_vec(),
            wd_haulers.to_vec(),
            tf_haulers.to_vec(),
            builders.to_vec(),
            repairers.to_vec(),
            upgraders.to_vec(),
            gatherers.to_vec(),
            defenders.to_vec(),
        );
    }

    // info!(
    //     "TOTAL: {:?} | HARV: {:?} | HAUL: {:?} | BUILD: {:?} | REPAIR: {:?} | UPGRADE: {:?} | GATHER: {:?} | CPU: {:.2}",
    //     total,
    //     harvesters.len(),
    //     wd_haulers.len() + tf_haulers.len(),
    //     builders.len(),
    //     repairers.len(),
    //     upgraders.len(),
    //     gatherers.len(),
    //     screeps::game::cpu::get_used(),
    // )
}
