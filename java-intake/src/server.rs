use self::super::{
	packet::{
		Packet,
		StatusResponse,
		StatusPing,
		StatusPong,
		LoginStart,
		LoginSuccess,
		PlayJoinGame,
		PlayPlayerPositionRotationServer,

		Dimension,
		Biome,
		DimensionCodec
	},
	socket::Socket
};
use hermaphrodite::interface::MinecraftServer;
use maplit::hashmap;
use std::{
	io::{Error, ErrorKind, Result}, sync::Arc,
	net::{TcpListener, ToSocketAddrs}
};

pub fn run_server<'s, S>(server: Arc<S>, address: impl ToSocketAddrs)
		where S: MinecraftServer<'s> {
	let socket = TcpListener::bind(address).unwrap();

	let mut socket = Socket::new(socket.accept().unwrap().0);

	loop {
		socket.recv().unwrap().into_iter()
			.try_for_each(|packet| process_packet(packet, &mut socket, &*server))
				.unwrap()
	}
}

pub fn process_packet<'s, S>(packet: Packet, socket: &mut Socket,
		server: &S) -> Result<()> where S: MinecraftServer<'s> {
	println!("{:?}", packet);
	match packet {
		Packet::Handshake(_) | Packet::PlayClientSettings(_) => Ok(()),
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
					x: 0.,
					y: 0.,
					z: 0.,
					yaw: 0.,
					pitch: 0.,
					flags: 0,
					teleport_id: 0
				}.into()
			])
		},
		Packet::PlayPluginMessageClient(packet) => {
			Ok(())
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
