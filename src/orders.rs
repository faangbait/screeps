use screeps::{RawObjectId, HasId, ResourceType, StructureType, RoomObjectProperties, LookResult, Attackable, HasStore};
use crate::jobs::{JobType, JobProperties};
use crate::util::RoomCustomActions;


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct JobOrder {
    pub request: JobType,
    pub resource: Option<screeps::ResourceType>,
    pub work_required: u32,
    pub priority: i32,
    pub target: RawObjectId,
}

pub trait WorkRequest {
    fn work_request(self: &Self) -> Option<JobOrder>;
}

impl WorkRequest for screeps::Source {
    fn work_request(self: &Self) -> Option<JobOrder> {
        Some(JobOrder {
            request: JobType::Harvest,
            resource: None,
            work_required: self.energy(),
            priority: -1 * self.ticks_to_regeneration() as i32,
            target: self.untyped_id()
        })
    }
}

impl WorkRequest for screeps::StructureController {
    fn work_request(self: &Self) -> Option<JobOrder> {
        Some(JobOrder {
            request: JobType::Upgrade,
            resource: Some(ResourceType::Energy),
            work_required: self.progress_total().unwrap_or(0) - self.progress().unwrap_or(0),
            priority: -1 * self.ticks_to_downgrade() as i32,
            target: self.untyped_id()
        })
    }
}

impl WorkRequest for screeps::ConstructionSite {
    fn work_request(self: &Self) -> Option<JobOrder> {
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

        let priority = match self.structure_type() {
            StructureType::Road => 1 * road_priority,
            StructureType::Extension => 3,
            StructureType::Observer => 1,
            StructureType::Spawn => -1,
            StructureType::PowerSpawn => 0,
            StructureType::Container => -2,
            StructureType::Storage => -4,
            StructureType::Factory => -5,
            StructureType::Tower => -6,
            StructureType::Link => -7,
            StructureType::Extractor => -8,
            StructureType::Lab => -9,
            StructureType::Terminal => -10,
            StructureType::Wall => -11,
            StructureType::Rampart => -12,
            StructureType::Nuker => -15,
            _ => -20
        };

        Some(JobOrder {
            request: JobType::Build,
            resource: Some(ResourceType::Energy),
            work_required: self.progress_total() - self.progress(),
            priority,
            target: self.untyped_id()
        })
    }
}

impl WorkRequest for screeps::StructureSpawn {
    fn work_request(self: &Self) -> Option<JobOrder> {

        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else if self.energy() < self.store_capacity(Some(ResourceType::Energy)) {
            Some(JobOrder {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy),
                work_required: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                priority: (self.store_capacity(Some(ResourceType::Energy)) - self.energy()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }
}


impl WorkRequest for screeps::StructureExtension {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else if self.energy() < self.store_capacity(Some(ResourceType::Energy)) {
            Some(JobOrder {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy),
                work_required: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                priority: (self.store_capacity(Some(ResourceType::Energy)) - self.energy()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }

}


impl WorkRequest for screeps::StructureContainer {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else if self.energy() > self.store_capacity(Some(ResourceType::Energy)) / 3 {
            Some(JobOrder {
                request: JobType::Withdraw,
                resource: Some(ResourceType::Energy),
                work_required: self.store_free_capacity(Some(ResourceType::Energy)) as u32,
                priority: self.store_used_capacity(Some(ResourceType::Energy)) as i32,
                target: self.untyped_id()
            })
        } else if self.energy() < self.store_capacity(Some(ResourceType::Energy)) {
            Some(JobOrder {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy),
                work_required: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                priority: (self.store_capacity(Some(ResourceType::Energy)) - self.energy()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }
}


impl WorkRequest for screeps::StructureStorage {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else if self.energy() < self.store_capacity(Some(ResourceType::Energy)) {
            Some(JobOrder {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy),
                work_required: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                priority: (self.store_capacity(Some(ResourceType::Energy)) - self.energy()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }
}


impl WorkRequest for screeps::StructureLink {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else if self.energy() < self.store_capacity(Some(ResourceType::Energy)) {
            Some(JobOrder {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy),
                work_required: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                priority: (self.store_capacity(Some(ResourceType::Energy)) - self.energy()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }

}

impl WorkRequest for screeps::StructureTower {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else if self.energy() < self.store_capacity(Some(ResourceType::Energy)) {
            Some(JobOrder {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy),
                work_required: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                priority: (self.store_capacity(Some(ResourceType::Energy)) - self.energy()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }
}

impl WorkRequest for screeps::Resource {
    fn work_request(self: &Self) -> Option<JobOrder> {
        Some(JobOrder {
            request: JobType::Pickup,
            resource: Some(self.resource_type()),
            work_required: self.amount(),
            priority: self.amount() as i32,
            target: self.untyped_id()
        })
    }
}

impl WorkRequest for screeps::StructureRoad {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }
}


impl WorkRequest for screeps::StructureWall {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }
}


impl WorkRequest for screeps::StructureRampart {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else if self.room().unwrap().count_baddies_here() > 0 {
            Some(JobOrder {
                request: JobType::DefendR,
                resource: None,
                work_required: self.room().unwrap().count_baddies_here(),
                priority: 100000,
                target: self.untyped_id()
            })
        } else { None }
    }
}

impl WorkRequest for screeps::StructureObserver {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }
}

impl WorkRequest for screeps::StructurePowerSpawn {
    fn work_request(self: &Self) -> Option<JobOrder> {

        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else if self.energy() < self.store_capacity(Some(ResourceType::Energy)) {
            Some(JobOrder {
                request: JobType::Transfer,
                resource: Some(ResourceType::Energy),
                work_required: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                priority: (self.store_capacity(Some(ResourceType::Energy)) - self.energy()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }
}

impl WorkRequest for screeps::StructureExtractor {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }
}

impl WorkRequest for screeps::StructureLab {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }
}

impl WorkRequest for screeps::StructureTerminal {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }
}

impl WorkRequest for screeps::StructureFactory {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }
}


impl WorkRequest for screeps::StructureNuker {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Repair,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else { None }
    }
}

impl WorkRequest for screeps::Creep {
    fn work_request(self: &Self) -> Option<JobOrder> {
        if self.hits() < self.hits_max() {
            Some(JobOrder {
                request: JobType::Heal,
                resource: Some(ResourceType::Energy),
                work_required: self.hits_max() - self.hits(),
                priority: (self.hits_max() -  self.hits()) as i32,
                target: self.untyped_id()
            })
        } else if self.resource_sink().is_some() {
            Some(JobOrder {
                request: JobType::Transfer,
                resource: self.resource_sink(),
                work_required: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                priority: (self.store_capacity(Some(ResourceType::Energy)) - self.energy()) as i32,
                target: self.untyped_id()
            })            
        } else if self.resource_source().is_some() {
            Some(JobOrder {
                request: JobType::Pickup, // TODO: Will this break?
                resource: self.resource_sink(),
                work_required: self.store_capacity(Some(ResourceType::Energy)) - self.energy(),
                priority: (self.store_capacity(Some(ResourceType::Energy)) - self.energy()) as i32,
                target: self.untyped_id()
            })  
        }
        
        else { None }
    }
}

//     StructureType::PowerBank => todo!(),
//     StructureType::KeeperLair => todo!(),
//     StructureType::Portal => todo!(),
//     StructureType::InvaderCore => todo!(),
