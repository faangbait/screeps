use screeps::traits::TryInto;
use screeps::{Creep, HasPosition, LookResult, ObjectId, RawObjectId, StructureContainer, HasStore};

use crate::filters;
use crate::jobs::JobProperties;
use crate::rtb::{JobAsk, JobBid, SinkNode, SourceNode};

/// Given a Linked List, Q, iterate through Q to find the highest ratio by comparing each ratio within the queue.
/// Once a ratio of element N is greater than the element M with the highest ratio, replace element M with element N
/// as the highest ratio element in the list. Once the end of the list is reached, dequeue the highest ratio element.
/// If the element is at the start of the list, dequeue it and set the list to its next element, returning the element.
/// Otherwise N's neighbors are reassigned to identify each other as their next and previous neighbor, returning the
/// result of N.
// pub fn hrrn(mut pq: ContextList) -> Option<Context> {
//     let highest_prio = pq.queue.pop();

//     match highest_prio {
//         Some((context, _)) => Some(context),
//         None => {},
//     }
// }

pub fn load_asks_from_memory() -> Option<Vec<JobAsk>> {
    None
}
pub fn load_bids_from_memory() -> Option<Vec<JobBid>> {
    None
}

pub fn load_contexts() {
    let mut creeps = screeps::game::creeps::values();

    let mut ask_vec = match load_asks_from_memory() {
        Some(asks) => asks,
        None => vec![],
    };

    let mut bid_vec = match load_bids_from_memory() {
        Some(bids) => bids,
        None => vec![],
    };

    // let ask_list = priority_queue::PriorityQueue::<JobAsk, i32>::from_iter(
    //     ask_vec.iter().map(|job| (*job, job.ask as i32))
    // );

    // let bid_list = priority_queue::PriorityQueue::<JobBid, i32>::from_iter(
    //     bid_vec.iter().map(|job| (*job, job.bid as i32))
    // );

    filters::get_my_rooms()
        .iter()
        .flat_map(|room| room.look_at_area(0, 0, 50, 50))
        .for_each(|res| {
            match &res.look_result {
                LookResult::Creep(st) => {
                    match st.sink_request() {
                        Some(req) => bid_vec.push(req),
                        None => {}
                    };
                    match st.source_request() {
                        Some(req) => ask_vec.push(req),
                        None => {}
                    };
                }
                LookResult::Energy(st) => {
                    match st.source_request() {
                        Some(req) => ask_vec.push(req),
                        None => {}
                    };
                }
                LookResult::Resource(st) => {
                    match st.source_request() {
                        Some(req) => ask_vec.push(req),
                        None => {}
                    };
                }
                LookResult::Source(st) => {
                    match st.source_request() {
                        Some(req) => ask_vec.push(req),
                        None => {}
                    };
                }
                LookResult::Mineral(st) => {
                    match st.source_request() {
                        Some(req) => ask_vec.push(req),
                        None => {}
                    };
                }
                LookResult::Deposit(st) => {
                    match st.source_request() {
                        Some(req) => ask_vec.push(req),
                        None => {}
                    };
                }
                LookResult::ConstructionSite(st) => {
                    match st.sink_request() {
                        Some(req) => bid_vec.push(req),
                        None => {}
                    };
                }
                LookResult::Tombstone(st) => {
                    match st.source_request() {
                        Some(req) => ask_vec.push(req),
                        None => {}
                    };
                }
                // LookResult::PowerCreep(st) => {
                //     match st.sink_request() {
                //         Some(req) => {bid_vec.push(req)},
                //         None => {},
                //     };
                //     match st.source_request() {
                //         Some(req) => {ask_vec.push(req)},
                //         None => {},
                //     };
                // },
                LookResult::Structure(structure) => match structure {
                    screeps::Structure::Container(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Controller(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Extension(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Extractor(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Factory(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Lab(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Link(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Nuker(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Observer(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::PowerSpawn(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Rampart(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Road(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Spawn(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Storage(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Terminal(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Tower(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Wall(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                    }
                    _ => {}
                },
                _ => {}
            }
        });

    creeps.sort_unstable_by_key(|c| -1 * c.body().len() as i8);
    ask_vec.sort_unstable_by_key(|job| job.ask);

    creeps.iter().for_each(|creep| {
        for job in &ask_vec {
            if !creep.has_parts_for_job(job.request) { // can't do this job
                continue;
            }

            if creep.store_free_capacity(job.resource) == 0 { continue; } // don't need to acquire resources
            // if creep.context()

            match job.ty {
                crate::rtb::SinkSources::Creep => {
                    let obj: Result<ObjectId<screeps::Creep>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Energy => {
                    let obj: Result<ObjectId<screeps::Resource>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Resource => {
                    let obj: Result<ObjectId<screeps::Resource>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Source => {
                    let obj: Result<ObjectId<screeps::Source>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Mineral => {
                    let obj: Result<ObjectId<screeps::Mineral>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Deposit => {
                    let obj: Result<ObjectId<screeps::Deposit>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::ConstructionSite => {
                    let obj: Result<ObjectId<screeps::ConstructionSite>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Tombstone => {
                    let obj: Result<ObjectId<screeps::Tombstone>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::PowerCreep => {
                    let obj: Result<ObjectId<screeps::PowerCreep>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Structure => {
                    let obj: Result<ObjectId<screeps::Structure>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Controller => {
                    let obj: Result<ObjectId<screeps::StructureController>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Container => {
                    let obj: Result<ObjectId<screeps::StructureContainer>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Extension => {
                    let obj: Result<ObjectId<screeps::StructureExtension>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Extractor => {
                    let obj: Result<ObjectId<screeps::StructureExtractor>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Factory => {
                    let obj: Result<ObjectId<screeps::StructureFactory>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Lab => {
                    let obj: Result<ObjectId<screeps::StructureLab>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Link => {
                    let obj: Result<ObjectId<screeps::StructureLink>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Nuker => {
                    let obj: Result<ObjectId<screeps::StructureNuker>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Observer => {
                    let obj: Result<ObjectId<screeps::StructureObserver>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::PowerSpawn => {
                    let obj: Result<ObjectId<screeps::StructurePowerSpawn>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Rampart => {
                    let obj: Result<ObjectId<screeps::StructureRampart>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Road => {
                    let obj: Result<ObjectId<screeps::StructureRoad>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Spawn => {
                    let obj: Result<ObjectId<screeps::StructureSpawn>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Storage => {
                    let obj: Result<ObjectId<screeps::StructureStorage>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Terminal => {
                    let obj: Result<ObjectId<screeps::StructureTerminal>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Tower => {
                    let obj: Result<ObjectId<screeps::StructureTower>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
                crate::rtb::SinkSources::Wall => {
                    let obj: Result<ObjectId<screeps::StructureWall>, _> =
                        screeps::traits::TryFrom::try_from(job.target.to_u128());
                    match obj {
                        Ok(o) => {
                            match o.try_resolve() {
                                Ok(opt) => match opt {
                                    Some(obj) => {
                                        if creep.pos().get_range_to(&obj) > 75
                                            && creep.body().len() < 5
                                        {
                                            continue;
                                        }
                                        //A
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
            };
        }
    })
}
