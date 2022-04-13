use std::{thread::spawn, collections::HashMap};

use log::{debug, warn, info};
use screeps::{HasStore, Part, SpawnOptions, ReturnCode, Creep, ResourceType, Spawning, StructureSpawn, SharedCreepProperties, mem_get, memory::MemoryReference};

use crate::flow::get_sources_here;

pub fn manage_spawns() {
    for spawn in screeps::game::spawns::values() {
        debug!("running spawn {}", spawn.name());
        if spawn.is_spawning() { continue; }
        let body = get_desired_body(&spawn);
        if spawn.energy() >= body.cost {
            let res = spawn.spawn_creep_with_options(&body.parts, &body.name, &body.opts);
            if res != ReturnCode::Ok {
                warn!("couldn't spawn: {:?}", res);
            }
        }
    }
}

pub struct BodyTemplate {
    name: String,
    parts: Vec<Part>,
    cost: u32,
    opts: SpawnOptions
}

impl BodyTemplate {
    fn new(templ:&str) -> BodyTemplate {
        let parts = match templ {
            "harvester" => vec![Part::Move, Part::Work, Part::Work],
            "hauler" => vec![Part::Move, Part::Work, Part::Carry, Part::Carry],
            _ => vec![Part::Move, Part::Move, Part::Carry, Part::Work]
        };
        
        let cost = parts.iter().map(|p| p.cost()).sum();
        let mem = MemoryReference::new();
        mem.set("role", templ.to_string());

        return BodyTemplate { 
            name: String::from(format!("{}-{}",templ,screeps::game::time())),
            parts,
            cost,
            opts: SpawnOptions::new().memory(mem)
        };
    }
}


pub fn get_desired_body(spawn: &StructureSpawn) -> BodyTemplate {
    let creeps = screeps::game::creeps::values();
   
    // creeps.iter().for_each(|creep| creep.body().iter().for_each(|bodypart| parts.push(bodypart.part)));
    // let useful_parts = parts.iter().filter(|&part| *part != Part::Move).collect::<Vec<&Part>>();
    // let total_parts = parts.len();

    // let work_ratio = total_parts / 3;
    // let carry_ratio = total_parts / 3;
    // let attack_ratio = total_parts / 6;
    // let heal_ratio = total_parts / 6;
    // let work_parts = parts.iter().filter(|&part| *part == Part::Work).collect::<Vec<&Part>>();
    // let carry_parts = parts.iter().filter(|&part| *part == Part::Carry).collect::<Vec<&Part>>();
    // let attack_parts = parts.iter().filter(|&part| *part == Part::Attack).collect::<Vec<&Part>>();
    // let heal_parts = parts.iter().filter(|&part| *part == Part::Heal).collect::<Vec<&Part>>();

    // let max_body_size = spawn.store_capacity(Some(ResourceType::Energy)) - 50;
    // let min_body_size = max_body_size;
    // let mut new_body = vec![];
    // let mut new_body_size = max_body_size;

    let harvesters = creeps.iter().filter(|&creep| {
        creep.memory().string("role").unwrap() == Some(String::from("harvester"))
    }).collect::<Vec<&Creep>>();
    
    let haulers = creeps.iter().filter(|&creep| {
        creep.memory().string("role").unwrap() == Some(String::from("hauler"))
    }).collect::<Vec<&Creep>>();
    
    if harvesters.len() < get_sources_here(&creeps[0]).len() {
        BodyTemplate::new("harvester")
    } else if haulers.len() < harvesters.len() {
        BodyTemplate::new("hauler")
    } else {
        BodyTemplate::new("average")
    }
    
}
