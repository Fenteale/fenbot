use serde::{Deserialize, Serialize};
use serde_yaml::{self};

use std::fs::File;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FRoles {
    pub fox: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub token: String,
    pub admin: u64,
    pub poll_id: u64,
    pub roles: FRoles,
}

pub fn load_config() -> Config {
    println!("Loading config from config.yaml...");
    let f = File::open("config.yaml").expect("Could not load config.yaml");
    let c: Config = serde_yaml::from_reader(f).expect("Could not deserialize yaml.");
    
    c
}

pub fn write_poll_id(pid: u64) {
    println!("Writing poll id to file...");

    let f = File::open("config.yaml").expect("Could not load config.yaml");
    let mut c: Config = serde_yaml::from_reader(f).expect("Could not deserialize yaml.");

    c.poll_id = pid;
    let f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("config.yaml")
        .expect("Could not edit config.yaml");
    serde_yaml::to_writer(f, &c).unwrap();
    println!("Poll id saved in config.yaml.");
}