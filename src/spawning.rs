

use log::info;
use screeps::{Part, ReturnCode, HasPosition, RoomObjectProperties};

use crate::{filters, util};

pub fn get_best_spawn_template(spawn: &screeps::StructureSpawn) -> SpawnTemplate {
    let opts = screeps::SpawnOptions::new();
    let creeps = screeps::game::creeps::values();


    match creeps.len() {
        // if we don't have enough harvesters...
        0..=1 | 6..=99 => match creeps.iter()
            .filter(|&c| util::count_bodyparts(c, Part::Move) == 1 && util::count_bodyparts(c, Part::Work) <= 5)
            .count() {
                0 => { return SpawnTemplate::new(opts, vec![Role::Harvester(6 as usize)]); },
                _ => {}
            } 
            _ => { return SpawnTemplate::new(opts, vec![Role::Worker(6 as usize)]); },
        };




    // let biggest_workers = creeps.iter().max_by_key(|&c| util::count_bodyparts(c, Part::Work));

    
    // if creeps.len() < 3 { 
    //     return SpawnTemplate::new(opts, vec![
    //         Role::Harvester(1 as usize),
    //         Role::Worker(3 as usize),
    //         ])
    // };

    // if we're barely into a cold boot...
    // if spawn.room().unwrap().energy_capacity_available() <= 400 {
    //     return SpawnTemplate::new(opts, vec![
    //         Role::Worker(3 as usize),
    //         Role::Hauler(1 as usize)
    //     ])
    // }

    // if we don't have enough harvesters...
    // if creeps.iter()
    //     .filter(|c| c.body().iter().fold(0, |acc,cur| match cur.part {
    //         Part::Move => acc+1,
    //         _ => acc
    //     }) == 1)
    //     .filter(|c| c.body().iter()
    //     .fold(0, |acc,cur| match cur.part {
    //         Part::Work => acc+1,
    //         _ => acc
    //     }) >= 5).count() < filters::get_my_sources().len() {
    //         return SpawnTemplate::new(opts, vec![
    //             Role::Harvester(6 as usize)
    //         ]);
    //     };
    
    SpawnTemplate::new(opts, vec![
        Role::Worker(6 as usize)
    ])
}


#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum Role<T> {
    Harvester(T),
    Hauler(T),
    Worker(T),
    Scout(T),
    Barbarian(T),
    Fighter(T),
    Cleric(T),
    Mage(T),
    Priest(T),
    Prospector(T)
}

impl Role<usize> {
    pub fn lvl(self) -> usize {
        match self {
            Role::Harvester(lvl) => lvl,
            Role::Hauler(lvl) => lvl,
            Role::Worker(lvl) => lvl,
            Role::Scout(lvl) => lvl,
            Role::Barbarian(lvl) => lvl,
            Role::Fighter(lvl) => lvl,
            Role::Cleric(lvl) => lvl,
            Role::Mage(lvl) => lvl,
            Role::Priest(lvl) => lvl,
            Role::Prospector(lvl) => lvl,
        }
    }

    pub fn components(self) -> Vec<Part> {
        match self {
            // 1 move, the rest work
            Role::Harvester(lvl) => match lvl {
                _ => [vec![Part::Work; lvl-1], vec![Part::Move]].concat()
            },
            // Equal parts carry and move
            Role::Hauler(lvl) => match lvl {
                1 => vec![Part::Carry],
                _ => [vec![Part::Carry; lvl / 2], vec![Part::Move; lvl / 2]].concat()
            },
            // Equal parts carry, work, and move; bonus move
            Role::Worker(lvl) => match lvl {
                1 => vec![Part::Work],
                _ => [vec![Part::Carry; lvl / 3], vec![Part::Work; lvl / 3], vec![Part::Move; lvl / 3 + lvl % 3]].concat()
            },
            // All move
            Role::Scout(lvl) => match lvl {
                _ => vec![Part::Move; lvl],
            },
            // Equal parts attack and move
            Role::Barbarian(lvl) => match lvl {
                1 => vec![Part::Attack],
                _ => [vec![Part::Attack; lvl / 2], vec![Part::Move; lvl / 2]].concat()
            },
            // Equal parts attack, tough, and move; bonus move
            Role::Fighter(lvl) => match lvl {
                1 => vec![Part::Attack],
                _ => [vec![Part::Attack; lvl / 3], vec![Part::Tough; lvl / 3], vec![Part::Move; lvl / 3 + lvl % 3]].concat()
            },
            // Equal parts heal, tough, and move; bonus move
            Role::Cleric(lvl) => match lvl {
                1 => vec![Part::Tough],
                _ => [vec![Part::Tough ; lvl / 3], vec![Part::Heal ; lvl / 3], vec![Part::Move ; lvl / 3 + lvl % 3]].concat()
            },
            // Equal parts rangedattack and move
            Role::Mage(lvl) => match lvl {
                1 => vec![Part::RangedAttack],
                _ => [vec![Part::RangedAttack; lvl / 2], vec![Part::Move; lvl / 2]].concat()
            },
            // Equal parts heal and move
            Role::Priest(lvl) => match lvl {
                1 => vec![Part::Heal],
                _ => [vec![Part::Heal ; lvl / 2], vec![Part::Move ; lvl / 2]].concat()
            },
            // One claim, the rest move
            Role::Prospector(lvl) => match lvl {
                1 => vec![Part::Claim],
                _ => [vec![Part::Move; lvl-1], vec![Part::Claim]].concat()
            },
        }
    }

    pub fn subcost(self) -> u32 {
        self.components().iter().map(|p| p.cost()).sum::<u32>()
    }

    pub fn name(self) -> String {
        match self {
            Role::Harvester(_) => String::from("Harvester"),
            Role::Hauler(_) => String::from("Hauler"),
            Role::Worker(_) => String::from("Worker"),
            Role::Scout(_) => String::from("Scout"),
            Role::Barbarian(_) => String::from("Barbarian"),
            Role::Fighter(_) => String::from("Fighter"),
            Role::Cleric(_) => String::from("Cleric"),
            Role::Mage(_) => String::from("Mage"),
            Role::Priest(_) => String::from("Priest"),
            Role::Prospector(_) => String::from("Prospector"),
        }
    }
}

pub struct SpawnTemplate {
    name: String,
    template: Vec<Role<usize>>,
    cost: u32,
    opts: screeps::SpawnOptions,

}

impl SpawnTemplate {
    pub fn new(opts: screeps::SpawnOptions, template: Vec<Role<usize>>) -> Self {
        let cost = template.iter().fold(50, |acc, c| acc + c.subcost());

        let mut name = template.iter().map(|s| { let mut t = s.name(); t.truncate(3); t }).collect::<String>();
        name.push_str("-");
        name.push_str(&screeps::game::time().to_string());

        Self { name, template, cost, opts }
    }

    pub fn desired_body(self: &Self) -> Vec<screeps::Part> {
        self.template.iter().flat_map(|s| s.components()).collect::<Vec<screeps::Part>>()
    }

    pub fn affordable_body(self: &mut Self, cost_cap: u32) -> Vec<screeps::Part> {

        while self.cost > cost_cap {
            self.template.sort_by_cached_key(|s| s.lvl());
            match self.template.pop() {
                Some(r) => if r.lvl() > 1 {
                    match r {
                        Role::Harvester(lvl) => self.template.push(Role::Harvester(lvl-1)),
                        Role::Hauler(lvl) => self.template.push(Role::Hauler(lvl-1)),
                        Role::Worker(lvl) => self.template.push(Role::Worker(lvl-1)),
                        Role::Scout(lvl) => self.template.push(Role::Scout(lvl-1)),
                        Role::Barbarian(lvl) => self.template.push(Role::Barbarian(lvl-1)),
                        Role::Fighter(lvl) => self.template.push(Role::Fighter(lvl-1)),
                        Role::Cleric(lvl) => self.template.push(Role::Cleric(lvl-1)),
                        Role::Mage(lvl) => self.template.push(Role::Mage(lvl-1)),
                        Role::Priest(lvl) => self.template.push(Role::Priest(lvl-1)),
                        Role::Prospector(lvl) => self.template.push(Role::Prospector(lvl-1)),
                    }
                },
                None => return vec![],
            }
            self.cost = self.desired_body().iter().fold(50, |acc, c| acc + c.cost());
        };
        info!("Spawning a creep with body {:?}", self.desired_body());
        return self.desired_body();
    }
}


// pub struct BodyTemplate {
//     name: String,
//     parts: Vec<Part>,
//     cost: u32,
//     opts: SpawnOptions
// }

// impl BodyTemplate {
//     fn new(templ: &str) -> BodyTemplate {
//         let parts = match templ {
//             "coldboot" => vec![
//                 Part::Move,
//                 Part::Move,
//                 Part::Work,
//                 Part::Carry
//             ],
//             "harvester" => vec![
//                 Part::Move,
//                 Part::Work,
//                 Part::Work,
//                 Part::Work,
//                 Part::Work,
//                 Part::Work,
//                 Part::Work,
//                 ],
//             "hauler" => vec![
//                 Part::Move, 
//                 Part::Move, 
//                 Part::Carry, 
//                 Part::Carry, 
//                 Part::Carry, 
//                 Part::Move, 
//                 Part::Carry, 
//                 Part::Move, 
//                 Part::Carry, 
//                 Part::Move,
//                 Part::Carry, 
//                 Part::Move,
//                 Part::Carry,
//                 Part::Move,
//                 ],
//             "builder" => vec![
//                 Part::Move, 
//                 Part::Work, 
//                 Part::Carry, 
//                 Part::Carry, 
//                 Part::Move, 
//                 Part::Work, 
//                 Part::Carry, 
//                 Part::Move, 
//                 Part::Carry, 
//                 Part::Carry, 
//                 Part::Work, 
//                 Part::Work, 
//                 Part::Carry, 
//                 Part::Carry, 
//                 Part::Work, 
//                 ],
//             _ => vec![
//                 Part::Move, 
//                 Part::Work, 
//                 Part::Carry, 
//                 Part::Carry, 
//                 Part::Move, 
//                 Part::Work, 
//                 Part::Carry, 
//                 Part::Move, 
//                 Part::Work, 
//                 Part::Carry, 
//                 Part::Move, 
//                 Part::Work, 
//                 Part::Carry, 
//                 Part::Move, 
//                 Part::Work, 
//                 ],
//         };
//         let cost = parts.iter().map(|p| p.cost()).sum::<u32>() + 50;
//         let mem = MemoryReference::new();
//         mem.set("role", templ.to_string());
//         return BodyTemplate { 
//             name: String::from(format!("{}-{}",templ,screeps::game::time())),
//             parts,
//             cost,
//             // opts: SpawnOptions::new().memory(mem).directions(&[screeps::Direction::Left, screeps::Direction::BottomLeft, screeps::Direction::TopLeft])
//             opts: SpawnOptions::new().memory(mem)
//         };
//     }
// }

pub fn init() -> ReturnCode {
    let creeps = screeps::game::creeps::values();
    let harvesters = get_by_role(creeps, "Har");
    if harvesters.len() < 1 {}

    let mut checked_rooms = vec![];

    for spawn in filters::get_my_spawns()
    .iter().filter(|sp| !sp.is_spawning()) {
        let room = screeps::RoomObjectProperties::room(spawn).unwrap();
        
        if checked_rooms.contains(&room.name()) { continue; }
        checked_rooms.push(room.name());

        if room.energy_available() >= 300 {
            let mut next = get_best_spawn_template(spawn);
            let affordable = next.affordable_body(room.energy_available());
            if next.cost > 150 { 
                return spawn.spawn_creep_with_options(&affordable, &next.name, &next.opts)
            }
        }
    }
    ReturnCode::NotEnough
}


// pub fn init() {
//     for spawn in screeps::game::spawns::values().iter()
//         .filter(|sp| !sp.is_spawning() && !sp.room().unwrap().energy_available() < 250) {
//             match get_desired_body(&spawn) {
//                 Some(body) => {
//                     match spawn.room() {
//                         Some(room) => {
//                             if room.energy_available() >= body.cost {
//                                 match spawn.spawn_creep_with_options(&body.parts, &body.name, &body.opts) {
//                                     ReturnCode::Ok => info!("Spawning {:?} at {:?}", &body.name, &spawn.name()),
//                                     ReturnCode::NoPath => info!("No place to spawn at {:?}", &spawn.name()),
//                                     ReturnCode::NameExists => info!("Name exists at {:?}", &spawn.name()),
//                                     ReturnCode::Busy => info!("{:?} busy, can't spawn", &spawn.name()),
//                                     // ReturnCode::NotEnough => {},
//                                     // ReturnCode::RclNotEnough => {},
//                                     // ReturnCode::GclNotEnough => {},
//                                     _ => warn!("Unhandled return at {:?}", &spawn.name())
//                                 }
//                             }
//                         },
//                         None => warn!("Unhandled return at {:?}", &spawn.name()),
//                     }    
//                 },
//                 None => return,
//             }
//         };
// }


// pub fn get_desired_body(spawn: &StructureSpawn) -> Option<BodyTemplate> {
//     let creeps = screeps::game::creeps::values();
//     let energy_cap =  spawn.room().expect("Error in room").energy_capacity_available();
//     let energy_avail = spawn.room().expect("Error in room").energy_available();
//     let energy_min =  300;
//     let creep_cap = filters::get_my_sources().len();
//     if creeps.len() <= 3 {
//         reduce_body_cost(BodyTemplate::new("coldboot"), energy_min)
//     } else if get_by_role(&creeps, "harvester").len() < creep_cap {
//         reduce_body_cost(BodyTemplate::new("harvester"), energy_avail.max(energy_min))
//     } else if get_by_role(&creeps, "builder").len() < creep_cap +1 {
//         reduce_body_cost(BodyTemplate::new("builder"), energy_avail.max(energy_min))
//     } else if get_by_role(&creeps, "hauler").len() < creep_cap +1 {
//         reduce_body_cost(BodyTemplate::new("hauler"), energy_avail.max(energy_min))
//     } else if get_by_role(&creeps, "builder").len() < creep_cap +4 {
//         reduce_body_cost(BodyTemplate::new("builder"), energy_cap)
//     } else { None }
// }
// pub fn reduce_body_cost(mut body: BodyTemplate, cap: u32) -> Option<BodyTemplate> {
//     while body.cost > cap {
//         body.parts.remove(body.parts.len()-1);
//         body.cost = body.parts.iter().map(|p| p.cost()).sum::<u32>() + 50;
//     }
//     if body.parts.len() > 3 {
//         Some(body)
//     } else { None }
// }

// pub fn get_by_role<'a>(creeps: &'a Vec<screeps::Creep>, role_name: &'a str) -> Vec<&'a screeps::Creep>  {
//     creeps
//         .iter()
//         .filter(|&c| c.my())
//         .filter(|&c| !c.spawning())
//         .filter(|&c| c.name().contains(role_name))
//         .collect::<Vec<&screeps::Creep>>()
// }
        
pub fn get_by_role(creeps: Vec<screeps::Creep>, role_name: &str) -> Vec<screeps::Creep> {
    creeps
        .iter()
        .filter(|&c| screeps::SharedCreepProperties::my(c))
        .filter(|&c| !c.spawning())
        .filter(|&c| screeps::SharedCreepProperties::name(c).contains(role_name))
        .map(|c| c.to_owned())
        .collect::<Vec<screeps::Creep>>()
}
