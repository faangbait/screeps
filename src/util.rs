use screeps::OwnedStructureProperties;

use crate::filters;
use crate::world::RoomDescription;

pub trait CreepCustomActions {
    fn count_bp_vec(self: &Self, part_array: Vec<screeps::Part>) -> Vec<u32>;
}

impl CreepCustomActions for screeps::Creep {
    /// Takes a vector of parts to check, e.g. vec![Part::Move, Part::Carry]
    /// Returns a vector of the counts of these parts, e.g. vec![4,2]
    fn count_bp_vec(self: &Self, part_array: Vec<screeps::Part>) -> Vec<u32> {
        let mut res = vec![];
        for part in part_array {
            res.push(self.get_active_bodyparts(part));
        }
        return res
    }
}
pub trait RoomCustomActions {
    fn count_baddies_here(self: &Self) -> u32;
    fn room_type(self: &Self) -> u8;
}

impl RoomCustomActions for screeps::Room {
    fn count_baddies_here(self: &Self) -> u32 {
        let hostiles = filters::get_hostility(self);
        (hostiles.0.len() + hostiles.1.len()) as u32
    }
    
    fn room_type(self: &Self) -> u8 {
        let my_username = String::from("test");

        match self.controller() {
            Some(c) => match c.reservation() {
                Some(r) => match r.username == my_username {
                    true => RoomDescription::MyReserved,
                    false => RoomDescription::HostileReserved,
                },
                None => match c.as_owned_structure().owner_name() {
                    Some(n) => match n == my_username {
                        true => RoomDescription::My,
                        false => RoomDescription::Hostile,
                    },
                    None => RoomDescription::Vacant,
                },
            },
            None => match self.energy_capacity_available() {
                0 => RoomDescription::Highway,
                1201..=1600 => RoomDescription::Center,
                1601..=u32::MAX => 0,
                _ => RoomDescription::SourceKeeper
            },
        }
    }
}
