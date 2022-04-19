use screeps::Creep;

use crate::architect;

pub fn get_hostiles() -> Vec<Creep>{
    architect::get_my_rooms()
    .iter()
    .flat_map(|room| room.find(screeps::find::HOSTILE_CREEPS))
    .collect::<Vec<Creep>>()
   
}
