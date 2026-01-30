mod core;
mod flight;
mod physics;
mod render;

use core::stage::Stage;

use miniquad::{conf::Conf, *};

fn main() {
    let conf: Conf = conf::Conf::default();
    start(conf, || Box::new(Stage::new()))
}
