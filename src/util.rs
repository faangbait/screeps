use crate::filters;

pub trait CreepCustomActions {
    fn count_bp_vec(self: &Self, part_array: Vec<screeps::Part>) -> Vec<u32>;
}

impl CreepCustomActions for screeps::Creep {
    /// Takes a vector of parts to check, e.g. vec![Part::Move, Part::Carry]
    /// Returns a vector of the counts of these parts, e.g. vec![4,2]
    fn count_bp_vec(self: &Self, part_array: Vec<screeps::Part>) -> Vec<u32> {
        let mut res = vec![0, part_array.len() as u32];
        for (idx, part) in part_array.iter().enumerate() {
            res[idx] = self.get_active_bodyparts(*part);
        }
        res
    }
}

pub trait RoomCustomActions {
    fn count_baddies_here(self: &Self) -> u32;
}

impl RoomCustomActions for screeps::Room {
    fn count_baddies_here(self: &Self) -> u32 {
        let hostiles = filters::get_hostility(self);
        (hostiles.0.len() + hostiles.1.len()) as u32
    }    
}
