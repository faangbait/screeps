
// pub fn get_shortest_distance(start_position: &Position, ending_positions: Vec<<T>, <u32>>) -> pathfinder::SearchResults {
//     let opts = pathfinder::SearchOptions::default();
//     let results = pathfinder::search_many(&start_position, ending_positions.into_iter(), opts);
//     return results
// }

use screeps::{Creep, find, Source, Position, HasPosition, pathfinder, RoomObjectProperties};

pub fn get_sources_here(creep: &Creep) -> Vec<Source> {
    creep.room().expect("room is not visible to you").find(find::SOURCES)
}

pub fn get_shortest_path(start: Position, ends: Vec<Position>) -> pathfinder::SearchResults {
    let opts = pathfinder::SearchOptions::new().swamp_cost(10).plain_cost(2);
    pathfinder::search_many(&start, ends.into_iter().map(|position| (position.pos(), 1)), opts)
    
}
