use self::super::{
	packet::{
		Packet,
		StatusResponse,
		StatusPing,
		StatusPong,
		LoginStart,
		LoginSuccess,
		PlayChunkData,
		PlayJoinGame,
		PlayPlayerPositionRotationServer,

		HeightMap,
		Dimension,
		Biome,
		DimensionCodec
	},
	socket::Socket
};
use hermaphrodite::interface::MinecraftServer;
use maplit::hashmap;
use std::{
	io::{Error, ErrorKind, Result},
	sync::{Arc, mpsc::{Receiver, TryRecvError, channel}},
	time::Duration,
	thread::{spawn as thread, sleep},
	net::{TcpListener, ToSocketAddrs}
};

pub fn run_server<'s, S>(server: Arc<S>, address: impl ToSocketAddrs)
		where S: MinecraftServer<'s> + 'static {
	let socket = TcpListener::bind(address).unwrap();
	let (sender, receiver) = channel();
	let client_server = server.clone();
	thread(move || run_clients(client_server, receiver));

	loop {
		let client = socket.accept().unwrap().0;
		sender.send(Socket::new(client)).unwrap()
	}
}

pub fn run_clients<'s, S>(server: Arc<S>, incoming: Receiver<Socket>)
		where S: MinecraftServer<'s> + 'static {
	let mut sockets = Vec::new();

	loop {
		match incoming.try_recv() {
			Err(TryRecvError::Disconnected) => panic!(),
			Ok(socket) => sockets.push(socket),
			_ => ()
		}

		sockets.iter_mut().try_for_each(|socket| {
			socket.recv().map_err(|error| error.0)?.into_iter()
				.try_for_each(|packet| process_packet(packet, socket, &*server))
		}).unwrap();

		sleep(Duration::from_micros(1))
	}
}

pub fn process_packet<'s, S>(packet: Packet, socket: &mut Socket, server: &S)
		-> Result<()> where S: MinecraftServer<'s> {
	match packet {
		Packet::Handshake(_)
			| Packet::PlayClientSettings(_)
			| Packet::PlayPluginMessageClient(_)
			| Packet::PlayTeleportConfirm(_)
			| Packet::PlayChatMessage(_)
			| Packet::PlayPlayerPositionClient(_)
			| Packet::PlayPlayerPositionRotationClient(_)
			| Packet::PlayPlayerRotationClient(_)
			| Packet::PlayPlayerAbilities(_) =>
				Ok(()),
		Packet::StatusRequest(_) => socket.send(vec![
			StatusResponse {
				protocol_name: "1.16.4".to_owned(),
				protocol_version: 754,
				players_online: 0,
				players_max: -834904539isize as usize,
				players_sample: vec![],
				display_motd: server.message_of_the_day()
			}.into()
		]),
		Packet::StatusPing(StatusPing(nonce)) => socket.send(vec![
			StatusPong(nonce).into()
		]),
		Packet::LoginStart(LoginStart(username)) => {
			server.new_pov(username.clone().into_boxed_str());

			let (dimension, dimension_codec) = dimension_and_codecs();

			socket.send(vec![
				LoginSuccess {
					uuid: 200,
					username: username
				}.into(),
				PlayJoinGame {
					entity_id: 0,
					gamemode_current: 1,
					gamemode_previous: 255,
					gamemode_hardcore: false,
					view_distance: 1,
					reduced_debug: false,
					respawn_screen: true,
					world_list: vec!["minecraft:overworld".to_owned()],
					world_name: "minecraft:overworld".to_owned(),
					seed_hashed: 0,
					world_debug: false,
					world_flat: true,
					dimension,
					dimension_codec
				}.into(),
				PlayPlayerPositionRotationServer {
					x: 8.,
					y: 1000.,
					z: 8.,
					yaw: 0.,
					pitch: 0.,
					flags: 0,
					teleport_id: 0
				}.into()
			])?;


			std::thread::sleep(std::time::Duration::from_secs(2));
			socket.send(vec![
				PlayChunkData {
					position: (0, 0),
					height_map: HeightMap {
						height_map: vec![0; 36]
					}
				}.into()
			])
		},
		_ => Err(Error::new(ErrorKind::InvalidData,
			format!("Idk this packet, {:?}.", packet)))
	}
}

fn dimension_and_codecs() -> (Dimension, DimensionCodec) {
	let this_dimension = Dimension {
		work_anchor: false,
		work_bed: true,
		work_piglin: false,
		work_raids: true,
		work_skylight: true,
		category_infiniburn: "minecraft:infiniburn_overworld".to_owned(),
		category_effects: "minecraft:overworld".to_owned(),
		light: 0.,
		height: 256,
		scale: 1.,
		natural: true,
		ceiling: false,
		ultrawarm: false
	};

	let dimensions = hashmap! {
		"minecraft:overworld".to_owned() => this_dimension.clone(),
	};

	let biomes = hashmap! {
		"minecraft:plains".to_owned() => Biome {
			precipitation: "rain".to_owned(),
			depth: 0.125,
			temperature: 0.8,
			scale: 0.05,
			downfall: 0.4,
			category: "plains".to_owned(),

			color_sky: 7907327,
			color_water_fog: 329011,
			color_fog: 12638463,
			color_water: 4159204,

			mood_tick_delay: 6000,
			mood_offset: 2.,
			mood_sound: "minecraft:ambient.cave".to_owned(),
			mood_block_search_extent: 8
		}
	};

	(this_dimension, DimensionCodec {dimensions, biomes})
}
