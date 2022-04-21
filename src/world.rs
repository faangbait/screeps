use serde::{Serialize, Deserialize};

// TODO: distance oracles that use get_time to solve new pair vertices when we have time available
pub fn scout() {}

#[derive(Debug, Serialize, Deserialize)]
pub enum RoomDescription {
    Vacant = 0,
    My = 1,
    MyReserved = 2,
    Hostile = 3,
    HostileReserved = 4,
    Highway = 5,
    SourceKeeper = 6,
    Center = 7
}
