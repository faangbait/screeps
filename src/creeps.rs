use screeps::HasPosition;

use crate::jobs::JobProperties;

pub fn init() {
    
}

pub fn get_nearest_harvestable(creep: screeps::Creep, sources: Vec<screeps::Source>) -> Option<screeps::Source> {
    match sources.iter()
    .filter(|&s| {
        s.pos().find_in_range(screeps::find::MY_CREEPS, 1).iter().fold(0, |acc, cur| {
            acc + cur.count_bp_vec(vec![screeps::Part::Work])[0]
        }) < 5
    }).min_by_key(|&s| creep.pos().get_range_to(&s.pos()))  {
        Some(s) => Some(s.to_owned()),
        None => None,
    }
}

