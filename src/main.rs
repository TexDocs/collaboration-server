extern crate cargo_style_logger;
#[macro_use]
extern crate log;
extern crate uuid;
extern crate websocket_api;
extern crate ws;

use log::LogLevel;
use cargo_style_logger::Logger;

mod server;
mod git;

const SOCKET_ADDR: &'static str = "0.0.0.0:1710";

use server::launch_server;

fn main() {
    Logger::init(LogLevel::Info);
    info!(
        "Collaboration server v{}-{}",
        env!("CARGO_PKG_VERSION"),
        git::COMMIT_HASH
    );

    launch_server(SOCKET_ADDR);
}
