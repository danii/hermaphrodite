use self::super::{
	Socket, State,
	super::interface::MinecraftServer,
	packet::{PacketEnum, StatusResponse, StatusPong}
};
use std::{sync::Arc, net::{TcpListener, ToSocketAddrs}};

pub fn run_server(internal: Arc<dyn MinecraftServer>, address: impl ToSocketAddrs) {
	let io = TcpListener::bind(address).unwrap();

	let mut io = Socket::new(io.accept().unwrap().0);

	loop {
		io.recv().unwrap().into_iter().for_each(|packet| {
			println!("{:?}", packet);
			match packet {
				PacketEnum::StatusRequest(_) => {
					io.send(vec![
						StatusResponse {
							protocol_name: "1.16.4".to_owned(),
							protocol_version: 754,
							players_max: 10,
							players_online: 0,
							players_sample: vec![],
							display_motd: internal.message_of_the_day()
						}.into()
					]).unwrap();
				},
				PacketEnum::StatusPing(packet) => {
					io.send(vec![
						StatusPong(packet.0).into()
					]).unwrap();
				}
				_ => {}
			}
		})
	}
}
