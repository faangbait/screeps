use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use log::warn;
use screeps::RawObjectId;

use crate::jobs::{JobProperties, JobType};
use crate::rtb::JobBid;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Context {
    c_thread_id: i32,
    target_id: screeps::RawObjectId,
    t_job_type: JobType,
    world_pos: screeps::Position,
    time_remaining: u32,
    job_contribution: u32,
    start_tick: u32,
    finish_tick: u32,
    status: ContextStatus,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]

pub enum ContextStatus {
    Active  = 1,
    Stopped = 2,
    Blocked = 3,
    Waiting = 4,
    Finished= 5,
}

impl Context {
    // pub fn new(creep: &screeps::Creep, target: &dyn screeps::HasId, job_type: JobType) -> Option<Self> { 
    pub fn new(creep: &screeps::Creep, target: &dyn screeps::HasId, request: JobBid) -> Option<Self> {         
        if !creep.has_parts_for_job(request.request) { return None };

        let c_thread_id;
        let target_id = target.untyped_id();
        let t_job_type = request.request;
        let world_pos = target.pos();
        let runtime = creep.job_runtime(target, request.request);

        let current = screeps::game::time();

        let time_remaining = runtime.0.checked_add(runtime.1).unwrap_or(u32::MAX);
        let job_contribution = runtime.2;

        let start_tick = current.checked_add(runtime.0).unwrap_or(u32::MAX);
        let finish_tick = start_tick.checked_add(runtime.1).unwrap_or(u32::MAX);

        // Get the c_thread_id from memory
        let mem = screeps::SharedCreepProperties::memory(creep);
        let c_thread_id_res = mem.i32("c_thread_id");

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
                finish_tick,
                status: ContextStatus::Stopped
            })
        } else { None }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct ContextMap {
    map: HashMap<RawObjectId, Context>,
}

impl ContextMap {
    pub fn new() -> Self { Self { map: HashMap::<RawObjectId, Context>::new() } }
    pub fn create(mut self: Self, obj: &RawObjectId, context: &Context) {
        self.map.insert(*obj, *context);

        let mem = screeps::memory::root();
        let mut path = "contexts.".to_string();
        path.push_str(&obj.to_string());
        let serialized_context = serde_json::to_string(context);
        
        match serialized_context {
            Ok(k) => mem.path_set(&path, k),
            Err(e) => warn!("Serialization error: {:?}", context),
        }
    }

    pub fn read(self: &Self, obj: &RawObjectId) -> Option<Context> {
        let kv = self.map.get_key_value(obj);

        match kv {
            Some(v) => return Some(*v.1),
            None => {
                let mem = screeps::memory::root();
                let mut path = "contexts.".to_string();
                path.push_str(&obj.to_string());

                let serialized_context = mem.get_path::<String>(&path);
                
                match serialized_context {
                    Ok(k) => match k {
                        Some(context_json) => match serde_json::from_str::<Context>(&context_json) {
                            Ok(context) => return Some(context),
                            Err(e) => warn!("Deerialization error: {:?}", e),
                        },
                        None => return None,
                    },
                    Err(e) => warn!("Path not found: {:?}", e),
                };
            },
        };
        return None

    }

    pub fn update(mut self: Self, obj: &RawObjectId, context: &Context) {
        self.map.insert(*obj, *context);
        let mem = screeps::memory::root();
        let mut path = "contexts.".to_string();
        path.push_str(&obj.to_string());
        let serialized_context = serde_json::to_string(context);
        
        match serialized_context {
            Ok(k) => mem.path_set(&path, k),
            Err(e) => warn!("Serialization error: {:?}", context),
        }
    }
    pub fn delete(mut self: Self, obj: &RawObjectId) {
        self.map.remove(obj);
        let mem = screeps::memory::root();
        let mut path = "contexts.".to_string();
        mem.del(&path);
    }
}

// pub struct ContextList {
//     pub queue: PriorityQueue<Context, i32>,
// }
