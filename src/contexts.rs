use priority_queue::PriorityQueue;

use crate::jobs::{JobProperties, JobType};


#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct Context {
    c_thread_id: i32,
    target_id: screeps::RawObjectId,
    t_job_type: JobType,
    world_pos: screeps::Position,
    time_remaining: u32,
    job_contribution: u32,
    start_tick: u32,
    finish_tick: u32
}

impl Context {
    pub fn new(creep: &screeps::Creep, target: &dyn screeps::HasId, job_type: JobType) -> Option<Self> { 
        let c_thread_id;
        let target_id = target.untyped_id();
        let t_job_type = job_type;
        let world_pos = target.pos();
        let runtime = creep.job_runtime(target, job_type);

        let current = screeps::game::time();

        let time_remaining = runtime.0.checked_add(runtime.1).unwrap_or(u32::MAX);
        let job_contribution = runtime.2;

        let start_tick = current.checked_add(runtime.0).unwrap_or(u32::MAX);
        let finish_tick = start_tick.checked_add(runtime.1).unwrap_or(u32::MAX);

        // Get the c_thread_id from memory
        let mem = screeps::SharedCreepProperties::memory(creep);
        let c_thread_id_res = mem.i32("c_thread_id");

        if !creep.has_parts_for_job(job_type) { return None };

        match c_thread_id_res {
            Ok(okval) => match okval {
                Some(cti) => c_thread_id = cti,
                None => return None,
            },
            Err(_) => return None,
        };
        
        if c_thread_id > 0 && time_remaining > 0 {
            Some(Self {
                c_thread_id,
                target_id,
                t_job_type,
                world_pos,
                time_remaining,
                job_contribution,
                start_tick,
                finish_tick
            })
        } else { None }
    }
}

pub struct ContextList {
    pub queue: PriorityQueue<Context, i32>,
}

impl ContextList {
    pub fn new() -> Self { Self { queue: PriorityQueue::<Context, i32>::new() } }
    pub fn add(mut self: Self, context: Context) -> Option<i32>{
        self.queue.push(context, -1 * context.time_remaining as i32)
    }

}
