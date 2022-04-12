use std::thread::spawn;

use log::{debug, warn, info};
use screeps::{HasStore, Part, SpawnOptions, ReturnCode, Creep, ResourceType, Spawning, StructureSpawn};

pub fn manage_spawns() {
    for spawn in screeps::game::spawns::values() {
        debug!("running spawn {}", spawn.name());
        if spawn.is_spawning() { continue; }
        let body = get_desired_body(&spawn);
        if spawn.energy() >= body.iter().map(|p| p.cost()).sum() {
            let opts = SpawnOptions::new();
            let res = spawn.spawn_creep_with_options(&body, &name_unit(), &opts);
            if res != ReturnCode::Ok {
                warn!("couldn't spawn: {:?}", res);
            }
        }
    }
}

pub fn name_unit() -> String {
    let name_base = screeps::game::time();
    format!("{}", name_base)
}

pub fn get_desired_body(spawn: &StructureSpawn) -> Vec<Part> {
    let creeps = screeps::game::creeps::values();
    let mut parts = vec![];
    creeps.iter().for_each(|creep| creep.body().iter().for_each(|bodypart| parts.push(bodypart.part)));
    let useful_parts = parts.iter().filter(|&part| *part != Part::Move).collect::<Vec<&Part>>();
    let total_parts = parts.len();

    let work_ratio = total_parts / 3;
    let carry_ratio = total_parts / 3;
    let attack_ratio = total_parts / 6;
    let heal_ratio = total_parts / 6;
    let work_parts = parts.iter().filter(|&part| *part == Part::Work).collect::<Vec<&Part>>();
    let carry_parts = parts.iter().filter(|&part| *part == Part::Carry).collect::<Vec<&Part>>();
    let attack_parts = parts.iter().filter(|&part| *part == Part::Attack).collect::<Vec<&Part>>();
    let heal_parts = parts.iter().filter(|&part| *part == Part::Heal).collect::<Vec<&Part>>();

    let max_body_size = spawn.store_capacity(Some(ResourceType::Energy)) - 50;
    let min_body_size = max_body_size;
    let mut new_body = vec![];
    let mut new_body_size = max_body_size;

    for _ in 0..max_body_size / 100 { 
        new_body.push(Part::Move); 
        new_body_size -= 50;
    }

    while new_body_size > 0 {
        if work_parts.len() < work_ratio { 
            new_body.push(Part::Work);
            new_body_size -= 100;
            new_body.push(Part::Carry);
            new_body_size -= 50;
        }
    }

    return new_body;
    
    // if new_body.len() > max_body_size as usize { new_body.remove(0); }
    // if new_body.len() >= min_body_size as usize {
    //     info!["Spawning a body of length {:?}", new_body.len()];
    //     return new_body;
    // } else { return vec![] }

}
