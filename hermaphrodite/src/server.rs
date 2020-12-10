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

use self::super::{interface::{Event, MinecraftServer}, util::{GenericTraitObject, generic_trait, generic_trait_downcast}};
use bit_range::BitRange;
use std::{any::{Any, TypeId}, collections::{HashMap, HashSet}, sync::Mutex};

pub struct Server<'l> {
	event_listeners: Mutex<HashMap<TypeId, Vec<GenericTraitObject<'l>>>>,
	entities: Mutex<HashSet<Player>>
	//orphanned_connections: Vec<()>,
	//loaded_chunks: HashMap<(u64, u64), Chunk>
}

pub struct Player {
	username: Box<str>,
	x: (u64, u16),
	y: (u64, u16),
	z: (u64, u16)
}

impl std::hash::Hash for Player {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		state.write_u8(0)
	}
}

impl std::cmp::PartialEq for Player {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl std::cmp::Eq for Player {}

impl<'l> Server<'l> {
	pub fn new() -> Self {
		Self {
			event_listeners: Mutex::new(HashMap::new()),
			entities: Mutex::new(HashSet::new())
		}
	}

	fn tick(&self) {

	}
}

impl<'l> MinecraftServer<'l> for Server<'l> {
	fn message_of_the_day(&self) -> String {
		"Hello, world!".to_owned()
	}

	fn event_listener_register<E>(&self, listener: &'l dyn Fn(&E, &Self))
			where E: Event + 'static {
		let mut event_listeners = self.event_listeners.lock().unwrap();

		let event_listeners = event_listeners.entry(TypeId::of::<E>())
			.or_insert_with(Vec::new);

		event_listeners.push(generic_trait!(listener));
	}

	fn event_dispatch<E>(&self, event: E)
			where E: Event + 'static {
		let event_listeners = self.event_listeners.lock().unwrap();

		event_listeners.get(&TypeId::of::<E>())
			.map(|listeners| listeners.iter()
				.for_each(|listener| unsafe {
					let func: &dyn Fn(&E, &Self) = generic_trait_downcast!(listener);
					func(&event, self);
				}));

		event.handle(self);
	 }
	 
	 fn new_pov(&self, name: Box<str>) {
			let mut entities = self.entities.lock().unwrap();
			entities.insert(Player {
				username: name,
				x: (0, 0),
				y: (0, 0),
				z: (0, 0)
			});
	 }
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
