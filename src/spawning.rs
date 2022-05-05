use crate::jobs::{JobProperties, JobType, SearchMove};
use crate::{filters, flags};
use screeps::memory::MemoryReference;
use screeps::{
    HasPosition, HasStore, Part, ResourceType, RoomObjectProperties, SharedCreepProperties,
    SpawnOptions, StructureSpawn,
};

pub fn get_hostile_needs(room: &screeps::Room, defenders: &Vec<screeps::Creep>) -> Vec<Part> {
    let mut needs = vec![];
    needs.push(Part::Move);
    needs.push(Part::RangedAttack);
    return needs;
}
pub fn get_needs(
    room: &screeps::Room,
    creeps: &Vec<screeps::Creep>,
    harvesters: &Vec<screeps::Creep>,
    haulers: &Vec<screeps::Creep>,
    builders: &Vec<screeps::Creep>,
    repairers: &Vec<screeps::Creep>,
    upgraders: &Vec<screeps::Creep>,
    gatherers: &Vec<screeps::Creep>,
) -> Vec<Part> {
    let sources = filters::get_my_sources();

    let mut needs = vec![];

    if creeps.len() < 1 {
        needs.extend(vec![Part::Carry, Part::Work, Part::Move]);
        return needs;
    };
    if creeps.len() < 2 {
        needs.extend(vec![Part::Carry, Part::Move]);
        return needs;
    };

    if flags::get_claim_flags().len() > 0
        && creeps
            .iter()
            .filter(|&c| c.get_active_bodyparts(Part::Claim) > 0)
            .count()
            == 0
        && room.energy_available() >= 700
    {
        needs.extend(&[
            Part::Move,
            Part::Claim,
            Part::Move,
            Part::Move,
            Part::Move,
            Part::Move,
            Part::Move,
        ]);
        return needs;
    }

    if harvesters
        .iter()
        .any(|c| c.get_active_bodyparts(screeps::Part::Work) < 5)
    {
        needs.push(Part::Move);
        needs.extend(vec![Part::Work; 5]);
        return needs;
    }

    // let drop_harvesters_parts = creeps
    //     .iter()
    //     .filter(|&c| {
    //         c.store_capacity(Some(ResourceType::Energy)) == 0
    //             && c.has_parts_for_job(JobType::Harvest)
    //     })
    //     .fold(0, |acc, cur| {
    //         acc + cur.get_active_bodyparts(screeps::Part::Work)
    //     });

    // if drop_harvesters_parts < sources.len() as u32 * 5 {
    //     needs.push(Part::Move);
    //     needs.extend(vec![Part::Work; 5]);
    //     return needs;
    // };

    let heavy_hauler_parts = creeps
        .iter()
        .filter(|&c| {
            c.has_parts_for_job(JobType::Transfer)
                && !c.has_parts_for_job(JobType::Harvest)
                && c.ticks_to_live().unwrap_or(0) > 15
        })
        .fold(0, |acc, cur| {
            acc + cur.get_active_bodyparts(Part::Carry) + cur.get_active_bodyparts(Part::Move)
        });

    if heavy_hauler_parts < sources.len() as u32 * 4 {
        needs.extend(&[Part::Move, Part::Carry].repeat(8));
        return needs;
    }

    let builder_parts = creeps
        .iter()
        .filter(|&c| c.has_parts_for_job(JobType::Build) && c.ticks_to_live().unwrap_or(0) > 0)
        .fold(0, |acc, cur| {
            acc + cur.get_active_bodyparts(Part::Carry) + cur.get_active_bodyparts(Part::Work)
        });

    if builder_parts < 1 {
        needs.extend(&[Part::Move, Part::Carry, Part::Work].repeat(8));
    };

    needs.extend(&[Part::Move, Part::Move, Part::Carry, Part::Work]);
    needs
}
pub fn oget_needs(
    room: &screeps::Room,
    creeps: &Vec<screeps::Creep>,
    harvesters: &Vec<screeps::Creep>,
    haulers: &Vec<screeps::Creep>,
    builders: &Vec<screeps::Creep>,
    repairers: &Vec<screeps::Creep>,
    upgraders: &Vec<screeps::Creep>,
    gatherers: &Vec<screeps::Creep>,
) -> Vec<Part> {
    let sources = filters::get_my_sources();
    // let source_slots = sources.iter().flat_map(|s| {
    //     s.pos().neighbors().iter()
    //         .filter(|&pos| { pos.move_cost().is_some() })
    //         .map(|&pos| pos)
    //         .collect::<Vec<screeps::Position>>()
    // }).collect::<Vec<screeps::Position>>();

    let mut needs = vec![];

    if creeps.len() < 3 {
        needs.extend(vec![Part::Carry, Part::Work, Part::Move, Part::Move]);
        return needs;
    }

    let drop_harvester_parts = creeps
        .iter()
        .filter(|&c| {
            c.has_parts_for_job(JobType::Harvest)
                && !c.has_parts_for_job(JobType::Claim)
                && !c.has_parts_for_job(JobType::Attack)
                && !c.has_parts_for_job(JobType::AttackR)
                && c.ticks_to_live().unwrap_or(0) > 15
                && c.store_capacity(Some(screeps::ResourceType::Energy)) == 0
        })
        .fold(0, |acc, cur| acc + cur.get_active_bodyparts(Part::Work));

    if drop_harvester_parts < sources.len() as u32 * 5 {
        needs.push(Part::Move);
        needs.extend(vec![Part::Work; 5]);
        return needs;
    }

    let heavy_hauler_parts = creeps
        .iter()
        .filter(|&c| {
            c.has_parts_for_job(JobType::Transfer)
                && !c.has_parts_for_job(JobType::Harvest)
                && c.ticks_to_live().unwrap_or(0) > 15
        })
        .fold(0, |acc, cur| {
            acc + cur.get_active_bodyparts(Part::Carry) + cur.get_active_bodyparts(Part::Move)
        });
    if heavy_hauler_parts < sources.len() as u32 * 4 || haulers.len() < 4 {
        needs.extend(&[Part::Move, Part::Carry].repeat(8));
        return needs;
    }

    needs.extend(&[Part::Move, Part::Move, Part::Carry, Part::Work]);

    return needs;
}
pub fn init(
    creeps: Vec<screeps::Creep>,
    harvesters: Vec<screeps::Creep>,
    haulers: Vec<screeps::Creep>,
    builders: Vec<screeps::Creep>,
    repairers: Vec<screeps::Creep>,
    upgraders: Vec<screeps::Creep>,
    gatherers: Vec<screeps::Creep>,
    defenders: Vec<screeps::Creep>,
) {
    let spawns = filters::get_my_spawns();

    let mut visited_rooms = vec![];
    spawns
        .iter()
        .filter(|spawn| !spawn.is_spawning())
        .for_each(|spawn| {
            if !visited_rooms.contains(&spawn) {
                visited_rooms.push(spawn);

                let energy_target = match spawn.room() {
                    Some(r) => r.energy_available(),
                    None => 0,
                };
                if energy_target < 250 {
                    return;
                }

                let mut needs = vec![];

                let hostile = filters::get_hostility(&spawn.room().unwrap());
                if hostile.0.is_empty()
                    && hostile.1.is_empty()
                    && hostile.2.is_empty()
                    && hostile.3.is_empty()
                    && hostile.4.is_empty()
                {
                    needs = get_needs(
                        &spawn.room().unwrap(),
                        &creeps,
                        &harvesters,
                        &haulers,
                        &builders,
                        &repairers,
                        &upgraders,
                        &gatherers,
                    );
                } else {
                    needs = get_hostile_needs(&spawn.room().unwrap(), &defenders);
                }

                match spawn.construct_creep(energy_target, needs, spawn) {
                    Some(mut tmpl) => {
                        tmpl.reduce_cost(energy_target);
                        tmpl.sort_body();
                        spawn.spawn_creep_with_options(&tmpl.body, &tmpl.name, &tmpl.opts);
                    }
                    None => {}
                }
            }
        })
}

pub trait SpawnProperties {
    fn construct_creep(
        self: &Self,
        energy_target: u32,
        needs: Vec<Part>,
        spawn: &screeps::StructureSpawn,
    ) -> Option<BodyTemplate>;
}

impl SpawnProperties for screeps::StructureSpawn {
    fn construct_creep(
        self: &Self,
        energy_target: u32,
        needs: Vec<Part>,
        spawn: &screeps::StructureSpawn,
    ) -> Option<BodyTemplate> {
        let mut body = BodyTemplate::new(needs.to_vec(), self);
        while body.cost < energy_target {
            needs.iter().for_each(|p| body.add_part(*p))
        }
        body.reduce_cost(energy_target);
        body.sort_body();
        if body.body.len() >= 3 {
            Some(body)
        } else {
            None
        }
    }
}

pub struct BodyTemplate {
    name: String,
    body: Vec<Part>,
    cost: u32,
    opts: SpawnOptions,
    spawned_at: screeps::Position,
}

impl BodyTemplate {
    fn new(body: Vec<Part>, spawn: &StructureSpawn) -> Self {
        let cost = body.iter().map(|p| p.cost()).sum::<u32>() + 50;
        let mem = MemoryReference::new();

        let count = body.iter().count();

        return BodyTemplate {
            name: String::from(format!("{}-{}-{}", "creep-", count, screeps::game::time())),
            body,
            cost,
            opts: SpawnOptions::new().memory(mem),
            spawned_at: spawn.pos(),
        };
    }
    fn sort_body(self: &mut Self) {
        self.body.sort_by_key(|bp| match bp {
            Part::Tough => 0,
            Part::Claim => 1,
            Part::Work => 2,
            Part::Carry => 3,
            Part::Attack => 4,
            Part::Move => 5,
            Part::RangedAttack => 6,
            Part::Heal => 7,
        });
    }

    fn recost(self: &mut Self) -> u32 {
        self.cost = self.body.iter().map(|p| p.cost()).sum::<u32>() + 50;
        self.cost
    }

    fn reduce_cost(self: &mut Self, parts_cost_cap: u32) {
        while self.cost > parts_cost_cap && self.body.len() > 0 {
            let extra = self
                .body
                .iter()
                .max_by_key(|&bp| self.body.iter().filter(|&p| p == bp).count())
                .unwrap()
                .to_owned();

            self.body
                .sort_unstable_by_key(|bp| if *bp == extra { 1 } else { 0 });
            self.body.pop();
            self.recost();
        }
    }

    fn add_part(self: &mut Self, part: Part) {
        self.body.push(part);
        self.recost();
    }
}

impl JobProperties for BodyTemplate {
    fn count_bp_vec(self: &Self, part_array: Vec<screeps::Part>) -> Vec<u32> {
        let mut res = vec![];
        for part in part_array {
            res.push(self.body.iter().filter(|&p| *p == part).count() as u32);
        }
        return res;
    }

    fn has_parts_for_job(&self, job_type: crate::jobs::JobType) -> bool {
        let bp_reqs = match job_type {
            JobType::Build => vec![Part::Move, Part::Carry, Part::Work],
            JobType::Repair => vec![Part::Move, Part::Carry, Part::Work],
            JobType::Station => vec![Part::Move],
            JobType::Upgrade => vec![Part::Move, Part::Carry, Part::Work],
            JobType::Transfer => vec![Part::Move, Part::Carry],
            JobType::Withdraw => vec![Part::Move, Part::Carry],
            JobType::Pickup => vec![Part::Move, Part::Carry],
            JobType::Harvest => vec![Part::Move, Part::Work],
            JobType::Claim => vec![Part::Move, Part::Claim],
            JobType::Reserve => vec![Part::Move, Part::Claim],
            JobType::Attack => vec![Part::Move, Part::Attack],
            JobType::AttackR => vec![Part::Move, Part::RangedAttack],
            JobType::Defend => vec![Part::Attack],
            JobType::DefendR => vec![Part::RangedAttack],
            JobType::Heal => vec![Part::Heal],
            JobType::Scout => vec![Part::Move],
        };

        bp_reqs
            .iter()
            .all(|req| self.body.iter().filter(|&p| *p == *req).count() > 0)
    }

    fn job_runtime(
        &self,
        target: &dyn screeps::HasId,
        job_type: crate::jobs::JobType,
    ) -> (u32, u32, u32) {
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

        return (
            start_ticks,
            finish_ticks,
            finish_ticks * contribution_per_tick,
        );
    }

    #[inline]
    fn astar(&self, target: &screeps::Position) -> SearchMove {
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

        if body_move == 0 {
            return SearchMove {
                arrive_ticks: u32::MAX,
                search_results: None,
            };
        }

        // let carry_weight = *body.get(6).unwrap_or(&0) as f32 * 50.;
        let loaded_carry = 0;

        // weight = how much gross fatigue is accumulated per move on road
        let weight = body
            .iter()
            .fold(loaded_carry.min(body_carry), |acc, cur| acc + cur);

        let ticks_road = (0.5 * weight as f32 / body_move as f32).ceil();
        let ticks_plain = (1.0 * weight as f32 / body_move as f32).ceil();
        let ticks_swamp = (5.0 * weight as f32 / body_move as f32).ceil();

        let heuristic = self.spawned_at.get_range_to(target) as f32 * ticks_road;

        let search_opts = screeps::pathfinder::SearchOptions::default()
            .plain_cost(ticks_plain as u8)
            .swamp_cost(ticks_swamp as u8)
            .heuristic_weight(heuristic.into());

        //TODO Visuals

        let results = screeps::pathfinder::search(&self.spawned_at, target, 1, search_opts);
        SearchMove {
            arrive_ticks: results.cost,
            search_results: Some(results),
        }
    }

    fn astar_move(&self, pos: &screeps::Position) {}

    fn distance_to(&self, pos: &screeps::Position) -> u32 {
        let linear = self.spawned_at.get_range_to(pos);
        linear
    }
}
