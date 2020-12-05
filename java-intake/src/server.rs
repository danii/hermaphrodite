use self::super::{
	packet::{
		Packet,
		StatusResponse,
		StatusPing,
		StatusPong,
		LoginStart,
		LoginSuccess,
		PlayPlayerPositionRotationServer
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
				players_max: usize::MAX,
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
