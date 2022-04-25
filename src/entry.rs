use std::collections::HashSet;

use log::{info, debug, warn};
// use screeps::game::cpu::HeapStatistics;

use crate::filters;

// pub struct Usage {
//     limit: u32,
//     tick_limit: u32,
//     bucket: u32,
//     heap: HeapStatistics
// }

// impl Usage {
//     pub fn new() -> Self { Self {
//         limit: screeps::game::cpu::limit(),
//         tick_limit: screeps::game::cpu::tick_limit(),
//         bucket: screeps::game::cpu::bucket(),
//         heap: screeps::game::cpu::get_heap_statistics()
//     } }
// }

// pub fn show_usage(cpu: &Usage) {
//     // info!("| {:.2}/{:?}({:?}) | {:?} | {:?} ", screeps::game::cpu::get_used(), cpu.tick_limit, cpu.bucket, cpu.heap.peak_malloced_memory, screeps::game::time());
// }
pub fn init() -> (
    Vec<screeps::Room>,
    Vec<screeps::Creep>,
    Vec<screeps::StructureSpawn>,
    Vec<screeps::Structure>,
    Vec<screeps::ConstructionSite>,
    Vec<screeps::objects::Resource>,
    Vec<screeps::Flag>,
    Vec<screeps::Source>,
    Vec<screeps::StructureController>,
){
    // let cpu = Usage::new();
    // show_usage(&cpu);
    return (
        screeps::game::rooms::values(),
        screeps::game::creeps::values(),
        screeps::game::spawns::values(),
        screeps::game::structures::values(),
        screeps::game::construction_sites::values(),
        filters::get_groundscores(),
        screeps::game::flags::values(),
        filters::get_my_sources(),
        filters::get_my_controllers(),
    )
    // screeps::game::gcl::level()
    // screeps::game::gpl::level()
    // screeps::game::map
    // screeps::game::market::
    // screeps::game::power_creeps
    // screeps::game::shards::name()
    
    // show_usage(&cpu);

}

pub fn endstep() {
    let time = screeps::game::time();

    if time % 32 == 3 {
        info!("running memory cleanup");
        cleanup_memory().expect("expected Memory.creeps format to be a regular memory object");
    }

    debug!("done! cpu: {}", screeps::game::cpu::get_used());

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
