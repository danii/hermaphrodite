use std::{cmp::Ordering, thread::sleep, time::{Duration, Instant}};
use bit_range::BitRange;

/*struct Stuff {
	last: Instant,
	duration: Duration
}

fn tick(this: &mut Stuff) {
	let now = Instant::now();
	let time_passed = now - this.last;
	let mut skip_ticks = false;

	match this.duration.checked_sub(time_passed) {
		Some(time) => {
			// We're on time.

			println!("Tick done in {}ms.", time_passed.as_micros());
			sleep(time);
		},
		None => match this.duration.checked_sub(time_passed - this.duration) {
			Some(time) => {
				// We're behind!

				let time = this.duration - time;
				println!("Tick done in {}ms, {}ms behind. Continuing without intervention.", time_passed.as_millis(), time.as_millis());
			},
			None => {
				// We're severly behind!!!

				let time = time_passed - this.duration;
				let ticks_skipped = time.as_secs_f32() / this.duration.as_secs_f32();
				println!("Tick done in {}ms, {}ms behind. Skipping approximately {} ticks.", time_passed.as_millis(), time.as_millis(), ticks_skipped);

				skip_ticks = true;
			}
		}
	}

	if skip_ticks {this.last = Instant::now()}
	else {this.last = this.last + this.duration}
}*/

mod java;

fn main() {
	let server = Arc::new(Server {});

	run_java_something_or_other(server);
}

use std::sync::Arc;
use std::io::{Read, Write};

fn run_java_something_or_other(server: Arc<MinecraftServer>) {
	let server = std::net::TcpListener::bind("127.0.0.1:25565").unwrap();

	let mut socket = server.accept().unwrap().0;

	std::thread::sleep(std::time::Duration::from_millis(1000));

	let mut b = [0; 1024];
	let count = socket.read(&mut b).unwrap();
	println!("{:?}", &b[..count]);

	socket.write(&[
		&[
			0b01111010, // Length
			0b00000000, // Packet ID
			0b01111000, // String Length
		],
		"{\"version\":{\"name\":\"1.16.4\",\"protocol\":754},\"players\":{\"max\":100,\"online\":5,\"sample\":[]},\"description\":{\"text\":\"epicc\"}}".as_bytes()
	].concat()).unwrap();
}

struct Server {
	//orphanned_connections: Vec<()>,
	//loaded_chunks: HashMap<(u64, u64), Chunk>
}

impl Server {
	fn tick(&self) {

	}
}

impl MinecraftServer for Server {
	fn message_of_the_day(&self) -> String {
		"Hello, world!".to_owned()
	}
}

trait MinecraftServer {
	/// Retrieves the message of the day.
	fn message_of_the_day(&self) -> String;
}

trait PlayerController {
	
}

struct Chunk {
	data: Box<[u8]>,
	diff_source: DiffSource,
	pallette: Option<Palette>,
	layer_mask: u16,
}

impl Chunk {
	fn solid(identifier: u16) -> Self {
		Self {
			data: Box::new([]),
			diff_source: DiffSource::Solid(identifier),
			pallette: None,
			layer_mask: 0b0000000000000000
		}
	}
}

struct Palette(u16, Box<[u8]>);

impl Palette {
	fn bits_per_block(&self) -> u8 {
		(self.0 as f32 + 1.).log2().ceil() as u8
	}

	fn global(&self, local: u16, global_bits_per_block: u8) -> u16 {
		let global_bits_per_block = global_bits_per_block as u32 + 1;
		let access = local as u32 * global_bits_per_block;
		self.1.get_bit_range(access..access + global_bits_per_block) as u16
	}
}

enum DiffSource {
	Generator(Option<u64>),
	Solid(u16)
}
