use log::info;
use screeps::SharedCreepProperties;
use screeps::{self, HasPosition};

use crate::architect;
use crate::combat;

pub fn tower_action(){
    let towers = architect::get_towers();
    let mut repairables = architect::get_my_repairables();
    let mut hostiles = combat::get_hostiles();
    
    for tower in towers {
        hostiles.sort_by_key(|loc| tower.pos().get_range_to(&loc.pos()));
        match hostiles.first() {
            Some(h) => match tower.attack(h) {
                _ => info!("{:?} tower attacked {:?}", &tower.pos(), &h.name())
            },
            None => {
                repairables.sort_by_key(|loc| tower.pos().get_range_to(&loc.pos()));
                match repairables.first() {
                    Some(r) => match tower.repair(r) {
                        // screeps::ReturnCode::Ok => todo!(),
                        // screeps::ReturnCode::NotOwner => todo!(),
                        // screeps::ReturnCode::NoPath => todo!(),
                        // screeps::ReturnCode::NameExists => todo!(),
                        // screeps::ReturnCode::Busy => todo!(),
                        // screeps::ReturnCode::NotFound => todo!(),
                        // screeps::ReturnCode::NotEnough => todo!(),
                        // screeps::ReturnCode::InvalidTarget => todo!(),
                        // screeps::ReturnCode::Full => todo!(),
                        // screeps::ReturnCode::NotInRange => todo!(),
                        // screeps::ReturnCode::InvalidArgs => todo!(),
                        // screeps::ReturnCode::Tired => todo!(),
                        // screeps::ReturnCode::NoBodypart => todo!(),
                        // screeps::ReturnCode::RclNotEnough => todo!(),
                        // screeps::ReturnCode::GclNotEnough => todo!(),
                        _ => continue,
                    },
                    None => continue,
                }
            },
        }
    }
}
