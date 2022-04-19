use priority_queue::PriorityQueue;
use std::any::Any;
use std::{collections::VecDeque, hash::Hash, marker::PhantomData};

use log::{info, warn};
use screeps::{HasPosition, Part, Creep, Position, find, HasStore, ResourceType, StructureContainer, Room, Source, StructureExtension, StructureTower, StructureStorage, Structure, ReturnCode, Attackable, RoomObjectProperties, game::market::calc_transaction_cost, SizedRoomObject, HasId, ObjectId, HasCooldown, RawObjectId, StructureController, ConstructionSite, IntoExpectedType, ConversionError};
use screeps::prelude::*;
use crate::memory::get_dest;
use crate::{filters, memory};


#[derive(Eq,PartialEq,Hash)]
pub struct Job {
    id: RawObjectId,
    fun: String
}
pub struct Assignment { 
    pub assigned: bool,
}

impl Assignment {
    pub fn new(assigned: bool) -> Self { Self { assigned } }
}



impl Job {
    pub fn new(id: RawObjectId, fun: String) -> Self { Self { id, fun } }

    pub fn creep(&self) -> Result<Option<Creep>, ConversionError> { screeps::game::get_object_typed::<Creep>(self.id.into()) }
    pub fn source(&self) -> Result<Option<Source>, ConversionError> { screeps::game::get_object_typed::<Source>(self.id.into()) }
    pub fn container(&self) -> Result<Option<StructureContainer>, ConversionError> { screeps::game::get_object_typed::<StructureContainer>(self.id.into()) }
    pub fn controller(&self) -> Result<Option<StructureController>, ConversionError> { screeps::game::get_object_typed::<StructureController>(self.id.into()) }
    pub fn structure(&self) -> Result<Option<Structure>, ConversionError> { screeps::game::get_object_typed::<Structure>(self.id.into()) }
    pub fn constructionsite(&self) -> Result<Option<ConstructionSite>, ConversionError> { screeps::game::get_object_typed::<ConstructionSite>(self.id.into()) }
    pub fn call(&self, actor: &Creep, target: RawObjectId) {
        actor.memory().set("busy", true);
        if self.fun == "harvest" { 
            let t = &screeps::game::get_object_typed::<Source>(target.into()).unwrap().unwrap();
            match actor.harvest(t) {
                ReturnCode::Ok => {},
                ReturnCode::NotOwner => todo!(),
                ReturnCode::NoPath => todo!(),
                ReturnCode::NameExists => todo!(),
                ReturnCode::Busy => todo!(),
                ReturnCode::NotFound => todo!(),
                ReturnCode::NotEnough => todo!(),
                ReturnCode::InvalidTarget => todo!(),
                ReturnCode::Full => todo!(),
                ReturnCode::NotInRange => {actor.move_to(t);},
                ReturnCode::InvalidArgs => todo!(),
                ReturnCode::Tired => todo!(),
                ReturnCode::NoBodypart => todo!(),
                ReturnCode::RclNotEnough => todo!(),
                ReturnCode::GclNotEnough => todo!(),
            }
        }
    }
}


pub fn scheduler() {
    let mut jobs = PriorityQueue::new();
    let sources = filters::get_my_sources();
    let rooms = filters::get_my_rooms();
    // let hostile_rooms = filters::get_hostile_rooms();
    let creeps = screeps::game::creeps::values();
    let containers = filters::get_my_containers();
    let extensions = filters::get_my_extensions();
    let towers = filters::get_my_towers();
    let storage = filters::get_my_storage();
    let repairables = filters::get_my_repairables();
    let buildables = filters::get_my_buildables();
    let controllers = filters::get_my_controllers();
    
    // heal
    // push a job when a creep needs healing
    for creep in creeps.iter().filter(|&c| c.hits_max() > c.hits()).collect::<Vec<&Creep>>() {
        let job = Job::new(creep.untyped_id(), "rangedHeal".to_string());
        // let job = Job::new(Target::Creep(creep.id()), creep.untyped_id(), (|| "rangedHeal")());
        let prio = 100 - (creep.hits_max() - creep.hits()) as i32;
        jobs.push(job, prio);
        
    }

    // defense
    // push a job when there are hostiles in my rooms
    // for room in rooms.iter().filter(|&r| r.find(find::HOSTILE_CREEPS).len() > 0).collect::<Vec<&Room>>() {
    //     jobs.push_back(Job {target: Box::new(*room)});
    // };

    // offense
    // push a job when there are hostiles in a target room
    // for room in hostile_rooms.iter().filter(|&r| r.find(find::HOSTILE_SPAWNS).len() > 0).collect::<Vec<&Room>>() {
    //     jobs.push_back(Job {target: Box::new(*room)});
    // };

    // haul creeps
    // push a job for any creep without move parts who has a destination
    for creep in creeps.iter().filter(|&c| !c.body().iter().any(|bp| bp.part == Part::Move) && memory::get_dest(&c).is_some()).collect::<Vec<&Creep>>() {
        let job = Job::new(creep.untyped_id(),"pull".to_string());
        // let job = Job::new(Target::Creep(creep.id()), creep.untyped_id(), (|| "pull")());
        let prio = 200 - creep.body().len() as i32;
        jobs.push(job, prio);
    }

    // drop-harvesting
    // push a job if any source is without a miner
    for source in sources.iter().filter(|&s| !creeps.iter().any(|c| c.pos().is_near_to(s))).collect::<Vec<&Source>>() {
        let mut ctrls = filters::get_my_controllers();
        ctrls.sort_unstable_by_key(|c| c.pos().get_range_to(source));
        let controller =  ctrls.first().unwrap();
        let job = Job::new(source.untyped_id(),"harvest".to_string());
        // let job = Job::new(Target::Source(source.id()), source.untyped_id(), (|| "harvest")());
        let prio = 300 - source.pos().get_range_to(controller) as i32;
        jobs.push(job, prio);
        
    }
    
    // centralize resources
    // push a job if any harvest containers are more than half full
    for container in containers.iter().filter(|&c| sources.iter().any(|s| c.pos().is_near_to(s)) && (c.store_used_capacity(Some(ResourceType::Energy)) > c.store_capacity(Some(ResourceType::Energy)) / 2)).collect::<Vec<&StructureContainer>>() {
        let mut ctrls = filters::get_my_controllers();
        ctrls.sort_unstable_by_key(|c| c.pos().get_range_to(container));
        let controller =  ctrls.first().unwrap();

        let job = Job::new(container.untyped_id(),"withdraw".to_string());
        // let job = Job::new(Target::Container(container.id()), container.untyped_id(), (|| "withdraw")());
        let prio = 500 - container.pos().get_range_to(controller) as i32;
        jobs.push(job, prio);

    }

    // repair anything that needs repair
    for repairable in repairables.iter().filter(|&r| r.as_can_decay().is_some()).filter(|r| r.as_attackable().map(|re| re.hits_max() > re.hits() + 50).unwrap_or(true)).collect::<Vec<&Structure>>() {
        let st = repairable.as_attackable();
        let mut damage = 0;

        if st.is_some() {
            let st = st.unwrap();
            damage = 100.min((st.hits_max() - st.hits()) / 10);
        }
        
        let job = Job::new(repairable.untyped_id(),"repair".to_string());
        // let job = Job::new(Target::Repairable(repairable.id()), repairable.untyped_id(), (|| "repair")());
        let prio = 600 - damage as i32;
        jobs.push(job, prio);
    }

    // build anything that needs building
    for buildable in buildables.iter() {
        let job = Job::new(buildable.untyped_id(),"build".to_string());
        // let job = Job::new(Target::Buildable(buildable.id()), buildable.untyped_id(), (|| "build")());
        let prio = 900 - (buildable.progress() / buildable.progress_total()) as i32;
        jobs.push(job, prio);
    }

    // upgrade controller
    for controller in controllers.iter() {
        let job = Job::new(controller.untyped_id(),"upgrade_controller".to_string());
        // let job = Job::new(Target::Controller(controller.id()), controller.untyped_id(), (|| "controller")());
        let prio = 1000;
        jobs.push(job, prio);
    }
    
    // // do something with held resources
    // for creep in creeps.iter().filter(|&c| c.store_used_capacity(Some(ResourceType::Energy)) > 0).collect::<Vec<&Creep>>() {
    //     // this creep is a hauler
    //     if creep.body().iter().all(|bp| bp.part != Part::Work) {
    //         let empty_extensions = extensions.iter().filter(|&e| e.store_free_capacity(Some(ResourceType::Energy)) > 0).collect::<Vec<&StructureExtension>>();
    //         let empty_towers = towers.iter().filter(|&t| t.store_free_capacity(Some(ResourceType::Energy)) > 0).collect::<Vec<&StructureTower>>();
    //         let empty_storage = storage.iter().filter(|&s| s.store_free_capacity(Some(ResourceType::Energy)) > 0).collect::<Vec<&StructureStorage>>();
    //         let empty_containers = containers.iter().filter(|&c| c.store_free_capacity(Some(ResourceType::Energy)) > 0 && sources.iter().all(|s| !c.pos().is_near_to(s))).collect::<Vec<&StructureContainer>>();
    //     } else {
    //         // creep can do work
    //         for repairable in &repairables {
    //             jobs.push_back(Job {});
    //         }

    //         for buildable in &buildables {
    //             jobs.push_back(Job {});
    //         }

    //         for controller in &controllers {
    //             jobs.push_back(Job {});
    //         }
    //     }

    //     jobs.push_back(Job {});
    // }
    /* clearing memory */
    for creep in &creeps {
        creep.memory().set("busy", false);
    }
    /* prioritizing creeps */
    loop {
        let j = jobs.pop();
        let mut costs: Vec<(&Creep, u32)> = vec![];
        if let Some(ref j) = j {
            for creep in &creeps {
                if creep.memory().bool("busy") { continue; }
                let job = &j.0;
                info!("Considering job: {}", job.fun);
                match job.creep() {
                    Ok(target) => match target {
                        Some(t) => {
                            costs.push((creep, calc_travel_fatigue(creep, &t.pos())));
                        },
                        None => warn!("Target not found.")
                    },
                    Err(_) => match job.source() {
                        Ok(target) => match target {
                            Some(t) => {
                                costs.push((creep, calc_travel_fatigue(creep, &t.pos())));
                            },
                            None => warn!("Target not found.")
                        },
                        Err(_) => match job.structure() {
                            Ok(target) => match target {
                                Some(t) => {
                                    costs.push((creep, calc_travel_fatigue(creep, &t.pos())));
                                },
                                None => warn!("Target not found.")
                            },
                            Err(_) => match job.constructionsite() {
                                Ok(target) => match target {
                                    Some(t) => {
                                        costs.push((creep, calc_travel_fatigue(creep, &t.pos())));
                                    },
                                    None => warn!("Target not found.")
                                },
                                Err(_) => match job.container() {
                                    Ok(target) => match target {
                                        Some(t) => {
                                            costs.push((creep, calc_travel_fatigue(creep, &t.pos())));
                                        },
                                        None => warn!("Target not found.")
                                    },
                                    Err(_) => match job.controller() {
                                        Ok(target) => match target {
                                            Some(t) => {
                                                costs.push((creep, calc_travel_fatigue(creep, &t.pos())));
                                            },
                                            None => warn!("Target not found.")
                                        },
                                        Err(_) => warn!("Couldn't find any matching type for job"),
                                    },
                                },
                            },
                        },
                    },
                }
            } 
        } else { break; }
        if costs.len() > 0 { 
            costs.sort_unstable_by_key(|(_, val)| *val);
        } else { continue; }

        let actor = costs.first().unwrap().0;
        let job = j.unwrap().0;
        info!("Selected job {:?} for {:?}", job.fun, actor.name());
        match job.creep() {
            Ok(target) => match target {
                Some(t) => {
                    job.call(actor, t.untyped_id());
                },
                None => warn!("Target not found.")
            },
            Err(_) => match job.source() {
                Ok(target) => match target {
                    Some(t) => {
                        job.call(actor, t.untyped_id());
                    },
                    None => warn!("Target not found.")
                },
                Err(_) => match job.structure() {
                    Ok(target) => match target {
                        Some(t) => {
                            job.call(actor, t.untyped_id());
                        },
                        None => warn!("Target not found.")
                    },
                    Err(_) => match job.constructionsite() {
                        Ok(target) => match target {
                            Some(t) => {
                            },
                            None => warn!("Target not found.")
                        },
                        Err(_) => match job.container() {
                            Ok(target) => match target {
                                Some(t) => {
                                },
                                None => warn!("Target not found.")
                            },
                            Err(_) => match job.controller() {
                                Ok(target) => match target {
                                    Some(t) => {
                                    },
                                    None => warn!("Target not found.")
                                },
                                Err(_) => warn!("Couldn't find any matching type for job"),
                            },
                        },
                    },
                },
            },
        }
        continue;


    }
}

