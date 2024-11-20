use std::{collections::HashMap, io, rc::Rc, time::Instant};
use log::info;
use model::*;
use strength::Trim;
pub use error::Error;
use crate::{
    load::*,
    area::*,
    icing_stab::*,
    math::*,
    displacement::*
};
use draught::Draught;
use icing_timber::IcingTimberBound;
//use data::api_server::*;

mod model;
mod error;
mod math;
mod strength;
mod draught;
mod trim;
mod parameters;
mod area;
mod icing_stab;
mod icing_timber;
mod load;
mod data;
mod displacement;

fn main() {
    let model = Model::new();
    let loa = model.loa();
    let middle_x = model.middle_x();    
    let bounds = Bounds::from_n(loa, middle_x, 20).unwrap();
    for b in bounds.iter() {
        dbg!(model.mass(b, LoadingType::Hull));
        dbg!(model.mass(b, LoadingType::Equipment));
        dbg!(model.value(b, 1., 1.));
    }
}
