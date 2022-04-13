
use log::{debug, warn, info};
use screeps::{HasStore, Part, SpawnOptions, ReturnCode, Creep, ResourceType, Spawning, StructureSpawn, SharedCreepProperties, mem_get, memory::MemoryReference, RoomObjectProperties};

use crate::flow::get_sources_here;

pub fn manage_spawns() {
    for spawn in screeps::game::spawns::values() {
        debug!("running spawn {}", spawn.name());
        if spawn.is_spawning() { continue; }
        let body = get_desired_body(&spawn);
        if spawn.energy() >= body.cost + 50 {
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
            "harvester" => vec![Part::Move, Part::Work, Part::Work, Part::Work, Part::Work, Part::Work, Part::Work, Part::Work, Part::Work],
            "hauler" => vec![Part::Move, Part::Move, Part::Carry, Part::Carry, Part::Move, Part::Carry, Part::Move, Part::Carry, Part::Move],
            "builder" => vec![Part::Move, Part::Work, Part::Carry, Part::Carry, Part::Move, Part::Work, Part::Carry, Part::Carry],
            _ => vec![Part::Move, Part::Move, Part::Carry, Part::Work, Part::Move, Part::Move, Part::Carry, Part::Work]
        };
        
        let cost = parts.iter().map(|p| p.cost()).sum();
        let mem = MemoryReference::new();
        mem.set("role", templ.to_string());

        return BodyTemplate { 
            name: String::from(format!("{}-{}",templ,screeps::game::time())),
            parts,
            cost,
            opts: SpawnOptions::new().memory(mem).directions(&[screeps::Direction::Left, screeps::Direction::BottomLeft, screeps::Direction::TopLeft])
        };
    }
}


pub fn get_desired_body(spawn: &StructureSpawn) -> BodyTemplate {
    let creeps = screeps::game::creeps::values();
    // let energy_cap =  spawn.room().expect("Error in room").energy_capacity_available();
    let energy_cap =  250;

    let harvesters = creeps.iter().filter(|&creep| {
        creep.memory().string("role").unwrap() == Some(String::from("harvester"))
    }).collect::<Vec<&Creep>>();
    
    let haulers = creeps.iter().filter(|&creep| {
        creep.memory().string("role").unwrap() == Some(String::from("hauler"))
    }).collect::<Vec<&Creep>>();

    let builders = creeps.iter().filter(|&creep| {
        creep.memory().string("role").unwrap() == Some(String::from("builder"))
    }).collect::<Vec<&Creep>>();
    
    if harvesters.len() < get_sources_here(&creeps[0]).len() {
        reduce_body_cost(BodyTemplate::new("harvester"), energy_cap)
    } else if builders.len() < harvesters.len() {
        reduce_body_cost(BodyTemplate::new("builder"), energy_cap)
    } else if haulers.len() < harvesters.len() {
        reduce_body_cost(BodyTemplate::new("hauler"), energy_cap)
    } else {
        reduce_body_cost(BodyTemplate::new("average"), energy_cap)
    }
}

pub fn reduce_body_cost(mut body: BodyTemplate, cap: u32) -> BodyTemplate {
    while body.cost > cap {
        body.parts.remove(body.parts.len()-1);
        body.cost = body.parts.iter().map(|p| p.cost()).sum();
    }

    body
}

pub fn get_role(creep: &Creep) -> String {
    String::from(creep.memory().string("role").unwrap().expect("No role specified"))
}
