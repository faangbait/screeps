use log::warn;
use screeps::{HasPosition, Position, Room, RoomName, RoomPosition};
use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum FlagRole {
    Source = 0,
    Scout = 1,
}

pub fn set_flagged_scouting_adj(room: &Room) {
    let adj = vec![(0, 50), (50, 0), (0, -50), (-50, 0)];

    for a in adj {
        let nearby_pos = &(room.controller().unwrap().pos() + a);
        let mut name = String::from("scout-");
        name.extend((&nearby_pos.room_name().to_string()).chars());

        if screeps::game::flags::get(&name).is_some() {
            continue;
        }

        let nearby_pos = RoomPosition::new(nearby_pos.x(), nearby_pos.y(), nearby_pos.room_name());

        match room.create_flag(
            &nearby_pos,
            &name,
            screeps::Color::Green,
            screeps::Color::Green,
        ) {
            Ok(f) => screeps::game::flags::get(&f)
                .unwrap()
                .set_position(nearby_pos),
            Err(o) => match o {
                screeps::ReturnCode::NameExists => {
                    warn!("NAME EXISTS:  {:?} {:?}", name, nearby_pos)
                }
                screeps::ReturnCode::Full => warn!("TOO MANY FLAGS:  {:?} {:?}", name, nearby_pos),
                screeps::ReturnCode::InvalidArgs => {
                    warn!("INVALID ARGS:  {:?} {:?}", name, nearby_pos)
                }
                _ => warn!("Unhandled return code in flag"),
            },
        };
    }
}
pub fn set_flagged_scouting_single(room_name: RoomName, pos: &Position) {
    let room = screeps::game::rooms::get(room_name);

    match room {
        Some(r) => {
            match r.get_position_at(pos.x(), pos.y()) {
                Some(pos) => {
                    let mut name = String::from("scout-");
                    name.extend(room_name.to_string().chars());
                    if screeps::game::flags::get(&name).is_some() {
                        return;
                    }

                    match r.create_flag(&pos, &name, screeps::Color::Green, screeps::Color::Green) {
                        Ok(o) => match screeps::game::flags::get(&o) {
                            Some(flag) => flag.set_position(pos),
                            None => warn!("Error moving flag to correct room"),
                        },
                        Err(e) => match e {
                            screeps::ReturnCode::NameExists => {
                                warn!("NAME EXISTS:  {:?} {:?}", name, pos)
                            }
                            screeps::ReturnCode::Full => {
                                warn!("TOO MANY FLAGS:  {:?} {:?}", name, pos)
                            }
                            screeps::ReturnCode::InvalidArgs => {
                                warn!("INVALID ARGS:  {:?} {:?}", name, pos)
                            }
                            _ => warn!("Unhandled return code in flag"),
                        },
                    }
                }
                None => warn!("Couldn't get position at {:?} for {:?}", pos, r.name()),
            };
        }
        None => warn!("No room named {:?}", room_name),
    }
}
pub fn get_source_flags() -> Vec<Position> {
    screeps::game::flags::values()
        .iter()
        .filter(|&f| f.name().starts_with("source"))
        .map(|f| f.pos())
        .collect::<Vec<Position>>()
}

pub fn get_claim_flags() -> Vec<Position> {
    screeps::game::flags::values()
        .iter()
        .filter(|&f| f.name().starts_with("claim"))
        .map(|f| f.pos())
        .collect::<Vec<Position>>()
}
