use self::super::{
	Bound::{self, Server, Client},
	State::{
		self,
		Handshake as HandshakeState, Status as StatusState, Login as LoginState
	},
	types::{Read, Write}
};
use enum_dispatch::enum_dispatch;
use serde::ser::{Serialize, SerializeMap, Serializer};
use serde_json::to_string;
use std::{
	fmt::Debug, io::{Error, ErrorKind, Result}, result::Result as STDResult
};

#[enum_dispatch]
pub trait Packet: Debug {
	fn serialize(&self, writer: &mut dyn Write) -> Result<()>;

	fn next_state(&self) -> Option<State> {
		None
	}
}

pub trait PacketNonObject: Packet + Sized {
	const PACKET_STATE: State;
	const PACKET_BOUND: Bound;
	const PACKET_ID: u32;

	fn deserialize(reader: &mut impl Read) -> Result<PacketEnum>;
}

#[enum_dispatch(Packet)]
#[derive(Debug)]
pub enum PacketEnum {
	Handshake,
	StatusRequest,
	StatusResponse,
	StatusPing,
	StatusPong,
	LoginStart,
	LoginCompression,
	LoginSuccess
}

macro deserialize_match($match_against:expr, $reader:ident, $($typ:ty),*) {
	match $match_against {
		$(
			(<$typ>::PACKET_STATE, <$typ>::PACKET_ID, <$typ>::PACKET_BOUND)
				=> <$typ>::deserialize($reader),
		)*
		_ => return None
	}
}

macro packet_info_match($selff:ident, $($typ:ident:$typ2:path),*) {
	match $selff {
		$(
			$typ2 (_) =>
				($typ::PACKET_STATE, $typ::PACKET_BOUND, $typ::PACKET_ID),
		)*
	}
}

impl PacketEnum {
	pub fn deserialize(reader: &mut impl Read, state: State, bound: Bound,
			packet: u32) -> Option<Result<Self>> {
		Some(deserialize_match!(
			(state, packet, bound), reader,
			Handshake, StatusRequest, StatusResponse, StatusPing, StatusPong,
			LoginStart, LoginCompression, LoginSuccess
		))
	}

	pub fn packet_info(&self) -> (State, Bound, u32) {
		packet_info_match!(
			self,
			Handshake:Self::Handshake, StatusRequest:Self::StatusRequest,
			StatusResponse:Self::StatusResponse, StatusPing:Self::StatusPing,
			StatusPong:Self::StatusPong, LoginStart:Self::LoginStart,
			LoginCompression:Self::LoginCompression, LoginSuccess:Self::LoginSuccess
		)
	}
}

#[derive(Debug)]
pub struct Handshake {
	pub protocol_version: u32,
	pub address: (String, u16),
	pub next_state: State
}

impl Packet for Handshake {
	fn serialize(&self, _writer: &mut dyn Write) -> Result<()> {
		todo!()
	}

	fn next_state(&self) -> Option<State> {
		Some(self.next_state)
	}
}

impl PacketNonObject for Handshake {
	const PACKET_STATE: State = HandshakeState;
	const PACKET_BOUND: Bound = Server;
	const PACKET_ID: u32 = 0;

	fn deserialize(reader: &mut impl Read) -> Result<PacketEnum> {
		Ok(PacketEnum::from(Self {
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
}

#[derive(Debug)]
pub struct StatusRequest;

impl Packet for StatusRequest {
	fn serialize(&self, _writer: &mut dyn Write) -> Result<()> {
		todo!()
	}
}

impl PacketNonObject for StatusRequest {
	const PACKET_STATE: State = StatusState;
	const PACKET_BOUND: Bound = Server;
	const PACKET_ID: u32 = 0;

	fn deserialize(_: &mut impl Read) -> Result<PacketEnum> {
		Ok(PacketEnum::from(Self))
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
	fn serialize(&self, writer: &mut dyn Write) -> Result<()> {
		writer.string(&to_string(self).unwrap())
	}
}

impl PacketNonObject for StatusResponse {
	const PACKET_STATE: State = StatusState;
	const PACKET_BOUND: Bound = Client;
	const PACKET_ID: u32 = 0;

	fn deserialize(_reader: &mut impl Read) -> Result<PacketEnum> {
		todo!()
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

#[derive(Debug)]
pub struct StatusPing(pub i64);

impl Packet for StatusPing {
	fn serialize(&self, writer: &mut dyn Write) -> Result<()> {
		writer.long(self.0)
	}
}

impl PacketNonObject for StatusPing {
	const PACKET_STATE: State = StatusState;
	const PACKET_BOUND: Bound = Server;
	const PACKET_ID: u32 = 1;

	fn deserialize(reader: &mut impl Read) -> Result<PacketEnum> {
		Ok(PacketEnum::from(Self(reader.long()?)))
	}
}

#[derive(Debug)]
pub struct StatusPong(pub i64);

impl Packet for StatusPong {
	fn serialize(&self, writer: &mut dyn Write) -> Result<()> {
		writer.long(self.0)
	}
}

impl PacketNonObject for StatusPong {
	const PACKET_STATE: State = StatusState;
	const PACKET_BOUND: Bound = Client;
	const PACKET_ID: u32 = 1;

	fn deserialize(reader: &mut impl Read) -> Result<PacketEnum> {
		Ok(PacketEnum::from(Self(reader.long()?)))
	}
}

#[derive(Debug)]
pub struct LoginStart(pub String);

impl Packet for LoginStart {
	fn serialize(&self, _: &mut dyn Write) -> Result<()> {
		todo!()
	}
}

impl PacketNonObject for LoginStart {
	const PACKET_STATE: State = LoginState;
	const PACKET_BOUND: Bound = Server;
	const PACKET_ID: u32 = 0;

	fn deserialize(reader: &mut impl Read) -> Result<PacketEnum> {
		Ok(PacketEnum::from(Self(reader.string()?)))
	}
}

#[derive(Debug)]
pub struct LoginCompression(pub u32);

impl PacketNonObject for LoginCompression {
	const PACKET_STATE: State = LoginState;
	const PACKET_BOUND: Bound = Client;
	const PACKET_ID: u32 = 3;

	fn deserialize(_: &mut impl Read) -> Result<PacketEnum> {
		todo!()
	}
}

impl Packet for LoginCompression {
	fn serialize(&self, writer: &mut dyn Write) -> Result<()> {
		writer.variable_integer(self.0 as i32)
	}
}

#[derive(Debug)]
pub struct LoginSuccess {
	pub uuid: u128,
	pub username: String
}

impl Packet for LoginSuccess {
	fn serialize(&self, writer: &mut dyn Write) -> Result<()> {
		writer.uuid(self.uuid)?;
		writer.string(&self.username)
	}
}

impl PacketNonObject for LoginSuccess {
	const PACKET_STATE: State = LoginState;
	const PACKET_BOUND: Bound = Client;
	const PACKET_ID: u32 = 2;

	fn deserialize(_: &mut impl Read) -> Result<PacketEnum> {
		todo!()
	}
}
