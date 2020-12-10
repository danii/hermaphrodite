use java_intake::server::run_server;
use std::sync::Arc;
use hermaphrodite::server::Server;

use java_intake::packet::*;
use java_intake::nbt::Serializer;
use serde::ser::Serialize as _;

fn main() {
	let server = Arc::new(Server::new());
	run_server(server, "127.0.0.1:25565");
}
