use screeps::{find, Room, Creep, StructureSpawn, ConstructionSite, PowerCreep, OwnedStructure};


pub fn get_hostility(room: &Room) -> (Vec<Creep>,Vec<PowerCreep>,Vec<StructureSpawn>,Vec<OwnedStructure>,Vec<ConstructionSite>) {
    (
        room.find(find::HOSTILE_CREEPS),
        room.find(find::HOSTILE_POWER_CREEPS),
        room.find(find::HOSTILE_SPAWNS),
        room.find(find::HOSTILE_STRUCTURES),
        room.find(find::HOSTILE_CONSTRUCTION_SITES),
    )
}
