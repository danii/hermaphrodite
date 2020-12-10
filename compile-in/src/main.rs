use hermaphrodite::server::Server;
use java_intake::server::run_server;
use std::sync::Arc;

fn main() {
	let server = Arc::new(Server::new());
	run_server(server, "127.0.0.1:25565");
}
