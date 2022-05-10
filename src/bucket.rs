use log::info;
use screeps::{
    HasId, HasPosition, HasStore, ObjectId, Position, ResourceType, RoomObjectProperties,
    SharedCreepProperties, StructureProperties,
};

use crate::jobs::{JobProperties, JobType};
use crate::{filters, flags};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Context {
    job: JobType,
    target: Position,
    priority: i32,
    work: u32,
}

impl Context {
    pub fn complete_context(self: Self, creep: &screeps::Creep) {
        info!(
            "{:?} assigned to {:?} ({:?}), {:?}",
            creep.name(),
            self.job,
            creep.contribution_per_tick(self.job),
            self.target
        );
        match self.job {
            JobType::Harvest => {
                if let Some(source) = self.target.find_closest_by_range(screeps::find::SOURCES) {
                    match creep.harvest(&source) {
                        screeps::ReturnCode::Ok => {
                            // harvesters.push(h.to_owned());
                        }
                        screeps::ReturnCode::NotInRange => {
                            creep.move_to(&source);
                        }
                        _ => {}
                    }
                }
            }
            JobType::Upgrade => {
                if let Some(st) = self
                    .target
                    .find_closest_by_range(screeps::find::MY_STRUCTURES)
                {
                    match st.as_structure() {
                        screeps::Structure::Controller(ctrl) => {
                            match creep.upgrade_controller(&ctrl) {
                                screeps::ReturnCode::Ok => {
                                    // upgraders.push(h.to_owned());
                                }
                                screeps::ReturnCode::NotInRange => {
                                    creep.move_to(&ctrl);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            JobType::Transfer => {
                if let Some(st) = self.target.find_closest_by_range(screeps::find::STRUCTURES) {
                    match st.as_transferable() {
                        Some(s) => match creep.transfer_all(s, ResourceType::Energy) {
                            screeps::ReturnCode::Ok => {
                                // haulers.push(h.to_owned());
                            }
                            screeps::ReturnCode::NotEnough => {
                                creep.transfer_amount(
                                    s,
                                    ResourceType::Energy,
                                    creep.store_capacity(Some(ResourceType::Energy)),
                                );
                            }
                            screeps::ReturnCode::Full => {
                                creep.drop(
                                    ResourceType::Energy,
                                    Some(creep.store_used_capacity(Some(ResourceType::Energy))),
                                );
                            }
                            screeps::ReturnCode::NotInRange => {
                                creep.move_to(&s.pos());
                            }
                            _ => {}
                        },
                        None => {}
                    }
                }
            }
            JobType::Pickup => {
                if let Some(r) = self
                    .target
                    .find_closest_by_range(screeps::find::DROPPED_RESOURCES)
                {
                    match creep.pickup(&r) {
                        screeps::ReturnCode::Ok => {
                            // gatherers.push(h.to_owned());
                        }
                        screeps::ReturnCode::NotInRange => {
                            creep.move_to(&r.pos());
                        }
                        _ => {}
                    }
                }
            }
            JobType::Build => {
                if let Some(st) = self
                    .target
                    .find_closest_by_range(screeps::find::CONSTRUCTION_SITES)
                {
                    match creep.build(&st) {
                        screeps::ReturnCode::Ok => {
                            // builders.push(h.to_owned());
                        }
                        screeps::ReturnCode::NotInRange => {
                            creep.move_to(&self.target);
                        }
                        _ => {
                            info!("Error building");
                        }
                    }
                }
            }
            JobType::Repair => {
                if let Some(st) = self.target.find_closest_by_range(screeps::find::STRUCTURES) {
                    match creep.repair(&st) {
                        screeps::ReturnCode::Ok => {
                            // repairers.push(h.to_owned());
                        }
                        screeps::ReturnCode::NotInRange => {
                            creep.move_to(&self.target);
                        }
                        _ => {}
                    }
                }
            }
            JobType::Withdraw => {
                if let Some(st) = self.target.find_closest_by_range(screeps::find::STRUCTURES) {
                    if vec![
                        screeps::StructureType::Storage,
                        screeps::StructureType::Container,
                    ]
                    .contains(&st.structure_type())
                    {
                        match st.as_withdrawable() {
                            Some(wd) => match creep.withdraw_amount(
                                wd,
                                ResourceType::Energy,
                                creep.store_free_capacity(None) as u32,
                            ) {
                                screeps::ReturnCode::Ok => {
                                    // haulers.push(h.to_owned());
                                }
                                screeps::ReturnCode::NotEnough => {
                                    creep.withdraw_all(wd, ResourceType::Energy);
                                }
                                screeps::ReturnCode::NotInRange => {
                                    creep.move_to(&self.target);
                                }
                                _ => {}
                            },
                            None => {}
                        }
                    }
                }
            }
            JobType::Station => {
                creep.move_to(&self.target);
            }
            JobType::Scout => todo!(),
            JobType::Reserve => todo!(),
            JobType::Attack => todo!(),
            JobType::AttackR => todo!(),
            JobType::Defend => todo!(),
            JobType::DefendR => todo!(),
            JobType::Heal => todo!(),
            JobType::Claim => {
                if creep.room().unwrap().name() != self.target.room_name() {
                    creep.move_to(&self.target);
                } else {
                    let l = self.target.look();
                    l.iter().for_each(|lr| match lr {
                        screeps::LookResult::Source(s) => match creep.harvest(s) {
                            screeps::ReturnCode::Ok => {}
                            screeps::ReturnCode::NotInRange => {
                                creep.move_to(s);
                            }
                            _ => {
                                info!("Error scout-harvesting");
                            }
                        },
                        screeps::LookResult::Structure(st) => match st.room() {
                            Some(rm) => match rm.controller() {
                                Some(ctrl) => match creep.claim_controller(&ctrl) {
                                    screeps::ReturnCode::Ok => {}
                                    screeps::ReturnCode::NotInRange => {
                                        creep.move_to(&ctrl);
                                    }
                                    screeps::ReturnCode::GclNotEnough => {
                                        match creep.reserve_controller(&ctrl) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creep.move_to(&ctrl);
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
                                    creep.move_to(&self.target);
                                }
                            },
                            None => {
                                creep.move_to(&self.target);
                            }
                        },
                        _ => {}
                    })
                }
            }
        }
    }
}

pub fn get_harvest_jobs(mut creeps: Vec<screeps::Creep>) -> Vec<Context> {
    let mut sources = filters::get_my_sources();
    sources.retain(|s| s.energy() > 0 || s.ticks_to_regeneration() < 20);
    sources.dedup_by_key(|s| s.pos().packed_repr());

    sources
        .iter()
        .map(|s| Context {
            job: JobType::Harvest,
            target: s.pos(),
            priority: 0,
            work: 10,
        })
        .collect::<Vec<Context>>()

    // // Holds a vector of positions near the source
    // let source_slots = sources
    //     .iter()
    //     .map(|s| {
    //         s.pos()
    //             .neighbors()
    //             .iter()
    //             .filter(|&pos| pos.move_cost().is_some())
    //             .map(|&pos| pos)
    //             .collect::<Vec<Position>>()
    //     })
    //     .flatten()
    //     .collect::<Vec<Position>>();

    // source_slots
    //     .iter()
    //     .map(|s| Context {
    //         job: JobType::Harvest,
    //         target: *s,
    //         priority: 0,
    //         work: 10,
    //     })
    //     .collect::<Vec<Context>>()
}

pub fn get_transfer_jobs(mut creeps: Vec<screeps::Creep>) -> Vec<Context> {
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
        .map(|d| Context {
            job: JobType::Transfer,
            target: d.pos(),
            work: d
                .as_has_store()
                .unwrap()
                .store_free_capacity(Some(screeps::ResourceType::Energy)) as u32,
            priority: match d.structure_type() {
                screeps::StructureType::Spawn => 2,
                screeps::StructureType::Extension => 1,
                screeps::StructureType::Link => 2,
                screeps::StructureType::Storage => 3,
                screeps::StructureType::Tower => 2,
                screeps::StructureType::Lab => 2,
                screeps::StructureType::Factory => 2,
                _ => 99,
            },
        })
        .collect::<Vec<Context>>()
}

pub fn get_gather_jobs(mut creeps: Vec<screeps::Creep>) -> Vec<Context> {
    let mut contexts = filters::get_groundscores()
        .iter()
        .map(|g| Context {
            job: JobType::Pickup,
            target: g.pos(),
            work: g.amount(),
            priority: 6,
        })
        .collect::<Vec<Context>>();

    filters::get_my_rooms().iter().for_each(|r| {
        let ruins = r.find(screeps::find::RUINS);
        ruins.iter().for_each(|r| {
            contexts.push(Context {
                job: JobType::Withdraw,
                target: r.pos(),
                work: r.store_used_capacity(None),
                priority: 10,
            })
        });
        let tomb = r.find(screeps::find::TOMBSTONES);
        tomb.iter().for_each(|r| {
            contexts.push(Context {
                job: JobType::Withdraw,
                target: r.pos(),
                work: r.store_used_capacity(None),
                priority: 10,
            })
        });
    });

    contexts
}

pub fn get_upgrade_jobs(mut creeps: Vec<screeps::Creep>) -> Vec<Context> {
    filters::get_my_controllers()
        .iter()
        .map(|c| Context {
            job: JobType::Upgrade,
            target: c.pos(),
            work: 1,
            priority: 6,
        })
        .collect::<Vec<Context>>()
}

pub fn get_build_jobs(mut creeps: Vec<screeps::Creep>) -> Vec<Context> {
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
        return vec![Context {
            job: JobType::Build,
            target: c.pos(),
            work: c.progress_total() - c.progress(),
            priority: 5,
        }];
    } else {
        return vec![];
    }
}

pub fn get_repair_jobs(mut creeps: Vec<screeps::Creep>) -> Vec<Context> {
    if creeps
        .iter()
        .filter(|c| c.has_parts_for_job(JobType::Repair))
        .count()
        == 0
    {
        return vec![];
    }
    if let Some(c) = filters::get_my_repairables().first() {
        return vec![Context {
            job: JobType::Repair,
            target: c.pos(),
            work: 100,
            priority: 3,
        }];
    } else {
        return vec![];
    }
}

pub fn get_scout_jobs(mut creeps: Vec<screeps::Creep>) -> Vec<Context> {
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
            work: 1,
            priority: 0,
        })
        .collect::<Vec<Context>>()
}
pub fn bucket_sort(mut creeps: Vec<screeps::Creep>) {
    let total = creeps.len();

    let mut contexts = vec![];

    contexts.extend(get_harvest_jobs(creeps.to_vec()));
    contexts.extend(get_transfer_jobs(creeps.to_vec()));
    contexts.extend(get_upgrade_jobs(creeps.to_vec()));
    contexts.extend(get_gather_jobs(creeps.to_vec()));
    contexts.extend(get_build_jobs(creeps.to_vec()));
    contexts.extend(get_repair_jobs(creeps.to_vec()));
    contexts.extend(get_scout_jobs(creeps.to_vec()));

    info!("{:?}", contexts);
    partition_select(creeps, contexts);
}

/// Performs a 'sieve' function on creeps; filling each Context as a 'bucket'
pub fn partition_select(mut creeps: Vec<screeps::Creep>, mut contexts: Vec<Context>) {
    contexts.sort_unstable_by(|a, b| a.priority.cmp(&b.priority));

    // let mut contexts = *contexts.as_slice().to_owned();

    for mut context in contexts {
        let mut pq = priority_queue::PriorityQueue::new();

        // build a heap of creep contributions
        creeps.iter().for_each(|c| {
            pq.push(c.id(), c.contribution_per_tick(context.job));
        });

        // fill bucket
        while context.work > 0 && pq.len() > 0 {
            if let Some(best_fit) = pq.pop() {
                if let Some(c) = best_fit.0.resolve() {
                    context.work -= best_fit.1;
                    context.complete_context(&c); // do work
                }
            }
        }
        // reconstruct the creeps array; less the popped values
        creeps = pq
            .iter()
            .filter_map(|h| h.0.resolve())
            .collect::<Vec<screeps::Creep>>();
    }
}
