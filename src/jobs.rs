use screeps::{Part, HasId, HasPosition, HasStore, SharedCreepProperties, ResourceType, RoomObjectProperties};

use crate::util::CreepCustomActions;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum JobType {
    Build,
    Repair,
    Station,
    Upgrade,
    Transfer,
    Withdraw,
    Pickup,
    Harvest,
    Claim,
    Reserve,
    Attack,
    AttackR,
    Defend,
    DefendR,
    Heal,
    Scout,
}

pub trait JobProperties {
    fn has_parts_for_job(&self, job_type: JobType) -> bool;
    fn job_runtime(&self, target: &dyn HasId, job_type: JobType) -> (u32, u32, u32);
    fn fatigue_to(&self, target: &dyn HasId) -> u32;
    fn resource_sink(&self) -> Option<ResourceType>;
    fn resource_source(&self) -> Option<ResourceType>;
}

impl JobProperties for screeps::Creep {
    fn has_parts_for_job(&self, job_type: JobType) -> bool {
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
        
        bp_reqs.iter().all(|req| self.get_active_bodyparts(*req) > 0)
    }

    fn job_runtime(&self, target: &dyn HasId, job_type: JobType) -> (u32, u32, u32) {
        let start_ticks = self.fatigue_to(target);

        let contribution_per_tick = match job_type {
            JobType::Harvest => self.get_active_bodyparts(Part::Work) * 2,
            JobType::Build => self.get_active_bodyparts(Part::Work) * 5,
            JobType::Repair => self.get_active_bodyparts(Part::Work) * 100,
            JobType::Station => 1,
            JobType::Upgrade => self.get_active_bodyparts(Part::Work) * 5, // TODO: Check this
            JobType::Transfer => self.store_used_capacity(None) as u32,
            JobType::Withdraw => self.store_free_capacity(None) as u32,
            JobType::Pickup => self.get_active_bodyparts(Part::Carry) * 50,
            JobType::Claim => self.get_active_bodyparts(Part::Claim),
            JobType::Reserve => self.get_active_bodyparts(Part::Claim),
            JobType::Attack => self.get_active_bodyparts(Part::Attack) * 30,
            JobType::AttackR => self.get_active_bodyparts(Part::RangedAttack) * 10,
            JobType::Defend => self.get_active_bodyparts(Part::Attack) * 30,
            JobType::DefendR => self.get_active_bodyparts(Part::RangedAttack) * 10,
            JobType::Heal => self.get_active_bodyparts(Part::Heal) * 12,
            JobType::Scout => 1,
        };
        //TODO: Harvest duration considers energy remaining
        let job_duration = match job_type {
            JobType::Harvest => self.ticks_to_live().unwrap_or(0),
            JobType::Build => self.store_used_capacity(Some(ResourceType::Energy)) / self.get_active_bodyparts(Part::Work),
            JobType::Repair => self.store_used_capacity(Some(ResourceType::Energy)) / self.get_active_bodyparts(Part::Work),
            JobType::Station => self.ticks_to_live().unwrap_or(0),
            JobType::Upgrade => self.store_used_capacity(Some(ResourceType::Energy)) / self.get_active_bodyparts(Part::Work), // TODO: Check this
            JobType::Transfer => 1,
            JobType::Withdraw => 1,
            JobType::Pickup => 1,
            JobType::Claim => 1,
            JobType::Reserve => self.ticks_to_live().unwrap_or(0),
            JobType::Attack => self.ticks_to_live().unwrap_or(0),
            JobType::AttackR => self.ticks_to_live().unwrap_or(0),
            JobType::Defend => self.ticks_to_live().unwrap_or(0),
            JobType::DefendR => self.ticks_to_live().unwrap_or(0),
            JobType::Heal => self.ticks_to_live().unwrap_or(0),
            JobType::Scout => self.ticks_to_live().unwrap_or(0),
        };
        
        let finish_ticks = (self.ticks_to_live().unwrap_or(0)
            .checked_sub(start_ticks).unwrap_or(0)).min(job_duration);
        
        return (start_ticks, finish_ticks, finish_ticks * contribution_per_tick);
    }

    /// calculates the fatigue required to arrive at a location
    /// returns the number of ticks the creep will take to arrive there
    fn fatigue_to(&self, target: &dyn HasId) -> u32 {
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

        if body_move == 0 && !self.pos().is_near_to(target) { return 100000; }

        let carry_weight = self.store_used_capacity(None) as f32;
        let loaded_carry = (carry_weight / 50.0).ceil() as u32;
        
        // weight = how much gross fatigue is accumulated per move on road
        let weight = body.iter()
            .fold(loaded_carry.min(body_carry),|acc, cur| acc + cur);

        let ticks_road = (0.5 * weight as f32 / body_move as f32).ceil();
        let ticks_plain = (1.0 * weight as f32 / body_move as f32).ceil();
        let ticks_swamp = (5.0 * weight as f32 / body_move as f32).ceil();
        
        let heuristic = self.pos().get_range_to(target) as f32 * ticks_road;

        let search_opts = screeps::pathfinder::SearchOptions::default()
            .plain_cost(ticks_plain as u8)
            .swamp_cost(ticks_swamp as u8)
            .heuristic_weight(heuristic.into());
        let search_results = screeps::pathfinder::search(&self.pos(), &target.pos(), 1, search_opts);
        
        if search_results.incomplete { return 100000; }
        
        //TODO Visuals

        return self.fatigue() / (2 * body_move) + self.fatigue() % (2 * body_move) + search_results.cost;
    }

    fn resource_sink(&self) -> Option<ResourceType> {
        None
    }
    fn resource_source(&self) -> Option<ResourceType> {
        None
    }
}

impl JobProperties for screeps::StructureTower{
    fn has_parts_for_job(&self, job_type: JobType) -> bool {
        match job_type {
            JobType::Repair
            | JobType::AttackR
            | JobType::DefendR
            | JobType::Heal => true,
            _ => false,
        }
    }
    fn job_runtime(&self, target: &dyn HasId, job_type: JobType) -> (u32, u32, u32) {
        let amount = match job_type {
            JobType::Repair => 800,
            JobType::AttackR => 600,
            JobType::DefendR => 600,
            JobType::Heal => 400,
            _ => 0,
        };

        let range = self.pos().get_range_to(target).min(20).max(5);
        (0,1, amount - (amount * (range - 5) / 20))

    }
    fn fatigue_to(&self, target: &dyn HasId) -> u32 { if target.room() == self.room() { 0 } else { 100000 } }
    fn resource_sink(&self) -> Option<ResourceType> { Some(ResourceType::Energy) }
    fn resource_source(&self) -> Option<ResourceType> { None }
}
