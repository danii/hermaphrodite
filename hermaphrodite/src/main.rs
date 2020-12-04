#![feature(decl_macro, try_blocks, raw)]

pub mod java;
pub mod interface;
pub mod server;
pub mod util;

use self::{java::server::run_server, interface::{Event, MinecraftServer}, server::Server};
use std::sync::{Arc, Mutex};

//use libloading::Library;

fn main() {
	/*let lib = Library::new("./../mc-test/target/release/libmc_test.so").unwrap();

	unsafe {
		let a = lib.get::<fn()>(b"__hermaphrodite_plugin\0").unwrap();
		a();
	}*/

	//println!("{:?}", lib);

	/*let a = std::time::Instant::now();
	let server = Arc::new(Server::new());
	server.event_listener_register(&|event: &MyEvent, _| {
		println!("{:?}", event);
		event.push_intent(MyIntent::Cancel);
	});

	server.event_dispatch(MyEvent(Mutex::new(Vec::new()), a));*/

	let server = Arc::new(Server::new());
	run_server(server.clone(), "127.0.0.1:25565");
}

#[derive(Debug)]
struct MyEvent(Mutex<Vec<MyIntent>>, std::time::Instant);

impl Event for MyEvent {
	type Intent = MyIntent;

	fn push_intent(&self, intent: Self::Intent) {
		self.0.lock().unwrap().push(intent);
	}

	fn handle<'l, S>(self, _: &S)
			where Self: Sized, S: MinecraftServer<'l> {
		println!("{:?}", std::time::Instant::now() - self.1);
		println!("{:?}", self.0.lock().unwrap());
	}
}

#[derive(Debug)]
enum MyIntent {
	Cancel
}
