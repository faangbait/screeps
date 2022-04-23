use std::convert::TryInto;

use hungarian::minimize;
use log::info;
use screeps::LookResult;

use crate::filters;
use crate::rtb::{SinkNode, SourceNode, JobAsk, JobBid};

pub fn calc() {

    let (ask_vec, bid_vec) = load_vectors();
    let mut matrix: Vec<u64> = vec![];

    // for i in &ask_vec {
    //     for j in &bid_vec {
    //         matrix.push((i.ask as u64).checked_sub(j.bid as u64).unwrap_or(0) as u64);
    //     }
    // }
    
    let height = ask_vec.len();
    let width = bid_vec.len();
    // Rectangular matrix (height < width)

    // let height = 2;
    // let width = 3;
    // let matrix = vec![
    //     1, 0, 5,
    //     2, 3, 1,
    // ];

    // let assignment = minimize(&matrix, height, width);
    
    // info!("{:?}", assignment);
    // let cost: u64 = assignment.iter()
    //     .enumerate()
    //     .filter_map(|(i, &a)| {
    //         a.map(|j| matrix[i*width + j])
    //     })
    //     .sum();

    // info!("{:?}", cost)
    // assert_eq!(&assignment, &vec![Some(1), Some(2)]);

    // let cost: u64 = assignment.iter()
    //     .enumerate()
    //     .filter_map(|(i, &a)| {
    //         a.map(|j| matrix[i*width + j])
    //     })
    //     .sum();

    // assert_eq!(cost, 1);

    // // Rectangular matrix (width < height)

    // let height = 3;
    // let width = 2;
    // let matrix = vec![
    //     5, 5,
    //     1, 0,
    //     2, 3,
    // ];

    // let assignment = minimize(&matrix, height, width);

    // assert_eq!(&assignment, &vec![None, Some(1), Some(0)]);

    // let cost: u64 = assignment.iter()
    //     .enumerate()
    //     .filter_map(|(i, &a)| {
    //         a.map(|j| matrix[i*width + j])
    //     })
    //     .sum();

    // assert_eq!(cost, 2);
}


pub fn load_asks_from_memory() -> Option<Vec<JobAsk>> {
    None
}
pub fn load_bids_from_memory() -> Option<Vec<JobBid>> {
    None
}

pub fn load_vectors() -> (Vec<JobAsk>,Vec<JobBid>) {
    let mut ask_vec = match load_asks_from_memory() {
        Some(asks) => asks,
        None => vec![],
    };

    let mut bid_vec = match load_bids_from_memory() {
        Some(bids) => bids,
        None => vec![],
    };
    filters::get_my_rooms()
        .iter()
        .flat_map(|room| room.look_at_area(0, 0, 50, 50))
        .for_each(|res| {
            match &res.look_result {
                LookResult::Creep(st) => {
                    match st.sink_request() {
                        Some(req) => bid_vec.push(req),
                        None => {}
                    };
                    match st.source_request() {
                        Some(req) => ask_vec.push(req),
                        None => {}
                    };
                }
                LookResult::Energy(st) => {
                    match st.source_request() {
                        Some(req) => ask_vec.push(req),
                        None => {}
                    };
                }
                LookResult::Resource(st) => {
                    match st.source_request() {
                        Some(req) => ask_vec.push(req),
                        None => {}
                    };
                }
                LookResult::Source(st) => {
                    match st.source_request() {
                        Some(req) => ask_vec.push(req),
                        None => {}
                    };
                }
                LookResult::Mineral(st) => {
                    match st.source_request() {
                        Some(req) => ask_vec.push(req),
                        None => {}
                    };
                }
                LookResult::Deposit(st) => {
                    match st.source_request() {
                        Some(req) => ask_vec.push(req),
                        None => {}
                    };
                }
                LookResult::ConstructionSite(st) => {
                    match st.sink_request() {
                        Some(req) => bid_vec.push(req),
                        None => {}
                    };
                }
                LookResult::Tombstone(st) => {
                    match st.source_request() {
                        Some(req) => ask_vec.push(req),
                        None => {}
                    };
                }
                // LookResult::PowerCreep(st) => {
                //     match st.sink_request() {
                //         Some(req) => {bid_vec.push(req)},
                //         None => {},
                //     };
                //     match st.source_request() {
                //         Some(req) => {ask_vec.push(req)},
                //         None => {},
                //     };
                // },
                LookResult::Structure(structure) => match structure {
                    screeps::Structure::Container(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Controller(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Extension(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Extractor(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Factory(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Lab(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Link(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Nuker(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Observer(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::PowerSpawn(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Rampart(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Road(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Spawn(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Storage(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Terminal(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Tower(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                        match st.source_request() {
                            Some(req) => ask_vec.push(req),
                            None => {}
                        };
                    }
                    screeps::Structure::Wall(st) => {
                        match st.sink_request() {
                            Some(req) => bid_vec.push(req),
                            None => {}
                        };
                    }
                    _ => {}
                },
                _ => {}
            }
        });
    (ask_vec, bid_vec)
}
