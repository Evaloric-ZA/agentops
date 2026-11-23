// A thin binary: everything lives in the library so it can be tested.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    agentops_lib::run()
}
