// TODO: distance oracles that use get_time to solve new pair vertices when we have time available
pub fn scout() {}

pub enum RoomDescription {
    Vacant,
    My,
    MyReserved,
    Hostile,
    HostileReserved,
    Highway,
    SourceKeeper,
    Center
}
