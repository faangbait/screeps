use screeps::{Source, Room, StructureContainer, StructureExtension, StructureStorage, StructureTower, Structure, ConstructionSite, StructureController, RoomObjectProperties, find, look, Resource, StructureSpawn, StructureFactory, StructureLab, StructureLink, StructureNuker, StructurePowerSpawn, StructureTerminal, HasId};

pub fn get_my_rooms() -> Vec<Room> {
    let mut rooms: Vec<Room> = screeps::game::spawns::values()
    .iter()
    .filter_map(|spawn| spawn.room())
    .collect();

    rooms.dedup_by_key(|room| room.name());
    rooms
}

pub fn get_my_structures() -> Vec<Structure> {
    get_my_rooms()
         .iter()
         .flat_map(|room| room.find(find::STRUCTURES))
         .collect()
 }
 
pub fn get_my_sources() -> Vec<Source> {
    let mut sources = get_my_rooms()
    .iter()
    .flat_map(|r| r.find(find::SOURCES))
    .collect::<Vec<Source>>();

    sources.sort_by_key(|st|st.id());
    sources
}

pub fn get_hostile_rooms() -> Vec<Room> {
    todo!();
}

pub fn get_my_spawns() -> Vec<StructureSpawn> {
    screeps::game::spawns::values()
}

pub fn get_my_containers() -> Vec<StructureContainer> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(find::STRUCTURES))
    .filter_map(|s| match s {
        Structure::Container(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_extensions() -> Vec<StructureExtension> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(find::STRUCTURES))
    .filter_map(|s| match s {
        Structure::Extension(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_towers() -> Vec<StructureTower> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(find::STRUCTURES))
    .filter_map(|s| match s {
        Structure::Tower(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_storages() -> Vec<StructureStorage> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(find::STRUCTURES))
    .filter_map(|s| match s {
        Structure::Storage(st) => Some(st),
        _ => None,
    }).collect()
}
pub fn get_my_repairables() -> Vec<Structure> {
    get_my_rooms()
    .iter()
    .flat_map(|room| room.find(find::STRUCTURES))
    .filter(|s| s.as_can_decay().map(|st| st.ticks_to_decay() > 0).unwrap_or(false))
    .filter(|s| s.as_attackable().map(|st| st.hits_max() > st.hits() + 50).unwrap_or(true))
    .collect()

    // repairables.sort_by_key(|s| s.as_attackable().map(|st| st.hits() as i32 - st.hits_max() as i32).unwrap_or(999999));
    // repairables

}

pub fn get_my_buildables() -> Vec<ConstructionSite> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(find::CONSTRUCTION_SITES))
    .collect()
}

pub fn get_my_controllers() -> Vec<StructureController> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(find::STRUCTURES))
    .filter_map(|s| match s {
        Structure::Controller(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_factories() -> Vec<StructureFactory> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(find::STRUCTURES))
    .filter_map(|s| match s {
        Structure::Factory(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_labs() -> Vec<StructureLab> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(find::STRUCTURES))
    .filter_map(|s| match s {
        Structure::Lab(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_links() -> Vec<StructureLink> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(find::STRUCTURES))
    .filter_map(|s| match s {
        Structure::Link(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_nukers() -> Vec<StructureNuker> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(find::STRUCTURES))
    .filter_map(|s| match s {
        Structure::Nuker(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_powerspawns() -> Vec<StructurePowerSpawn> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(find::STRUCTURES))
    .filter_map(|s| match s {
        Structure::PowerSpawn(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_terminals() -> Vec<StructureTerminal> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(find::STRUCTURES))
    .filter_map(|s| match s {
        Structure::Terminal(st) => Some(st),
        _ => None,
    }).collect()
}

// pub fn get_my(structure_type: Structure) -> Vec<Structure> {
//     get_my_rooms()
//     .iter()
//     .flat_map(|r| r.find(find::STRUCTURES))
//     .filter_map(|s| -> Option<StructureContainer> {match s {
//         Structure::Container(st) if structure_type == screeps::Structure::Container(st) => Some(screeps::Structure::Container(st)),
//         Structure::Container(st) => None,
//         Structure::Controller(st) if structure_type == screeps::Structure::Controller(st) => Some(st),
//         Structure::Controller(st) => None,
//         Structure::Extension(st) if structure_type == screeps::Structure::Extension(st) => Some(st),
//         Structure::Extension(st) => None,
//         Structure::Extractor(st) if structure_type == screeps::Structure::Extractor(st) => Some(st),
//         Structure::Extractor(st) => None,
//         Structure::Factory(st) if structure_type == screeps::Structure::Factory(st) => Some(st),
//         Structure::Factory(st) => None,
//         Structure::InvaderCore(st) if structure_type == screeps::Structure::InvaderCore(st) => Some(st),
//         Structure::InvaderCore(st) => None,
//         Structure::KeeperLair(st) if structure_type == screeps::Structure::KeeperLair(st) => Some(st),
//         Structure::KeeperLair(st) => None,
//         Structure::Lab(st) if structure_type == screeps::Structure::Lab(st) => Some(st),
//         Structure::Lab(st) => None,
//         Structure::Link(st) if structure_type == screeps::Structure::Link(st) => Some(st),
//         Structure::Link(st) => None,
//         Structure::Nuker(st) if structure_type == screeps::Structure::Nuker(st) => Some(st),
//         Structure::Nuker(st) => None,
//         Structure::Observer(st) if structure_type == screeps::Structure::Observer(st) => Some(st),
//         Structure::Observer(st) => None,
//         Structure::PowerBank(st) if structure_type == screeps::Structure::PowerBank(st) => Some(st),
//         Structure::PowerBank(st) => None,
//         Structure::PowerSpawn(st) if structure_type == screeps::Structure::PowerSpawn(st) => Some(st),
//         Structure::PowerSpawn(st) => None,
//         Structure::Portal(st) if structure_type == screeps::Structure::Portal(st) => Some(st),
//         Structure::Portal(st) => None,
//         Structure::Rampart(st) if structure_type == screeps::Structure::Rampart(st) => Some(st),
//         Structure::Rampart(st) => None,
//         Structure::Road(st) if structure_type == screeps::Structure::Road(st) => Some(st),
//         Structure::Road(st) => None,
//         Structure::Spawn(st) if structure_type == screeps::Structure::Spawn(st) => Some(st),
//         Structure::Spawn(st) => None,
//         Structure::Storage(st) if structure_type == screeps::Structure::Storage(st) => Some(st),
//         Structure::Storage(st) => None,
//         Structure::Terminal(st) if structure_type == screeps::Structure::Terminal(st) => Some(st),
//         Structure::Terminal(st) => None,
//         Structure::Tower(st) if structure_type == screeps::Structure::Tower(st) => Some(st),
//         Structure::Tower(st) => None,
//         Structure::Wall(st) if structure_type == screeps::Structure::Wall(st) => Some(st),
//         Structure::Wall(st) => None,
//     }}).collect::<Vec<Structure>>()
// }

pub fn get_groundscores() -> Vec<Resource> {
    get_my_rooms()
        .iter()
        .flat_map(|room| room.look_for_at_area(look::RESOURCES, 0..50, 0..50))
        .collect()
}


