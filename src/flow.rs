use std::collections::HashSet;
use priority_queue::PriorityQueue;

use log::{debug, info, warn};
use screeps::{Creep, SharedCreepProperties, HasStore, ResourceType, Part, find::{self, MY_STRUCTURES}, RoomObjectProperties, HasPosition, Source, ReturnCode, Structure, Attackable, look, Room, LookResult};

pub fn start_loop() {
    debug!("top of loop");
}


pub fn prioritize_actions() {
    let construction = screeps::game::construction_sites::values();
    let creeps = screeps::game::creeps::values();
    let structures = screeps::game::structures::values();
    let my_structures = screeps::Room::find(&creeps[0].room().unwrap(),MY_STRUCTURES);
    // semaphores:
    //      "pathing"
    //      "busy"

    let my_creeps = creeps.iter()
        .filter(|&creep| creep.my() && !creep.spawning())
        .collect::<Vec<&Creep>>();

    let waiting_creeps = creeps.iter()
        .filter(|&creep| !creep.memory().bool("busy") && !creep.memory().bool("pathing"))
        .collect::<Vec<&Creep>>();

    let active_creeps = creeps.iter()
        .filter(|&creep| creep.memory().bool("busy") || creep.memory().bool("pathing"))
        .collect::<Vec<&Creep>>();

    let harvesters = creeps.iter()
        .filter(|&creep| creep.store_capacity(Some(ResourceType::Energy)) > 0)
        .collect::<Vec<&Creep>>();

    let builders = creeps.iter()
        .filter(|&creep| creep.store_used_capacity(Some(ResourceType::Energy)) > 0)
        .filter(|&creep| creep.body().iter().any(| bodypart| Part::Work == bodypart.clone().part))
        .collect::<Vec<&Creep>>();

        
    let sources = get_sources_here(&creeps[0]);

    for (idx, &creep) in my_creeps.iter()
        .filter(|&&creep| creep.memory().string("role").unwrap() == Some(String::from("harvester")))
        .enumerate() {
            let target_source = &sources[idx % sources.len()];
            if creep.pos().is_near_to(target_source) {
                let r = creep.harvest(target_source);
                if r != ReturnCode::Ok {
                    warn!("couldn't harvest: {:?}", r);
                }
            } else {
                creep.move_to(target_source);
            }
    }
    
    for (idx, &creep) in my_creeps.iter()
        .filter(|&&creep| creep.memory().string("role").unwrap() == Some(String::from("hauler")))
        .enumerate() {
            
            if creep.store_used_capacity(Some(ResourceType::Energy)) > 0 {
                if let Some(c) = creep.room().expect("room is not visible to you").controller() {
                    if creep.pos().in_range_to(&c,6) {
                        creep.drop(ResourceType::Energy, Some(creep.store_used_capacity(Some(ResourceType::Energy))));
                    } else {
                        creep.move_to(&c);
                    }
                }
            }else {
                let mut resources = creep.pos().find_in_range(find::DROPPED_RESOURCES, 2500);
                resources.sort_unstable_by_key(|res| creep.pos().get_range_to(res));
                resources.reverse();
                let target_source = &resources[0];
                creep.move_to(target_source);
            }
    }
    
    for (idx, &creep) in my_creeps.iter()
        .filter(|&&creep| creep.memory().string("role").unwrap() == Some(String::from("average")))
        .enumerate() {
            if creep.store_used_capacity(Some(ResourceType::Energy)) > 0 {
                if construction.len() > 0 {
                    if creep.pos().is_near_to(&construction[0]) {
                        creep.build(&construction[0]);
                    } else {
                        creep.move_to(&construction[0]);
                    }
                } else {
                    if let Some(c) = creep.room().expect("room is not visible to you").controller() {
                        let r = creep.upgrade_controller(&c);
                        if r == ReturnCode::NotInRange {
                        creep.move_to(&c);
                    } else if r == ReturnCode::NoBodypart {
                        creep.drop(ResourceType::Energy, Some(creep.store_used_capacity(Some(ResourceType::Energy))));
                    }
                }
            }
        } else {
            if let Some(target_source) = creep.pos().find_closest_by_range(find::DROPPED_RESOURCES) {
                if target_source.pos().is_near_to(creep) {
                    creep.pickup(&target_source);
                } else {
                    creep.move_to(&target_source);
                }
            } else {
                warn!("Nothing to do...");
            }
        }
    }
}

pub fn get_sources_here(creep: &Creep) -> Vec<Source> {
    let mut sources = creep.room().expect("room is not visible to you").find(find::SOURCES);
    sources.sort_unstable_by_key(|s| {
        s.pos().get_range_to(creep);
    });
    sources
}

pub fn end_loop() {}

pub fn manage_memory() {
    let time = screeps::game::time();

    if time % 32 == 3 {
        info!("running memory cleanup");
        cleanup_memory().expect("expected Memory.creeps format to be a regular memory object");
    }

}

fn cleanup_memory() -> Result<(), Box<dyn std::error::Error>> {
    let alive_creeps: HashSet<String> = screeps::game::creeps::keys().into_iter().collect();

    let screeps_memory = match screeps::memory::root().dict("creeps")? {
        Some(v) => v,
        None => {
            warn!("not cleaning game creep memory: no Memory.creeps dict");
            return Ok(());
        }
    };

    for mem_name in screeps_memory.keys() {
        if !alive_creeps.contains(&mem_name) {
            debug!("cleaning up creep memory of dead creep {}", mem_name);
            screeps_memory.del(&mem_name);
        }
    }

    Ok(())
}
