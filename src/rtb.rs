use itertools::{zip, Itertools, izip};
use screeps::{RoomObjectProperties, LookResult, RawObjectId, ResourceType, HasId, Attackable, HasStore, SharedCreepProperties, HasPosition};

use crate::jobs::JobType;
use crate::util::RoomCustomActions;


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct JobBid {
    pub request: JobType,
    pub resource: Option<screeps::ResourceType>,
    pub max: u32, // the max quantity of resources that can be spent here
    pub bid: u32, // the amount the job pays per resource [or tick, if resource is none]; basic repairs are 10
    pub target: RawObjectId
}


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct JobAsk {
    pub request: JobType,
    pub resource: Option<screeps::ResourceType>,
    pub max: u32,
    pub ask: u32,
    pub target: RawObjectId
}

pub trait Sink {
    fn bid(self: &Self) -> u32;
    fn sink_request(self: &Self) -> Option<JobBid>;
}

impl Sink for screeps::Creep {
    fn bid(self: &Self) -> u32 {
        let mem = self.memory().i32("workValue");
        match mem {
            Ok(m) => match m {
                Some(val) => val.checked_sub(1).unwrap_or(0) as u32,
                None => 0, // todo: calculate value
            },
            Err(_) => 0,
        }
    }

    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() {
            Some(JobBid {
                request: JobType::Heal,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 0, // TODO: Body cost
                target: self.untyped_id()
            })
        } else {
            Some(JobBid {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy), // TODO: More resources
                max: self.store_free_capacity(Some(ResourceType::Energy)) as u32,
                bid: self.bid(),
                target: self.untyped_id(),
            })
        }
    }
}
impl Sink for screeps::StructureRoad {
    fn bid(self: &Self) -> u32 { 0 }

    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() - 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: (self.hits_max() - self.hits()) / 100,
                bid: 10,
                target: self.untyped_id()
            })
        } else { None }
    }
}
impl Sink for screeps::StructureWall {
    fn bid(self: &Self) -> u32 { 0 }

    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() - 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10 * 1.max(self.room().unwrap().count_baddies_here()),
                target: self.untyped_id()
            })
        } else { None }
    }
}
impl Sink for screeps::StructureRampart {
    fn bid(self: &Self) -> u32 {
        self.room().unwrap().count_baddies_here() * 100
    }

    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() - 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10 * 1.max(self.room().unwrap().count_baddies_here()),
                target: self.untyped_id()
            })
        } else if self.room().unwrap().count_baddies_here() > 0 {
            if self.pos().find_in_range(screeps::find::MY_CREEPS, 0).len() > 0 {
                // a creep is stationed here
                Some(JobBid {
                    request: JobType::DefendR,
                    resource: None,
                    max: self.room().unwrap().count_baddies_here(),
                    bid: self.bid(),
                    target: self.untyped_id()
                })
            } else {
                // no creep stationed here; let's get someone here
                Some(JobBid {
                    request: JobType::DefendR,
                    resource: None,
                    max: self.room().unwrap().count_baddies_here(),
                    bid: self.bid() * 2,
                    target: self.untyped_id()
                })
            }
        } else { None }
    }
}

impl Sink for screeps::StructureController {
    fn bid(self: &Self) -> u32 {
        // https://www.wolframalpha.com/input?i=plot+.025+*+%28%28.2%28x%5E2%29%29+-+%28log%283.14159%2C+45000%29x%29+-+%283.14159*x%29+%2B+400%29+from+x%3D0+to+100
        // scales from ~10 to ~40

        let progress = match self.progress() {
            Some(p) => match self.progress_total() {
                Some(pt) => 20.max((0.025 * 
                    ((0.2 * (p as f32 / pt as f32).powi(2)) -
                    ((pt as f32).log(3.14159) * pt as f32) -
                    (3.14159 * pt as f32) + 400.0)
                    ) as u32),
                None => 0,
            },
            None => 0,
        };

        return match self.level() {
            0 => 0,
            1 => progress.max( 20000_u32.checked_sub(self.ticks_to_downgrade()).unwrap_or(0)),
            2 => progress.max( 10000_u32.checked_sub(self.ticks_to_downgrade()).unwrap_or(0)),
            3 => progress.max( 20000_u32.checked_sub(self.ticks_to_downgrade()).unwrap_or(0)),
            4 => progress.max( 40000_u32.checked_sub(self.ticks_to_downgrade()).unwrap_or(0)),
            5 => progress.max( 80000_u32.checked_sub(self.ticks_to_downgrade()).unwrap_or(0)),
            6 => progress.max(120000_u32.checked_sub(self.ticks_to_downgrade()).unwrap_or(0)),
            7 => progress.max(150000_u32.checked_sub(self.ticks_to_downgrade()).unwrap_or(0)),
            8 => 20.max(200000 - self.ticks_to_downgrade()),
            _ => 20,
        };
    }

    fn sink_request(self: &Self) -> Option<JobBid> {
            Some(JobBid {
                request: JobType::Upgrade,
                resource: Some(ResourceType::Energy),
                max: self.progress_total().unwrap_or(0) - self.progress().unwrap_or(0),
                bid: self.bid(),
                target: self.untyped_id()
            })
            
            // TODO
            // Some(JobOrder {
            //     request: JobType::Reserve,
            //     resource: None,
            //     work_required: 10,
            //     priority: -1 * self.ticks_to_downgrade() as i32,
            //     target: self.untyped_id()
            // })
            
    }
}

impl Sink for screeps::StructureLink {
    fn bid(self: &Self) -> u32 { 20 }  // TODO

    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() - 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10,
                target: self.untyped_id()
            })
        } else if self.energy() < self.store_capacity(Some(ResourceType::Energy)) {
            Some(JobBid {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy),
                max: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                bid: self.bid(),
                target: self.untyped_id()
            })
        } else { None }
    }
}

impl Sink for screeps::StructureObserver {

    fn bid(self: &Self) -> u32 { 0 }
    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() - 100{
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10,
                target: self.untyped_id()
            })
        } else { None }
    }

}

impl Sink for screeps::StructureLab {

    fn bid(self: &Self) -> u32 { 0 }
    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() - 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10,
                target: self.untyped_id()
            })
        } else { None }
    }
}

impl Sink for screeps::StructureStorage {
    fn bid(self: &Self) -> u32 { 5 } // TODO
    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() - 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10,
                target: self.untyped_id()
            })
        } else if self.energy() < self.store_capacity(Some(ResourceType::Energy)) {
            Some(JobBid {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy),
                max: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                bid: self.bid(),
                target: self.untyped_id()
            })
        } else { None }
    }
}
impl Sink for screeps::StructureTower {
    fn bid(self: &Self) -> u32 { 3 } // TODO
    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() - 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10,
                target: self.untyped_id()
            })
        } else if self.energy() < self.store_capacity(Some(ResourceType::Energy)) {
            Some(JobBid {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy),
                max: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                bid: self.bid(),
                target: self.untyped_id()
            })
        } else { None }
    }
}
impl Sink for screeps::StructurePowerSpawn {
    fn bid(self: &Self) -> u32 { 10 } // TODO
    fn sink_request(self: &Self) -> Option<JobBid> {

        if self.hits() < self.hits_max() - 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10,
                target: self.untyped_id()
            })
        } else if self.energy() < self.store_capacity(Some(ResourceType::Energy)) {
            Some(JobBid {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy),
                max: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                bid: self.bid(),
                target: self.untyped_id()
            })
        } else { None }
    }
}

impl Sink for screeps::StructureSpawn {

    fn bid(self: &Self) -> u32 { 6 } // TODO
    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() - 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10,
                target: self.untyped_id()
            })
        } else if self.energy() < self.store_capacity(Some(ResourceType::Energy)) {
            Some(JobBid {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy),
                max: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                bid: self.bid(),
                target: self.untyped_id()
            })
        } else { None }
    }
}

impl Sink for screeps::StructureExtractor {

    fn bid(self: &Self) -> u32 { 0 }
    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() - 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10,
                target: self.untyped_id()
            })
        } else { None }
    }
}
impl Sink for screeps::StructureExtension {

    fn bid(self: &Self) -> u32 {
        match self.pos().find_closest_by_range(screeps::find::MY_SPAWNS) {
            Some(spawn) => spawn.bid(),
            None => 5,
        }
    }
    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() - 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10,
                target: self.untyped_id()
            })
        } else if self.energy() < self.store_capacity(Some(ResourceType::Energy)) {
            Some(JobBid {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy),
                max: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                bid: self.bid(),
                target: self.untyped_id()
            })
        } else { None }
    }

}
impl Sink for screeps::StructureTerminal {

    fn bid(self: &Self) -> u32 { 0 }
    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() - 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10,
                target: self.untyped_id()
            })
        } else { None }
    }
}
impl Sink for screeps::StructureContainer {
    fn bid(self: &Self) -> u32 { 5 } // TODO

    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() - 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10,
                target: self.untyped_id()
            })
        } else if self.energy() < self.store_capacity(Some(ResourceType::Energy)) {
            Some(JobBid {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy),
                max: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                bid: self.bid(),
                target: self.untyped_id()
            })
        } else { None }
    }

}
impl Sink for screeps::StructureNuker {

    fn bid(self: &Self) -> u32 { 0 }
    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max()- 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10,
                target: self.untyped_id()
            })
        } else { None }
    }
}
impl Sink for screeps::StructureFactory {
    fn bid(self: &Self) -> u32 { 0 }

    fn sink_request(self: &Self) -> Option<JobBid> {
        if self.hits() < self.hits_max() - 100 {
            Some(JobBid {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                max: self.hits_max() - self.hits(),
                bid: 10,
                target: self.untyped_id()
            })
        } else { None }
    }
}
impl Sink for screeps::ConstructionSite {
    fn bid(self: &Self) -> u32 {
        let mult = (self.progress_total() as f32 / (self.progress_total() as f32 / (self.progress() as f32 + 1.0))) / 12.0;
        
        (match self.structure_type() {
            screeps::StructureType::Spawn => mult * 30.,
            screeps::StructureType::Extension => mult * 125.,
            screeps::StructureType::Road => {
                let here = self.room().unwrap().look_at(self);
                let mut terrain = screeps::Terrain::Wall;
                let road_priority;
        
                for obj in here {
                    match obj {
                        LookResult::Terrain(t) => terrain = t,
                        _ => {}
                    }
                }
                
                match terrain {
                    screeps::Terrain::Plain => road_priority = 2,
                    screeps::Terrain::Swamp => road_priority = 10,
                    screeps::Terrain::Wall => road_priority = 0,
                }

                mult * 25. * road_priority as f32
            },
            screeps::StructureType::Wall => mult * 5.,
            screeps::StructureType::Rampart => mult * 6.,
            screeps::StructureType::Link => mult * 30.,
            screeps::StructureType::Storage => mult * 17.,
            screeps::StructureType::Tower => mult * 16.,
            screeps::StructureType::Observer => mult * 18.,
            screeps::StructureType::PowerSpawn => mult * 18.,
            screeps::StructureType::Extractor => mult * 15.,
            screeps::StructureType::Lab => mult * 12.,
            screeps::StructureType::Terminal => mult * 18.,
            screeps::StructureType::Container => mult * 17.,
            screeps::StructureType::Nuker => mult * 6.,
            screeps::StructureType::Factory => mult * 10.,
            _ => 0.,
        }) as u32
    }

    fn sink_request(self: &Self) -> Option<JobBid> {
        Some(JobBid {
            request: JobType::Build,
            resource: Some(ResourceType::Energy),
            max: self.progress_total() - self.progress(),
            bid: self.bid(),
            target: self.untyped_id()
        })
    }

}
// impl Sink for dyn screeps::Transferable {
//     fn bid(self: &Self) -> u32 {
//         todo!()
//     }
// }

pub trait Source {
    fn ask(self: &Self) -> u32;
    fn source_request(self: &Self) -> Option<JobAsk>;

}

impl Source for screeps::Creep {
    fn ask(self: &Self) -> u32 {
        let mem = self.memory().i32("workValue");
        match mem {
            Ok(m) => match m {
                Some(val) => 1 + val as u32,
                None => 0, // todo: calculate value
            },
            Err(_) => 0,
        }
    }
    fn source_request(self: &Self) -> Option<JobAsk> {
        Some(JobAsk {
            request: JobType::Withdraw,
            resource: Some(ResourceType::Energy), // todo
            max: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
            ask: self.ask(),
            target: self.untyped_id()
        })  
    }
}
impl Source for screeps::Source {
    fn ask(self: &Self) -> u32 { 1 } // sources have minimal cost
    fn source_request(self: &Self) -> Option<JobAsk> {
        //TODO "first harvest" to start ticks to regen
        Some(JobAsk {
            request: JobType::Harvest,
            resource: Some(ResourceType::Energy),
            max: self.energy(),
            ask: self.ask(),
            target: self.untyped_id()
        })
    }
}
impl Source for screeps::Deposit {
    fn ask(self: &Self) -> u32 { 1 } // sources have minimal cost

    fn source_request(self: &Self) -> Option<JobAsk> {
        Some(JobAsk {
            request: JobType::Harvest,
            resource: Some(self.deposit_type()),
            max: if self.last_cooldown() > 0 { 400 / self.last_cooldown() } else { 400 },
            ask: self.ask(),
            target: self.untyped_id()
        })
    }
}
impl Source for screeps::Resource {
    fn ask(self: &Self) -> u32 { 1 } // sources have minimal cost

    fn source_request(self: &Self) -> Option<JobAsk> {
        Some(JobAsk {
            request: JobType::Pickup,
            resource: Some(self.resource_type()),
            max: self.amount(),
            ask: self.ask(),
            target: self.untyped_id()
        })
    }

}
impl Source for screeps::Mineral {
    fn ask(self: &Self) -> u32 { 1 } // sources have minimal cost

    fn source_request(self: &Self) -> Option<JobAsk> {
        Some(JobAsk {
            request: JobType::Harvest,
            resource: Some(self.mineral_type()),
            max: self.mineral_amount(),
            ask: self.ask(),
            target: self.untyped_id()
        })
    }
}
impl Source for screeps::Tombstone {
    fn ask(self: &Self) -> u32 { 1 } // sources have minimal cost
    fn source_request(self: &Self) -> Option<JobAsk> {

        let biggest = self.store_types().iter()
        .map(|&rt| (rt, self.store_used_capacity(Some(rt))))
        .max_by_key(|(_rt,quant)| *quant);

        if biggest.is_some() {
            Some(JobAsk {
                request: JobType::Pickup,
                resource: Some(biggest.unwrap().0),
                max: biggest.unwrap().1,
                ask: self.ask(),
                target: self.untyped_id()
            })            
        } else { None }
    }
}

// impl Source for dyn screeps::Withdrawable {
//     fn ask(self: &Self) -> u32 { 1 } // sources have minimal cost

//     fn source_request(self: &Self) -> Option<JobAsk> {
        
//     }
// }
impl Source for screeps::StructureSpawn {
    fn ask(self: &Self) -> u32 { self.bid() + 5 }

    fn source_request(self: &Self) -> Option<JobAsk> {
        Some(JobAsk {
            request: JobType::Withdraw,
            resource: Some(ResourceType::Energy),
            max: self.energy(),
            ask: self.ask(),
            target: self.untyped_id()
        })
    }
}

impl Source for screeps::StructureExtension {
    fn ask(self: &Self) -> u32 { self.bid() + 5 }

    fn source_request(self: &Self) -> Option<JobAsk> {
        Some(JobAsk {
            request: JobType::Withdraw,
            resource: Some(ResourceType::Energy),
            max: self.energy(),
            ask: self.ask(),
            target: self.untyped_id()
        })
    }
}
impl Source for screeps::StructureLink {
    fn ask(self: &Self) -> u32 { self.bid() + 5 }

    fn source_request(self: &Self) -> Option<JobAsk> {
        Some(JobAsk {
            request: JobType::Withdraw,
            resource: Some(ResourceType::Energy),
            max: self.energy(),
            ask: self.ask(),
            target: self.untyped_id()
        })
    }
}

impl Source for screeps::StructureStorage {
    fn ask(self: &Self) -> u32 { self.bid() + 1 }

    fn source_request(self: &Self) -> Option<JobAsk> {
        Some(JobAsk {
            request: JobType::Withdraw,
            resource: Some(ResourceType::Energy),
            max: self.energy(),
            ask: self.ask(),
            target: self.untyped_id()
        })
    }
}
impl Source for screeps::StructureTower {
    fn ask(self: &Self) -> u32 { self.bid() + 1 }

    fn source_request(self: &Self) -> Option<JobAsk> {
        Some(JobAsk {
            request: JobType::Withdraw,
            resource: Some(ResourceType::Energy),
            max: self.energy(),
            ask: self.ask(),
            target: self.untyped_id()
        })
    }
}
impl Source for screeps::StructurePowerSpawn {
    fn ask(self: &Self) -> u32 { self.bid() + 5 }

    fn source_request(self: &Self) -> Option<JobAsk> {
        Some(JobAsk {
            request: JobType::Withdraw,
            resource: Some(ResourceType::Energy),
            max: self.energy(),
            ask: self.ask(),
            target: self.untyped_id()
        })
    }
}
impl Source for screeps::StructureTerminal {
    fn ask(self: &Self) -> u32 { self.bid() + 1 }

    fn source_request(self: &Self) -> Option<JobAsk> {
        Some(JobAsk {
            request: JobType::Withdraw,
            resource: Some(ResourceType::Energy),
            max: self.energy(),
            ask: self.ask(),
            target: self.untyped_id()
        })
    }
}
impl Source for screeps::StructureContainer {
    fn ask(self: &Self) -> u32 { self.bid() + 1 }

    fn source_request(self: &Self) -> Option<JobAsk> {
        if self.energy() > self.store_capacity(Some(ResourceType::Energy)) / 3 {
            Some(JobAsk {
                request: JobType::Withdraw,
                resource: Some(ResourceType::Energy),
                max: self.store_free_capacity(Some(ResourceType::Energy)) as u32,
                ask: self.ask(),
                target: self.untyped_id()
            })
        } else { None }
    }
}
impl Source for screeps::StructureNuker {
    fn ask(self: &Self) -> u32 { self.bid() + 5 }

    fn source_request(self: &Self) -> Option<JobAsk> {
        Some(JobAsk {
            request: JobType::Withdraw,
            resource: Some(ResourceType::Energy),
            max: self.energy(),
            ask: self.ask(),
            target: self.untyped_id()
        })
    }
}

