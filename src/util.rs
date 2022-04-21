use screeps::OwnedStructureProperties;

use crate::filters;
use crate::world::RoomDescription;

pub trait RoomCustomActions {
    fn count_baddies_here(self: &Self) -> u32;
    fn room_type(self: &Self) -> RoomDescription;
}

impl RoomCustomActions for screeps::Room {
    fn count_baddies_here(self: &Self) -> u32 {
        let hostiles = filters::get_hostility(self);
        (hostiles.0.len() + hostiles.1.len()) as u32
    }
    
    fn room_type(self: &Self) -> RoomDescription {
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
                1601..=u32::MAX => RoomDescription::SourceKeeper,
                _ => RoomDescription::SourceKeeper
            },
        }
    }
}
