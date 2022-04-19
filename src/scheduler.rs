use log::{warn, info, debug};
use priority_queue::PriorityQueue;
use screeps::{ReturnCode, HasPosition, SharedCreepProperties, HasStore, ResourceType, Creep, Part};

use crate::{spawning, filters, travel, logistics::{ResourceRequest, WorkRequest, WorkerRequest, self, WorkerLogistic}, util};


pub fn oscheduler() {
    for creep in spawning::get_by_role(screeps::game::creeps::values(), "coldboot") {
        let e = creep.store_used_capacity(Some(ResourceType::Energy));
        let f = creep.store_capacity(Some(ResourceType::Energy));
        let g = creep.store_free_capacity(Some(ResourceType::Energy));
        match e {
            0..=49 => match creep.groundscore(ResourceType::Energy, f) {
                Some(t) => match creep.pickup(&t) {
                    ReturnCode::Ok => {info!("{} targeting a groundscore at {:?}", creep.name(), t.pos())},
                    ReturnCode::NotInRange => {creep.move_to(&t.pos());},
                    _ => warn!("Error pickup")
                },
                None => {
                    let sources = filters::get_my_sources();
                    let nearest = sources.iter().min_by_key(|s|travel::calc_travel_fatigue(&creep, &s.pos()));
                    match nearest {
                        Some(s) => match creep.harvest(s) {
                            ReturnCode::NotInRange => {creep.move_to(&s.pos());},
                            _ => {}
                        },
                        None => warn!("Nothing to harvest?"),
                    }
                },
            },
            _ => match creep.provide_resources(ResourceType::Energy, e) {
                Some(t) => match creep.transfer_all(t.target.as_transferable().unwrap(),ResourceType::Energy) {
                    ReturnCode::Ok => {},
                    ReturnCode::NotInRange => {creep.move_to(&t.target.pos());},
                    _ => warn!("Error transfer")
                },
                None => warn!("{} has nowhere to store energy ({}).", creep.name(), e),
            }
        }
    }
    for (idx, creep) in spawning::get_by_role(screeps::game::creeps::values(), "harvester")
        .iter()
        .enumerate()
    {
        // harvesting provides 2 energy per workpart per tick
        // sources regen every 3000 ticks
        // 5 workparts will max out a source

        let sources = filters::get_my_sources();
       
        let mut untapped_source = sources.iter()
            .filter(|&s| screeps::game::creeps::values().iter()
                    .filter(|&c| c.pos().is_near_to(&s.pos()) && c != creep)
                    .fold(0,|acc, c| acc + c.get_active_bodyparts(screeps::Part::Work)) + creep.get_active_bodyparts(screeps::Part::Work) <= 6)
                    .collect::<Vec<&screeps::Source>>();
                   
        untapped_source.sort_by_key(|s| travel::calc_travel_fatigue(creep, &s.pos()));
        // untapped_source.sort_by_key(|s| s.pos().find_in_range(screeps::find::CREEPS, 1).iter().fold(0, |acc, c| acc + c.get_active_bodyparts(screeps::Part::Work)));

        match untapped_source.first() {
            Some(target) => match creep.harvest(*target) {
                ReturnCode::Ok | ReturnCode::Busy | ReturnCode::Tired | ReturnCode::Full => {
                    continue;
                }
                ReturnCode::NotOwner => {
                    warn!("{:?} is not the owner of {:?}", creep.name(), target.pos());
                }
                ReturnCode::NoPath => {
                    travel::set_waypoint(&creep, &target.pos());
                    info!(
                        "{:?} nopath to {:?}, retrying",
                        &creep.name(),
                        &target.pos()
                    );
                }
                ReturnCode::InvalidArgs
                | ReturnCode::NameExists
                | ReturnCode::NotFound
                | ReturnCode::InvalidTarget => warn!("{} Invalid args given to harvester", creep.name()),
                ReturnCode::NotEnough => warn!("{} is out of resources to mine.", creep.name()),
                ReturnCode::NotInRange => {
                    travel::set_waypoint(&creep, &target.pos());
                    debug!("{:?} headed to {:?}", &creep.name(), &target.pos());
                }
                ReturnCode::NoBodypart => warn!(
                    "{:?} doesn't have the required body parts for his job",
                    &creep.name()
                ),
                ReturnCode::RclNotEnough | ReturnCode::GclNotEnough => warn!("RCL/GCL"),
            },
            None => continue,
        }


    }

    for (idx, creep) in spawning::get_by_role(screeps::game::creeps::values(), "hauler").iter().enumerate(){
        let e = creep.store_used_capacity(Some(ResourceType::Energy));
        let f = creep.store_capacity(Some(ResourceType::Energy));
        
        match e {
            20..=9999 => match creep.provide_resources(ResourceType::Energy, e) {
                Some(t) => match creep.transfer_all(t.target.as_transferable().unwrap(),ResourceType::Energy) {
                    ReturnCode::Ok => {},
                    ReturnCode::NotInRange => {creep.move_to(&t.target.pos());},
                    _ => warn!("Error transfer")
                },
                None => warn!("{} has nowhere to store energy ({}).", creep.name(), e),
            }
            _ => match creep.groundscore(ResourceType::Energy, f) {
                Some(t) => match creep.pickup(&t) {
                    ReturnCode::Ok => {info!("{} targeting a groundscore at {:?}", creep.name(), t.pos())},
                    ReturnCode::NotInRange => {creep.move_to(&t.pos());},
                    _ => warn!("Error pickup")
                },
                None => warn!("{} has nothing to do.", creep.name()),
            }
        }

    }

    for creep in spawning::get_by_role(screeps::game::creeps::values(), "builder") {
        let e = creep.store_used_capacity(Some(ResourceType::Energy));
        let f = creep.store_capacity(Some(ResourceType::Energy));
        let g = creep.store_free_capacity(Some(ResourceType::Energy));
        match e {
            0 => match creep.request_resources(ResourceType::Energy, g as u32) {
                Some(t) => match creep.withdraw_amount(t.target.as_withdrawable().unwrap(), ResourceType::Energy, g as u32) {
                    ReturnCode::Ok => {},
                    ReturnCode::NotInRange => {creep.move_to(&t.target.pos());},
                    _ => info!("Unhandled error for {}", creep.name()),
                },
                None => match creep.groundscore(ResourceType::Energy, f) {
                    Some(t) => match creep.pickup(&t) {
                        ReturnCode::Ok => {info!("{} targeting a groundscore at {:?}", creep.name(), t.pos())},
                        ReturnCode::NotInRange => {creep.move_to(&t.pos());},
                        _ => warn!("Error pickup")
                    },
                    None => warn!("{} has nothing to do.", creep.name()),
                },
            },
            _ => match creep.get_active_bodyparts(screeps::Part::Work) {
                0 => {},
                _ => creep.request_work(ResourceType::Energy),
            }
        }
    }
    //     let mut constructions = architect::get_my_buildables();
    //     let mut repairs = architect::get_my_repairables();
    //     let mut groundscores = architect::get_groundscores();
    //     let mut containers = architect::get_wd_containers_f();

    //     containers.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));
    //     constructions.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));
    //     repairs.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));

    //     let groundscore = groundscores.first();
    //     let container = containers.first();
    //     let repair = repairs.first();
    //     let construction = constructions.first();

    //     match creep.store_used_capacity(Some(ResourceType::Energy)) {
    //         // nothing carried
    //         0 => match container {
    //             // there is a container
    //             Some(c) => match c.as_withdrawable() {
    //                 Some(c) => match creep.withdraw_amount(
    //                     c,
    //                     ResourceType::Energy,
    //                     creep.store_capacity(Some(ResourceType::Energy))
    //                         - creep.store_used_capacity(Some(ResourceType::Energy)),
    //                 ) {
    //                     ReturnCode::Ok => debug!("{:?} picked up resources.", creep.name()),
    //                     ReturnCode::NoPath => {
    //                         info!("{:?} nopathed to {:?}", creep.name(), &c.pos())
    //                     }
    //                     ReturnCode::NotEnough => {
    //                         creep.withdraw_all(c, ResourceType::Energy);
    //                     }
    //                     ReturnCode::NotInRange => {
    //                         pathing::set_waypoint(&creep, &c.pos());
    //                         debug!("{:?} headed to {:?}; pickup", creep.name(), &c.pos());
    //                     }
    //                     _ => warn!(
    //                         "{:?} ran into an invalid branch. Withdrawing from container",
    //                         creep.name()
    //                     ),
    //                 },
    //                 None => warn!(
    //                     "{:?} ran into an invalid branch. Container unwithdrawable",
    //                     creep.name()
    //                 ),
                    
    //                 // no container
    //                 // no ground loot
    //             },
    //             None => match groundscore {
    //                 // there is ground loot
    //                 Some(c) => match creep.pickup(&c) {
    //                     ReturnCode::Ok => debug!("{:?} picked up resources.", creep.name()),
    //                     ReturnCode::NotInRange => {
    //                         pathing::set_waypoint(&creep, &c.pos());
    //                         debug!("{:?} headed to {:?}; pickup", creep.name(), &c.pos());
    //                     }
    //                     _ => warn!(
    //                         "{:?} ran into an invalid branch. {:?}",
    //                         creep.name(),
    //                         creep.pickup(&c)
    //                     ),
    //                 },
    //                 None => warn!("{:?} has nothing to do; no resources", creep.name()),
    //             },
    //         },
    //         // we have resources
    //         _ => match construction {
    //             // there are construction projects
    //             Some(c) => match creep.build(&c) {
    //                 ReturnCode::Ok => debug!("{:?} built {:?}.", creep.name(), c.structure_type()),
    //                 ReturnCode::NotInRange => {
    //                     pathing::set_waypoint(&creep, &c.pos());
    //                     info!("{:?} headed to {:?}, building", &creep.name(), &c.pos());
    //                 }
    //                 _ => warn!(
    //                     "{:?} ran into an invalid branch. {:?}",
    //                     creep.name(),
    //                     creep.build(&c)
    //                 ),
    //             },
    //             // no construction projects
    //             None => match repair {
    //                 // there are repair projects
    //                 Some(c) => match creep.repair(c) {
    //                     ReturnCode::Ok => debug!("{:?} repaired {:?}.", creep.name(), c.pos()),
    //                     ReturnCode::NotInRange => {
    //                         pathing::set_waypoint(&creep, &c.pos());
    //                         info!("{:?} headed to {:?}, repairing", &creep.name(), &c.pos());
    //                     }
    //                     _ => warn!(
    //                         "{:?} ran into an invalid branch. {:?}",
    //                         creep.name(),
    //                         creep.repair(c)
    //                     ),
    //                 },
    //                 // no repair projects
    //                 None => {
    //                     warn!("{:?} has nothing to do; upgrading controller", creep.name());
    //                     let mut controllers = architect::get_my_controllers();
    //                     let controller = controllers.first();
    //                     match controller {
    //                         Some(c) => match creep.upgrade_controller(&c) {
    //                             ReturnCode::Ok => debug!("{:?} upgraded controller.", creep.name()),
    //                             ReturnCode::NotInRange => {
    //                                 pathing::set_waypoint(&creep, &c.pos());
    //                                 debug!("{:?} headed to {:?}; controller", creep.name(), &c.pos());
    //                             }
    //                             _ => warn!(
    //                                 "{:?} ran into an invalid branch. No controller.",
    //                                 creep.name()
    //                             ),
    //                         },
    //                         None => warn!(
    //                             "{:?} ran into an invalid branch. No controller.",
    //                             creep.name()
    //                         ),
    //                     }
    //                 }
    //             },
    //         },
    //     }
    // }

    // for creep in spawning::get_by_role(&screeps::game::creeps::values(), "average") {
    //     let mut groundscores = architect::get_groundscores();
    //     let mut containers = architect::get_wd_containers_f();
    //     let mut controllers = architect::get_my_controllers();

    //     containers.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));
    //     controllers.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));

    //     let groundscore = groundscores.first();
    //     let container = containers.first();
    //     let controller = controllers.first();

    //     match creep.store_used_capacity(Some(ResourceType::Energy)) {
    //         // nothing carried
    //         0 => match container {
    //             // there is a container
    //             Some(c) => match c.as_withdrawable() {
    //                 Some(c) => match creep.withdraw_amount(
    //                     c,
    //                     ResourceType::Energy,
    //                     creep.store_capacity(Some(ResourceType::Energy))
    //                         - creep.store_used_capacity(Some(ResourceType::Energy)),
    //                 ) {
    //                     ReturnCode::Ok => debug!("{:?} picked up resources.", creep.name()),
    //                     ReturnCode::NoPath => {
    //                         info!("{:?} nopathed to {:?}", creep.name(), &c.pos())
    //                     }
    //                     ReturnCode::NotEnough => {
    //                         creep.withdraw_all(c, ResourceType::Energy);
    //                     }
    //                     ReturnCode::NotInRange => {
    //                         pathing::set_waypoint(&creep, &c.pos());
    //                         debug!("{:?} headed to {:?}; pickup", creep.name(), &c.pos());
    //                     }
    //                     _ => warn!(
    //                         "{:?} ran into an invalid branch. Withdrawing from container",
    //                         creep.name()
    //                     ),
    //                 },
    //                 None => warn!(
    //                     "{:?} ran into an invalid branch. Container unwithdrawable",
    //                     creep.name()
    //                 ),
    //             },

    //             // no container
    //             None => match groundscore {
    //                 // there is ground loot
    //                 Some(c) => match creep.pickup(&c) {
    //                     ReturnCode::Ok => debug!("{:?} picked up resources.", creep.name()),
    //                     ReturnCode::NotInRange => {
    //                         pathing::set_waypoint(&creep, &c.pos());
    //                         debug!("{:?} headed to {:?}; pickup", creep.name(), &c.pos());
    //                     }
    //                     _ => warn!(
    //                         "{:?} ran into an invalid branch. {:?}",
    //                         creep.name(),
    //                         creep.pickup(&c)
    //                     ),
    //                 },
    //                 // no ground loot
    //                 None => warn!("{:?} has nothing to do", creep.name()),
    //             },
    
    //         },
            
    //         // we have resources
    //         _ => match controller {
    //             Some(c) => match creep.upgrade_controller(&c) {
    //                 ReturnCode::Ok => debug!("{:?} upgraded controller.", creep.name()),
    //                 ReturnCode::NotInRange => {
    //                     pathing::set_waypoint(&creep, &c.pos());
    //                     debug!("{:?} headed to {:?}; controller", creep.name(), &c.pos());
    //                 }
    //                 _ => warn!(
    //                     "{:?} ran into an invalid branch. No controller.",
    //                     creep.name()
    //                 ),
    //             },
    //             None => warn!(
    //                 "{:?} ran into an invalid branch. No controller.",
    //                 creep.name()
    //             ),
    //         },
    //     }
    // }
}

pub fn scheduler() {
    let mut prq = PriorityQueue::<WorkerLogistic,i32>::new();
    let creeps = screeps::game::creeps::values();
    
    for source in filters::get_my_sources() {
        let request = source.request_workers();
        match request {
            Some(req) => { prq.push(req, req.priority); },
            None => {},
        };
    }
    for controller in filters::get_my_controllers() {
        let request = controller.request_workers();
        match request {
            Some(req) => { prq.push(req, req.priority); },
            None => {},
        };
    }

    for spawn in filters::get_my_spawns() {
        let request = spawn.request_workers();
        match request {
            Some(req) => { prq.push(req, req.priority); },
            None => {},
        };
    }
    
    for extension in filters::get_my_extensions() {
        let request = extension.request_workers();
        match request {
            Some(req) => { prq.push(req, req.priority); },
            None => {},
        };
    }
    
    for storage in filters::get_my_storages() {
        let request = storage.request_workers();
        match request {
            Some(req) => { prq.push(req, req.priority); },
            None => {},
        };
    }

    for container in filters::get_my_containers() {
        let request = container.request_workers();
        match request {
            Some(req) => { prq.push(req, req.priority); },
            None => {},
        };
    }
    
    for link in filters::get_my_links() {
        let request = link.request_workers();
        match request {
            Some(req) => { prq.push(req, req.priority); },
            None => {},
        };
    }
    
    for tower in filters::get_my_towers() {
        let request = tower.request_workers();
        match request {
            Some(req) => { prq.push(req, req.priority); },
            None => {},
        };
    }

    for site in filters::get_my_buildables() {
        let request = site.request_workers();
        match request {
            Some(req) => { prq.push(req, req.priority); },
            None => {},
        };
    }

    for resource in filters::get_groundscores() {
        let request = resource.request_workers();
        match request {
            Some(req) => { prq.push(req, req.priority); },
            None => {},
        };
    }
    
    
    

    let mut pq = prq.into_sorted_vec();
    
    let mut workers = creeps.iter()
    .filter(|c| c.body().iter()
        .fold(0, |acc,cur| match cur.part {
            screeps::Part::Work => acc+1,
            _ => acc,
        }) > 0)
        .collect::<Vec<&Creep>>();
    
    info!("Workers to process: {} ", workers.len());
    while workers.len() > 0 {
        let job = pq.pop();
        workers.sort_unstable_by_key(|&c| util::count_bodyparts(c, Part::Work) + util::count_bodyparts(c, Part::Carry));
        
        match job {
            Some(request) => {
                let target = request.target.look();
                debug!("Processing job at {:?}", request.target);
                let mut available_workers = workers.iter()
                    .map(|c| *c)
                    .filter(|&c| request.resource_min <= c.store_used_capacity(Some(request.resource_type)) && c.store_used_capacity(Some(request.resource_type)) <= request.resource_max )
                    .filter(|&c| request.target.in_range_to(c, 40))
                    .collect::<Vec<&Creep>>();

                for res in target {
                    match res {
                        screeps::LookResult::Source(s) => {
                            let w = available_workers.iter()
                                .max_by_key(|&c| util::count_bodyparts(c, Part::Work));
                            match w {
                                Some(&c) => match c.harvest(&s) {
                                    ReturnCode::NotInRange => {
                                        travel::set_waypoint(c, &request.target);
                                        workers.retain(|&x| *x.name() != *c.name());
                                        continue;
                                    },
                                    ReturnCode::Ok => { workers.retain(|&x| *x != *c); continue; },
                                    _ => warn!("Unhandled error in harvest at {:?}", request.target)
                                },
                                None => warn!("No qualified workers for harvest at {:?}", request.target),
                            }
                        },
                        screeps::LookResult::ConstructionSite(s) => {
                            let w = available_workers.iter()
                                .filter(|c| util::count_bodyparts(c, Part::Carry) > 0)
                                .min_by_key(|&c| travel::calc_travel_fatigue(&c, &s.pos()));

                            match w {
                                Some(&c) => match c.build(&s) {
                                    ReturnCode::NotInRange => {
                                        travel::set_waypoint(c, &request.target);
                                        workers.retain(|&x| *x.name() != *c.name());
                                        continue;
                                    },
                                    ReturnCode::Ok => { workers.retain(|&x| *x != *c); continue; },
                                    _ => warn!("Unhandled error in construction at {:?}", request.target)
                                },
                                None => debug!("No qualified workers for construction at {:?}", request.target),
                            }
                        },
                        screeps::LookResult::Structure(s) => {
                            let w = available_workers.iter()
                            .filter(|c| util::count_bodyparts(c, Part::Carry) > 0)
                            .min_by_key(|&c| travel::calc_travel_fatigue(&c, &s.pos()));

                            match w {
                                Some(c) => {
                                    match request.request {
                                        logistics::RequestType::Deposit => {
                                            match s.as_owned() {
                                                Some(st) => {
                                                    let room = st.room().unwrap();
                                                    match st.structure_type() {
                                                        screeps::StructureType::Container 
                                                        | screeps::StructureType::Extension 
                                                        | screeps::StructureType::Factory 
                                                        | screeps::StructureType::Spawn 
                                                        | screeps::StructureType::Lab 
                                                        | screeps::StructureType::Link 
                                                        | screeps::StructureType::Nuker 
                                                        | screeps::StructureType::Tower 
                                                        | screeps::StructureType::Storage 
                                                        | screeps::StructureType::PowerSpawn 
                                                        | screeps::StructureType::Terminal => {
                                                            
                                                            match c.transfer_all(s.as_transferable().unwrap(), request.resource_type) {
                                                                ReturnCode::NotInRange => {
                                                                    travel::set_waypoint(c, &request.target);
                                                                    workers.retain(|&x| *x.name() != *c.name());
                                                                    continue;
                                                                },
                                                                ReturnCode::Ok => { 
                                                                    workers.retain(|&x| *x.name() != *c.name()); 
                                                                    continue; 
                                                                },
                                                                ReturnCode::NotEnough => {
                                                                    match c.transfer_amount(s.as_transferable().unwrap(), request.resource_type, c.store_used_capacity(Some(request.resource_type))) {
                                                                        ReturnCode::NotInRange => {
                                                                            travel::set_waypoint(c, &request.target);
                                                                            workers.retain(|&x| *x.name() != *c.name());
                                                                            continue;
                                                                        },
                                                                        ReturnCode::Ok => { 
                                                                            workers.retain(|&x| *x.name() != *c.name());
                                                                            continue; 
                                                                        },
                                                                        _ => warn!("Unhandled error in transfer at {:?}", request.target)
                                                                    }
                                                                },
                                                                ReturnCode::Full => warn!("Full transfer?"),
                                                                _ => warn!("Unhandled error in transfer at {:?}", request.target)
                                                            }
                                                        }
                                                        _ => warn!("Invalid deposit request at {:?}", request.target),
                                                    }
                                                }
                                                None => warn!("Unknown building type at {:?}", request.target),
                                            }
                                        },
                                        logistics::RequestType::Repair => {
                                            match s.as_owned() {
                                                Some(s) => {
                                                    let room = s.room().unwrap();
                                                    match s.structure_type() {
                                                        screeps::StructureType::Spawn 
                                                        | screeps::StructureType::Extension 
                                                        | screeps::StructureType::Road 
                                                        | screeps::StructureType::Wall 
                                                        | screeps::StructureType::Rampart 
                                                        | screeps::StructureType::Link 
                                                        | screeps::StructureType::Storage 
                                                        | screeps::StructureType::Tower 
                                                        | screeps::StructureType::Observer 
                                                        | screeps::StructureType::PowerSpawn 
                                                        | screeps::StructureType::Extractor 
                                                        | screeps::StructureType::Lab 
                                                        | screeps::StructureType::Terminal 
                                                        | screeps::StructureType::Container 
                                                        | screeps::StructureType::Nuker 
                                                        | screeps::StructureType::Factory => {
                                                            match c.repair(s) {
                                                                ReturnCode::NotInRange => {
                                                                    travel::set_waypoint(c, &request.target);
                                                                    workers.retain(|&x| *x.name() != *c.name());
                                                                    continue;
                                                                },
                                                                ReturnCode::Ok => { 
                                                                    workers.retain(|&x| *x.name() != *c.name()); 
                                                                    continue; 
                                                                },
                                                                ReturnCode::Full => warn!("Full repair?"),
                                                                _ => warn!("Unhandled error in construction at {:?}", request.target)
                                                            }
                                                        }
                                                        _ => warn!("Invalid repair request at {:?}", request.target),
                                                    }
                                                }
                                                None => warn!("Unknown building type at {:?}", request.target),
                                            }
                                        },
                                        logistics::RequestType::Harvest => {
                                            match s.as_owned() {
                                                Some(s) => {
                                                    let room = s.room().unwrap();
                                                    match s.structure_type() {
                                                        _ => warn!("Invalid harvest request at {:?}", request.target),
                                                    }
                                                }
                                                None => warn!("Unknown building type at {:?}", request.target),
                                            }
                                        },
                                        logistics::RequestType::Upgrade => {
                                            match s.as_owned() {
                                                Some(s) => {
                                                    let room = s.room().unwrap();
                                                    match s.structure_type() {
                                                        screeps::StructureType::Controller => {
                                                            match c.upgrade_controller(&room.controller().unwrap()) {
                                                                ReturnCode::NotInRange => {
                                                                    travel::set_waypoint(c, &request.target);
                                                                    workers.retain(|&x| *x.name() != *c.name());
                                                                    continue;
                                                                },
                                                                ReturnCode::Ok => {
                                                                    workers.retain(|&x| *x.name() != *c.name());
                                                                    continue;
                                                                },
                                                                _ => warn!("Unhandled branch on controller uprgade: {:?}", request.target),
                                                            }        
                                                        },
                                                        _ => warn!("Invalid upgrade request at {:?}", request.target),
                                                    }
                                                }
                                                None => warn!("Unknown building type at {:?}", request.target),
                                            }
                                        },
                                        logistics::RequestType::Build => {
                                            match s.as_owned() {
                                                Some(s) => {
                                                    let room = s.room().unwrap();
                                                    match s.structure_type() {
                                                       _ => warn!("Invalid build request at {:?}", request.target),
                                                    }
                                                }
                                                None => warn!("Unknown building type at {:?}", request.target),
                                            }
                                        },
                                        logistics::RequestType::Pickup => {
                                            debug!("Got pickup request for structure");
                                        },
                                        _ => warn!("Unhandled request type"),
                                    }
                                }
                                None => debug!("No qualified workers for repair at {:?}", request.target),
                            }
                        },

                        // screeps::LookResult::Creep(_) => todo!(),
                        // screeps::LookResult::Energy(_) => todo!(),
                        screeps::LookResult::Resource(s) => {
                            let w = available_workers.iter()
                            .filter(|c| util::count_bodyparts(c, Part::Carry) > 0)
                            .min_by_key(|&c| travel::calc_travel_fatigue(&c, &s.pos()));

                            match w {
                                Some(&c) => match c.pickup(&s) {
                                    ReturnCode::NotInRange => {
                                        travel::set_waypoint(c, &request.target);
                                        workers.retain(|&x| *x.name() != *c.name());
                                        continue;
                                    },
                                    ReturnCode::Ok => { workers.retain(|&x| *x != *c); continue; },
                                    ReturnCode::Full => warn!("Tried to pick up but we're full"),
                                    ReturnCode::NoBodypart => warn!("A unit with no carry tried to pick up resource"),
                                    ReturnCode::Tired => {}
                                    _ => warn!("Unhandled error in pickup at {:?}", request.target)
                                },
                                None => debug!("No qualified workers for pickup at {:?}", request.target),
                            }
                        },
                        // screeps::LookResult::Mineral(_) => todo!(),
                        // screeps::LookResult::Deposit(_) => todo!(),
                        // screeps::LookResult::Tombstone(_) => todo!(),
                        _ => debug!("Got a job request at {:?} but nothing is there", request.target),
                    }
                }
            }
            None => return,
        }
    }
    warn!("Not enough workers found");
}
