use log::warn;
use screeps::{Room, HasPosition, RoomPosition, Position};

pub fn flag_adjacent_rooms(room: &Room) {
    let adj = vec![(0,50),(50,0),(0,-50),(-50,0)];

    for a in adj {
        let nearby_pos = &(room.controller().unwrap().pos() + a);

        let mut name = nearby_pos.room_name().to_string();
        name.extend("-adj".chars());

        if screeps::game::flags::get(&name).is_some() { continue; }

        let nearby_pos = RoomPosition::new(nearby_pos.x(), nearby_pos.y(), nearby_pos.room_name());

        match room.create_flag(&nearby_pos, &name, screeps::Color::Green,screeps::Color::Green) {
            Ok(f) => { screeps::game::flags::get(&f).unwrap().set_position(nearby_pos) },
            Err(o) => match o {
                screeps::ReturnCode::NameExists => warn!("NAME EXISTS:  {:?} {:?}", name, nearby_pos),
                screeps::ReturnCode::Full => warn!("TOO MANY FLAGS:  {:?} {:?}", name, nearby_pos),
                screeps::ReturnCode::InvalidArgs => warn!("INVALID ARGS:  {:?} {:?}", name, nearby_pos),
                _ => warn!("Unhandled return code in flag"),
            },
        };
    }
}

pub fn get_flagged_sources() -> Vec<Position>{
    screeps::game::flags::values()
        .iter()
        .filter(|f|f.name().starts_with("source-"))
        .map(|f| f.pos())
        .collect()
}
