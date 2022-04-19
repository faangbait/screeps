use screeps::RoomObjectProperties;


pub fn count_bodyparts(creep: &screeps::Creep, part: screeps::Part) -> usize {
    creep.body().iter().filter(|&bp| bp.part == part).count()
}

/// Decays a value given its age
pub fn time_decay(val: f64, age: f64, mut gravity: Option<f64>, ) -> f64 {
    if gravity.is_none() {
        gravity = Some(0.2);
    }
    
    val / (2. + age).powf(gravity.unwrap())
}

/// Indexes the value between 0 and 1
pub fn time_decay_index(age: f64, gravity: Option<f64>) -> f64 {
    1. / time_decay(1., age, gravity)
}

pub fn position_to_room(pos: &screeps::Position) -> Option<screeps::Room> {
    let lookres = pos.look();
    for l in lookres {
        match l {
            screeps::LookResult::Terrain(x) => {},
            screeps::LookResult::Creep(x) => return x.room(),
            screeps::LookResult::Energy(x) => return x.room(),
            screeps::LookResult::Resource(x) => return x.room(),
            screeps::LookResult::Source(x) => return x.room(),
            screeps::LookResult::Mineral(x) => return x.room(),
            screeps::LookResult::Deposit(x) => return x.room(),
            screeps::LookResult::Structure(x) => return x.room(),
            screeps::LookResult::Flag(x) => return x.room(),
            screeps::LookResult::ConstructionSite(x) => return x.room(),
            screeps::LookResult::Nuke(x) => return x.room(),
            screeps::LookResult::Tombstone(x) => return x.room(),
            screeps::LookResult::PowerCreep(x) => return x.room(),
            screeps::LookResult::Ruin(x) => return x.room(),
        }
    };

    None
}
