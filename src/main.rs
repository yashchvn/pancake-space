mod components;
mod core;
mod render;
mod systems;

use core::stage::Stage;

use miniquad::{conf::Conf, *};

fn main() {
    let conf: Conf = conf::Conf::default();
    start(conf, || Box::new(Stage::new()))
}
