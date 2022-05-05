use log::info;
use screeps::{HasStore, Part, RoomObjectProperties};

use crate::filters::{self, get_hostility};

pub fn init(
    count: usize,
    harvesters: usize,
    haulers: usize,
    builders: usize,
    upgraders: usize,
    gatherers: usize,
) {
    let spawns = filters::get_my_spawns();
    let timestamp = screeps::game::time();
    if let Some(spawner) = spawns
        .iter()
        .max_by_key(|s| s.room().unwrap().energy_available())
    {
        let (c, pc, sp, st, cs) = get_hostility(&spawner.room().unwrap());
        if c.len() > 0 || pc.len() > 0 || sp.len() > 0 || st.len() > 0 || cs.len() > 0 {
            spawner.spawn_creep(
                &[Part::Move, Part::RangedAttack, Part::RangedAttack],
                &timestamp.to_string(),
            );
        }
        spawner.spawn_creep(
            &[Part::Move, Part::Work, Part::Carry],
            &timestamp.to_string(),
        )

        //     match count {
        //         0 => {
        //             spawner.spawn_creep(
        //                 &[Part::Move, Part::Work, Part::Carry, Part::Move],
        //                 &timestamp.to_string(),
        //             );
        //         }
        //         1 => {
        //             spawner.spawn_creep(
        //                 &[Part::Move, Part::Move, Part::Carry, Part::Work],
        //                 &timestamp.to_string(),
        //             );
        //         }
        //         2 => {
        //             spawner.spawn_creep(
        //                 &[Part::Move, Part::Work, Part::Work],
        //                 &timestamp.to_string(),
        //             );
        //         }
        //         3..=16 => {
        //             if harvesters < 2 {
        //                 spawner.spawn_creep(
        //                     &[Part::Move, Part::Work, Part::Work],
        //                     &timestamp.to_string(),
        //                 );
        //             } else if builders < 3 {
        //                 spawner.spawn_creep(
        //                     &[Part::Move, Part::Work, Part::Carry, Part::Carry],
        //                     &timestamp.to_string(),
        //                 );
        //             } else if upgraders < 3 {
        //                 spawner.spawn_creep(
        //                     &[Part::Move, Part::Work, Part::Work, Part::Carry],
        //                     &timestamp.to_string(),
        //                 );
        //             } else if gatherers < 3 {
        //                 spawner.spawn_creep(
        //                     &[
        //                         Part::Move,
        //                         Part::Move,
        //                         Part::Move,
        //                         Part::Work,
        //                         Part::Carry,
        //                         Part::Carry,
        //                     ],
        //                     &timestamp.to_string(),
        //                 );
        //             } else if haulers < 3 {
        //                 spawner.spawn_creep(
        //                     &[Part::Move, Part::Move, Part::Carry, Part::Carry],
        //                     &timestamp.to_string(),
        //                 );
        //             } else {
        //                 spawner.spawn_creep(
        //                     &[Part::Move, Part::Work, Part::Work, Part::Carry],
        //                     &timestamp.to_string(),
        //                 );
        //             }
        //         }
        //         _ => {
        //             if harvesters < 2 {
        //                 spawner.spawn_creep(
        //                     &[
        //                         Part::Move,
        //                         Part::Work,
        //                         Part::Work,
        //                         Part::Work,
        //                         Part::Work,
        //                         Part::Work,
        //                     ],
        //                     &timestamp.to_string(),
        //                 );
        //             } else if builders < 6 {
        //                 spawner.spawn_creep(
        //                     &[
        //                         Part::Work,
        //                         Part::Move,
        //                         Part::Move,
        //                         Part::Move,
        //                         Part::Work,
        //                         Part::Move,
        //                         Part::Carry,
        //                         Part::Carry,
        //                     ],
        //                     &timestamp.to_string(),
        //                 );
        //             } else if upgraders < 4 {
        //                 spawner.spawn_creep(
        //                     &[
        //                         Part::Move,
        //                         Part::Work,
        //                         Part::Work,
        //                         Part::Carry,
        //                         Part::Carry,
        //                         Part::Carry,
        //                         Part::Carry,
        //                         Part::Carry,
        //                     ],
        //                     &timestamp.to_string(),
        //                 );
        //             } else if gatherers < 3 {
        //                 spawner.spawn_creep(
        //                     &[
        //                         Part::Move,
        //                         Part::Move,
        //                         Part::Move,
        //                         Part::Work,
        //                         Part::Carry,
        //                         Part::Carry,
        //                     ],
        //                     &timestamp.to_string(),
        //                 );
        //             } else if haulers < 6 {
        //                 spawner.spawn_creep(
        //                     &[
        //                         Part::Move,
        //                         Part::Move,
        //                         Part::Carry,
        //                         Part::Carry,
        //                         Part::Move,
        //                         Part::Move,
        //                         Part::Carry,
        //                         Part::Carry,
        //                     ],
        //                     &timestamp.to_string(),
        //                 );
        //             } else {
        //                 spawner.spawn_creep(
        //                     &[
        //                         Part::Move,
        //                         Part::Move,
        //                         Part::Move,
        //                         Part::Move,
        //                         Part::Work,
        //                         Part::Work,
        //                         Part::Carry,
        //                         Part::Carry,
        //                     ],
        //                     &timestamp.to_string(),
        //                 );
        //             }
        //         }
        //     }
    }
}
