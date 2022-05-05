use log::info;
use screeps::{HasPosition, Position};

pub fn get_source_flags() -> Vec<Position> {
    screeps::game::flags::values()
        .iter()
        .filter(|&f| f.name().starts_with("source"))
        .map(|f| f.pos())
        .collect::<Vec<Position>>()
}

pub fn get_claim_flags() -> Vec<Position> {
    screeps::game::flags::values()
        .iter()
        .filter(|&f| f.name().starts_with("claim"))
        .map(|f| f.pos())
        .collect::<Vec<Position>>()
}
