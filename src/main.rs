use messenger::{Config, Messanger};
use std::{env, process};

fn main() {
    println!("Starting...");
    Messanger::build().run().expect("Error");
}
