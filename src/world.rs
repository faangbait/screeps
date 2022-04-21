use log::warn;
use screeps::{HasId, Room, RoomObjectProperties, RoomPosition,};
use serde::{Deserialize, Serialize};

use crate::contexts::{Context, ContextMap};
use crate::jobs::{JobProperties, JobType};
use crate::rtb::{BidMap, JobBid, SinkSources};

// TODO: distance oracles that use get_time to solve new pair vertices when we have time available

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ScoutingReport {
    room_name: String,
    objects: Vec<u8>,
    terrain: Vec<u8>,
}

pub fn gen_scouting_report(creep: &screeps::Creep) -> ScoutingReport {
    todo!();
}

pub fn write_scouting_report(report: Vec<ScoutingReport>) {
    let mem = screeps::memory::root();
    let mut path = "world.scouting";

    let serialized_report = serde_json::to_string(&report);
    match serialized_report {
        Ok(k) => mem.path_set(&path, k),
        Err(e) => warn!("Error setting scouting report"),
    };
}

pub fn read_scouting_report() -> Option<Vec<ScoutingReport>> {
    let mem = screeps::memory::root();
    let mut path = "world.scouting";
    match mem.get_path::<String>(path) {
        Ok(o) => match o {
            Some(report_json) => match serde_json::from_str::<Vec<ScoutingReport>>(&report_json) {
                Ok(report) => Some(report),
                Err(e) => {
                    warn!("Error deserializing scouting report");
                    None
                }
            },
            None => None,
        },
        Err(_) => {
            warn!("Error getting scouting report");
            None
        }
    }
}

pub fn scout(room: Room) {
    let creeps = screeps::game::creeps::values();

    let scout = creeps
        .iter()
        .filter(|&c| match c.room() {
            Some(r) => r == room,
            None => false,
        })
        .max_by_key(|&c| c.get_active_bodyparts(screeps::Part::Move));

    match scout {
        Some(c) => {
            let scouting_report = read_scouting_report();
            match scouting_report {
                Some(mut report) => {
                    report.push(gen_scouting_report(c));
                    write_scouting_report(report);
                }
                None => {
                    let empty_vec: Vec<ScoutingReport> = vec![];
                    write_scouting_report(empty_vec);
                }
            }
        }
        None => {

            let room_position = RoomPosition::new(25, 25, room.name());
            let pos = &room.get_position_at(room_position.x(), room_position.y());
            match pos {
                Some(p) => {
                    room.create_named_construction_site(p, screeps::StructureType::Road, "scout");
                    match room
                        .look_at(p)
                        .iter()
                        .filter_map(|lr| match lr {
                            screeps::LookResult::ConstructionSite(site) => Some(site),
                            _ => None,
                        })
                        .min_by_key(|&k| k.progress())
                    {
                        Some(site) => {
                            let scouter = creeps
                                .iter()
                                .filter(|&c| c.context().is_none())
                                .min_by_key(|&c| c.fatigue_to(site));
                            match scouter {
                                Some(c) => {
                                    // directly assign this creep
                                    let mut cmap = ContextMap::new();

                                    match Context::new(
                                        c,
                                        site,
                                        JobBid {
                                            request: JobType::Scout,
                                            resource: None,
                                            max: 1,
                                            bid: 1000,
                                            target: site.untyped_id(),
                                            ty: SinkSources::ConstructionSite,
                                        },
                                    ) {
                                        Some(context) => {
                                            cmap.create(&c.untyped_id(), &context);
                                        }
                                        None => {
                                            warn!("Error creating scout context");
                                        }
                                    }
                                }
                                None => {
                                    // make a bid; potentially spawn a creep
                                    let mut bmap = BidMap::new();
                                    bmap.create(
                                        &site.untyped_id(),
                                        &JobBid {
                                            request: JobType::Scout,
                                            resource: None,
                                            max: 1,
                                            bid: 1000,
                                            target: site.untyped_id(),
                                            ty: SinkSources::ConstructionSite,
                                        },
                                    );
                                }
                            }
                        }
                        None => warn!("Couldn't find construction site"),
                    }
                }
                None => {}
            };
        }
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RoomDescription {
    Vacant = 0,
    My = 1,
    MyReserved = 2,
    Hostile = 3,
    HostileReserved = 4,
    Highway = 5,
    SourceKeeper = 6,
    Center = 7,
}
