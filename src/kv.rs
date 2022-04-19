use screeps::{Position, Creep, SharedCreepProperties, mem_get, RoomName};

pub fn get_dest(creep: &Creep) -> Option<Position> {
    let mem = creep.memory();
    let r = mem_get!(mem._move.dest.room.string);
    let x = mem_get!(mem._move.dest.x.i32);
    let y = mem_get!(mem._move.dest.y.i32);

    let r = r.map_or(None, |f| f.map(|o| o));
    let x = x.map_or(None, |f| f.map(|o| o));
    let y = y.map_or(None, |f| f.map(|o| o));

    if r.is_none() || x.is_none() || y.is_none() { return None; }
    
    let r = RoomName::new(&r.unwrap());
    if r.is_err() { return None; }

    let r = r.unwrap();
    let x = x.unwrap() as u32;
    let y = y.unwrap() as u32;

    Some(Position::new(x,y,r))

}
