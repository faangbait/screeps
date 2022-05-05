use jobs::JobProperties;
use log::info;
use screeps::{game, HasPosition, Position, SharedCreepProperties};
use stdweb::js;

mod constructionsites;
mod creeps;
mod entry;
mod filters;
mod flags;
mod jobs;
mod logging;
mod relogic;
mod rooms;
mod sink;
mod source;
mod spawning;
mod structures;
mod world;

fn main() {
    logging::setup_logging(logging::Info);

    js! {
        var game_loop = @{game_loop};

        module.exports.loop = function() {
            // Provide actual error traces.
            try {
                game_loop();
            } catch (error) {
                // console_error function provided by 'screeps-game-api'
                console_error("caught exception:", error);
                if (error.stack) {
                    console_error("stack trace:", error.stack);
                }
                console_error("resetting VM next tick.");
                // reset the VM since we don't know if everything was cleaned up and don't
                // want an inconsistent state.
                module.exports.loop = wasm_initialize;
            }
        }
    }
}

fn game_loop() {
    // info!("Starting loop...");
    let (rooms, creeps, spawns, structures, constructionsites, resources, flags, sources) =
        entry::init();

    // creeps.iter().filter(|&c| {
    //     match c.ticks_to_live() {
    //         Ok(o) => o % 10 == 0,
    //         Err(_) => false,
    //     }
    // }).for_each(|c| {
    //     let room = c.memory().path_string("_move.dest.room").unwrap_or(None);
    //     let x = c.memory().path_i32("_move.dest.x").unwrap_or(None);
    //     let y = c.memory().path_i32("_move.dest.y").unwrap_or(None);

    //     // if let Some(room_name) = room {
    //     //     if let Some(x) = x {
    //     //         if let Some(y) = y {
    //     //             let position = Position::new(x as u32,y as u32,screeps::RoomName::new(&room_name).unwrap());
    //     //             let search = c.astar(&position).search_results.unwrap();
    //     //             c.memory().path_set("_move.astar.path", search.opaque_path());
    //     //             c.memory().path_set("_move.astar.cost", search.cost);
    //     //         }
    //     //     }
    //     // }

    // });
    // logic::prioritize(creeps, sources, structures, constructionsites, resources);
    relogic::prioritize(creeps.to_vec());

    entry::endstep();
}
