
use log::{debug, warn, info};
use screeps::{Part, SpawnOptions, ReturnCode, Creep, StructureSpawn, SharedCreepProperties, memory::MemoryReference, RoomObjectProperties, HasStore};

use crate::architect;

pub fn manage_spawns() {
    for spawn in screeps::game::spawns::values() {
        debug!("running spawn {}", spawn.name());
        if spawn.is_spawning() { continue; }
        if spawn.energy() < 250 { continue; }
        match get_desired_body(&spawn) {
            Some(body) => {
                match spawn.room() {
                    Some(room) => {
                        if room.energy_available() >= body.cost {
                            match spawn.spawn_creep_with_options(&body.parts, &body.name, &body.opts) {
                                ReturnCode::Ok => info!("Spawning {:?} at {:?}", &body.name, &spawn.name()),
                                ReturnCode::NoPath => info!("No place to spawn at {:?}", &spawn.name()),
                                ReturnCode::NameExists => info!("Name exists at {:?}", &spawn.name()),
                                ReturnCode::Busy => info!("{:?} busy, can't spawn", &spawn.name()),
                                // ReturnCode::NotEnough => {},
                                // ReturnCode::RclNotEnough => {},
                                // ReturnCode::GclNotEnough => {},
                                _ => warn!("Unhandled return at {:?}", &spawn.name())
                            }
                        }
                    },
                    None => warn!("Unhandled return at {:?}", &spawn.name()),
                }    
            },
            None => return,
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
            "harvester" => vec![
                Part::Move,
                Part::Work,
                Part::Work,
                Part::Work,
                Part::Work,
                Part::Work,
                Part::Work,
                Part::Move,
                Part::Move,
                ],
            "hauler" => vec![
                Part::Move, 
                Part::Move, 
                Part::Carry, 
                Part::Carry, 
                Part::Carry, 
                Part::Move, 
                Part::Carry, 
                Part::Move, 
                Part::Carry, 
                Part::Move,
                Part::Carry, 
                Part::Move,
                Part::Carry,
                Part::Move,
                ],
            "builder" => vec![
                Part::Move, 
                Part::Work, 
                Part::Carry, 
                Part::Carry, 
                Part::Move, 
                Part::Work, 
                Part::Carry, 
                Part::Move, 
                Part::Carry, 
                Part::Carry, 
                Part::Work, 
                Part::Work, 
                Part::Carry, 
                Part::Carry, 
                Part::Work, 
                ],
            _ => vec![
                Part::Move, 
                Part::Work, 
                Part::Carry, 
                Part::Carry, 
                Part::Move, 
                Part::Work, 
                Part::Carry, 
                Part::Move, 
                Part::Work, 
                Part::Carry, 
                Part::Move, 
                Part::Work, 
                Part::Carry, 
                Part::Move, 
                Part::Work, 
                ],
                
        };
        
        let cost = parts.iter().map(|p| p.cost()).sum::<u32>() + 50;
        
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


pub fn get_desired_body(spawn: &StructureSpawn) -> Option<BodyTemplate> {
    let creeps = screeps::game::creeps::values();
    let energy_cap =  spawn.room().expect("Error in room").energy_capacity_available();
    let energy_avail = spawn.room().expect("Error in room").energy_available();
    let energy_min =  300;

    let creep_cap = architect::get_sources().len();
    
    if get_by_role(&creeps, "harvester").len() < creep_cap {
        reduce_body_cost(BodyTemplate::new("harvester"), energy_avail.max(energy_min))
    } else if get_by_role(&creeps, "builder").len() < creep_cap +1 {
        reduce_body_cost(BodyTemplate::new("builder"), energy_avail.max(energy_min))
    } else if get_by_role(&creeps, "hauler").len() < creep_cap +1 {
        reduce_body_cost(BodyTemplate::new("hauler"), energy_avail.max(energy_min))
    } else if get_by_role(&creeps, "average").len() < creep_cap +1 {
        reduce_body_cost(BodyTemplate::new("average"), energy_cap)
    } else { None }
}

pub fn reduce_body_cost(mut body: BodyTemplate, cap: u32) -> Option<BodyTemplate> {
    while body.cost > cap {
        body.parts.remove(body.parts.len()-1);
        body.cost = body.parts.iter().map(|p| p.cost()).sum::<u32>() + 50;
    }
    Some(body)
}

pub fn get_role(creep: &Creep) -> String {
    String::from(creep.memory().string("role").unwrap().expect("No role specified"))
}

pub fn get_by_role<'a>(creeps: &'a Vec<Creep>, role_name: &'a str) -> Vec<&'a Creep>  {

    creeps
        .iter()
        .filter(|&c| c.my())
        .filter(|&c| !c.spawning())
        .filter(|&c| {
            c.memory().string("role")
                .unwrap_or(Some("notamatch".to_string()))
                .unwrap_or("notamatch".to_string()) == role_name.to_string()
            })
        .collect::<Vec<&Creep>>()

}
        
//         .filter_map(|c| String::from(c.memory().string("role")
//         .filter(|c|  == role_name.to_string())
//         .collect::<Vec<Creep>>()
// }
