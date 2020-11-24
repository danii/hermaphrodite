#![feature(decl_macro, try_blocks)]

pub mod java;
pub mod interface;
pub mod server;

use self::{java::server::run_server, server::Server};
use std::sync::Arc;

fn main() {
	let server = Arc::new(Server::new());
	run_server(server.clone(), "127.0.0.1:25565");
}
