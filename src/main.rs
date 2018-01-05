extern crate ws;
extern crate uuid;
#[macro_use]
extern crate log;
extern crate cargo_style_logger;
extern crate websocket_api;

use log::LogLevel;
use cargo_style_logger::Logger;

mod server;
mod git;

const SOCKET_ADDR: &'static str = "0.0.0.0:1710";

use server::launch_server;

fn main() {
    Logger::init(LogLevel::Info);
    info!("Collaboration server v{}-{}", env!("CARGO_PKG_VERSION"), git::COMMIT_HASH);

    launch_server(SOCKET_ADDR);
}
