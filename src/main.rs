use log::*;

use screeps::prelude::*;
use stdweb::js;

mod logging;
mod flow;
mod spawning;
mod pathing;
mod architect; 

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
    debug!("loop starting! CPU: {}", screeps::game::cpu::get_used());

    flow::start_loop();
    spawning::manage_spawns();

    flow::prioritize_actions();
    
    flow::manage_memory();
    flow::end_loop();

    info!("done! cpu: {}", screeps::game::cpu::get_used())
}

