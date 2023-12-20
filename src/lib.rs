use std::env;

#[macro_use]
extern crate rocket;

pub mod api;
pub mod db;

pub fn get_env(key: &str) -> String {
    let not_set = format!("env variable '{}' required", key);
    env::var(key).expect(&not_set)
}
