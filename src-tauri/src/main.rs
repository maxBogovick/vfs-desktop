// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use llm_utl::api::Scan;

fn main() {
//run();
    vfdir_lib::run()
}

fn run() {
    Scan::dir("../")
        .allow_only(vec!("**/*.rs", "**/*.ts", "**/*.vue"))
        .run().unwrap();
}

