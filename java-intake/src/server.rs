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
		Packet::Handshake(_) => Ok(()),
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

			std::thread::sleep(std::time::Duration::from_millis(1000));

			socket.send(vec![
				LoginSuccess {
					uuid: 200,
					username: username
				}.into()
			])?;

			std::thread::sleep(std::time::Duration::from_millis(1000));

			let dim = Dimension {
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

			let mut bs = std::collections::HashMap::new();
			bs.insert("minecraft:overworld".to_owned(), dim.clone());

			let mut bs2 = std::collections::HashMap::new();
			bs2.insert("minecraft:plains".to_owned(), Biome {
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
			});

			socket.send(vec![
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
					dimension: dim.clone(),
					dimension_codec: DimensionCodec {
						dimensions: bs,
						biomes: bs2
					}
				}.into()
			])?;

			std::thread::sleep(std::time::Duration::from_millis(1000));

			socket.send(vec![
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
		_ => Err(Error::new(ErrorKind::InvalidData,
			format!("Idk this packet, {:?}.", packet)))
	}
}
