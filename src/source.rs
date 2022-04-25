use screeps::{ResourceType, HasStore, RawObjectId, HasId, Position, HasPosition};
use serde::{Serialize, Deserialize};
use crate::basic_enums::{JobType, SourceType};
use crate::sink::ResourceValued;


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SourceWork {
    pub target: RawObjectId,
    pub position: Position,
    pub price: u32,
    pub job_type: JobType,
    pub resource_type: Option<ResourceType>,
    pub resource_max: Option<i32>,
    pub ty: SourceType,
}
pub trait SourceNode {
    fn ask(self: &Self) -> Option<SourceWork>;
}

pub trait BufferStorage: ResourceValued + HasStore + HasId {
    fn needs_withdraw(self: &Self, ty: SourceType) -> Option<SourceWork> {
        if self.store_used_capacity(Some(ResourceType::Energy)) > 0 {
            let proposed_work = SourceWork {
                target: self.untyped_id(),
                position: self.pos(),
                price: 1 + self.get_resource_val(ResourceType::Energy) as u32,
                job_type: JobType::Withdraw,
                resource_type: Some(ResourceType::Energy), // TODO: More resources
                resource_max: Some(self.store_used_capacity(Some(ResourceType::Energy)) as i32),
                ty,                   
            };

            if proposed_work.price > 0 {
                Some(proposed_work)
            } else { None }
        } else { None }
        //     if let Ok(root) = screeps::memory::root().dict_or_create("workVal") {
        //         if let Ok(mem) = root.i32(&self.untyped_id().to_string()) {
        //             if let Some(work_val) = mem {
        //                 if work_val > 0 {
        //                     Some(SourceWork {
        //                         target: self.untyped_id(),
        //                         price: 1 + work_val as u32,
        //                         job_type: JobType::Withdraw,
        //                         resource_type: Some(ResourceType::Energy), // TODO: More resources
        //                         resource_max: Some(self.store_used_capacity(Some(ResourceType::Energy)) as i32),
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
impl BufferStorage for screeps::Tombstone {}

impl SourceNode for screeps::Creep {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Creep;
        self.needs_withdraw(source_type)
    }
}

impl SourceNode for screeps::PowerCreep {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::PowerCreep;
        self.needs_withdraw(source_type)
    }
}

impl SourceNode for screeps::StructureContainer {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Container;
        self.needs_withdraw(source_type)
    }
}


impl SourceNode for screeps::StructureExtension {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Extension;
        self.needs_withdraw(source_type)
    }
}


impl SourceNode for screeps::StructureFactory {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Factory;
        self.needs_withdraw(source_type)
    }
}


impl SourceNode for screeps::StructureLab {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Lab;
        self.needs_withdraw(source_type)
    }
}

impl SourceNode for screeps::StructureLink {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Link;
        self.needs_withdraw(source_type)
    }
}

impl SourceNode for screeps::StructureSpawn {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Spawn;
        self.needs_withdraw(source_type)
    }
}
impl SourceNode for screeps::StructureStorage {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Storage;
        self.needs_withdraw(source_type)
    }
}
impl SourceNode for screeps::StructureTower {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Tower;
        self.needs_withdraw(source_type)
    }
}
impl SourceNode for screeps::StructurePowerSpawn {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::PowerSpawn;
        self.needs_withdraw(source_type)
    }
}
impl SourceNode for screeps::StructureTerminal {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Terminal;
        self.needs_withdraw(source_type)
    }
}

impl SourceNode for screeps::Tombstone {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Tombstone;
        self.needs_withdraw(source_type)
    }
}

impl SourceNode for screeps::Source {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Source;
        Some(SourceWork {
            target: self.untyped_id(),
            position: self.pos(),
            price: 0,
            job_type: JobType::Harvest,
            resource_type: Some(ResourceType::Energy),
            resource_max: Some(self.energy() as i32),
            ty: source_type,
        })
    }
}

impl SourceNode for screeps::Mineral {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Mineral;
        Some(SourceWork {
            target: self.untyped_id(),
            position: self.pos(),
            price: 0,
            job_type: JobType::Harvest,
            resource_type: Some(self.mineral_type()),
            resource_max: Some(self.mineral_amount() as i32),
            ty: source_type,
        })
    }
}

impl SourceNode for screeps::Deposit {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Deposit;
        Some(SourceWork {
            target: self.untyped_id(),
            position: self.pos(),
            price: 0,
            job_type: JobType::Harvest,
            resource_type: Some(self.deposit_type()),
            resource_max: Some(if self.last_cooldown() > 0 { 400 / self.last_cooldown() as i32 } else { 400 }),
            ty: source_type,
        })
    }
}

impl SourceNode for screeps::objects::Resource {
    fn ask(self: &Self) -> Option<SourceWork> {
        let source_type = SourceType::Resource;
        Some(SourceWork {
            target: self.untyped_id(),
            position: self.pos(),
            price: 0,
            job_type: JobType::Pickup,
            resource_type: Some(self.resource_type()),
            resource_max: Some(self.amount() as i32),
            ty: source_type,
        })
    }
}

impl SourceNode for screeps::Structure {
    fn ask(self: &Self) -> Option<SourceWork> {
        match self {
            screeps::Structure::Container(st) => st.ask(),
            screeps::Structure::Extension(st) => st.ask(),
            screeps::Structure::Factory(st) => st.ask(),
            screeps::Structure::Lab(st) => st.ask(),
            screeps::Structure::Link(st) => st.ask(),
            screeps::Structure::PowerSpawn(st) => st.ask(),
            screeps::Structure::Spawn(st) => st.ask(),
            screeps::Structure::Storage(st) => st.ask(),
            screeps::Structure::Terminal(st) => st.ask(),
            screeps::Structure::Tower(st) => st.ask(),
            _ => None,
        }
    }
}
