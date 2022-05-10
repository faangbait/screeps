use log::info;
use screeps::{
    find, look, Attackable, Creep, HasPosition, HasStore, OwnedStructureProperties, Position,
    ResourceType, RoomObjectProperties, SharedCreepProperties, StructureProperties,
};

use crate::jobs::{JobProperties, JobType};
use crate::{bucket, filters, flags, spawning};

// pub fn select_upgraders(mut creeps: Vec<Creep>) -> Vec<Creep> {
//     creeps.retain(|c| {
//         c.has_parts_for_job(JobType::Upgrade)
//             && c.store_used_capacity(Some(ResourceType::Energy)) > 0
//             && if let Some(room) = c.room() {
//                 if let Some(controller) = room.controller() {
//                     if controller.owner_name() == Some("sage".to_string()) {
//                         true
//                     } else {
//                         false
//                     }
//                 } else {
//                     false
//                 }
//             } else {
//                 false
//             }
//     });

//     creeps.sort_unstable_by_key(|c| {
//         -1 * (c.get_active_bodyparts(screeps::Part::Work)
//             + c.get_active_bodyparts(screeps::Part::Carry)) as i32
//     });

//     creeps.truncate(3);

//     creeps.iter().for_each(|c| {
//         match c.upgrade_controller(&c.room().unwrap().controller().unwrap()) {
//             screeps::ReturnCode::Ok => {}
//             screeps::ReturnCode::NotInRange => {
//                 c.move_to(&c.room().unwrap().controller().unwrap());
//             }
//             _ => {}
//         }
//     });

//     creeps
// }

// pub fn select_harvesters(mut creeps: Vec<Creep>, mut sources: Vec<screeps::Source>) -> Vec<Creep> {
//     creeps.retain(|c| {
//         c.store_free_capacity(Some(ResourceType::Energy)) > 0
//             || c.store_capacity(Some(ResourceType::Energy)) == 0
//     });

//     creeps.sort_unstable_by_key(|c| {
//         (-1 * c.get_active_bodyparts(screeps::Part::Work) as i32) + c.body().len() as i32
//     });

//     sources.retain(|s| s.energy() > 0 || s.ticks_to_regeneration() < 20);

//     // Holds a vector of positions near the source
//     let mut source_slots = sources
//         .iter()
//         .map(|s| {
//             s.pos()
//                 .neighbors()
//                 .iter()
//                 .filter(|&pos| pos.move_cost().is_some())
//                 .map(|&pos| pos)
//                 .collect::<Vec<Position>>()
//         })
//         .flatten()
//         .collect::<Vec<Position>>();

//     source_slots.dedup_by_key(|s| s.packed_repr());

//     creeps.truncate(source_slots.len());

//     let mut matrix = vec![];

//     creeps.iter().for_each(|c| {
//         source_slots
//             .iter()
//             .for_each(|s| matrix.push(s.get_range_to(c)));
//     });

//     let height = creeps.len();
//     let width = source_slots.len();

//     let assignments = hungarian::minimize(&matrix, height, width);

//     assignments
//         .iter()
//         .filter_map(|&a| a)
//         .enumerate()
//         .for_each(|(i, j)| {
//             let h = &creeps[i];
//             let s = &source_slots[j];

//             // info!("{:?} assigned to {:?}", h.name(), &s.pos());

//             match s.find_closest_by_range(find::SOURCES) {
//                 Some(source) => match h.harvest(&source) {
//                     screeps::ReturnCode::Ok => {}
//                     screeps::ReturnCode::NotInRange => {
//                         h.move_to(s);
//                     }
//                     _ => info!("Error, unhandled return in harvest"),
//                 },
//                 None => {}
//             };
//         });

//     creeps
// }

// pub fn select_haulers(mut creeps: Vec<Creep>) -> Vec<Creep> {
//     creeps.retain(|c| c.has_parts_for_job(JobType::Transfer));

//     let empty = creeps
//         .iter()
//         .filter(|&c| c.store_used_capacity(None) == 0)
//         .collect::<Vec<&Creep>>();

//     let unempty = creeps
//         .iter()
//         .filter(|&c| !empty.contains(&c))
//         .map(|c| c.to_owned())
//         .collect::<Vec<Creep>>();

//     unempty
// }
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Context {
    job: JobType,
    target: Position,
}

pub fn get_harvest_jobs(mut creeps: Vec<screeps::Creep>) -> Vec<Context> {
    let mut sources = filters::get_my_sources();
    sources.retain(|s| s.energy() > 0 || s.ticks_to_regeneration() < 20);

    // Holds a vector of positions near the source
    let source_slots = sources
        .iter()
        .map(|s| {
            s.pos()
                .neighbors()
                .iter()
                .filter(|&pos| pos.move_cost().is_some())
                .map(|&pos| pos)
                .collect::<Vec<Position>>()
        })
        .flatten()
        .collect::<Vec<Position>>();

    source_slots
        .iter()
        .map(|s| Context {
            job: JobType::Harvest,
            target: *s,
        })
        .collect::<Vec<Context>>()
}

pub fn get_transfer_jobs(mut creeps: Vec<Creep>) -> Vec<Context> {
    filters::get_my_structures()
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
        .flat_map(|d| {
            [Context {
                job: JobType::Transfer,
                target: d.pos(),
            }]
            .repeat(match d.structure_type() {
                screeps::StructureType::Spawn => 2,
                screeps::StructureType::Extension => 1,
                screeps::StructureType::Link => 2,
                screeps::StructureType::Storage => 3,
                screeps::StructureType::Tower => 2,
                screeps::StructureType::Lab => 1,
                screeps::StructureType::Factory => 1,
                _ => 0,
            })
        })
        .collect::<Vec<Context>>()
}
pub fn get_gather_jobs(mut creeps: Vec<Creep>) -> Vec<Context> {
    let mut contexts = filters::get_groundscores()
        .iter()
        .flat_map(|g| {
            [Context {
                job: JobType::Pickup,
                target: g.pos(),
            }]
            .repeat((g.amount() / 100).max(1) as usize)
        })
        .collect::<Vec<Context>>();

    filters::get_my_rooms().iter().for_each(|r| {
        let ruins = r.find(find::RUINS);
        ruins.iter().for_each(|r| {
            contexts.push(Context {
                job: JobType::Withdraw,
                target: r.pos(),
            })
        });
        let tomb = r.find(find::TOMBSTONES);
        tomb.iter().for_each(|r| {
            contexts.push(Context {
                job: JobType::Withdraw,
                target: r.pos(),
            })
        });
    });

    contexts
}

pub fn get_upgrade_jobs(mut creeps: Vec<Creep>) -> Vec<Context> {
    filters::get_my_controllers()
        .iter()
        .flat_map(|c| {
            [Context {
                job: JobType::Upgrade,
                target: c.pos(),
            }]
            .repeat(match filters::get_my_buildables().len() > 0 {
                true => 1,
                false => 3,
            })
        })
        .collect::<Vec<Context>>()
}

pub fn get_build_jobs(mut creeps: Vec<Creep>) -> Vec<Context> {
    if creeps
        .iter()
        .filter(|c| c.has_parts_for_job(JobType::Build))
        .count()
        == 0
    {
        return vec![];
    }
    let mut buildables = filters::get_my_buildables();
    buildables.sort_by_key(|cs| match cs.structure_type() {
        screeps::StructureType::Spawn => 1,
        screeps::StructureType::Extension => 0,
        screeps::StructureType::Road => 2,
        screeps::StructureType::Wall => 5,
        screeps::StructureType::Rampart => 6,
        screeps::StructureType::Link => 3,
        screeps::StructureType::Storage => 2,
        screeps::StructureType::Tower => 3,
        screeps::StructureType::Observer => 5,
        screeps::StructureType::PowerSpawn => 5,
        screeps::StructureType::Extractor => 4,
        screeps::StructureType::Lab => 4,
        screeps::StructureType::Terminal => 4,
        screeps::StructureType::Container => 4,
        screeps::StructureType::Nuker => 4,
        screeps::StructureType::Factory => 4,
        _ => 99,
    });

    if let Some(c) = buildables.first() {
        return [Context {
            job: JobType::Build,
            target: c.pos(),
        }]
        .repeat(5);
    } else {
        return vec![];
    }
}
pub fn get_repair_jobs(mut creeps: Vec<Creep>) -> Vec<Context> {
    if creeps
        .iter()
        .filter(|c| c.has_parts_for_job(JobType::Repair))
        .count()
        == 0
    {
        return vec![];
    }
    if let Some(c) = filters::get_my_repairables().first() {
        return [Context {
            job: JobType::Repair,
            target: c.pos(),
        }]
        .repeat(3);
    } else {
        return vec![];
    }
}

pub fn get_withdraw_jobs(mut creeps: Vec<Creep>) -> Vec<Context> {
    vec![]
}

pub fn get_scout_jobs(mut creeps: Vec<Creep>) -> Vec<Context> {
    if creeps
        .iter()
        .filter(|c| c.has_parts_for_job(JobType::Claim))
        .count()
        == 0
    {
        return vec![];
    }

    flags::get_claim_flags()
        .iter()
        .map(|pos| Context {
            job: JobType::Claim,
            target: *pos,
        })
        .collect::<Vec<Context>>()
}

pub fn get_defense_jobs(mut creeps: Vec<Creep>, room: &screeps::Room) -> Vec<Creep> {
    let (hc, hpc, hsp, hst, hcs) = filters::get_hostility(&room);
    if hc.len() > 0 || hpc.len() > 0 || hsp.len() > 0 || hst.len() > 0 || hcs.len() > 0 {
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

        let hurt = creeps
            .iter()
            .filter(|&c| c.hits_max() > c.hits())
            .collect::<Vec<&Creep>>();

        let mut towers = filters::get_my_towers();
        let mut ramparts = filters::get_my_ramparts();
        ramparts.retain(|r| r.room().unwrap() == *room);

        let mut assigned_defenders = vec![];
        let mut assigned_towers = vec![];
        let mut assigned_healers = vec![];
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
    } else {
        vec![]
    }
}

pub fn assign_harvesters(mut creeps: Vec<screeps::Creep>) -> Vec<screeps::Creep> {
    let mut assigned_harvesters = vec![];
    let mut sources = filters::get_my_sources();
    sources.retain(|s| s.energy() > 0 || s.ticks_to_regeneration() < 20);

    // Holds a vector of positions near the source
    let source_slots = sources
        .iter()
        .map(|s| {
            s.pos()
                .neighbors()
                .iter()
                .filter(|&pos| pos.move_cost().is_some())
                .map(|&pos| pos)
                .collect::<Vec<Position>>()
        })
        .flatten()
        .collect::<Vec<Position>>();

    creeps.retain(|c| c.has_parts_for_job(JobType::Harvest));

    creeps.sort_unstable_by_key(|c| -1 * c.get_active_bodyparts(screeps::Part::Work) as i32);

    let mut bp = 0;

    if let Some(best) = creeps.first() {
        bp = best.get_active_bodyparts(screeps::Part::Work);
    }

    creeps.retain(|c| c.get_active_bodyparts(screeps::Part::Work) >= bp);

    let matrix = creeps
        .iter()
        .map(|c| sources.iter().map(move |s| c.pos().get_range_to(s)))
        .flatten()
        .collect::<Vec<u32>>();

    let height = creeps.len();
    let width = sources.len();

    let assignments = hungarian::minimize(&matrix, height, width);
    assignments
        .iter()
        .filter_map(|&a| a)
        .enumerate()
        .for_each(|(i, j)| {
            let h = &creeps[j];
            let s = &sources[i];
            info!("{:?} assigned to harvest {:?}", h.name(), &s.pos());
            assigned_harvesters.push(h.to_owned());
            match h.harvest(s) {
                    screeps::ReturnCode::Ok => {}
                    screeps::ReturnCode::NotInRange => {
                        h.move_to(s);
                    }
                    screeps::ReturnCode::NoPath => {
                        info!("No path to {:?}", s.pos());
                    }
                    _ => { info!("Problem harvesting");}
                    // screeps::ReturnCode::NotOwner => todo!(),
                    // screeps::ReturnCode::NameExists => todo!(),
                    // screeps::ReturnCode::Busy => todo!(),
                    // screeps::ReturnCode::NotFound => todo!(),
                    // screeps::ReturnCode::NotEnough => todo!(),
                    // screeps::ReturnCode::InvalidTarget => todo!(),
                    // screeps::ReturnCode::Full => todo!(),
                    // screeps::ReturnCode::InvalidArgs => todo!(),
                    // screeps::ReturnCode::Tired => todo!(),
                    // screeps::ReturnCode::NoBodypart => todo!(),
                    // screeps::ReturnCode::RclNotEnough => todo!(),
                    // screeps::ReturnCode::GclNotEnough => todo!(),
                };
            // }
        });

    // for source in sources {
    //     let mut bp = 0;
    //     while creeps.len() > 0 && bp <= 5 {
    //         if let Some(c) = creeps.pop() {
    //             bp += c.get_active_bodyparts(screeps::Part::Work);
    //             assigned_harvesters.push(c.to_owned());
    //             match c.harvest(&source) {
    //                 screeps::ReturnCode::Ok => {}
    //                 screeps::ReturnCode::NotInRange => {
    //                     c.move_to(&source);
    //                 }
    //                 _ => {}
    //             }
    //         }
    //     }
    // }

    assigned_harvesters
}

pub fn prioritize(mut creeps: Vec<screeps::Creep>) {
    let total = creeps.len();

    let defenders = filters::get_my_rooms()
        .iter()
        .flat_map(|room| get_defense_jobs(creeps.to_vec(), room))
        .collect::<Vec<Creep>>();

    creeps.retain(|c| !defenders.contains(c));

    // let mut harvesters = assign_harvesters(creeps.to_vec());
    // creeps.retain(|c| !harvesters.contains(c));

    let mut contexts = vec![];

    contexts.extend(get_harvest_jobs(creeps.to_vec()));
    contexts.extend(get_transfer_jobs(creeps.to_vec()));
    contexts.extend(get_upgrade_jobs(creeps.to_vec()));
    contexts.extend(get_gather_jobs(creeps.to_vec()));
    contexts.extend(get_build_jobs(creeps.to_vec()));
    contexts.extend(get_repair_jobs(creeps.to_vec()));
    contexts.extend(get_scout_jobs(creeps.to_vec()));
    // contexts.extend(get_withdraw_j?obs(creeps.to_vec()));

    let height = creeps.len();
    let width = contexts.len();

    let mut harvesters = vec![];
    let mut upgraders = vec![];
    let mut haulers = vec![];
    let mut builders = vec![];
    let mut repairers = vec![];
    let mut gatherers = vec![];

    let matrix = creeps
        .iter()
        .map(|c| {
            contexts.iter().map(move |ctx| match ctx.job {
                JobType::Harvest if c.has_parts_for_job(JobType::Harvest) => {
                    100_u32
                        .checked_sub(c.get_active_bodyparts(screeps::Part::Work) * 5)
                        .unwrap_or(0)
                        + c.get_active_bodyparts(screeps::Part::Move)
                        + ctx.target.get_range_to(c) * 3
                }
                JobType::Harvest => u32::MAX,
                JobType::Transfer
                    if c.has_parts_for_job(JobType::Transfer)
                        && c.store_used_capacity(None) > 0 =>
                {
                    (100 - (c.get_active_bodyparts(screeps::Part::Carry) * 10))
                        + (ctx
                            .target
                            .get_range_to(c)
                            .checked_sub(c.store_used_capacity(None))
                            .unwrap_or(0)
                            / c.get_active_bodyparts(screeps::Part::Move))
                }
                JobType::Transfer => u32::MAX,
                JobType::Upgrade
                    if c.has_parts_for_job(JobType::Upgrade)
                        && c.store_used_capacity(Some(ResourceType::Energy)) > 0 =>
                {
                    (100 - (c.get_active_bodyparts(screeps::Part::Work) as u32 * 5))
                        + ctx.target.get_range_to(c)
                }
                JobType::Upgrade => u32::MAX,
                JobType::Pickup
                    if c.has_parts_for_job(JobType::Transfer)
                        && c.store_used_capacity(Some(ResourceType::Energy)) < 10 =>
                {
                    (100 - 5
                        * (c.get_active_bodyparts(screeps::Part::Carry)
                            + c.get_active_bodyparts(screeps::Part::Move) as u32))
                        + (ctx.target.get_range_to(c) / c.get_active_bodyparts(screeps::Part::Move))
                }
                JobType::Pickup => u32::MAX,
                JobType::Build
                    if c.has_parts_for_job(JobType::Build)
                        && c.store_used_capacity(Some(ResourceType::Energy)) > 0 =>
                {
                    (100 - (5 * c.get_active_bodyparts(screeps::Part::Work) as u32))
                        + (ctx.target.get_range_to(c) / c.get_active_bodyparts(screeps::Part::Move))
                }
                JobType::Build => u32::MAX,
                JobType::Repair
                    if c.has_parts_for_job(JobType::Repair)
                        && c.store_used_capacity(Some(ResourceType::Energy)) > 0 =>
                {
                    (100 - (5 * c.get_active_bodyparts(screeps::Part::Work) as u32))
                        + (ctx.target.get_range_to(c) / c.get_active_bodyparts(screeps::Part::Move))
                }
                JobType::Repair => u32::MAX,
                JobType::Station => todo!(),
                JobType::Withdraw
                    if c.has_parts_for_job(JobType::Withdraw)
                        && c.store_free_capacity(None) > 0 =>
                {
                    (100 - (5 * c.get_active_bodyparts(screeps::Part::Carry) as u32))
                        + ctx.target.get_range_to(c)
                }
                JobType::Withdraw => u32::MAX,
                JobType::Claim if c.has_parts_for_job(JobType::Claim) => {
                    10 - c.get_active_bodyparts(screeps::Part::Move) as u32
                }
                JobType::Claim => u32::MAX,
                JobType::Reserve => todo!(),
                JobType::Attack => todo!(),
                JobType::AttackR => todo!(),
                JobType::Defend => todo!(),
                JobType::DefendR => todo!(),
                JobType::Heal => todo!(),
                JobType::Scout if c.has_parts_for_job(JobType::Claim) => {
                    10 - c.get_active_bodyparts(screeps::Part::Move) as u32
                }
                JobType::Scout => u32::MAX,
            })
        })
        .flatten()
        .collect::<Vec<u32>>();

    let assignments = hungarian::minimize(&matrix, height, width);

    assignments
        .iter()
        .filter_map(|&a| a)
        .enumerate()
        .for_each(|(i, j)| {
            let h = &creeps[i];
            let s = &contexts[j];
            info!("{:?} assigned to {:?}", h.name(), s,);
            if h.has_parts_for_job(s.job) {
                match s.job {
                    JobType::Harvest => {
                        if let Some(source) = s.target.find_closest_by_range(find::SOURCES) {
                            match h.harvest(&source) {
                                screeps::ReturnCode::Ok => {
                                    harvesters.push(h.to_owned());
                                }
                                screeps::ReturnCode::NotInRange => {
                                    h.move_to(&source);
                                }
                                _ => {}
                            }
                        }
                    }
                    JobType::Upgrade => {
                        if let Some(st) = s.target.find_closest_by_range(find::MY_STRUCTURES) {
                            match st.as_structure() {
                                screeps::Structure::Controller(ctrl) => {
                                    match h.upgrade_controller(&ctrl) {
                                        screeps::ReturnCode::Ok => {
                                            upgraders.push(h.to_owned());
                                        }
                                        screeps::ReturnCode::NotInRange => {
                                            h.move_to(&ctrl);
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    JobType::Transfer => {
                        if let Some(st) = s.target.find_closest_by_range(find::STRUCTURES) {
                            match st.as_transferable() {
                                Some(s) => match h.transfer_all(s, ResourceType::Energy) {
                                    screeps::ReturnCode::Ok => {
                                        haulers.push(h.to_owned());
                                    }
                                    screeps::ReturnCode::NotEnough => {
                                        h.transfer_amount(
                                            s,
                                            ResourceType::Energy,
                                            h.store_capacity(Some(ResourceType::Energy)),
                                        );
                                    }
                                    screeps::ReturnCode::Full => {
                                        h.drop(
                                            ResourceType::Energy,
                                            Some(h.store_used_capacity(Some(ResourceType::Energy))),
                                        );
                                    }
                                    screeps::ReturnCode::NotInRange => {
                                        h.move_to(&s.pos());
                                    }
                                    _ => {}
                                },
                                None => {}
                            }
                        }
                    }
                    JobType::Pickup => {
                        if let Some(r) = s.target.find_closest_by_range(find::DROPPED_RESOURCES) {
                            match h.pickup(&r) {
                                screeps::ReturnCode::Ok => {
                                    gatherers.push(h.to_owned());
                                }
                                screeps::ReturnCode::NotInRange => {
                                    h.move_to(&r.pos());
                                }
                                _ => {}
                            }
                        }
                    }
                    JobType::Build => {
                        if let Some(st) = s.target.find_closest_by_range(find::CONSTRUCTION_SITES) {
                            match h.build(&st) {
                                screeps::ReturnCode::Ok => {
                                    builders.push(h.to_owned());
                                }
                                screeps::ReturnCode::NotInRange => {
                                    h.move_to(&s.target);
                                }
                                _ => {
                                    info!("Error building");
                                }
                            }
                        }
                    }
                    JobType::Repair => {
                        if let Some(st) = s.target.find_closest_by_range(find::STRUCTURES) {
                            match h.repair(&st) {
                                screeps::ReturnCode::Ok => {
                                    repairers.push(h.to_owned());
                                }
                                screeps::ReturnCode::NotInRange => {
                                    h.move_to(&s.target);
                                }
                                _ => {}
                            }
                        }
                    }
                    JobType::Withdraw => {
                        if let Some(st) = s.target.find_closest_by_range(find::STRUCTURES) {
                            if vec![
                                screeps::StructureType::Storage,
                                screeps::StructureType::Container,
                            ]
                            .contains(&st.structure_type())
                            {
                                match st.as_withdrawable() {
                                    Some(wd) => match h.withdraw_amount(
                                        wd,
                                        ResourceType::Energy,
                                        h.store_free_capacity(None) as u32,
                                    ) {
                                        screeps::ReturnCode::Ok => {
                                            haulers.push(h.to_owned());
                                        }
                                        screeps::ReturnCode::NotEnough => {
                                            h.withdraw_all(wd, ResourceType::Energy);
                                        }
                                        screeps::ReturnCode::NotInRange => {
                                            h.move_to(&s.target);
                                        }
                                        _ => {}
                                    },
                                    None => {}
                                }
                            }
                        }
                    }
                    JobType::Station => {
                        h.move_to(&s.target);
                    }
                    JobType::Scout => todo!(),
                    JobType::Reserve => todo!(),
                    JobType::Attack => todo!(),
                    JobType::AttackR => todo!(),
                    JobType::Defend => todo!(),
                    JobType::DefendR => todo!(),
                    JobType::Heal => todo!(),
                    JobType::Claim => {
                        if h.room().unwrap().name() != s.target.room_name() {
                            h.move_to(&s.target);
                        } else {
                            let l = s.target.look();
                            l.iter().for_each(|lr| match lr {
                                screeps::LookResult::Source(s) => match h.harvest(s) {
                                    screeps::ReturnCode::Ok => {}
                                    screeps::ReturnCode::NotInRange => {
                                        h.move_to(s);
                                    }
                                    _ => {
                                        info!("Error scout-harvesting");
                                    }
                                },
                                screeps::LookResult::Structure(st) => match st.room() {
                                    Some(rm) => match rm.controller() {
                                        Some(ctrl) => match h.claim_controller(&ctrl) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                h.move_to(&ctrl);
                                            }
                                            screeps::ReturnCode::GclNotEnough => {
                                                match h.reserve_controller(&ctrl) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        h.move_to(&ctrl);
                                                    }
                                                    _ => {
                                                        info!("Error scouting");
                                                    }
                                                }
                                            }
                                            _ => {
                                                info!("Error scouting");
                                            }
                                        },
                                        None => {
                                            h.move_to(&s.target);
                                        }
                                    },
                                    None => {
                                        h.move_to(&s.target);
                                    }
                                },
                                _ => {}
                            })
                        }
                    }
                }
            }
        });

    if (width > height || total < 6) && total < 30 {
        if screeps::game::time() % 15 == 1 {
            spawning::init(
                creeps.to_vec(),
                harvesters.to_vec(),
                haulers.to_vec(),
                builders.to_vec(),
                repairers.to_vec(),
                upgraders.to_vec(),
                gatherers.to_vec(),
                defenders.to_vec(),
            );
        }
    }
}
