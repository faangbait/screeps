// use std::collections::HashMap;

// use log::{info, warn};
use screeps::{Creep, Position, SharedCreepProperties, ReturnCode};


pub fn set_waypoint(creep: &Creep, position: &Position) -> ReturnCode {
    // creep.memory().set("waypoint", Position::from(*position));
    // creep.memory().set("waypoint", *position);
    // creep_waypoints.insert(creep.name(), *position);
    creep.move_to(position)
}

// pub fn get_waypoint(creep: &Creep) -> Option<Position> {
//     match creep_waypoints.get_key_value(&creep.name()) {
//         Some(v) => Some(*v.1),
//         None => None,
//     }
// }

// pub fn arrived_at_waypoint(creep: &Creep) -> bool {
//     match get_waypoint(creep) {
//         Some(v) => { 
//             if v.is_near_to(&creep.pos()) {
//                 creep_waypoints.remove(&creep.name());
//                 true
//             } else { false } },
//         None => false,
//     }
// }

// pub fn reset_waypoint(creep: &Creep) {
//     creep.memory().del("waypoint")
// }

