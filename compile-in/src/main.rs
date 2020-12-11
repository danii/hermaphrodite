#![feature(try_blocks)]

use hermaphrodite::server::Server;
use java_intake::server::run_server;
use std::{sync::Arc, thread::spawn as thread};

fn main() {
	let server = Arc::new(Server::new());

	let java_intake = server.clone();
	thread(move || {
		run_server(java_intake, "0.0.0.0:25565");
	});

	server.run();
}
