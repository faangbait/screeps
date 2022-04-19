
use std::ops::Range;

use log::{info, warn};
use screeps::{HasPosition, HasStore, ResourceType};

use crate::{filters, travel::{self, calc_travel_fatigue}};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct WorkerLogistic {
    pub id: u64,
    pub request: RequestType,
    pub resource_type: screeps::ResourceType,
    pub resource_min: u32,
    pub resource_max: u32,
    pub work_parts: i32,
    pub work_type: screeps::Part,
    pub priority: i32,
    pub target: screeps::Position,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RequestType {
    Deposit,
    Repair,
    Harvest,
    Upgrade,
    Build,
    Pickup,
}
pub trait WorkerRequest {
    fn request_workers(self: &Self) -> Option<WorkerLogistic>;
}

impl WorkerRequest for screeps::Source {
    fn request_workers(self: &Self) -> Option<WorkerLogistic> {
        let creeps = screeps::game::creeps::values();
        // let work_parts = creeps.iter()
        //     .filter(|c| self.pos().is_near_to(*c))
        //     .fold(0, |acc,cur| acc + cur.body().iter()
        //         .fold(0, |acc,cur| match cur.part {
        //             screeps::Part::Work => acc+1,
        //             screeps::Part::Carry => acc-1,
        //             _ => acc
        //         })
        //     );

        Some(
            WorkerLogistic {
                id: 1, 
                request: RequestType::Harvest,
                resource_type: ResourceType::Energy, 
                resource_min: 0,
                resource_max: 1,
                work_parts: 2, 
                work_type: screeps::Part::Work,
                priority: -8000, 
                target: self.pos()
            }
        )

    }
}

impl WorkerRequest for screeps::StructureController {
    fn request_workers(self: &Self) -> Option<WorkerLogistic> {
        let creeps = screeps::game::creeps::values();
        let work_parts = creeps.iter()
            .filter(|c| self.pos().in_range_to(*c, 2))
            .fold(0, |acc,cur| acc + cur.body().iter()
                .fold(0, |acc,cur| match cur.part {
                    screeps::Part::Work => acc+1,
                    screeps::Part::Carry => acc+1,
                    _ => acc
                })
            );

        if work_parts < 12 {
            Some(
                WorkerLogistic {
                    id: 1, 
                    request: RequestType::Upgrade,
                    resource_type: ResourceType::Energy, 
                    resource_min: 49,
                    resource_max: u32::MAX,
                    work_parts, 
                    work_type: screeps::Part::Work,
                    priority: 205, 
                    target: self.pos()
                }
            )
        } else { None }
    }
}

impl WorkerRequest for screeps::ConstructionSite {
    fn request_workers(self: &Self) -> Option<WorkerLogistic> {
        let creeps = screeps::game::creeps::values();
        // let work_parts = creeps.iter()
        //     .filter(|c| self.pos().in_range_to(*c, 2))
        //     .fold(0, |acc,cur| acc + cur.body().iter()
        //         .fold(0, |acc,cur| match cur.part {
        //             screeps::Part::Work => acc+1,
        //             screeps::Part::Carry => acc+1,
        //             _ => acc
        //         })
        //     );

        let deficit = self.progress_total() - self.progress();

        let priority = match self.structure_type() {
            screeps::StructureType::Extension => 0,
            screeps::StructureType::Road => 50,
            screeps::StructureType::Wall => 100,
            screeps::StructureType::Rampart => 200,
            // screeps::StructureType::Link => todo!(),
            // screeps::StructureType::Storage => todo!(),
            // screeps::StructureType::Tower => todo!(),
            // screeps::StructureType::Observer => todo!(),
            // screeps::StructureType::PowerBank => todo!(),
            // screeps::StructureType::PowerSpawn => todo!(),
            // screeps::StructureType::Extractor => todo!(),
            // screeps::StructureType::Lab => todo!(),
            // screeps::StructureType::Terminal => todo!(),
            // screeps::StructureType::Container => todo!(),
            // screeps::StructureType::Nuker => todo!(),
            // screeps::StructureType::Factory => todo!(),
            // screeps::StructureType::InvaderCore => todo!(),
            _ => 50,
        };

        Some(
            WorkerLogistic {
                id: 1, 
                request: RequestType::Build,
                resource_type: ResourceType::Energy, 
                resource_min: 1,
                resource_max: u32::MAX,
                work_parts: 1, 
                work_type: screeps::Part::Work,
                priority, 
                target: self.pos()
            }
        )

    }
}

impl WorkerRequest for screeps::StructureSpawn {
    fn request_workers(self: &Self) -> Option<WorkerLogistic> {
        let creeps = screeps::game::creeps::values();
        let deficit = self.store_free_capacity(Some(ResourceType::Energy));
        let haul_parts = deficit / 50;

        if deficit == 0 { None } else {
            Some(
                WorkerLogistic {
                    id: 1, 
                    request: RequestType::Deposit,
                    resource_type: ResourceType::Energy, 
                    resource_min: 50,
                    resource_max: u32::MAX,
                    work_parts: haul_parts, 
                    work_type: screeps::Part::Carry,
                    priority: -deficit * 10, 
                    target: self.pos()
                }
            )
        }
    }
}

impl WorkerRequest for screeps::StructureExtension {
    fn request_workers(self: &Self) -> Option<WorkerLogistic> {
        let creeps = screeps::game::creeps::values();
        let deficit = self.store_free_capacity(Some(ResourceType::Energy));
        if deficit == 0 { None } else {
            Some(
                WorkerLogistic {
                    id: 1, 
                    request: RequestType::Deposit,
                    resource_type: ResourceType::Energy, 
                    resource_min: 50,
                    resource_max: u32::MAX,
                    work_parts: 1, 
                    work_type: screeps::Part::Carry,
                    priority: -deficit * 100, 
                    target: self.pos()
                }
            )
        }
    }
}

impl WorkerRequest for screeps::StructureContainer {
    fn request_workers(self: &Self) -> Option<WorkerLogistic> {
        let creeps = screeps::game::creeps::values();
        let deficit = self.store_free_capacity(Some(ResourceType::Energy));
        let haul_parts = deficit / 50;

        if deficit < 200 { None } else { 
            Some(
                WorkerLogistic {
                    id: 1, 
                    request: RequestType::Deposit,
                    resource_type: ResourceType::Energy, 
                    resource_min: 200,
                    resource_max: deficit as u32,
                    work_parts: haul_parts, 
                    work_type: screeps::Part::Carry,
                    priority: -deficit, 
                    target: self.pos()
                }
            )
        }
    }
}

impl WorkerRequest for screeps::StructureStorage {
    fn request_workers(self: &Self) -> Option<WorkerLogistic> {
        let creeps = screeps::game::creeps::values();
        let deficit = self.store_free_capacity(Some(ResourceType::Energy));
        let haul_parts = deficit / 50;

        if haul_parts == 0 { None }
        else if haul_parts <= 5 {
            Some(
                WorkerLogistic {
                    id: 1, 
                    request: RequestType::Deposit,
                    resource_type: ResourceType::Energy, 
                    resource_min: deficit as u32,
                    resource_max: u32::MAX,
                    work_parts: haul_parts, 
                    work_type: screeps::Part::Carry,
                    priority: -deficit, 
                    target: self.pos()
                }
            )
        } else { 
            Some(
                WorkerLogistic {
                    id: 1, 
                    request: RequestType::Deposit,
                    resource_type: ResourceType::Energy, 
                    resource_min: 200,
                    resource_max: deficit as u32,
                    work_parts: haul_parts, 
                    work_type: screeps::Part::Carry,
                    priority: -deficit / 100, 
                    target: self.pos()
                }
            )
        }
    }
}


impl WorkerRequest for screeps::StructureLink {
    fn request_workers(self: &Self) -> Option<WorkerLogistic> {
        let creeps = screeps::game::creeps::values();
        let deficit = self.store_free_capacity(Some(ResourceType::Energy));
        let haul_parts = deficit / 50;

        if haul_parts == 0 { None }
        else if haul_parts <= 5 {
            Some(
                WorkerLogistic {
                    id: 1, 
                    request: RequestType::Deposit,
                    resource_type: ResourceType::Energy, 
                    resource_min: deficit as u32,
                    resource_max: u32::MAX,
                    work_parts: haul_parts, 
                    work_type: screeps::Part::Carry,
                    priority: -deficit, 
                    target: self.pos()
                }
            )
        } else { 
            Some(
                WorkerLogistic {
                    id: 1, 
                    request: RequestType::Deposit,
                    resource_type: ResourceType::Energy, 
                    resource_min: 200,
                    resource_max: deficit as u32,
                    work_parts: haul_parts, 
                    work_type: screeps::Part::Carry,
                    priority: -deficit / 100, 
                    target: self.pos()
                }
            )
        }
    }
}


impl WorkerRequest for screeps::StructureTower {
    fn request_workers(self: &Self) -> Option<WorkerLogistic> {
        let creeps = screeps::game::creeps::values();
        let deficit = self.store_free_capacity(Some(ResourceType::Energy));
        let haul_parts = deficit / 50;

        if haul_parts == 0 { None }
        else if haul_parts <= 5 {
            Some(
                WorkerLogistic {
                    id: 1, 
                    request: RequestType::Deposit,
                    resource_type: ResourceType::Energy, 
                    resource_min: deficit as u32,
                    resource_max: u32::MAX,
                    work_parts: haul_parts, 
                    work_type: screeps::Part::Carry,
                    priority: -deficit, 
                    target: self.pos()
                }
            )
        } else { 
            Some(
                WorkerLogistic {
                    id: 1, 
                    request: RequestType::Deposit,
                    resource_type: ResourceType::Energy, 
                    resource_min: 200,
                    resource_max: deficit as u32,
                    work_parts: haul_parts, 
                    work_type: screeps::Part::Carry,
                    priority: -deficit / 100, 
                    target: self.pos()
                }
            )
        }
    }
}


impl WorkerRequest for screeps::Resource {
    fn request_workers(self: &Self) -> Option<WorkerLogistic> {
        let creeps = screeps::game::creeps::values();
        let size = self.amount();
        let resource_type = self.resource_type();
        let haul_parts = size / 50;


        Some(
            WorkerLogistic {
                id: 1, 
                request: RequestType::Pickup,
                resource_type, 
                resource_min: 0,
                resource_max: 5,
                work_parts: haul_parts as i32, 
                work_type: screeps::Part::Carry,
                priority: size as i32 * -1, 
                target: self.pos()
            }
        )
    
    }
}


/// A request for resources in or out of a container
/// 
/// Args:
/// - id: a unique identifier used for matching/hashing
/// - resource_type: The type of resource to transport
/// - amount: the current quantity of resources that need to be moved; positive for deposit
/// - priority: a minheap to prioritize requests
/// - target: an enum of potential targets
pub struct ResourceLogistic {
    pub id: u64,
    pub resource_type: screeps::ResourceType,
    pub amount: i32,
    pub priority: i32,
    pub target: screeps::Structure,
}

pub trait ResourceRequest {
    fn request_resources(self: &Self, resource_type: screeps::ResourceType, amount: u32) -> Option<ResourceLogistic>;
    fn provide_resources(self: &Self, resource_type: screeps::ResourceType, amount: u32) -> Option<ResourceLogistic>;
    fn groundscore(self: &Self, resource_type: screeps::ResourceType, amount: u32) -> Option<screeps::Resource>;
}

impl ResourceRequest for screeps::Creep {
    fn request_resources(self: &Self, resource_type: screeps::ResourceType, amount: u32) -> Option<ResourceLogistic> {
        match get_nearest_logi_store(self, resource_type, true) {
            Some(t) => Some(ResourceLogistic {
                id: 1,
                resource_type,
                amount: amount as i32,
                priority: 1,
                target: t,
            }),
            None => None,
        }
    }

    fn provide_resources(self: &Self, resource_type: screeps::ResourceType, amount: u32) -> Option<ResourceLogistic> {
        match get_nearest_logi_store(self, resource_type, false) {
            Some(t) => Some(ResourceLogistic {
                id: 1,
                resource_type,
                amount: amount as i32,
                priority: 1,
                target: t,
            }),
            None => None,
        }
       
    }

    fn groundscore(self: &Self, resource_type: screeps::ResourceType, amount: u32) -> Option<screeps::Resource> {
        get_nearest_groundscore(self, resource_type, amount)
    }
}

pub fn get_nearest_groundscore(creep: &screeps::Creep, resource_type: screeps::ResourceType, amount: u32) -> Option<screeps::Resource> {
    // gs = gs.iter().filter(|s| s.amount() >= creep.store_free_capacity(Some(resource_type)));
    match filters::get_groundscores().iter()
    .filter(|s| s.amount() >= amount)
    .min_by_key(|&c| travel::calc_travel_fatigue(creep, &screeps::HasPosition::pos(c))) {
        Some(first) => Some(first.to_owned()),
        None => {info!("No groundscores"); None},
    }
}

pub fn get_nearest_logi_store(creep: &screeps::Creep, resource_type: screeps::ResourceType, withdraw: bool) -> Option<screeps::Structure> {
    let mut storages = vec![];

    if withdraw {
        filters::get_my_containers().iter()
            .filter(|s| filters::get_my_sources().iter().any(|s| s.pos().is_near_to(s)) &&  // container is near source
                (s.store_used_capacity(Some(resource_type)) > s.store_capacity(Some(resource_type)) / 2)) // container more than half full
            // .for_each(|s| storages.push(screeps::Structure::Container(s.to_owned())));
            .for_each(|s| storages.push(screeps::Structure::Container(s.to_owned())));

        filters::get_my_storages().iter()
            .filter(|s| s.store_used_capacity(Some(resource_type)) as i32 >= creep.store_free_capacity(Some(resource_type)) as i32) 
            // .for_each(|&s| storages.push(screeps::Structure::Storage(s.to_owned())));
            .for_each(|s| storages.push(screeps::Structure::Storage(s.to_owned())));

    } else {

        filters::get_my_storages().iter()
            .filter(|s| s.store_free_capacity(Some(resource_type)) >= creep.store_used_capacity(Some(resource_type)) as i32)
            .for_each(|s| storages.push(screeps::Structure::Storage(s.to_owned())));
        filters::get_my_containers().iter()
            // .filter(|s| s.store_free_capacity(Some(resource_type)) >= creep.store_used_capacity(Some(resource_type)) as i32)
            .filter(|s| filters::get_my_sources().iter().all(|s| !s.pos().is_near_to(s)) &&  // container is near source
                s.store_free_capacity(Some(resource_type)) >= creep.store_used_capacity(Some(resource_type)) as i32) // container more than half full
            .for_each(|s| storages.push(screeps::Structure::Container(s.to_owned())));
        filters::get_my_extensions().iter()
            .filter(|s| s.store_free_capacity(Some(resource_type)) > 0)
            .for_each(|s| storages.push(screeps::Structure::Extension(s.to_owned())));
        filters::get_my_towers().iter()
            .filter(|s| s.store_free_capacity(Some(resource_type)) >= creep.store_used_capacity(Some(resource_type)) as i32)
            .filter(|s| s.store_used_capacity(Some(resource_type)) > s.store_capacity(Some(resource_type)) / 2)
            .for_each(|s| storages.push(screeps::Structure::Tower(s.to_owned())));
        filters::get_my_spawns().iter()
            .filter(|s| s.store_free_capacity(Some(resource_type)) > 50)
            .for_each(|s| storages.push(screeps::Structure::Spawn(s.to_owned())));
        filters::get_my_factories().iter()
            .filter(|s| s.store_free_capacity(Some(resource_type)) >= creep.store_used_capacity(Some(resource_type)) as i32)
            .for_each(|s| storages.push(screeps::Structure::Factory(s.to_owned())));
        filters::get_my_labs().iter()
            .filter(|s| s.store_free_capacity(Some(resource_type)) >= creep.store_used_capacity(Some(resource_type)) as i32)
            .for_each(|s| storages.push(screeps::Structure::Lab(s.to_owned())));
        filters::get_my_links().iter()
            .filter(|s| s.store_free_capacity(Some(resource_type)) >= creep.store_used_capacity(Some(resource_type)) as i32)
            .for_each(|s| storages.push(screeps::Structure::Link(s.to_owned())));
        filters::get_my_nukers().iter()
            .filter(|s| s.store_free_capacity(Some(resource_type)) >= creep.store_used_capacity(Some(resource_type)) as i32)
            .for_each(|s| storages.push(screeps::Structure::Nuker(s.to_owned())));
        filters::get_my_powerspawns().iter()
            .filter(|s| s.store_free_capacity(Some(resource_type)) >= creep.store_used_capacity(Some(resource_type)) as i32)
            .for_each(|s| storages.push(screeps::Structure::PowerSpawn(s.to_owned())));
        filters::get_my_terminals().iter()
            .filter(|s| s.store_free_capacity(Some(resource_type)) >= creep.store_used_capacity(Some(resource_type)) as i32)
            .for_each(|s| storages.push(screeps::Structure::Terminal(s.to_owned())));
    }


    match storages.iter().min_by_key(|&st| travel::calc_travel_fatigue(creep, &st.pos())) {
        Some(t) => { 
            info!("{} is providing to {:?}", screeps::SharedCreepProperties::name(creep), t.pos());
            Some(t.to_owned())
        },
        None => None,
    }
}

pub struct WorkLogistic {
    pub id: u64,
    pub resource_type: screeps::ResourceType,
    // pub amount: i32,
    pub priority: i32,
    pub target: screeps::Structure,
}

pub trait WorkRequest {
    fn request_work(self: &Self, resource_type: screeps::ResourceType);
}

impl WorkRequest for screeps::Creep {
    fn request_work(self: &Self, resource_type: screeps::ResourceType) {
        match get_nearest_logi_repair(self, resource_type) {
            Some(r) => match self.repair(&r) {
                screeps::ReturnCode::Ok => {},
                screeps::ReturnCode::NotInRange => {
                    screeps::SharedCreepProperties::move_to(self,&r);
                },
                _ => return
            },
            None => match get_nearest_logi_build(self, resource_type) {
                Some(r) => {
                    match self.build(&r) {
                        screeps::ReturnCode::Ok => {},
                        screeps::ReturnCode::NotInRange => {
                            screeps::SharedCreepProperties::move_to(self,&r);
                        },
                        _ => return
                    }
                },
                None => match get_nearest_logi_controller(self, resource_type){
                    Some(r) => {
                        match self.upgrade_controller(&r) {
                            screeps::ReturnCode::Ok => {},
                            screeps::ReturnCode::NotInRange => {
                                screeps::SharedCreepProperties::move_to(self, &r);
                            }
                            _ => return
                        }
                    },
                    None => return,
                },
            },
        };
    }
}

pub fn get_nearest_logi_repair(creep: &screeps::Creep, resource_type: screeps::ResourceType) -> Option<screeps::Structure>{
    match resource_type {
        screeps::ResourceType::Energy => {
            let repairables = filters::get_my_repairables();
            let nearest_repair = repairables.iter().min_by_key(|st| travel::calc_travel_fatigue(creep, &st.pos()));

            match nearest_repair {
                Some(r) => Some(r.to_owned()),
                None => None
            }
        },
        _ => todo!(),
        // screeps::ResourceType::Power => todo!(),
        // screeps::ResourceType::Hydrogen => todo!(),
        // screeps::ResourceType::Oxygen => todo!(),
        // screeps::ResourceType::Utrium => todo!(),
        // screeps::ResourceType::Lemergium => todo!(),
        // screeps::ResourceType::Keanium => todo!(),
        // screeps::ResourceType::Zynthium => todo!(),
        // screeps::ResourceType::Catalyst => todo!(),
        // screeps::ResourceType::Ghodium => todo!(),
        // screeps::ResourceType::Hydroxide => todo!(),
        // screeps::ResourceType::ZynthiumKeanite => todo!(),
        // screeps::ResourceType::UtriumLemergite => todo!(),
        // screeps::ResourceType::UtriumHydride => todo!(),
        // screeps::ResourceType::UtriumOxide => todo!(),
        // screeps::ResourceType::KeaniumHydride => todo!(),
        // screeps::ResourceType::KeaniumOxide => todo!(),
        // screeps::ResourceType::LemergiumHydride => todo!(),
        // screeps::ResourceType::LemergiumOxide => todo!(),
        // screeps::ResourceType::ZynthiumHydride => todo!(),
        // screeps::ResourceType::ZynthiumOxide => todo!(),
        // screeps::ResourceType::GhodiumHydride => todo!(),
        // screeps::ResourceType::GhodiumOxide => todo!(),
        // screeps::ResourceType::UtriumAcid => todo!(),
        // screeps::ResourceType::UtriumAlkalide => todo!(),
        // screeps::ResourceType::KeaniumAcid => todo!(),
        // screeps::ResourceType::KeaniumAlkalide => todo!(),
        // screeps::ResourceType::LemergiumAcid => todo!(),
        // screeps::ResourceType::LemergiumAlkalide => todo!(),
        // screeps::ResourceType::ZynthiumAcid => todo!(),
        // screeps::ResourceType::ZynthiumAlkalide => todo!(),
        // screeps::ResourceType::GhodiumAcid => todo!(),
        // screeps::ResourceType::GhodiumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedUtriumAcid => todo!(),
        // screeps::ResourceType::CatalyzedUtriumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedKeaniumAcid => todo!(),
        // screeps::ResourceType::CatalyzedKeaniumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedLemergiumAcid => todo!(),
        // screeps::ResourceType::CatalyzedLemergiumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedZynthiumAcid => todo!(),
        // screeps::ResourceType::CatalyzedZynthiumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedGhodiumAcid => todo!(),
        // screeps::ResourceType::CatalyzedGhodiumAlkalide => todo!(),
        // screeps::ResourceType::Ops => todo!(),
        // screeps::ResourceType::Silicon => todo!(),
        // screeps::ResourceType::Metal => todo!(),
        // screeps::ResourceType::Biomass => todo!(),
        // screeps::ResourceType::Mist => todo!(),
        // screeps::ResourceType::UtriumBar => todo!(),
        // screeps::ResourceType::LemergiumBar => todo!(),
        // screeps::ResourceType::ZynthiumBar => todo!(),
        // screeps::ResourceType::KeaniumBar => todo!(),
        // screeps::ResourceType::GhodiumMelt => todo!(),
        // screeps::ResourceType::Oxidant => todo!(),
        // screeps::ResourceType::Reductant => todo!(),
        // screeps::ResourceType::Purifier => todo!(),
        // screeps::ResourceType::Battery => todo!(),
        // screeps::ResourceType::Composite => todo!(),
        // screeps::ResourceType::Crystal => todo!(),
        // screeps::ResourceType::Liquid => todo!(),
        // screeps::ResourceType::Wire => todo!(),
        // screeps::ResourceType::Switch => todo!(),
        // screeps::ResourceType::Transistor => todo!(),
        // screeps::ResourceType::Microchip => todo!(),
        // screeps::ResourceType::Circuit => todo!(),
        // screeps::ResourceType::Device => todo!(),
        // screeps::ResourceType::Cell => todo!(),
        // screeps::ResourceType::Phlegm => todo!(),
        // screeps::ResourceType::Tissue => todo!(),
        // screeps::ResourceType::Muscle => todo!(),
        // screeps::ResourceType::Organoid => todo!(),
        // screeps::ResourceType::Organism => todo!(),
        // screeps::ResourceType::Alloy => todo!(),
        // screeps::ResourceType::Tube => todo!(),
        // screeps::ResourceType::Fixtures => todo!(),
        // screeps::ResourceType::Frame => todo!(),
        // screeps::ResourceType::Hydraulics => todo!(),
        // screeps::ResourceType::Machine => todo!(),
        // screeps::ResourceType::Condensate => todo!(),
        // screeps::ResourceType::Concentrate => todo!(),
        // screeps::ResourceType::Extract => todo!(),
        // screeps::ResourceType::Spirit => todo!(),
        // screeps::ResourceType::Emanation => todo!(),
        // screeps::ResourceType::Essence => todo!(),
    }
}


pub fn get_nearest_logi_build(creep: &screeps::Creep, resource_type: screeps::ResourceType) -> Option<screeps::ConstructionSite>{
    match resource_type {
        screeps::ResourceType::Energy => {
            let buildables = filters::get_my_buildables();
            let nearest_build = buildables.iter().min_by_key(|st| travel::calc_travel_fatigue(creep, &st.pos()));
            match nearest_build {
                Some(r) => Some(r.to_owned()),
                None => None,
            }
        },
        _ => todo!(),
        // screeps::ResourceType::Power => todo!(),
        // screeps::ResourceType::Hydrogen => todo!(),
        // screeps::ResourceType::Oxygen => todo!(),
        // screeps::ResourceType::Utrium => todo!(),
        // screeps::ResourceType::Lemergium => todo!(),
        // screeps::ResourceType::Keanium => todo!(),
        // screeps::ResourceType::Zynthium => todo!(),
        // screeps::ResourceType::Catalyst => todo!(),
        // screeps::ResourceType::Ghodium => todo!(),
        // screeps::ResourceType::Hydroxide => todo!(),
        // screeps::ResourceType::ZynthiumKeanite => todo!(),
        // screeps::ResourceType::UtriumLemergite => todo!(),
        // screeps::ResourceType::UtriumHydride => todo!(),
        // screeps::ResourceType::UtriumOxide => todo!(),
        // screeps::ResourceType::KeaniumHydride => todo!(),
        // screeps::ResourceType::KeaniumOxide => todo!(),
        // screeps::ResourceType::LemergiumHydride => todo!(),
        // screeps::ResourceType::LemergiumOxide => todo!(),
        // screeps::ResourceType::ZynthiumHydride => todo!(),
        // screeps::ResourceType::ZynthiumOxide => todo!(),
        // screeps::ResourceType::GhodiumHydride => todo!(),
        // screeps::ResourceType::GhodiumOxide => todo!(),
        // screeps::ResourceType::UtriumAcid => todo!(),
        // screeps::ResourceType::UtriumAlkalide => todo!(),
        // screeps::ResourceType::KeaniumAcid => todo!(),
        // screeps::ResourceType::KeaniumAlkalide => todo!(),
        // screeps::ResourceType::LemergiumAcid => todo!(),
        // screeps::ResourceType::LemergiumAlkalide => todo!(),
        // screeps::ResourceType::ZynthiumAcid => todo!(),
        // screeps::ResourceType::ZynthiumAlkalide => todo!(),
        // screeps::ResourceType::GhodiumAcid => todo!(),
        // screeps::ResourceType::GhodiumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedUtriumAcid => todo!(),
        // screeps::ResourceType::CatalyzedUtriumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedKeaniumAcid => todo!(),
        // screeps::ResourceType::CatalyzedKeaniumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedLemergiumAcid => todo!(),
        // screeps::ResourceType::CatalyzedLemergiumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedZynthiumAcid => todo!(),
        // screeps::ResourceType::CatalyzedZynthiumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedGhodiumAcid => todo!(),
        // screeps::ResourceType::CatalyzedGhodiumAlkalide => todo!(),
        // screeps::ResourceType::Ops => todo!(),
        // screeps::ResourceType::Silicon => todo!(),
        // screeps::ResourceType::Metal => todo!(),
        // screeps::ResourceType::Biomass => todo!(),
        // screeps::ResourceType::Mist => todo!(),
        // screeps::ResourceType::UtriumBar => todo!(),
        // screeps::ResourceType::LemergiumBar => todo!(),
        // screeps::ResourceType::ZynthiumBar => todo!(),
        // screeps::ResourceType::KeaniumBar => todo!(),
        // screeps::ResourceType::GhodiumMelt => todo!(),
        // screeps::ResourceType::Oxidant => todo!(),
        // screeps::ResourceType::Reductant => todo!(),
        // screeps::ResourceType::Purifier => todo!(),
        // screeps::ResourceType::Battery => todo!(),
        // screeps::ResourceType::Composite => todo!(),
        // screeps::ResourceType::Crystal => todo!(),
        // screeps::ResourceType::Liquid => todo!(),
        // screeps::ResourceType::Wire => todo!(),
        // screeps::ResourceType::Switch => todo!(),
        // screeps::ResourceType::Transistor => todo!(),
        // screeps::ResourceType::Microchip => todo!(),
        // screeps::ResourceType::Circuit => todo!(),
        // screeps::ResourceType::Device => todo!(),
        // screeps::ResourceType::Cell => todo!(),
        // screeps::ResourceType::Phlegm => todo!(),
        // screeps::ResourceType::Tissue => todo!(),
        // screeps::ResourceType::Muscle => todo!(),
        // screeps::ResourceType::Organoid => todo!(),
        // screeps::ResourceType::Organism => todo!(),
        // screeps::ResourceType::Alloy => todo!(),
        // screeps::ResourceType::Tube => todo!(),
        // screeps::ResourceType::Fixtures => todo!(),
        // screeps::ResourceType::Frame => todo!(),
        // screeps::ResourceType::Hydraulics => todo!(),
        // screeps::ResourceType::Machine => todo!(),
        // screeps::ResourceType::Condensate => todo!(),
        // screeps::ResourceType::Concentrate => todo!(),
        // screeps::ResourceType::Extract => todo!(),
        // screeps::ResourceType::Spirit => todo!(),
        // screeps::ResourceType::Emanation => todo!(),
        // screeps::ResourceType::Essence => todo!(),
    }
}

pub fn get_nearest_logi_controller(creep: &screeps::Creep, resource_type: screeps::ResourceType) -> Option<screeps::StructureController>{
    match resource_type {
        screeps::ResourceType::Energy => {
            let controllers = filters::get_my_controllers();
            let nearest_controller = controllers.iter().min_by_key(|st| travel::calc_travel_fatigue(creep, &st.pos()));
            match nearest_controller {
                Some(r) => Some(r.to_owned()),
                None => None,
            }
        },
        _ => todo!(),
        // screeps::ResourceType::Power => todo!(),
        // screeps::ResourceType::Hydrogen => todo!(),
        // screeps::ResourceType::Oxygen => todo!(),
        // screeps::ResourceType::Utrium => todo!(),
        // screeps::ResourceType::Lemergium => todo!(),
        // screeps::ResourceType::Keanium => todo!(),
        // screeps::ResourceType::Zynthium => todo!(),
        // screeps::ResourceType::Catalyst => todo!(),
        // screeps::ResourceType::Ghodium => todo!(),
        // screeps::ResourceType::Hydroxide => todo!(),
        // screeps::ResourceType::ZynthiumKeanite => todo!(),
        // screeps::ResourceType::UtriumLemergite => todo!(),
        // screeps::ResourceType::UtriumHydride => todo!(),
        // screeps::ResourceType::UtriumOxide => todo!(),
        // screeps::ResourceType::KeaniumHydride => todo!(),
        // screeps::ResourceType::KeaniumOxide => todo!(),
        // screeps::ResourceType::LemergiumHydride => todo!(),
        // screeps::ResourceType::LemergiumOxide => todo!(),
        // screeps::ResourceType::ZynthiumHydride => todo!(),
        // screeps::ResourceType::ZynthiumOxide => todo!(),
        // screeps::ResourceType::GhodiumHydride => todo!(),
        // screeps::ResourceType::GhodiumOxide => todo!(),
        // screeps::ResourceType::UtriumAcid => todo!(),
        // screeps::ResourceType::UtriumAlkalide => todo!(),
        // screeps::ResourceType::KeaniumAcid => todo!(),
        // screeps::ResourceType::KeaniumAlkalide => todo!(),
        // screeps::ResourceType::LemergiumAcid => todo!(),
        // screeps::ResourceType::LemergiumAlkalide => todo!(),
        // screeps::ResourceType::ZynthiumAcid => todo!(),
        // screeps::ResourceType::ZynthiumAlkalide => todo!(),
        // screeps::ResourceType::GhodiumAcid => todo!(),
        // screeps::ResourceType::GhodiumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedUtriumAcid => todo!(),
        // screeps::ResourceType::CatalyzedUtriumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedKeaniumAcid => todo!(),
        // screeps::ResourceType::CatalyzedKeaniumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedLemergiumAcid => todo!(),
        // screeps::ResourceType::CatalyzedLemergiumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedZynthiumAcid => todo!(),
        // screeps::ResourceType::CatalyzedZynthiumAlkalide => todo!(),
        // screeps::ResourceType::CatalyzedGhodiumAcid => todo!(),
        // screeps::ResourceType::CatalyzedGhodiumAlkalide => todo!(),
        // screeps::ResourceType::Ops => todo!(),
        // screeps::ResourceType::Silicon => todo!(),
        // screeps::ResourceType::Metal => todo!(),
        // screeps::ResourceType::Biomass => todo!(),
        // screeps::ResourceType::Mist => todo!(),
        // screeps::ResourceType::UtriumBar => todo!(),
        // screeps::ResourceType::LemergiumBar => todo!(),
        // screeps::ResourceType::ZynthiumBar => todo!(),
        // screeps::ResourceType::KeaniumBar => todo!(),
        // screeps::ResourceType::GhodiumMelt => todo!(),
        // screeps::ResourceType::Oxidant => todo!(),
        // screeps::ResourceType::Reductant => todo!(),
        // screeps::ResourceType::Purifier => todo!(),
        // screeps::ResourceType::Battery => todo!(),
        // screeps::ResourceType::Composite => todo!(),
        // screeps::ResourceType::Crystal => todo!(),
        // screeps::ResourceType::Liquid => todo!(),
        // screeps::ResourceType::Wire => todo!(),
        // screeps::ResourceType::Switch => todo!(),
        // screeps::ResourceType::Transistor => todo!(),
        // screeps::ResourceType::Microchip => todo!(),
        // screeps::ResourceType::Circuit => todo!(),
        // screeps::ResourceType::Device => todo!(),
        // screeps::ResourceType::Cell => todo!(),
        // screeps::ResourceType::Phlegm => todo!(),
        // screeps::ResourceType::Tissue => todo!(),
        // screeps::ResourceType::Muscle => todo!(),
        // screeps::ResourceType::Organoid => todo!(),
        // screeps::ResourceType::Organism => todo!(),
        // screeps::ResourceType::Alloy => todo!(),
        // screeps::ResourceType::Tube => todo!(),
        // screeps::ResourceType::Fixtures => todo!(),
        // screeps::ResourceType::Frame => todo!(),
        // screeps::ResourceType::Hydraulics => todo!(),
        // screeps::ResourceType::Machine => todo!(),
        // screeps::ResourceType::Condensate => todo!(),
        // screeps::ResourceType::Concentrate => todo!(),
        // screeps::ResourceType::Extract => todo!(),
        // screeps::ResourceType::Spirit => todo!(),
        // screeps::ResourceType::Emanation => todo!(),
        // screeps::ResourceType::Essence => todo!(),
    }
}

