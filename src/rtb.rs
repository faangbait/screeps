
pub trait Sink {}
impl Sink for screeps::Creep {}
impl Sink for screeps::StructureRoad {}
impl Sink for screeps::StructureWall {}
impl Sink for screeps::StructureRampart {}
impl Sink for screeps::StructureController {}
impl Sink for screeps::StructureLink {}
impl Sink for screeps::StructureStorage {}
impl Sink for screeps::StructureTower {}
impl Sink for screeps::StructurePowerSpawn {}
impl Sink for screeps::StructureTerminal {}
impl Sink for screeps::StructureContainer {}
impl Sink for screeps::StructureNuker {}
impl Sink for screeps::StructureFactory {}
impl Sink for screeps::ConstructionSite {}
impl Sink for dyn screeps::Transferable {}

pub trait Source {}
impl Source for screeps::Creep {}
impl Source for screeps::Source {}
impl Source for screeps::Deposit {}
impl Source for screeps::Resource {}
impl Source for screeps::Mineral {}
impl Source for screeps::Tombstone {}
impl Source for dyn screeps::Withdrawable {}
impl Source for screeps::StructureSpawn {}
impl Source for screeps::StructureExtension {}
impl Source for screeps::StructureLink {}
impl Source for screeps::StructureStorage {}
impl Source for screeps::StructureTower {}
impl Source for screeps::StructurePowerSpawn {}
impl Source for screeps::StructureTerminal {}
impl Source for screeps::StructureContainer {}
impl Source for screeps::StructureNuker {}

