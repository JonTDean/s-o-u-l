mod app;
mod core;
mod input;
mod network;  
mod intelligence_engine;
mod state;
mod ui;
mod tests;
mod dev_utils;

fn main() {
    app::runner::run();
}
