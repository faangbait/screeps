
use screeps::{Creep, RoomObjectProperties, Room, find::{MY_CONSTRUCTION_SITES, STRUCTURES, self}, ConstructionSite, Structure, Source, look, Resource, ResourceType, StructureController, HasPosition, StructureTower};


pub fn get_rooms() -> Vec<Room>{
    let mut rooms = screeps::game::creeps::values()
    .iter()
    .filter_map(|creep| creep.room())
    .collect::<Vec<Room>>();

    rooms.dedup_by_key(|room| room.name());
    rooms
}

pub fn get_my_rooms() -> Vec<Room>{
    let mut rooms = screeps::game::spawns::values()
    .iter()
    .filter_map(|spawn| spawn.room())
    .collect::<Vec<Room>>();

    rooms.dedup_by_key(|room| room.name());
    rooms
}

pub fn get_all_structures() -> Vec<Structure> {
   get_my_rooms()
        .iter()
        .flat_map(|room| room.find(STRUCTURES))
        .collect::<Vec<Structure>>()
}

pub fn get_my_repairables() -> Vec<Structure> {
    let mut repairables = get_my_rooms()
    .iter()
    .flat_map(|room| room.find(find::STRUCTURES))
    .filter(|s| s.as_can_decay().map(|st| st.ticks_to_decay() > 0).unwrap_or(false))
    .filter(|s| s.as_attackable().map(|st| st.hits_max() > st.hits() + 50).unwrap_or(true))
    .collect::<Vec<Structure>>();

    repairables.sort_by_key(|s| s.as_attackable().map(|st| st.hits() as i32 - st.hits_max() as i32).unwrap_or(999999));
    repairables
}

pub fn get_my_buildables() -> Vec<ConstructionSite> {
    get_my_rooms()
        .iter()
        .flat_map(|room| room.find(MY_CONSTRUCTION_SITES))
        .collect::<Vec<ConstructionSite>>()
}

pub fn get_sources() -> Vec<Source> {
    get_my_rooms()
        .iter()
        .flat_map(|room| room.find(find::SOURCES))
        .collect::<Vec<Source>>()
}

pub fn get_groundscores() -> Vec<Resource> {
    let mut gs = get_my_rooms()
        .iter()
        .flat_map(|room| room.look_for_at_area(look::RESOURCES, 0..50, 0..50))
        .collect::<Vec<Resource>>();
    gs.sort_by_key(|r| r.amount());
    gs.reverse();
    gs
}

pub fn get_my_controllers() -> Vec<StructureController>{
    get_my_rooms()
        .iter()
        .flat_map(|room| room.controller())
        .collect::<Vec<StructureController>>()
}

pub fn get_wd_containers() -> Vec<Structure> {
    get_my_rooms()
        .iter()
        .flat_map(|room| room.find(find::STRUCTURES))
        .filter_map(|s| match s {
            Structure::Container(_) => Some(s),
            Structure::Storage(_) => Some(s),
            _ => None,
        })
        .collect::<Vec<Structure>>()
}

pub fn get_deposit_containers() -> Vec<Structure> {
    get_my_rooms()
        .iter()
        .flat_map(|room| room.find(find::STRUCTURES))
        .filter_map(|s| match s {
            Structure::Container(_)
            | Structure::Extension(_)
            | Structure::Spawn(_)
            | Structure::Storage(_)
            | Structure::Tower(_) => Some(s),
            _ => None,
        })
        .collect::<Vec<Structure>>()
}

pub fn get_deposit_containers_e() -> Vec<Structure> {
    get_deposit_containers()
    .iter()
    .filter(|&s| s.as_has_store().map(|st| st.store_free_capacity(Some(ResourceType::Energy)) > 0).unwrap_or(false))
    .map(|s| s.to_owned())
    .collect::<Vec<Structure>>()
}

pub fn get_deposit_containers_f() -> Vec<Structure> {
    get_deposit_containers()
    .iter()
    .filter(|&s| s.as_has_store().map(|st| st.store_used_capacity(Some(ResourceType::Energy)) == st.store_capacity(Some(ResourceType::Energy))).unwrap_or(false))
    .map(|s| s.to_owned())
    .collect::<Vec<Structure>>()
}

pub fn get_wd_containers_e() -> Vec<Structure> {
    get_wd_containers()
    .iter()
    .filter(|&s| s.as_has_store().map(|st| st.store_free_capacity(Some(ResourceType::Energy)) > 0).unwrap_or(false))
    .map(|s| s.to_owned())
    .collect::<Vec<Structure>>()
}

pub fn get_wd_containers_f() -> Vec<Structure> {
    get_wd_containers()
    .iter()
    .filter(|&s| s.as_has_store().map(|st| st.store_used_capacity(Some(ResourceType::Energy)) > 0).unwrap_or(false))
    .map(|s| s.to_owned())
    .collect::<Vec<Structure>>()
}

pub fn get_my_containers() -> Vec<Structure> {
    get_my_rooms()
        .iter()
        .flat_map(|room| room.find(find::STRUCTURES))
        .filter(|s| s.as_has_store().map(|st| st.store_capacity(Some(ResourceType::Energy)) > 0).unwrap_or(false))
        .collect::<Vec<Structure>>()
}

pub fn get_unfull_containers() -> Vec<Structure> {
    get_my_containers()
        .iter()
        .filter(|&s| s.as_has_store().map(|st| st.store_free_capacity(Some(ResourceType::Energy)) > 0).unwrap_or(false))
        .map(|s| s.to_owned())
        .collect::<Vec<Structure>>()
}

pub fn get_full_containers() -> Vec<Structure> {
    get_my_containers()
    .iter()
    .filter(|&s| s.as_has_store().map(|st| st.store_used_capacity(Some(ResourceType::Energy)) == st.store_capacity(Some(ResourceType::Energy))).unwrap_or(false))
    .map(|s|s.to_owned())
    .collect::<Vec<Structure>>()
}

pub fn get_towers() -> Vec<StructureTower> {
    get_my_rooms()
    .iter()
    .flat_map(|room| room.find(find::STRUCTURES))
    .filter_map(|s| match s {
        Structure::Tower(st) => Some(st),
        _ => None
    }).collect::<Vec<StructureTower>>()
}
// pub fn nearest_controller(creep: &Creep, controllers: Vec<StructureController>) -> Option<StructureController> {
//     match creep.room() {
//         Some(r) => r.controller(),
//         None => None,
//     }
// }

// pub fn nearest_structure(creep: &Creep, locations: Vec<Structure>) -> Option<&Structure> {
//     locations.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));
//     locations.first()
// }

// pub fn nearest_resource(creep: &Creep, resources: Vec<Resource>) -> Option<&Resource> {
//     resources.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));
//     // if resources.len() > 0 { Some(resources[0]) } else { None }
//     resources.first()
// }

// pub fn nearest_construction_site(creep: &Creep, sites: Vec<ConstructionSite>) -> Option<&ConstructionSite> {
//     sites.sort_by_key(|loc| creep.pos().get_range_to(&loc.pos()));
//     // if sites.len() > 0 { Some(sites[0]) } else { None }
//     sites.first()
// }
