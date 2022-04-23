use log::warn;
use screeps::{RoomTerrain, Room, PositionedLookResult};

pub trait Caching {
    fn load_cache(room: &Room, path: &str) -> Option<Vec<u8>>;
    fn save_cache(room: &Room, path: &str) -> Vec<u8>;
}

impl Caching for RoomTerrain {
    fn load_cache(room: &Room, path: &str) -> Option<Vec<u8>> {
        let mem = screeps::memory::root();
        let mut path = "terrain.".to_string();
        path.push_str(&room.name().to_string());
        mem.path_arr(&path).ok().unwrap_or(None)
    }
    fn save_cache(room: &Room, path: &str) -> Vec<u8> {
        let buffer = room.get_terrain().get_raw_buffer();
        let mem = screeps::memory::root();
        let mut path = "terrain.".to_string();
        path.push_str(&room.name().to_string());
        let serialized = serde_json::to_vec(&buffer);
        match serialized {
            Ok(k) => {mem.path_set(&path, k);},
            Err(e) => { warn!("Serialization error: load_cache for RoomTerrain : {:?}", e.classify()); },
        }

        buffer
    }
}

impl Caching for PositionedLookResult {
    fn load_cache(room: &Room, path: &str) -> Option<Vec<PositionedLookResult>> {
        let mem = screeps::memory::root();
        let mut path = "lookresult.".to_string();
        path.push_str(&room.name().to_string());
        mem.path_arr(&path).ok().unwrap_or(None)
    }
    fn save_cache(room: &Room, path: &str) -> Vec<PositionedLookResult> {
        let buffer = room.look_at_area(0,0,50,50,);
        let mem = screeps::memory::root();
        let mut path = "lookresult.".to_string();
        path.push_str(&room.name().to_string());
        let serialized = serde_json::to_string(&buffer);
        match serialized {
            Ok(k) => {mem.path_set(&path, k);},
            Err(e) => { warn!("Serialization error: load_cache for RoomTerrain : {:?}", e.classify()); },
        }

        buffer
    }
}
pub fn load_room_terrain(room: &Room) -> Vec<u8> {
    let mut path = "terrain.".to_string();
    path.push_str(&room.name().to_string());
    
    match RoomTerrain::load_cache(room, &path) {
        Some(s) => { s },
        None => RoomTerrain::save_cache(room, &path),
    }
}

