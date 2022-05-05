use screeps::{
    HasId, HasPosition, HasStore, Part, Position, ResourceType, RoomObjectProperties,
    SharedCreepProperties,
};
use serde::{Deserialize, Serialize};

pub struct SearchMove {
    pub arrive_ticks: u32,
    pub search_results: Option<screeps::pathfinder::SearchResults>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum JobType {
    Build = 1,
    Repair = 2,
    Station = 3,
    Upgrade = 4,
    Transfer = 5,
    Withdraw = 6,
    Pickup = 7,
    Harvest = 8,
    Claim = 9,
    Reserve = 10,
    Attack = 11,
    AttackR = 12,
    Defend = 13,
    DefendR = 14,
    Heal = 15,
    Scout = 16,
}

pub trait JobProperties {
    fn count_bp_vec(self: &Self, part_array: Vec<screeps::Part>) -> Vec<u32>;
    fn has_parts_for_job(&self, job_type: JobType) -> bool;
    fn job_runtime(&self, target: &dyn HasId, job_type: JobType) -> (u32, u32, u32);

    fn distance_to(&self, pos: &screeps::Position) -> u32;
    fn astar(&self, target: &screeps::Position) -> SearchMove;
    fn astar_move(&self, pos: &screeps::Position);
}

impl JobProperties for screeps::Creep {
    fn count_bp_vec(self: &Self, part_array: Vec<screeps::Part>) -> Vec<u32> {
        let mut res = vec![];
        for part in part_array {
            res.push(self.get_active_bodyparts(part));
        }
        return res;
    }

    fn has_parts_for_job(&self, job_type: JobType) -> bool {
        // let mut bp_reqs = vec![];
        let bp_reqs;

        match job_type {
            JobType::Build => bp_reqs = vec![Part::Move, Part::Carry, Part::Work],
            JobType::Repair => bp_reqs = vec![Part::Move, Part::Carry, Part::Work],
            JobType::Station => bp_reqs = vec![Part::Move],
            JobType::Upgrade => bp_reqs = vec![Part::Move, Part::Carry, Part::Work],
            JobType::Transfer => bp_reqs = vec![Part::Move, Part::Carry],
            JobType::Withdraw => bp_reqs = vec![Part::Move, Part::Carry],
            JobType::Pickup => bp_reqs = vec![Part::Move, Part::Carry],
            JobType::Harvest => bp_reqs = vec![Part::Move, Part::Work],
            JobType::Claim => bp_reqs = vec![Part::Move, Part::Claim],
            JobType::Reserve => bp_reqs = vec![Part::Move, Part::Claim],
            JobType::Attack => bp_reqs = vec![Part::Move, Part::Attack],
            JobType::AttackR => bp_reqs = vec![Part::Move, Part::RangedAttack],
            JobType::Defend => bp_reqs = vec![Part::Attack],
            JobType::DefendR => bp_reqs = vec![Part::RangedAttack],
            JobType::Heal => bp_reqs = vec![Part::Heal],
            JobType::Scout => bp_reqs = vec![Part::Move],
        }

        bp_reqs
            .iter()
            .all(|req| self.get_active_bodyparts(*req) > 0)
    }

    fn job_runtime(&self, target: &dyn HasId, job_type: JobType) -> (u32, u32, u32) {
        let start_ticks = self.astar(&target.pos()).arrive_ticks;

        let contribution_per_tick = match job_type {
            JobType::Harvest => self.get_active_bodyparts(Part::Work) * 2,
            JobType::Build => self.get_active_bodyparts(Part::Work) * 5,
            JobType::Repair => self.get_active_bodyparts(Part::Work) * 100,
            JobType::Station => 1,
            JobType::Upgrade => self.get_active_bodyparts(Part::Work) * 1,
            JobType::Transfer => self.store_used_capacity(None) as u32, // TODO: None?
            JobType::Withdraw => self.store_free_capacity(None) as u32, // TODO: None?
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
            JobType::Build => {
                self.store_used_capacity(Some(ResourceType::Energy))
                    / self.get_active_bodyparts(Part::Work)
            }
            JobType::Repair => {
                self.store_used_capacity(Some(ResourceType::Energy))
                    / self.get_active_bodyparts(Part::Work)
            }
            JobType::Station => self.ticks_to_live().unwrap_or(0),
            JobType::Upgrade => {
                self.store_used_capacity(Some(ResourceType::Energy))
                    / self.get_active_bodyparts(Part::Work)
            } // TODO: Check this
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

        let finish_ticks = (self
            .ticks_to_live()
            .unwrap_or(0)
            .checked_sub(start_ticks)
            .unwrap_or(0))
        .min(job_duration);

        return (
            start_ticks,
            finish_ticks,
            finish_ticks * contribution_per_tick,
        );
    }

    /// Returns the number of ticks it will take to reach a target; roughly
    fn distance_to(&self, pos: &screeps::Position) -> u32 {
        screeps::pathfinder::search(
            &self.pos(),
            pos,
            1,
            screeps::pathfinder::SearchOptions::default(),
        )
        .cost
    }

    #[inline]
    /// calculates the fatigue required to arrive at a location
    /// returns the number of ticks the creep will take to arrive there
    fn astar(&self, pos: &screeps::Position) -> SearchMove {
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

        if body_move == 0 && !self.pos().is_near_to(pos) {
            return SearchMove {
                arrive_ticks: u32::MAX,
                search_results: None,
            };
        };

        let carry_weight = self.store_used_capacity(None) as f32;
        let loaded_carry = (carry_weight / 50.0).ceil() as u32;

        // weight = how much gross fatigue is accumulated per move on road
        let weight = body
            .iter()
            .fold(loaded_carry.min(body_carry), |acc, cur| acc + cur);

        let ticks_road = (0.5 * weight as f32 / body_move as f32).ceil();
        let ticks_plain = (1.0 * weight as f32 / body_move as f32).ceil();
        let ticks_swamp = (5.0 * weight as f32 / body_move as f32).ceil();
        let heuristic = self.pos().get_range_to(pos) as f32 * ticks_road;

        let search_opts = screeps::pathfinder::SearchOptions::default()
            .plain_cost(ticks_plain as u8)
            .swamp_cost(ticks_swamp as u8)
            .heuristic_weight(heuristic.into());
        let results = screeps::pathfinder::search(&self.pos(), pos, 1, search_opts);

        //TODO Visuals
        SearchMove {
            arrive_ticks: self.fatigue() / (2 * body_move)
                + self.fatigue() % (2 * body_move)
                + results.cost,
            search_results: Some(results),
        }
        // Some(SearchMove {
        //             fatigue_ticks: self.fatigue() / (2 * body_move) + self.fatigue() % (2 * body_move) + results.cost,
        //             search_results: results,
        //             path: results.load_local_path(),
        //             incomplete: results.incomplete
        //         })
    }

    fn astar_move(&self, pos: &screeps::Position) {
        let astar = self.astar(pos);

        if let Some(sr) = astar.search_results {
            self.move_by_path_search_result(&sr);
        }
    }
}

impl JobProperties for screeps::StructureTower {
    fn count_bp_vec(self: &Self, part_array: Vec<screeps::Part>) -> Vec<u32> {
        vec![]
    }
    fn has_parts_for_job(&self, job_type: JobType) -> bool {
        match job_type {
            JobType::Repair | JobType::AttackR | JobType::DefendR | JobType::Heal => true,
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
        (0, 1, amount - (amount * (range - 5) / 20))
    }

    fn astar(&self, target: &screeps::Position) -> SearchMove {
        SearchMove {
            arrive_ticks: 0,
            search_results: None,
        }
    }

    fn astar_move(&self, pos: &screeps::Position) {}

    fn distance_to(&self, pos: &screeps::Position) -> u32 {
        let linear = self.pos().get_range_to(pos);
        linear
    }
}
