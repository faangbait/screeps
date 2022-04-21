use log::{warn, info};
use screeps::{Part, HasStore, RoomObjectProperties, SpawnOptions};
use itertools::Itertools;

use crate::filters;

pub fn init() {
    let spawns = filters::get_my_spawns();
    let energy_cap = 300;

    let mut visited_rooms = vec![];
    
    for spawn in spawns {
        if visited_rooms.contains(&spawn) { continue; } else { visited_rooms.push(spawn); }

        let energy_cap = match spawn.room() {
            Some(r) => r.energy_available(),
            None => 0,
        };
        
        match construct_creep(energy_cap) {
            Some(body) => {
                let feasible = sort_body(body);
                let opts = SpawnOptions::new();
                let dry_run = spawn.spawn_creep_with_options(&body, "name", &opts.dry_run(true));
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

    body_parts
}

pub fn construct_creep(energy_cap: u32) -> Option<Vec<Part>> {
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
        .max_by_key(|body| rtb_spawn(body))
}

/// calculates the most this body would pay to have itself spawned given
/// current rtb prices
pub fn rtb_spawn(body: Vec<Part>) -> i32 {

    -5
}
