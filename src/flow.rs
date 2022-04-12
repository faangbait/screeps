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

    for (idx, &creep) in my_creeps.iter().enumerate() {

        if creep.memory().bool("harvesting") {
            if creep.store_free_capacity(Some(ResourceType::Energy)) == 0 {
                creep.memory().set("harvesting", false);
            }
        } else {
            if creep.store_used_capacity(None) == 0 {
                creep.memory().set("harvesting", true);
            }
        }

        if creep.memory().bool("harvesting") {
            let target_source = &sources[idx % sources.len()];
            if creep.pos().is_near_to(target_source) {
                let r = creep.harvest(target_source);
                if r != ReturnCode::Ok {
                    warn!("couldn't harvest: {:?}", r);
                }
            } else {
                creep.move_to(target_source);
            }
        } else {
            for site in &construction {
                if creep.pos().in_range_to(site, 1) {
                    creep.build(site);
                }
            }
            for structure in &my_structures {
                if creep.pos().in_range_to(structure, 1) && structure.hits() < structure.hits_max() {
                    creep.repair(structure);
                }
            }
            if let Some(c) = creep
            .room()
            .expect("room is not visible to you")
            .controller()
            {
                let r = creep.upgrade_controller(&c);
                if r == ReturnCode::NotInRange {
                    creep.move_to(&c);
                } else if r != ReturnCode::Ok {
                    warn!("couldn't upgrade: {:?}", r);
                }
            } else {
                warn!("creep room has no controller!");
            }
        }
   
        }
    }





    // my_creeps.iter()
    //     .map(|creep| *creep)
    //     .for_each(|creep| {
    //         let x = creep.pos().x() as u8;
    //         let y = creep.pos().y() as u8;
    //         let room = creep.room().unwrap();
    //         let n_sources = room.look_for_at_area(look::SOURCES, x-5..x+5, y-5..y+5);
    //         let n_sites = room.look_for_at_area(look::CONSTRUCTION_SITES, x-5..x+5, y-5..y+5);
    //         let n_structs = room.look_for_at_area(look::STRUCTURES, x-5..x+5, y-5..y+5);
    //         let n_deposits = room.look_for_at_area(look::DEPOSITS, x-5..x+5, y-5..y+5);
            
    //         if creep.store_free_capacity(Some(ResourceType::Energy)) > 5 {
    //             if creep.pos().in_range_to(&n_sources[0], 1) {
    //                 creep.harvest(&n_sources[0]);
    //             } else {
    //                 creep.move_to(&n_sources[0]);
    //             }
    //         }
            
    //         if creep.store_used_capacity(Some(ResourceType::Energy)) > 5 {
    //             if creep.pos().in_range_to(&n_sites[0], 1) {
    //                 creep.build(&n_sites[0]);
    //             } else {
    //                 creep.move_to(&n_sites[0]);
    //             }
    //         }
    //     });




    // for i in 0..harvesters.len().max(sources.len()) {
    //     let res = harvesters[i].move_to(&sources[i % sources.len()]);
    //     if res == ReturnCode::Ok {
    //     } else {
    //         warn!("couldn't harvest: {:?}", res);
    //     }
    // }


    // harvesters.iter()
    //     .filter(|creep| creep.store_free_capacity(Some(ResourceType::Energy)) < 5 )
    //     .for_each(|creep| {
    //         if let Some(c) = creep.room().expect("room is not visible to you").controller() {
    //             let res = creep.upgrade_controller(&c);
    //             if res == ReturnCode::NotInRange {
    //                 creep.move_to(&c);
    //             } else if res != ReturnCode::Ok {
    //                 warn!("couldn't upgrade: {:?}", res);
    //             } else {
    //                 if creep.store_used_capacity(Some(ResourceType::Energy)) == 0 {
    //                 } else {
    //                 }
    //             }

    //         } else { warn! ("creep room has no controller!");}
    //     });

    // sources.iter()
    //     .for_each(|source| {
    //     harvesters.iter()
    //         .filter(|&creep| creep.pos().is_near_to(source) && creep.store_free_capacity(Some(ResourceType::Energy)) > 0)
    //         .for_each(|&creep| { 
    //             creep.harvest(source); 
    //         });
    //     });

    // for &i in builders.iter().filter(|&creep| !&creep.memory().bool("busy")) {
    //     my_structures.iter()
    //     .filter(|&structure| structure.hits_max() > structure.hits())
    //     .for_each(|structure| { i.repair(structure); })
    // }
        



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
