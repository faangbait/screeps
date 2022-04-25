use log::info;
use screeps::SharedCreepProperties;
use stdweb::js;

mod logging;
mod entry;
mod rooms;
mod creeps;
mod spawns;
mod structures;
mod constructionsites;
mod flags;
mod world;
mod sink;
mod source;
mod logic;
mod basic_enums;
mod filters;
mod jobs;

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
    let (
        mut rooms,
        mut creeps,
        mut spawns,
        mut structures,
        mut constructionsites,
        mut resources,
        mut flags,
        mut sources,
        mut controllers,
    ) = entry::init();

    creeps.sort_unstable_by_key(|c| c.body().len());

    logic::prioritize(creeps, structures, constructionsites, resources, sources);
    
    entry::endstep();
}

