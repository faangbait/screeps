use grid;
use log::info;
use pathfinding::kuhn_munkres::{kuhn_munkres_min, Weights};
use pathfinding::matrix;
use screeps::{
    ConstructionSite, HasId, HasStore, ObjectId, SharedCreepProperties, Structure, Withdrawable,
};
use std::collections::HashMap;

use crate::basic_enums::JobType;
use crate::jobs::JobProperties;
use crate::sink::{SinkNode, SinkWork, ResourceValued};
use crate::source::{SourceNode, SourceWork};

pub fn prioritize(
    creeps: Vec<screeps::Creep>,
    structures: Vec<screeps::Structure>,
    sites: Vec<screeps::ConstructionSite>,
    resources: Vec<screeps::objects::Resource>,
    sources: Vec<screeps::objects::Source>,
) {
    // let mut bids = HashMap::<SinkWork, u32>::new();
    // let mut asks = HashMap::<SourceWork, u32>::new();
    let mut asks = vec![];
    let mut bids = vec![];
    // creeps.iter().for_each(|c| {
    //     if let Some(bid) = c.bid() {
    //         bids.insert(bid, bid.price);
    //     }
    //     if let Some(ask) = c.ask() {
    //         asks.insert(ask, ask.price);
    //     }
    // });
    structures.iter().for_each(|st| {
        if let Some(bid) = st.bid() {
            bids.push(bid);
        }
        if let Some(ask) = st.ask() {
            asks.push(ask);
        }
    });
    sites.iter().for_each(|st| {
        if let Some(bid) = st.bid() {
            bids.push(bid);
        }
    });
    resources.iter().for_each(|st| {
        if let Some(ask) = st.ask() {
            asks.push(ask);
        }
    });
    sources.iter().for_each(|st| {
        if let Some(ask) = st.ask() {
            asks.push(ask);
        }
    });

    let mut potential_contracts: Vec<(u32, SinkWork, SourceWork)> = vec![];

    // bids.keys().for_each(|bid| {
    let mut considered_bids: Vec<&SinkWork> = vec![];
    let mut considered_asks: Vec<&SourceWork> = vec![];

    let mut grid = creeps
        .iter()
        .map(|c| {
            let ask_map = asks
                .iter()
                .map(|ask| {
                    if c.untyped_id() == ask.target {
                        999
                    } else if !c.has_parts_for_job(ask.job_type) {
                        999
                    } else if let Some(bid) = c.bid() {
                        if let Some(nearby) = considered_asks
                            .clone()
                            .iter()
                            .filter(|&p| p.position.in_range_to(&bid.position, 2))
                            .max_by_key(|&a| a.price)
                        {
                            considered_asks.push(ask);
                            if nearby.price < ask.price {
                                let max_bid = bid.price as i32 * bid.resource_max.unwrap();
                                let max_ask = ask.price as i32 * ask.resource_max.unwrap();

                                if let Some(cost) = max_ask.checked_sub(max_bid) {
                                    (c.fatigue_to_pos(&ask.position) + cost as u32) as i32
                                } else {
                                    999
                                }
                            } else {
                                999
                            }
                        } else {
                            999
                        }
                    } else {
                        999
                    }
                })
                .collect::<Vec<i32>>();

            let bid_map = bids
                .iter()
                .map(|bid| {
                    if c.untyped_id() == bid.target {
                        999
                    } else if !c.has_parts_for_job(bid.job_type) {
                        999
                    } else if let Some(ask) = c.ask() {
                        let max_bid = bid.price as i32 * bid.resource_max.unwrap();
                        let max_ask = ask.price as i32 * ask.resource_max.unwrap();
                        if let Some(cost) = max_bid.checked_sub(max_ask) {
                            (c.fatigue_to_pos(&bid.position) + cost as u32) as i32
                        } else {
                            999
                        }
                    } else {
                        999
                    }
                })
                .collect::<Vec<i32>>();
            [ask_map, bid_map].concat()
        })
        .collect::<Vec<Vec<i32>>>();

    let matrix = matrix![grid.concat()];

    let assignments = kuhn_munkres_min(&matrix);

    assignments.1.iter().enumerate().for_each(|(i, u)| {
        if *u > asks.len() {
            let sink = &bids[*u - asks.len()];
            info!("{:?}, {:?}", creeps[i].name(), sink.position);
            creeps[i].set_resource_val(Some(*sink));

            match sink.ty {
                crate::basic_enums::SinkType::Creep => {
                    let obj: Result<ObjectId<screeps::Creep>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            JobType::Station => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::Heal => {
                                                //HEAL
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Resource => {
                    let obj: Result<ObjectId<screeps::objects::Resource>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => match o.try_resolve() {
                            Err(_) => {}
                            Ok(opt) => match opt {
                                None => {}
                                Some(obj) => match sink.job_type {
                                    JobType::Pickup => match creeps[i].pickup(&obj) {
                                        screeps::ReturnCode::Ok => {}
                                        screeps::ReturnCode::NotInRange => {
                                            creeps[i].move_to(&sink.position);
                                        }
                                        _ => {}
                                    },
                                    _ => {}
                                },
                            },
                        },
                    }
                }
                crate::basic_enums::SinkType::ConstructionSite => {
                    let obj: Result<ObjectId<screeps::ConstructionSite>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Build => match creeps[i].build(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Tombstone => {
                    let obj: Result<ObjectId<screeps::Tombstone>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Withdraw => {
                                                // match creeps[i].withdraw_amount(target, ty, amount)
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::PowerCreep => {
                    let obj: Result<ObjectId<screeps::PowerCreep>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::Heal => {
                                                //HEAL
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Structure => {
                    let obj: Result<ObjectId<screeps::Structure>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Station => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Controller => {
                    let obj: Result<ObjectId<screeps::StructureController>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Upgrade => {
                                                match creeps[i].upgrade_controller(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Claim => {
                                                match creeps[i].claim_controller(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Reserve => {
                                                match creeps[i].reserve_controller(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::AttackR => {
                                                match creeps[i].attack_controller(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            },
                                            
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Container => {
                    let obj: Result<ObjectId<screeps::StructureContainer>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            },
                                            JobType::Withdraw => {
                                                match creeps[i].withdraw_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::Full => {
                                                        creeps[i].withdraw_amount(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                            creeps[i].store_free_capacity(
                                                                sink.resource_type,
                                                            )
                                                                as u32,
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            },
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            },
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            },
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            },
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Extension => {
                    let obj: Result<ObjectId<screeps::StructureExtension>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Withdraw => {
                                                match creeps[i].withdraw_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::Full => {
                                                        creeps[i].withdraw_amount(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                            creeps[i].store_free_capacity(
                                                                sink.resource_type,
                                                            )
                                                                as u32,
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Extractor => {
                    let obj: Result<ObjectId<screeps::StructureExtractor>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Factory => {
                    let obj: Result<ObjectId<screeps::StructureFactory>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Withdraw => {
                                                match creeps[i].withdraw_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::Full => {
                                                        creeps[i].withdraw_amount(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                            creeps[i].store_free_capacity(
                                                                sink.resource_type,
                                                            )
                                                                as u32,
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Lab => {
                    let obj: Result<ObjectId<screeps::StructureLab>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Withdraw => {
                                                match creeps[i].withdraw_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::Full => {
                                                        creeps[i].withdraw_amount(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                            creeps[i].store_free_capacity(
                                                                sink.resource_type,
                                                            )
                                                                as u32,
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Link => {
                    let obj: Result<ObjectId<screeps::StructureLink>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Withdraw => {
                                                match creeps[i].withdraw_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::Full => {
                                                        creeps[i].withdraw_amount(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                            creeps[i].store_free_capacity(
                                                                sink.resource_type,
                                                            )
                                                                as u32,
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Nuker => {
                    let obj: Result<ObjectId<screeps::StructureNuker>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Observer => {
                    let obj: Result<ObjectId<screeps::StructureObserver>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::PowerSpawn => {
                    let obj: Result<ObjectId<screeps::StructurePowerSpawn>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Withdraw => {
                                                match creeps[i].withdraw_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::Full => {
                                                        creeps[i].withdraw_amount(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                            creeps[i].store_free_capacity(
                                                                sink.resource_type,
                                                            )
                                                                as u32,
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Rampart => {
                    let obj: Result<ObjectId<screeps::StructureRampart>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Station => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Road => {
                    let obj: Result<ObjectId<screeps::StructureRoad>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Spawn => {
                    let obj: Result<ObjectId<screeps::StructureSpawn>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Withdraw => {
                                                match creeps[i].withdraw_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::Full => {
                                                        creeps[i].withdraw_amount(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                            creeps[i].store_free_capacity(
                                                                sink.resource_type,
                                                            )
                                                                as u32,
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Storage => {
                    let obj: Result<ObjectId<screeps::StructureStorage>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Withdraw => {
                                                match creeps[i].withdraw_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::Full => {
                                                        creeps[i].withdraw_amount(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                            creeps[i].store_free_capacity(
                                                                sink.resource_type,
                                                            )
                                                                as u32,
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Terminal => {
                    let obj: Result<ObjectId<screeps::StructureTerminal>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Withdraw => {
                                                match creeps[i].withdraw_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::Full => {
                                                        creeps[i].withdraw_amount(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                            creeps[i].store_free_capacity(
                                                                sink.resource_type,
                                                            )
                                                                as u32,
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Tower => {
                    let obj: Result<ObjectId<screeps::StructureTower>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Withdraw => {
                                                match creeps[i].withdraw_amount(
                                                    &obj,
                                                    sink.resource_type.unwrap(),
                                                    sink.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::Full => {
                                                        creeps[i].withdraw_amount(
                                                            &obj,
                                                            sink.resource_type.unwrap(),
                                                            creeps[i].store_free_capacity(
                                                                sink.resource_type,
                                                            )
                                                                as u32,
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
                crate::basic_enums::SinkType::Wall => {
                    let obj: Result<ObjectId<screeps::StructureWall>, _> =
                        screeps::traits::TryFrom::try_from(sink.target.to_u128());
                    match obj {
                        Err(_) => {}
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {}
                                Ok(opt) => match opt {
                                    None => {}
                                    Some(obj) => {
                                        match sink.job_type {
                                            //A
                                            JobType::Repair => match creeps[i].repair(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&sink.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&sink.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&sink.position);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
            }
        } else {
            let source = &asks[*u];
            info!("{:?}, {:?}", creeps[i].name(), source.position);
            creeps[i].set_resource_val(creeps[i].bid());
            match source.ty {
                crate::basic_enums::SourceType::Creep => {
                    let obj: Result<ObjectId<screeps::Creep>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => {
                                        match source.job_type {
                                            JobType::Station => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    source.resource_type.unwrap(),
                                                    source.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            source.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&source.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&source.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            JobType::Heal => {
                                                //HEAL
                                            }
                                            _ => {}

                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::Resource => {
                    let obj: Result<ObjectId<screeps::objects::Resource>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        JobType::Pickup => match creeps[i].pickup(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            _ => {}
                                        },
                                        _ => {}
                                    },
    
                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::Source => {
                    let obj: Result<ObjectId<screeps::Source>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        JobType::Harvest => match creeps[i].harvest(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            _ => {}
                                        },
                                        _ => {}
                                    },
                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::Mineral => {
                    let obj: Result<ObjectId<screeps::Mineral>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        JobType::Pickup => match creeps[i].harvest(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            _ => {}
                                        },
                                        _ => {}
                                    },
                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::Deposit => {
                    let obj: Result<ObjectId<screeps::Deposit>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        JobType::Pickup => match creeps[i].harvest(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            _ => {}
                                        },
                                        _ => {}
                                    },
                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::Tombstone => {
                    let obj: Result<ObjectId<screeps::Tombstone>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        JobType::Withdraw => {
                                            match creeps[i].withdraw_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::Full => {
                                                    creeps[i].withdraw_amount(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                        creeps[i].store_free_capacity(
                                                            source.resource_type,
                                                        )
                                                            as u32,
                                                    );
                                                }
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                }
                                                _ => {}
                                            }
                                        }
                                        _ => {}
                                    },
                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::PowerCreep => {
                    let obj: Result<ObjectId<screeps::PowerCreep>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => {
                                        match source.job_type {
                                            //A
                                            JobType::Transfer => {
                                                match creeps[i].transfer_amount(
                                                    &obj,
                                                    source.resource_type.unwrap(),
                                                    source.resource_max.unwrap() as u32,
                                                ) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotEnough => {
                                                        creeps[i].transfer_all(
                                                            &obj,
                                                            source.resource_type.unwrap(),
                                                        );
                                                    }
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&source.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Attack => match creeps[i].attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                }
                                                _ => {}
                                            },
                                            JobType::AttackR => {
                                                match creeps[i].ranged_attack(&obj) {
                                                    screeps::ReturnCode::Ok => {}
                                                    screeps::ReturnCode::NotInRange => {
                                                        creeps[i].move_to(&source.position);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            JobType::Defend => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            JobType::DefendR => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            JobType::Heal => {
                                                //HEAL
                                            }
                                            _ => {}
                                        }
                                    }

                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::Container => {
                    let obj: Result<ObjectId<screeps::StructureContainer>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        //A
                                        JobType::Repair => match creeps[i].repair(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            },
                                            _ => {},
                                        },
                                        JobType::Transfer => {
                                            match creeps[i].transfer_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::NotEnough => {
                                                    creeps[i].transfer_all(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Withdraw => {
                                            match creeps[i].withdraw_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::Full => {
                                                    creeps[i].withdraw_amount(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                        creeps[i].store_free_capacity(
                                                            source.resource_type,
                                                        )
                                                            as u32,
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Attack => match creeps[i].attack(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            _ => {}
                                        },
                                        JobType::AttackR => {
                                            match creeps[i].ranged_attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                }
                                                _ => {}
                                            }
                                        },
                                        JobType::Defend => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        JobType::DefendR => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::Extension => {
                    let obj: Result<ObjectId<screeps::StructureExtension>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        //A
                                        JobType::Repair => match creeps[i].repair(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            },
                                            _ => {},
                                        },
                                        JobType::Transfer => {
                                            match creeps[i].transfer_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::NotEnough => {
                                                    creeps[i].transfer_all(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Withdraw => {
                                            match creeps[i].withdraw_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::Full => {
                                                    creeps[i].withdraw_amount(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                        creeps[i].store_free_capacity(
                                                            source.resource_type,
                                                        )
                                                            as u32,
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Attack => match creeps[i].attack(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            _ => {}
                                        },
                                        JobType::AttackR => {
                                            match creeps[i].ranged_attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                }
                                                _ => {}
                                            }
                                        },
                                        JobType::Defend => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        JobType::DefendR => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        _ => {}
                                    }

                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::Factory => {
                    let obj: Result<ObjectId<screeps::StructureFactory>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        //A
                                        JobType::Repair => match creeps[i].repair(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            },
                                            _ => {},
                                        },
                                        JobType::Transfer => {
                                            match creeps[i].transfer_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::NotEnough => {
                                                    creeps[i].transfer_all(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Withdraw => {
                                            match creeps[i].withdraw_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::Full => {
                                                    creeps[i].withdraw_amount(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                        creeps[i].store_free_capacity(
                                                            source.resource_type,
                                                        )
                                                            as u32,
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Attack => match creeps[i].attack(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            _ => {}
                                        },
                                        JobType::AttackR => {
                                            match creeps[i].ranged_attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                }
                                                _ => {}
                                            }
                                        },
                                        JobType::Defend => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        JobType::DefendR => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        _ => {}
                                    }

                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::Lab => {
                    let obj: Result<ObjectId<screeps::StructureLab>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        //A
                                        JobType::Repair => match creeps[i].repair(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            },
                                            _ => {},
                                        },
                                        JobType::Transfer => {
                                            match creeps[i].transfer_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::NotEnough => {
                                                    creeps[i].transfer_all(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Withdraw => {
                                            match creeps[i].withdraw_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::Full => {
                                                    creeps[i].withdraw_amount(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                        creeps[i].store_free_capacity(
                                                            source.resource_type,
                                                        )
                                                            as u32,
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Attack => match creeps[i].attack(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            _ => {}
                                        },
                                        JobType::AttackR => {
                                            match creeps[i].ranged_attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                }
                                                _ => {}
                                            }
                                        },
                                        JobType::Defend => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        JobType::DefendR => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        _ => {}
                                    }

                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::Link => {
                    let obj: Result<ObjectId<screeps::StructureLink>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        //A
                                        JobType::Repair => match creeps[i].repair(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            },
                                            _ => {},
                                        },
                                        JobType::Transfer => {
                                            match creeps[i].transfer_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::NotEnough => {
                                                    creeps[i].transfer_all(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Withdraw => {
                                            match creeps[i].withdraw_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::Full => {
                                                    creeps[i].withdraw_amount(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                        creeps[i].store_free_capacity(
                                                            source.resource_type,
                                                        )
                                                            as u32,
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Attack => match creeps[i].attack(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            _ => {}
                                        },
                                        JobType::AttackR => {
                                            match creeps[i].ranged_attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                }
                                                _ => {}
                                            }
                                        },
                                        JobType::Defend => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        JobType::DefendR => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        _ => {}
                                    }

                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::PowerSpawn => {
                    let obj: Result<ObjectId<screeps::StructurePowerSpawn>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        //A
                                        JobType::Repair => match creeps[i].repair(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            },
                                            _ => {},
                                        },
                                        JobType::Transfer => {
                                            match creeps[i].transfer_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::NotEnough => {
                                                    creeps[i].transfer_all(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Withdraw => {
                                            match creeps[i].withdraw_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::Full => {
                                                    creeps[i].withdraw_amount(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                        creeps[i].store_free_capacity(
                                                            source.resource_type,
                                                        )
                                                            as u32,
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Attack => match creeps[i].attack(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            _ => {}
                                        },
                                        JobType::AttackR => {
                                            match creeps[i].ranged_attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                }
                                                _ => {}
                                            }
                                        },
                                        JobType::Defend => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        JobType::DefendR => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        _ => {}
                                    }

                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::Spawn => {
                    let obj: Result<ObjectId<screeps::StructureSpawn>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        //A
                                        JobType::Repair => match creeps[i].repair(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            },
                                            _ => {},
                                        },
                                        JobType::Transfer => {
                                            match creeps[i].transfer_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::NotEnough => {
                                                    creeps[i].transfer_all(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Withdraw => {
                                            match creeps[i].withdraw_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::Full => {
                                                    creeps[i].withdraw_amount(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                        creeps[i].store_free_capacity(
                                                            source.resource_type,
                                                        )
                                                            as u32,
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Attack => match creeps[i].attack(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            _ => {}
                                        },
                                        JobType::AttackR => {
                                            match creeps[i].ranged_attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                }
                                                _ => {}
                                            }
                                        },
                                        JobType::Defend => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        JobType::DefendR => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        _ => {}
                                    }

                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::Storage => {
                    let obj: Result<ObjectId<screeps::StructureStorage>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        //A
                                        JobType::Repair => match creeps[i].repair(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            },
                                            _ => {},
                                        },
                                        JobType::Transfer => {
                                            match creeps[i].transfer_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::NotEnough => {
                                                    creeps[i].transfer_all(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Withdraw => {
                                            match creeps[i].withdraw_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::Full => {
                                                    creeps[i].withdraw_amount(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                        creeps[i].store_free_capacity(
                                                            source.resource_type,
                                                        )
                                                            as u32,
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Attack => match creeps[i].attack(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            _ => {}
                                        },
                                        JobType::AttackR => {
                                            match creeps[i].ranged_attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                }
                                                _ => {}
                                            }
                                        },
                                        JobType::Defend => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        JobType::DefendR => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        _ => {}
                                    }

                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::Terminal => {
                    let obj: Result<ObjectId<screeps::StructureTerminal>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        //A
                                        JobType::Repair => match creeps[i].repair(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            },
                                            _ => {},
                                        },
                                        JobType::Transfer => {
                                            match creeps[i].transfer_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::NotEnough => {
                                                    creeps[i].transfer_all(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Withdraw => {
                                            match creeps[i].withdraw_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::Full => {
                                                    creeps[i].withdraw_amount(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                        creeps[i].store_free_capacity(
                                                            source.resource_type,
                                                        )
                                                            as u32,
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Attack => match creeps[i].attack(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            _ => {}
                                        },
                                        JobType::AttackR => {
                                            match creeps[i].ranged_attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                }
                                                _ => {}
                                            }
                                        },
                                        JobType::Defend => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        JobType::DefendR => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        _ => {}
                                    }

                                }
                            }
                        }
                    }
                },
                crate::basic_enums::SourceType::Tower => {
                    let obj: Result<ObjectId<screeps::StructureTower>, _> =
                        screeps::traits::TryFrom::try_from(source.target.to_u128());
                    match obj {
                        Err(_) => {},
                        Ok(o) => {
                            match o.try_resolve() {
                                Err(_) => {},
                                Ok(opt) => match opt {
                                    None => {},
                                    Some(obj) => match source.job_type {
                                        //A
                                        JobType::Repair => match creeps[i].repair(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            },
                                            _ => {},
                                        },
                                        JobType::Transfer => {
                                            match creeps[i].transfer_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::NotEnough => {
                                                    creeps[i].transfer_all(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Withdraw => {
                                            match creeps[i].withdraw_amount(
                                                &obj,
                                                source.resource_type.unwrap(),
                                                source.resource_max.unwrap() as u32,
                                            ) {
                                                screeps::ReturnCode::Ok => {},
                                                screeps::ReturnCode::Full => {
                                                    creeps[i].withdraw_amount(
                                                        &obj,
                                                        source.resource_type.unwrap(),
                                                        creeps[i].store_free_capacity(
                                                            source.resource_type,
                                                        )
                                                            as u32,
                                                    );
                                                },
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                },
                                                _ => {}
                                            }
                                        },
                                        JobType::Attack => match creeps[i].attack(&obj) {
                                            screeps::ReturnCode::Ok => {}
                                            screeps::ReturnCode::NotInRange => {
                                                creeps[i].move_to(&source.position);
                                            }
                                            _ => {}
                                        },
                                        JobType::AttackR => {
                                            match creeps[i].ranged_attack(&obj) {
                                                screeps::ReturnCode::Ok => {}
                                                screeps::ReturnCode::NotInRange => {
                                                    creeps[i].move_to(&source.position);
                                                }
                                                _ => {}
                                            }
                                        },
                                        JobType::Defend => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        JobType::DefendR => {
                                            creeps[i].move_to(&source.position);
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                },
            }
        }
    })
}
