use self::super::{
	interface::{Event, MinecraftServer},
	util::{GenericTraitObject, generic_trait_downcast, generic_trait}
};
use bit_range::BitRange;
use std::{
	any::TypeId, borrow::Borrow, collections::{HashSet, HashMap},
	hash::{Hash, Hasher}, sync::Mutex, thread::sleep, time::{Duration, Instant}
};

pub struct Server<'l> {
	event_listeners: Mutex<HashMap<TypeId, Vec<GenericTraitObject<'l>>>>,
	entities: Mutex<HashSet<Player>>,
	chunks: Mutex<HashSet<Chunk>>
	//orphanned_connections: Vec<()>,
}

#[derive(Eq, PartialEq)]
pub struct Player {
	username: Box<str>,
	x: (u64, u16),
	y: (u64, u16),
	z: (u64, u16)
}

impl Hash for Player {
	fn hash<H>(&self, hasher: &mut H)
			where H: Hasher {
		hasher.write_u8(0)
	}
}

impl<'l> Server<'l> {
	pub fn new() -> Self {
		Self {
			event_listeners: Mutex::new(HashMap::new()),
			entities: Mutex::new(HashSet::new()),
			chunks: Mutex::new(HashSet::new())
		}
	}

	pub fn run(&self) {
		let duration = Duration::from_nanos(1_000_000_000 / 1);
		println!("Running @{:?}/Tick", duration);

		loop {
			let then = Instant::now();

			self.tick();

			let elapsed = Instant::now() - then;
			match duration.checked_sub(elapsed) {
				Some(time) => {
					// We're on time.

					println!("Tick completed in {:?}.", elapsed);
					sleep(time);
				},
				None => panic!()
			}
		}
	}

	fn tick(&self) {
		self.manage_chunks();
	}

	fn manage_chunks(&self) {
		let players = self.entities.lock().unwrap();
		let mut chunks = self.chunks.lock().unwrap();

		//let render_distance = 2;
		//let load_max = 10000;
		players.iter().for_each(|player| {
			let chunk_pos = (player.x.0 / 16, player.z.0 / 16);

			chunks.get_or_insert_with(&chunk_pos, |_| {
				println!("Loaded chunk @{},{}.", chunk_pos.0, chunk_pos.1);
				Chunk::solid(chunk_pos, 0)
			});
		});
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

#[derive(Eq, PartialEq)]
struct Chunk {
	data: Box<[u8]>,
	diff_source: DiffSource,
	pallette: Option<Palette>,
	layer_mask: u16,
	position: (u64, u64)
}

impl Chunk {
	fn solid(position: (u64, u64), identifier: u16) -> Self {
		Self {
			data: Box::new([]),
			diff_source: DiffSource::Solid(identifier),
			pallette: None,
			layer_mask: 0b0000000000000000,
			position
		}
	}
}

impl Hash for Chunk {
	fn hash<H>(&self, hasher: &mut H)
			where H: Hasher {
		self.position.hash(hasher)
	}
}

impl Borrow<(u64, u64)> for Chunk {
	fn borrow(&self) -> &(u64, u64) {
		&self.position
	}
}

#[derive(Eq, PartialEq)]
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

#[derive(Eq, PartialEq)]
enum DiffSource {
	Generator(Option<u64>),
	Solid(u16)
}
