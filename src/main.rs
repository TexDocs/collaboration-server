extern crate websocket;
extern crate futures;
extern crate tokio_core;
#[macro_use] extern crate log;
extern crate cargo_style_logger;

use log::LogLevel;
use cargo_style_logger::Logger;

mod server;
mod git;

const SOCKET_ADDR: &'static str = "0.0.0.0:1710";

fn main() {
    Logger::init(LogLevel::Info);
    info!("Collaboration server v{}-{}", env!("CARGO_PKG_VERSION"), git::COMMIT_HASH);

    info!("Listening on {}", SOCKET_ADDR);
    server::start_server(SOCKET_ADDR);
}
