use serde::{Serialize, Deserialize};

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


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SinkType {
    Creep = 1,
    Resource = 3,
    ConstructionSite = 7,
    Tombstone = 8,
    PowerCreep = 9,
    Structure = 10,
    Controller = 11,
    Container = 12,
    Extension = 13,
    Extractor = 14,
    Factory = 15,
    Lab = 16,
    Link = 17,
    Nuker = 18,
    Observer = 19,
    PowerSpawn = 20,
    Rampart = 21,
    Road = 22,
    Spawn = 23,
    Storage = 24,
    Terminal = 25,
    Tower = 26,
    Wall = 27,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SourceType {
    Creep = 1,
    Resource = 3,
    Source = 4,
    Mineral = 5,
    Deposit = 6,
    Tombstone = 8,
    PowerCreep = 9,
    Container = 12,
    Extension = 13,
    Factory = 15,
    Lab = 16,
    Link = 17,
    PowerSpawn = 20,
    Spawn = 23,
    Storage = 24,
    Terminal = 25,
    Tower = 26,
}
