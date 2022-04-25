
use std::fmt::Error;

use screeps::{ResourceType, HasStore, RawObjectId, HasId, Attackable, SharedCreepProperties, HasCooldown, RoomObjectProperties, SizedRoomObject, Position, HasPosition};

use crate::basic_enums::{JobType, SinkType};
use serde::{Serialize,Deserialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SinkWork {
    pub target: RawObjectId,
    pub position: Position,
    pub price: u32,
    pub job_type: JobType,
    pub resource_type: Option<ResourceType>,
    pub resource_max: Option<i32>,
    pub ty: SinkType,
}

pub trait SinkNode: HasId {
    fn bid(self: &Self) -> Option<SinkWork>;
}

pub trait ResourceValued : HasId + SizedRoomObject {
    fn set_resource_val(self: &Self, work_opt: Option<SinkWork>) {
        if let Some(work) = work_opt {
            if let Some(resource_type) = work.resource_type {
                if let Ok(root) = screeps::memory::root().dict_or_create("ResourceVals") {
                    let r_key = match resource_type {
                        ResourceType::Energy => "Energy",
                        ResourceType::Power => "Power",
                        ResourceType::Hydrogen => "Hydrogen",
                        ResourceType::Oxygen => "Oxygen",
                        ResourceType::Utrium => "Utrium",
                        ResourceType::Lemergium => "Lemergium",
                        ResourceType::Keanium => "Keanium",
                        ResourceType::Zynthium => "Zynthium",
                        ResourceType::Catalyst => "Catalyst",
                        ResourceType::Ghodium => "Ghodium",
                        ResourceType::Hydroxide => "Hydroxide",
                        ResourceType::ZynthiumKeanite => "ZynthiumKeanite",
                        ResourceType::UtriumLemergite => "UtriumLemergite",
                        ResourceType::UtriumHydride => "UtriumHydride",
                        ResourceType::UtriumOxide => "UtriumOxide",
                        ResourceType::KeaniumHydride => "KeaniumHydride",
                        ResourceType::KeaniumOxide => "KeaniumOxide",
                        ResourceType::LemergiumHydride => "LemergiumHydride",
                        ResourceType::LemergiumOxide => "LemergiumOxide",
                        ResourceType::ZynthiumHydride => "ZynthiumHydride",
                        ResourceType::ZynthiumOxide => "ZynthiumOxide",
                        ResourceType::GhodiumHydride => "GhodiumHydride",
                        ResourceType::GhodiumOxide => "GhodiumOxide",
                        ResourceType::UtriumAcid => "UtriumAcid",
                        ResourceType::UtriumAlkalide => "UtriumAlkalide",
                        ResourceType::KeaniumAcid => "KeaniumAcid",
                        ResourceType::KeaniumAlkalide => "KeaniumAlkalide",
                        ResourceType::LemergiumAcid => "LemergiumAcid",
                        ResourceType::LemergiumAlkalide => "LemergiumAlkalide",
                        ResourceType::ZynthiumAcid => "ZynthiumAcid",
                        ResourceType::ZynthiumAlkalide => "ZynthiumAlkalide",
                        ResourceType::GhodiumAcid => "GhodiumAcid",
                        ResourceType::GhodiumAlkalide => "GhodiumAlkalide",
                        ResourceType::CatalyzedUtriumAcid => "CatalyzedUtriumAcid",
                        ResourceType::CatalyzedUtriumAlkalide => "CatalyzedUtriumAlkalide",
                        ResourceType::CatalyzedKeaniumAcid => "CatalyzedKeaniumAcid",
                        ResourceType::CatalyzedKeaniumAlkalide => "CatalyzedKeaniumAlkalide",
                        ResourceType::CatalyzedLemergiumAcid => "CatalyzedLemergiumAcid",
                        ResourceType::CatalyzedLemergiumAlkalide => "CatalyzedLemergiumAlkalide",
                        ResourceType::CatalyzedZynthiumAcid => "CatalyzedZynthiumAcid",
                        ResourceType::CatalyzedZynthiumAlkalide => "CatalyzedZynthiumAlkalide",
                        ResourceType::CatalyzedGhodiumAcid => "CatalyzedGhodiumAcid",
                        ResourceType::CatalyzedGhodiumAlkalide => "CatalyzedGhodiumAlkalide",
                        ResourceType::Ops => "Ops",
                        ResourceType::Silicon => "Silicon",
                        ResourceType::Metal => "Metal",
                        ResourceType::Biomass => "Biomass",
                        ResourceType::Mist => "Mist",
                        ResourceType::UtriumBar => "UtriumBar",
                        ResourceType::LemergiumBar => "LemergiumBar",
                        ResourceType::ZynthiumBar => "ZynthiumBar",
                        ResourceType::KeaniumBar => "KeaniumBar",
                        ResourceType::GhodiumMelt => "GhodiumMelt",
                        ResourceType::Oxidant => "Oxidant",
                        ResourceType::Reductant => "Reductant",
                        ResourceType::Purifier => "Purifier",
                        ResourceType::Battery => "Battery",
                        ResourceType::Composite => "Composite",
                        ResourceType::Crystal => "Crystal",
                        ResourceType::Liquid => "Liquid",
                        ResourceType::Wire => "Wire",
                        ResourceType::Switch => "Switch",
                        ResourceType::Transistor => "Transistor",
                        ResourceType::Microchip => "Microchip",
                        ResourceType::Circuit => "Circuit",
                        ResourceType::Device => "Device",
                        ResourceType::Cell => "Cell",
                        ResourceType::Phlegm => "Phlegm",
                        ResourceType::Tissue => "Tissue",
                        ResourceType::Muscle => "Muscle",
                        ResourceType::Organoid => "Organoid",
                        ResourceType::Organism => "Organism",
                        ResourceType::Alloy => "Alloy",
                        ResourceType::Tube => "Tube",
                        ResourceType::Fixtures => "Fixtures",
                        ResourceType::Frame => "Frame",
                        ResourceType::Hydraulics => "Hydraulics",
                        ResourceType::Machine => "Machine",
                        ResourceType::Condensate => "Condensate",
                        ResourceType::Concentrate => "Concentrate",
                        ResourceType::Extract => "Extract",
                        ResourceType::Spirit => "Spirit",
                        ResourceType::Emanation => "Emanation",
                        ResourceType::Essence => "Essence",
                    };
                    if let Ok(resource) = root.dict_or_create(r_key) {
                        let last = resource.i32(&self.untyped_id().to_string());
                        if let Ok(last_opt) = last {
                            if let Some(last_val) = last_opt {
                                resource.set(&self.untyped_id().to_string(), ((last_val*3) + work.price as i32) / 4);
                            } else { resource.set(&self.untyped_id().to_string(), work.price);}
                        } else { resource.set(&self.untyped_id().to_string(), work.price);}
                    }
                }
            }
        }
    }

    fn get_resource_val(self: &Self, resource_type: ResourceType) -> i32 {
            if let Ok(root) = screeps::memory::root().dict_or_create("ResourceVals") {
                let r_key = match resource_type {
                    ResourceType::Energy => "Energy",
                    ResourceType::Power => "Power",
                    ResourceType::Hydrogen => "Hydrogen",
                    ResourceType::Oxygen => "Oxygen",
                    ResourceType::Utrium => "Utrium",
                    ResourceType::Lemergium => "Lemergium",
                    ResourceType::Keanium => "Keanium",
                    ResourceType::Zynthium => "Zynthium",
                    ResourceType::Catalyst => "Catalyst",
                    ResourceType::Ghodium => "Ghodium",
                    ResourceType::Hydroxide => "Hydroxide",
                    ResourceType::ZynthiumKeanite => "ZynthiumKeanite",
                    ResourceType::UtriumLemergite => "UtriumLemergite",
                    ResourceType::UtriumHydride => "UtriumHydride",
                    ResourceType::UtriumOxide => "UtriumOxide",
                    ResourceType::KeaniumHydride => "KeaniumHydride",
                    ResourceType::KeaniumOxide => "KeaniumOxide",
                    ResourceType::LemergiumHydride => "LemergiumHydride",
                    ResourceType::LemergiumOxide => "LemergiumOxide",
                    ResourceType::ZynthiumHydride => "ZynthiumHydride",
                    ResourceType::ZynthiumOxide => "ZynthiumOxide",
                    ResourceType::GhodiumHydride => "GhodiumHydride",
                    ResourceType::GhodiumOxide => "GhodiumOxide",
                    ResourceType::UtriumAcid => "UtriumAcid",
                    ResourceType::UtriumAlkalide => "UtriumAlkalide",
                    ResourceType::KeaniumAcid => "KeaniumAcid",
                    ResourceType::KeaniumAlkalide => "KeaniumAlkalide",
                    ResourceType::LemergiumAcid => "LemergiumAcid",
                    ResourceType::LemergiumAlkalide => "LemergiumAlkalide",
                    ResourceType::ZynthiumAcid => "ZynthiumAcid",
                    ResourceType::ZynthiumAlkalide => "ZynthiumAlkalide",
                    ResourceType::GhodiumAcid => "GhodiumAcid",
                    ResourceType::GhodiumAlkalide => "GhodiumAlkalide",
                    ResourceType::CatalyzedUtriumAcid => "CatalyzedUtriumAcid",
                    ResourceType::CatalyzedUtriumAlkalide => "CatalyzedUtriumAlkalide",
                    ResourceType::CatalyzedKeaniumAcid => "CatalyzedKeaniumAcid",
                    ResourceType::CatalyzedKeaniumAlkalide => "CatalyzedKeaniumAlkalide",
                    ResourceType::CatalyzedLemergiumAcid => "CatalyzedLemergiumAcid",
                    ResourceType::CatalyzedLemergiumAlkalide => "CatalyzedLemergiumAlkalide",
                    ResourceType::CatalyzedZynthiumAcid => "CatalyzedZynthiumAcid",
                    ResourceType::CatalyzedZynthiumAlkalide => "CatalyzedZynthiumAlkalide",
                    ResourceType::CatalyzedGhodiumAcid => "CatalyzedGhodiumAcid",
                    ResourceType::CatalyzedGhodiumAlkalide => "CatalyzedGhodiumAlkalide",
                    ResourceType::Ops => "Ops",
                    ResourceType::Silicon => "Silicon",
                    ResourceType::Metal => "Metal",
                    ResourceType::Biomass => "Biomass",
                    ResourceType::Mist => "Mist",
                    ResourceType::UtriumBar => "UtriumBar",
                    ResourceType::LemergiumBar => "LemergiumBar",
                    ResourceType::ZynthiumBar => "ZynthiumBar",
                    ResourceType::KeaniumBar => "KeaniumBar",
                    ResourceType::GhodiumMelt => "GhodiumMelt",
                    ResourceType::Oxidant => "Oxidant",
                    ResourceType::Reductant => "Reductant",
                    ResourceType::Purifier => "Purifier",
                    ResourceType::Battery => "Battery",
                    ResourceType::Composite => "Composite",
                    ResourceType::Crystal => "Crystal",
                    ResourceType::Liquid => "Liquid",
                    ResourceType::Wire => "Wire",
                    ResourceType::Switch => "Switch",
                    ResourceType::Transistor => "Transistor",
                    ResourceType::Microchip => "Microchip",
                    ResourceType::Circuit => "Circuit",
                    ResourceType::Device => "Device",
                    ResourceType::Cell => "Cell",
                    ResourceType::Phlegm => "Phlegm",
                    ResourceType::Tissue => "Tissue",
                    ResourceType::Muscle => "Muscle",
                    ResourceType::Organoid => "Organoid",
                    ResourceType::Organism => "Organism",
                    ResourceType::Alloy => "Alloy",
                    ResourceType::Tube => "Tube",
                    ResourceType::Fixtures => "Fixtures",
                    ResourceType::Frame => "Frame",
                    ResourceType::Hydraulics => "Hydraulics",
                    ResourceType::Machine => "Machine",
                    ResourceType::Condensate => "Condensate",
                    ResourceType::Concentrate => "Concentrate",
                    ResourceType::Extract => "Extract",
                    ResourceType::Spirit => "Spirit",
                    ResourceType::Emanation => "Emanation",
                    ResourceType::Essence => "Essence",
                };
                if let Ok(resource) = root.dict_or_create(r_key) {
                    let last = resource.i32(&self.untyped_id().to_string());
                    if let Ok(last_opt) = last {
                        if let Some(last_val) = last_opt { return last_val.max(0) };
                    };
                };
        };
        return 0
    }
}

pub trait NeedsRepair: Attackable + SinkNode + HasId {
    fn needs_repair(self: &Self, ty: SinkType) -> Option<SinkWork> {
        if self.hits() < self.hits_max() - 100 {
            Some(SinkWork {
                target: self.untyped_id(),
                position: self.pos(),
                price: 10,
                job_type: JobType::Transfer,
                resource_type: Some(ResourceType::Energy),
                resource_max: Some(((self.hits_max() - self.hits()) / 100) as i32),
                ty
            })
        } else { None }
    }
}

impl NeedsRepair for screeps::StructureContainer {}
impl NeedsRepair for screeps::StructureExtension {}
impl NeedsRepair for screeps::StructureExtractor {}
impl NeedsRepair for screeps::StructureFactory {}
impl NeedsRepair for screeps::StructureLab {}
impl NeedsRepair for screeps::StructureLink {}
impl NeedsRepair for screeps::StructureNuker {}
impl NeedsRepair for screeps::StructureObserver {}
impl NeedsRepair for screeps::StructurePowerSpawn {}
impl NeedsRepair for screeps::StructureRampart {}
impl NeedsRepair for screeps::StructureRoad {}
impl NeedsRepair for screeps::StructureSpawn {}
impl NeedsRepair for screeps::StructureStorage {}
impl NeedsRepair for screeps::StructureTerminal {}
impl NeedsRepair for screeps::StructureTower {}
impl NeedsRepair for screeps::StructureWall {}

pub trait NeedsHeal: Attackable + SinkNode + HasId {
    fn needs_heal(self: &Self, ty: SinkType) -> Option<SinkWork> {
        if self.hits() < self.hits_max() {
            Some(SinkWork {
                target: self.untyped_id(),
                position: self.pos(),
                price: 0, // TODO: Body cost
                job_type: JobType::Heal,
                resource_type: None,
                resource_max: None,
                ty,
            })
        } else { None }
    }
}

impl NeedsHeal for screeps::Creep {}
impl NeedsHeal for screeps::PowerCreep {}

pub trait BufferStorage: ResourceValued + HasStore + HasId {
    fn needs_transfer(self: &Self, ty: SinkType) -> Option<SinkWork> {
        if self.store_free_capacity(Some(ResourceType::Energy)) > 0 {
            Some(SinkWork {
                target: self.untyped_id(),
                position: self.pos(),
                price: self.get_resource_val(ResourceType::Energy) as u32,
                job_type: JobType::Transfer,
                resource_type: Some(ResourceType::Energy), // TODO: More resources
                resource_max: Some(self.store_free_capacity(Some(ResourceType::Energy))),
                ty,
            })
            // if proposed_work.price > 0 {
            //     Some(proposed_work)
            // } else { None }
        } else { None }
        //     if let Ok(root) = screeps::memory::root().dict_or_create("workVal") {
        //         if let Ok(mem) = root.i32(&self.untyped_id().to_string()) {
        //             if let Some(work_val) = mem {
        //                 if work_val > 0 {
        //                     Some(SinkWork {
        //                         target: self.untyped_id(),
        //                         price: work_val as u32,
        //                         job_type: JobType::Transfer,
        //                         resource_type: Some(ResourceType::Energy), // TODO: More resources
        //                         resource_max: Some(self.store_free_capacity(Some(ResourceType::Energy))),
        //                         ty,
        //                     })             
        //                 } else { None }
        //             } else { None }
        //         } else { None }
        //     } else { None }
        // } else { None }
    }
}
impl BufferStorage for screeps::Creep {}
impl BufferStorage for screeps::PowerCreep {}
impl BufferStorage for screeps::StructureContainer {}
impl BufferStorage for screeps::StructureExtension {}
impl BufferStorage for screeps::StructureFactory {}
impl BufferStorage for screeps::StructureLab {}
impl BufferStorage for screeps::StructureLink {}
impl BufferStorage for screeps::StructureNuker {}
impl BufferStorage for screeps::StructureSpawn {}
impl BufferStorage for screeps::StructureStorage {}
impl BufferStorage for screeps::StructureTower {}
impl BufferStorage for screeps::StructurePowerSpawn {}
impl BufferStorage for screeps::StructureTerminal {}

impl ResourceValued for screeps::Creep {}
impl ResourceValued for screeps::PowerCreep {}
impl ResourceValued for screeps::StructureContainer {}
impl ResourceValued for screeps::StructureExtension {}
impl ResourceValued for screeps::StructureFactory {}
impl ResourceValued for screeps::StructureLab {}
impl ResourceValued for screeps::StructureLink {}
impl ResourceValued for screeps::StructureNuker {}
impl ResourceValued for screeps::StructureSpawn {}
impl ResourceValued for screeps::StructureStorage {}
impl ResourceValued for screeps::StructureTower {}
impl ResourceValued for screeps::StructurePowerSpawn {}
impl ResourceValued for screeps::StructureTerminal {}
impl ResourceValued for screeps::Tombstone {}

impl SinkNode for screeps::Creep {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Creep;
        if self.needs_heal(sink_type).is_some() {
            self.needs_heal(sink_type)
        } else if self.needs_transfer(sink_type).is_some() { 
            self.needs_transfer(sink_type)
        } else { None }
    }
}

impl SinkNode for screeps::PowerCreep {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::PowerCreep;
        if self.needs_heal(sink_type).is_some() {
            self.needs_heal(sink_type)
        } else if self.needs_transfer(sink_type).is_some() { 
            self.needs_transfer(sink_type)
        } else { None }
    }
}

impl SinkNode for screeps::StructureRoad {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Road;
        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else { None }
    }
}

impl SinkNode for screeps::StructureContainer {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Container;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else if self.needs_transfer(sink_type).is_some() { 
            self.needs_transfer(sink_type)
        } else { None }
    }
}

impl SinkNode for screeps::StructureExtension {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Extension;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else if self.needs_transfer(sink_type).is_some() { 
            self.needs_transfer(sink_type)
        } else { None }


    }
}


impl SinkNode for screeps::StructureStorage {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Storage;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else if self.needs_transfer(sink_type).is_some() { 
            self.needs_transfer(sink_type)
        } else { None }


    }
}


impl SinkNode for screeps::StructureTower {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Tower;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else if self.needs_transfer(sink_type).is_some() { 
            self.needs_transfer(sink_type)
        } else { None }


    }
}


impl SinkNode for screeps::StructureSpawn {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Spawn;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else if self.needs_transfer(sink_type).is_some() { 
            self.needs_transfer(sink_type)
        } else { None }

    }
}


impl SinkNode for screeps::StructurePowerSpawn {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::PowerSpawn;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else if self.needs_transfer(sink_type).is_some() { 
            self.needs_transfer(sink_type)
        } else { None }

    }
}


impl SinkNode for screeps::StructureTerminal {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Terminal;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else if self.needs_transfer(sink_type).is_some() { 
            self.needs_transfer(sink_type)
        } else { None }

    }
}


impl SinkNode for screeps::StructureNuker {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Nuker;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else if self.needs_transfer(sink_type).is_some() { 
            self.needs_transfer(sink_type)
        } else { None }
    }
}

impl SinkNode for screeps::StructureLink {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Link;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else if self.needs_transfer(sink_type).is_some() { 
            self.needs_transfer(sink_type)
        } else { None }
    }
}


impl SinkNode for screeps::StructureLab {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Lab;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else if self.needs_transfer(sink_type).is_some() { 
            self.needs_transfer(sink_type)
        } else { None }
    }
}


impl SinkNode for screeps::StructureFactory {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Factory;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else if self.needs_transfer(sink_type).is_some() { 
            self.needs_transfer(sink_type)
        } else { None }
    }
}



impl SinkNode for screeps::StructureExtractor {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Extractor;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else { None }
    }
}


impl SinkNode for screeps::StructureObserver {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Observer;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else { None }
    }
}


impl SinkNode for screeps::StructureWall {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Wall;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else { None }
    }
}

impl SinkNode for screeps::StructureRampart {
    fn bid(self: &Self) -> Option<SinkWork> {
        let sink_type = SinkType::Rampart;

        if self.needs_repair(sink_type).is_some() {
            self.needs_repair(sink_type)
        } else { None }
    }
}

impl SinkNode for screeps::ConstructionSite {
    fn bid(self: &Self) -> Option<SinkWork> {
        let mult = (self.progress_total() as f32 / (self.progress_total() as f32 / (self.progress() as f32 + 1.0))) / 12.0;
        
        let price = (match self.structure_type() {
            screeps::StructureType::Spawn => mult * 30.,
            screeps::StructureType::Extension => mult * 125.,
            screeps::StructureType::Road => {
                let here = self.room().unwrap().look_at(self);
                let mut terrain = screeps::Terrain::Wall;
                let road_priority;
        
                for obj in here {
                    match obj {
                        screeps::LookResult::Terrain(t) => terrain = t,
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
        }) as u32;

        Some(SinkWork {
            target: self.untyped_id(),
            position: self.pos(),
            price,
            job_type: JobType::Build,
            resource_type: Some(ResourceType::Energy),
            resource_max: Some((self.progress_total() - self.progress()) as i32),
            ty: SinkType::ConstructionSite
        })

    }
}

impl SinkNode for screeps::StructureController {
    fn bid(self: &Self) -> Option<SinkWork> {
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

        let price = match self.level() {
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

        Some(SinkWork {
            target: self.untyped_id(),
            position: self.pos(),
            price,
            job_type: JobType::Upgrade,
            resource_type: Some(ResourceType::Energy),
            resource_max: Some(5),
            ty: SinkType::Controller
        })
    }
}

impl SinkNode for screeps::Structure {
    fn bid(self: &Self) -> Option<SinkWork> { 
        match self {
            screeps::Structure::Container(st) => st.bid(),
            screeps::Structure::Controller(st) => st.bid(),
            screeps::Structure::Extension(st) => st.bid(),
            screeps::Structure::Extractor(st) => st.bid(),
            screeps::Structure::Factory(st) => st.bid(),
            screeps::Structure::Lab(st) => st.bid(),
            screeps::Structure::Link(st) => st.bid(),
            screeps::Structure::Nuker(st) => st.bid(),
            screeps::Structure::Observer(st) => st.bid(),
            screeps::Structure::PowerSpawn(st) => st.bid(),
            screeps::Structure::Rampart(st) => st.bid(),
            screeps::Structure::Road(st) => st.bid(),
            screeps::Structure::Spawn(st) => st.bid(),
            screeps::Structure::Storage(st) => st.bid(),
            screeps::Structure::Terminal(st) => st.bid(),
            screeps::Structure::Tower(st) => st.bid(),
            screeps::Structure::Wall(st) => st.bid(),
            _ => None
        } 
    }
}

