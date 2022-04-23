use log::info;
use screeps::memory::MemoryReference;
use screeps::{Part, RoomObjectProperties, SpawnOptions, StructureSpawn, HasPosition};
use itertools::Itertools;

use crate::contexts::Context;
use crate::filters;
use crate::jobs::{JobProperties, JobType};

pub fn init() {
    let spawns = filters::get_my_spawns();
    let energy_cap = 300;

    let mut visited_rooms = vec![];
    
    spawns.iter().for_each(|spawn| {
        if !visited_rooms.contains(&spawn) {
            visited_rooms.push(spawn);

            let energy_cap = match spawn.room() {
                Some(r) => r.energy_available(),
                None => 0,
            };
        
            match construct_creep(energy_cap, spawn) {
                Some(body) => {
                    let feasible = sort_body(body);
                    let opts = SpawnOptions::new();
                    let dry_run = spawn.spawn_creep_with_options(&feasible, "name", &opts.dry_run(true));
                    info!("spawn creep: {:?}", dry_run);

                    // match spawn.spawn_creep_with_options(&body, "name", &opts) {
                    //     screeps::ReturnCode::Ok => info!("Spawning creep, {:?}", body),
                    // screeps::ReturnCode::NotOwner => todo!(),
                    // screeps::ReturnCode::NoPath => todo!(),
                    // screeps::ReturnCode::NameExists => todo!(),
                    // screeps::ReturnCode::Busy => todo!(),
                    // screeps::ReturnCode::NotFound => todo!(),
                    // screeps::ReturnCode::NotEnough => todo!(),
                    // screeps::ReturnCode::InvalidTarget => todo!(),
                    // screeps::ReturnCode::Full => todo!(),
                    // screeps::ReturnCode::NotInRange => todo!(),
                    // screeps::ReturnCode::InvalidArgs => todo!(),
                    // screeps::ReturnCode::Tired => todo!(),
                    // screeps::ReturnCode::NoBodypart => todo!(),
                    // screeps::ReturnCode::RclNotEnough => todo!(),
                    // screeps::ReturnCode::GclNotEnough => todo!(),
                    //     _ => warn!("Unhandled return in spawn for body {:?}", body),
                    // }
                },
                None => {},
            }
        }
    });
}

pub fn sort_body(mut body_parts: Vec<Part>) -> Vec<Part> {
    
    body_parts.sort_by_key(|bp| {
        match bp {
            Part::Tough => 0,
            Part::Claim => 1,
            Part::Work => 2,
            Part::Carry => 3,
            Part::Attack => 4,
            Part::Move => 5,
            Part::RangedAttack => 6,
            Part::Heal => 7,
        }
    });

    body_parts.to_vec()
}

pub fn construct_creep(energy_cap: u32, spawn: &StructureSpawn) -> Option<Vec<Part>> {
    // generate a combination of feasible body parts given our max
    // energy to spend constructing this creep. 

    vec![
        Part::Move,
        Part::Work,
        Part::Carry,
        Part::Tough,
        Part::Attack,
        Part::RangedAttack,
        Part::Heal,
        Part::Claim].iter()
        .combinations_with_replacement((energy_cap / 50) as usize)
        .filter_map(|body| match body.iter()
            .fold(0, |acc, &part| acc + part.cost()) <= energy_cap {
            true => Some(body.iter().map(|&part| *part).collect::<Vec<Part>>()),
            false => None,
        })
        .max_by_key(|body| rtb_spawn(body, spawn))
}

/// calculates the most this body would pay to have itself spawned given
/// current rtb prices
pub fn rtb_spawn(body: &Vec<Part>, spawn: &StructureSpawn) -> i32 {
    0
}

pub struct BodyTemplate {
    name: String,
    body: Vec<Part>,
    cost: u32,
    opts: SpawnOptions,
    spawn: StructureSpawn
}

impl BodyTemplate {
    fn new(body: Vec<Part>, spawn: StructureSpawn) -> BodyTemplate {
        let cost = body.iter().map(|p| p.cost()).sum::<u32>() + 50;
        let mem = MemoryReference::new();

        let count = body.iter().count();

        return BodyTemplate { 
            name: String::from(format!("{}-{}-{}","creep-", count, screeps::game::time())),
            body,
            cost,
            opts: SpawnOptions::new().memory(mem),
            spawn
        };

    }
}

impl JobProperties for BodyTemplate {
    fn count_bp_vec(self: &Self, part_array: Vec<screeps::Part>) -> Vec<u32> {
        let mut res = vec![];
        for part in part_array {
            res.push(self.body.iter().filter(|&p| *p == part).count() as u32);
        }
        return res
    }

    fn has_parts_for_job(&self, job_type: crate::jobs::JobType) -> bool {
        let mut bp_reqs = vec![];

        match job_type {
            JobType::Build => { bp_reqs.extend([Part::Move, Part::Carry, Part::Work].iter()) },
            JobType::Repair => { bp_reqs.extend([Part::Move, Part::Carry, Part::Work].iter()) },
            JobType::Station => { bp_reqs.extend([Part::Move].iter()) },
            JobType::Upgrade => { bp_reqs.extend([Part::Move, Part::Carry, Part::Work].iter()) },
            JobType::Transfer => { bp_reqs.extend([Part::Move, Part::Carry].iter()) },
            JobType::Withdraw => { bp_reqs.extend([Part::Move, Part::Carry].iter()) },
            JobType::Pickup => { bp_reqs.extend([Part::Move, Part::Carry].iter()) },
            JobType::Harvest => { bp_reqs.extend([Part::Move, Part::Work].iter()) },
            JobType::Claim => { bp_reqs.extend([Part::Move, Part::Claim].iter()) },
            JobType::Reserve => { bp_reqs.extend([Part::Move, Part::Claim].iter()) },
            JobType::Attack => { bp_reqs.extend([Part::Move, Part::Attack].iter()) },
            JobType::AttackR => { bp_reqs.extend([Part::Move, Part::RangedAttack].iter()) },
            JobType::Defend => { bp_reqs.extend([Part::Attack].iter()) },
            JobType::DefendR => { bp_reqs.extend([Part::RangedAttack].iter()) },
            JobType::Heal => { bp_reqs.extend([Part::Heal].iter()) },
            JobType::Scout => { bp_reqs.extend([Part::Move].iter()) },
        }
        
        bp_reqs.iter().all(|req| self.body.iter().filter(|&p| *p == *req).count() > 0)
    }

    fn job_runtime(&self, target: &dyn screeps::HasId, job_type: crate::jobs::JobType) -> (u32, u32, u32) {
        let start_ticks = (self.body.len() * 3) as u32;
        let mut body = self.count_bp_vec(vec![
            Part::Attack,
            Part::Claim,
            Part::Heal,
            Part::RangedAttack,
            Part::Tough,
            Part::Work,
            Part::Carry,
            Part::Move,
            ]);

        let contribution_per_tick = match job_type {
            JobType::Harvest => *body.get(5).unwrap_or(&0) * 2,
            JobType::Build => *body.get(5).unwrap_or(&0) * 5,
            JobType::Repair => *body.get(5).unwrap_or(&0) * 100,
            JobType::Station => 1,
            JobType::Upgrade => *body.get(5).unwrap_or(&0) * 5, // TODO: Check this
            JobType::Transfer => *body.get(6).unwrap_or(&0) * 50,
            JobType::Withdraw => *body.get(6).unwrap_or(&0) * 50,
            JobType::Pickup => *body.get(6).unwrap_or(&0) * 50,
            JobType::Claim => *body.get(1).unwrap_or(&0),
            JobType::Reserve => *body.get(1).unwrap_or(&0),
            JobType::Attack => *body.get(0).unwrap_or(&0) * 30,
            JobType::AttackR => *body.get(3).unwrap_or(&0) * 10,
            JobType::Defend => *body.get(0).unwrap_or(&0) * 30,
            JobType::DefendR => *body.get(3).unwrap_or(&0) * 10,
            JobType::Heal => *body.get(2).unwrap_or(&0) * 12,
            JobType::Scout => *body.get(7).unwrap_or(&0).min(&1),
        };

        //TODO: Harvest duration considers energy remaining
        let job_duration = match job_type {
            JobType::Harvest => 1500,
            JobType::Build => (*body.get(6).unwrap_or(&0) * 50) / *body.get(5).unwrap_or(&0),
            JobType::Repair => (*body.get(6).unwrap_or(&0) * 50) / *body.get(5).unwrap_or(&0),
            JobType::Station => 1500,
            JobType::Upgrade => (*body.get(6).unwrap_or(&0) * 50) / *body.get(5).unwrap_or(&0), // TODO: Check this
            JobType::Transfer => 1,
            JobType::Withdraw => 1,
            JobType::Pickup => 1,
            JobType::Claim => 1,
            JobType::Reserve => 1500,
            JobType::Attack => 1500,
            JobType::AttackR => 1500,
            JobType::Defend => 1500,
            JobType::DefendR => 1500,
            JobType::Heal => 1500,
            JobType::Scout => 1500,
        };
        
        let finish_ticks = (1500 - start_ticks).min(job_duration);
        
        return (start_ticks, finish_ticks, finish_ticks * contribution_per_tick);
    }

    fn fatigue_to(&self, target: &dyn screeps::HasId) -> u32 {
        return self.fatigue_to_pos(&HasPosition::pos(target));
    }

    fn fatigue_to_pos(&self, pos: &screeps::Position) -> u32 {
        let mut body = self.count_bp_vec(vec![
            Part::Attack,
            Part::Claim,
            Part::Heal,
            Part::RangedAttack,
            Part::Tough,
            Part::Work,
            Part::Carry,
            Part::Move,
            ]);
        
        let body_move = body.pop().unwrap_or(0);
        let body_carry = body.pop().unwrap_or(0);

        if body_move == 0 { return 100000; }

        // let carry_weight = *body.get(6).unwrap_or(&0) as f32 * 50.;
        let loaded_carry = 0;
        
        // weight = how much gross fatigue is accumulated per move on road
        let weight = body.iter()
            .fold(loaded_carry.min(body_carry),|acc, cur| acc + cur);

        let ticks_road = (0.5 * weight as f32 / body_move as f32).ceil();
        let ticks_plain = (1.0 * weight as f32 / body_move as f32).ceil();
        let ticks_swamp = (5.0 * weight as f32 / body_move as f32).ceil();
        
        let heuristic = self.spawn.pos().get_range_to(pos) as f32 * ticks_road;

        let search_opts = screeps::pathfinder::SearchOptions::default()
            .plain_cost(ticks_plain as u8)
            .swamp_cost(ticks_swamp as u8)
            .heuristic_weight(heuristic.into());
        let search_results = screeps::pathfinder::search(&self.spawn.pos(), pos, 1, search_opts);
        
        if search_results.incomplete { return 100000; }
        
        //TODO Visuals

        return search_results.cost;

    }

    fn context(&self) -> Option<Context> {
        None
    }
}
