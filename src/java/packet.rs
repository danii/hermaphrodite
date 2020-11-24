use self::super::{
	Bound::{self, Client, Server},
	State::{
		self,
		Handshake as HandshakeState, Status as StatusState, Login as LoginState
	},
	types::{Read, Write}
};
use serde::ser::{Serialize, SerializeMap, Serializer};
use serde_json::to_string;
use std::{
	fmt::Debug, io::{Error, ErrorKind, Result}, result::Result as STDResult
};

pub fn deserialize(reader: &mut impl Read, state: State, packet: u32,
		bound: Bound) -> Option<Result<Box<dyn Packet>>> {
	Some(match (state, packet, bound) {
		(HandshakeState, 0, Server) => Handshake::deserialize(reader),
		(StatusState, 0, Server) => StatusRequest::deserialize(reader),
		_ => return None
	})
}

pub trait Packet: Debug {
	fn deserialize(reader: &mut impl Read) -> Result<Box<dyn Packet>>
		where Self: Sized;
	fn serialize(&self, writer: &mut dyn Write) -> Result<()>;
	fn packet_info(&self) -> &'static (State, u32);

	fn next_state(&self) -> Option<State> {
		None
	}
}

#[derive(Debug)]
pub struct Handshake {
	pub protocol_version: u32,
	pub address: (String, u16),
	pub next_state: State
}

impl Packet for Handshake {
	fn deserialize(reader: &mut impl Read) -> Result<Box<dyn Packet>> {
		Ok(Box::new(Self {
			protocol_version: reader.variable_integer()? as u32,
			address: (reader.string()?, reader.unsigned_short()?),
			next_state: match reader.variable_integer()? {
				1 => StatusState,
				2 => LoginState,
				num => return Err(Error::new(ErrorKind::InvalidData,
					format!("Expected 1 or 2, found {}.", num)))
			}
		}))
	}

	fn serialize(&self, writer: &mut dyn Write) -> Result<()> {
		todo!()
	}

	fn packet_info(&self) -> &'static (State, u32) {
		&(HandshakeState, 0)
	}

	fn next_state(&self) -> Option<State> {
		Some(self.next_state)
	}
}

#[derive(Debug)]
pub struct StatusRequest;

impl Packet for StatusRequest {
	fn deserialize(_: &mut impl Read) -> Result<Box<dyn Packet>> {
		Ok(Box::new(Self))
	}

	fn serialize(&self, writer: &mut dyn Write) -> Result<()> {
		todo!()
	}

	fn packet_info(&self) -> &'static (State, u32) {
		&(StatusState, 0)
	}
}

#[derive(Debug)]
pub struct StatusResponse {
	pub protocol_name: String,
	pub protocol_version: u32,
	pub players_max: usize,
	pub players_online: usize,
	pub players_sample: Vec<(String, u128)>,
	pub display_motd: String
}

impl Packet for StatusResponse {
	fn deserialize(reader: &mut impl Read) -> Result<Box<dyn Packet>> {
		todo!()
	}

	fn serialize(&self, writer: &mut dyn Write) -> Result<()> {
		writer.string(&to_string(self).unwrap())
	}

	fn packet_info(&self) -> &'static (State, u32) {
		&(StatusState, 0)
	}
}

impl Serialize for StatusResponse {
	fn serialize<S>(&self, serializer: S) -> STDResult<S::Ok, S::Error>
			where S: Serializer {
		struct Protocol<'p>(&'p StatusResponse);

		impl<'p> Serialize for Protocol<'p> {
			fn serialize<S>(&self, serializer: S) -> STDResult<S::Ok, S::Error>
					where S: Serializer {
				let mut map = serializer.serialize_map(Some(2))?;
				map.serialize_entry("name", &self.0.protocol_name)?;
				map.serialize_entry("protocol", &self.0.protocol_version)?;
				map.end()
			}
		}

		struct Players<'p>(&'p StatusResponse);

		impl<'p> Serialize for Players<'p> {
			fn serialize<S>(&self, serializer: S) -> STDResult<S::Ok, S::Error>
					where S: Serializer {
				let mut map = serializer.serialize_map(Some(3))?;
				map.serialize_entry("max", &self.0.players_max)?;
				map.serialize_entry("online", &self.0.players_online)?;
				map.serialize_entry("sample", &self.0.players_sample)?;
				map.end()
			}
		}

		struct MessageOfTheDay<'p>(&'p StatusResponse);

		impl<'p> Serialize for MessageOfTheDay<'p> {
			fn serialize<S>(&self, serializer: S) -> STDResult<S::Ok, S::Error>
					where S: Serializer {
				let mut map = serializer.serialize_map(Some(1))?;
				map.serialize_entry("text", &self.0.display_motd)?;
				map.end()
			}
		}

		let mut map = serializer.serialize_map(Some(3))?;
		map.serialize_entry("version", &Protocol(&self))?;
		map.serialize_entry("players", &Players(&self))?;
		map.serialize_entry("description", &MessageOfTheDay(&self))?;
		map.end()
	}
}
