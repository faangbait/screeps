pub fn get_my_rooms() -> Vec<screeps::Room> {
    let mut rooms: Vec<screeps::Room> = screeps::game::spawns::values()
    .iter()
    .filter_map(|spawn| screeps::RoomObjectProperties::room(spawn))
    .collect();

    rooms.dedup_by_key(|room| room.name());
    rooms
}

pub fn get_hostility(room: &screeps::Room) -> (
    Vec<screeps::Creep>,
    Vec<screeps::PowerCreep>,
    Vec<screeps::StructureSpawn>,
    Vec<screeps::OwnedStructure>,
    Vec<screeps::ConstructionSite>
) {
    (
        room.find(screeps::find::HOSTILE_CREEPS),
        room.find(screeps::find::HOSTILE_POWER_CREEPS),
        room.find(screeps::find::HOSTILE_SPAWNS),
        room.find(screeps::find::HOSTILE_STRUCTURES),
        room.find(screeps::find::HOSTILE_CONSTRUCTION_SITES),
    )
}

pub fn get_my_structures() -> Vec<screeps::Structure> {
    get_my_rooms()
         .iter()
         .flat_map(|room| room.find(screeps::find::STRUCTURES))
         .collect()
 }

 pub fn get_my_sources() -> Vec<screeps::Source> {
    let mut sources = get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::SOURCES))
    .collect::<Vec<screeps::Source>>();

    sources.sort_by_key(|st|screeps::HasId::id(st));
    sources
}

pub fn get_my_spawns() -> Vec<screeps::StructureSpawn> {
    screeps::game::spawns::values()
}

// pub fn get_sinks() -> Vec<dyn Sink> {
//     todo!();
// }

// pub fn get_sources() -> Vec<dyn Source> {
//     todo!();
// }

use screeps::SharedCreepProperties;


pub fn get_my_containers() -> Vec<screeps::StructureContainer> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::STRUCTURES))
    .filter_map(|s| match s {
        screeps::Structure::Container(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_extensions() -> Vec<screeps::StructureExtension> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::STRUCTURES))
    .filter_map(|s| match s {
        screeps::Structure::Extension(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_towers() -> Vec<screeps::StructureTower> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::STRUCTURES))
    .filter_map(|s| match s {
        screeps::Structure::Tower(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_storages() -> Vec<screeps::StructureStorage> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::STRUCTURES))
    .filter_map(|s| match s {
        screeps::Structure::Storage(st) => Some(st),
        _ => None,
    }).collect()
}
pub fn get_my_repairables() -> Vec<screeps::Structure> {
    get_my_rooms()
    .iter()
    .flat_map(|room| room.find(screeps::find::STRUCTURES))
    .filter(|s| s.as_can_decay().is_some())
    .filter(|s| s.as_attackable().map(|st| st.hits_max() > st.hits() + 1000).unwrap_or_else(||true))
    .collect()

}

pub fn get_my_buildables() -> Vec<screeps::ConstructionSite> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::CONSTRUCTION_SITES))
    .collect()
}

pub fn get_my_controllers() -> Vec<screeps::StructureController> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::STRUCTURES))
    .filter_map(|s| match s {
        screeps::Structure::Controller(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_factories() -> Vec<screeps::StructureFactory> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::STRUCTURES))
    .filter_map(|s| match s {
        screeps::Structure::Factory(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_labs() -> Vec<screeps::StructureLab> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::STRUCTURES))
    .filter_map(|s| match s {
        screeps::Structure::Lab(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_links() -> Vec<screeps::StructureLink> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::STRUCTURES))
    .filter_map(|s| match s {
        screeps::Structure::Link(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_nukers() -> Vec<screeps::StructureNuker> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::STRUCTURES))
    .filter_map(|s| match s {
        screeps::Structure::Nuker(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_powerspawns() -> Vec<screeps::StructurePowerSpawn> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::STRUCTURES))
    .filter_map(|s| match s {
        screeps::Structure::PowerSpawn(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_terminals() -> Vec<screeps::StructureTerminal> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::STRUCTURES))
    .filter_map(|s| match s {
        screeps::Structure::Terminal(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_groundscores() -> Vec<screeps::objects::Resource> {
    get_my_rooms()
        .iter()
        .flat_map(|room| room.look_for_at_area(screeps::look::RESOURCES, 0..50, 0..50))
        .collect()
}


pub fn get_my_roads() -> Vec<screeps::StructureRoad> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::STRUCTURES))
    .filter_map(|s| match s {
        screeps::Structure::Road(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_walls() -> Vec<screeps::StructureWall> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::STRUCTURES))
    .filter_map(|s| match s {
        screeps::Structure::Wall(st) => Some(st),
        _ => None,
    }).collect()
}

pub fn get_my_ramparts() -> Vec<screeps::StructureRampart> {
    get_my_rooms()
    .iter()
    .flat_map(|r| r.find(screeps::find::STRUCTURES))
    .filter_map(|s| match s {
        screeps::Structure::Rampart(st) => Some(st),
        _ => None,
    }).collect()
}

pub(crate) fn get_my_creeps() -> Vec<screeps::Creep> {
    screeps::game::creeps::values()
    .iter()
    .filter(|&c| c.my() && !c.spawning())
    .map(|c| c.to_owned())
    .collect()
}
