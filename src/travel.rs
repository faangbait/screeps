
use crate::{util::count_bodyparts, kv};


/// calculates the fatigue required to arrive at a location
/// returns the number of ticks the creep will take to arrive there
pub fn calc_travel_fatigue(creep: &screeps::Creep, pos: &screeps::Position) -> u32 {

    let body_move = count_bodyparts(creep, screeps::Part::Move);
    let body_other = creep.body().len() - body_move;
    if body_move == 0 { return u32::MAX - 1; }

    // TODO: Carry does not cause fatigue unless loaded

    let search_opts = screeps::pathfinder::SearchOptions::default().plain_cost(2).swamp_cost(10);
    let search_results = screeps::pathfinder::search(&screeps::HasPosition::pos(creep), pos, 1, search_opts);
    if search_results.incomplete { return u32::MAX - 1; }

    let fatigue_cost = (search_results.cost * body_other as u32 ) + creep.fatigue();
    let fatigue_reduction_per_tick = body_move as u32 * 2;

    fatigue_cost / fatigue_reduction_per_tick
}

pub fn patrol(creep: &screeps::Creep, pos1: &screeps::Position, pos2: &screeps::Position) {
   
    match kv::get_dest(&creep) {
        Some(dest) => match dest.in_range_to(creep, 1) {
            true => match dest.in_range_to(pos1, 1) {
                // true => creep.move_to(pos2),
                // false => creep.move_to(pos1),
                true => screeps::SharedCreepProperties::move_to(creep, pos2),
                false => screeps::SharedCreepProperties::move_to(creep, pos1),
            },
            // false => creep.move_to(&dest),
            false => screeps::SharedCreepProperties::move_to(creep, &dest),
        },
        // None => creep.move_to(pos1),
        None => screeps::SharedCreepProperties::move_to(creep, pos1),
    };     
}


pub fn set_waypoint(creep: &screeps::Creep, position: &screeps::Position) -> screeps::ReturnCode {
    // creep.memory().set("waypoint", Position::from(*position));
    // creep.memory().set("waypoint", *position);
    // creep_waypoints.insert(creep.name(), *position);
    screeps::SharedCreepProperties::move_to(creep, position)
}

pub fn get_adjacent_terrain(pos: &screeps::Position) -> Vec<screeps::Terrain> {
    let terrain = screeps::game::map::get_room_terrain(pos.room_name());
    let mut tiles = vec![];
    let upper = *pos+(2,2);
    let lower = *pos-(1,1);

    for x in lower.x()..upper.x() {
        for y in lower.y()..upper.y() {
            let t = terrain.get(x,y);
            tiles.push(t);
        };
    };
    tiles
}

pub fn get_adjacent_open_spaces(pos: &screeps::Position) -> usize {
    let terrain = get_adjacent_terrain(pos)
        .iter()
        .filter(|&s| match s {
            screeps::Terrain::Wall => false,
            _ => true
        }).count();

    // constructed walls?
    terrain
}
