use self::super::types::{Bound, Read, State, Write};
use serde::ser::{Serialize, SerializeMap, Serializer};
use serde_json::to_string;
use std::{
	io::{Error, ErrorKind, Result},
	fmt::{Debug, Formatter, Result as FMTResult},
	result::Result as STDResult
};

pub trait PacketLiterate: Debug + Sized {
	const PACKET_STATE: State;
	const PACKET_BOUND: Bound;
	const PACKET_ID: u32;

	fn serialize(&self, writer: &mut impl Write) -> Result<()>;
	fn deserialize(reader: &mut impl Read) -> Result<Packet>;

	fn next_state(&self) -> Option<State> {
		None
	}
}

pub enum Packet {
	Handshake(Handshake),
	StatusRequest(StatusRequest),
	StatusResponse(StatusResponse),
	StatusPing(StatusPing),
	StatusPong(StatusPong),
	LoginStart(LoginStart),
	LoginCompression(LoginCompression),
	LoginSuccess(LoginSuccess),
	PlayPlayerPositionRotationServer(PlayPlayerPositionRotationServer)
}

macro constant_fetcher($name:ident(), $constant:ident, $result:ident) {
	pub fn $name(&self) -> $result {
		match self {
			// Handshake
			Self::Handshake(_) =>
				Handshake::$constant,

			// Status
			Self::StatusRequest(_) =>
				StatusRequest::$constant,
			Self::StatusResponse(_) =>
				StatusResponse::$constant,
			Self::StatusPing(_) =>
				StatusPing::$constant,
			Self::StatusPong(_) =>
				StatusPong::$constant,

			// Login
			Self::LoginStart(_) =>
				LoginStart::$constant,
			Self::LoginCompression(_) =>
				LoginCompression::$constant,
			Self::LoginSuccess(_) =>
				LoginSuccess::$constant,

			// Play
			Self::PlayPlayerPositionRotationServer(_) =>
				PlayPlayerPositionRotationServer::$constant
		}
	}
}

macro trait_impl($traitt:ident::$name:ident($($arg:ident: $arg_ty:ty),*), $result:ty) {
	pub fn $name(&self, $($arg: $arg_ty),*) -> $result {
		match self {
			// Handshake
			Self::Handshake(packet) =>
				$traitt::$name(packet, $($arg),*),

			// Status
			Self::StatusRequest(packet) =>
				$traitt::$name(packet, $($arg),*),
			Self::StatusResponse(packet) =>
				$traitt::$name(packet, $($arg),*),
			Self::StatusPing(packet) =>
				$traitt::$name(packet, $($arg),*),
			Self::StatusPong(packet) =>
				$traitt::$name(packet, $($arg),*),

			// Login
			Self::LoginStart(packet) =>
				$traitt::$name(packet, $($arg),*),
			Self::LoginCompression(packet) =>
				$traitt::$name(packet, $($arg),*),
			Self::LoginSuccess(packet) =>
				$traitt::$name(packet, $($arg),*),

			// Play
			Self::PlayPlayerPositionRotationServer(packet) =>
				$traitt::$name(packet, $($arg),*)
		}
	}
}

impl Packet {
	constant_fetcher!(packet_state(), PACKET_STATE, State);
	constant_fetcher!(packet_bound(), PACKET_BOUND, Bound);
	constant_fetcher!(packet_id(), PACKET_ID, u32);
	trait_impl!(PacketLiterate::serialize(writer: &mut impl Write), Result<()>);
	trait_impl!(PacketLiterate::next_state(), Option<State>);

	pub fn deserialize(reader: &mut impl Read, state: State, bound: Bound, id: u32) -> Option<Result<Packet>> {
		Some(match (state, bound, id) {
			// Handshake
			(Handshake::PACKET_STATE, Handshake::PACKET_BOUND, Handshake::PACKET_ID) =>
				Handshake::deserialize(reader),

			// Status
			(StatusRequest::PACKET_STATE, StatusRequest::PACKET_BOUND, StatusRequest::PACKET_ID) =>
				StatusRequest::deserialize(reader),
			(StatusResponse::PACKET_STATE, StatusResponse::PACKET_BOUND, StatusResponse::PACKET_ID) =>
				StatusResponse::deserialize(reader),
			(StatusPing::PACKET_STATE, StatusPing::PACKET_BOUND, StatusPing::PACKET_ID) =>
				StatusPing::deserialize(reader),
			(StatusPong::PACKET_STATE, StatusPong::PACKET_BOUND, StatusPong::PACKET_ID) =>
				StatusPong::deserialize(reader),

			// Login
			(LoginStart::PACKET_STATE, LoginStart::PACKET_BOUND, LoginStart::PACKET_ID) =>
				LoginStart::deserialize(reader),
			(LoginCompression::PACKET_STATE, LoginCompression::PACKET_BOUND, LoginCompression::PACKET_ID) =>
				LoginCompression::deserialize(reader),
			(LoginSuccess::PACKET_STATE, LoginSuccess::PACKET_BOUND, LoginSuccess::PACKET_ID) =>
				LoginSuccess::deserialize(reader),

			// Play
			(PlayPlayerPositionRotationServer::PACKET_STATE, PlayPlayerPositionRotationServer::PACKET_BOUND, PlayPlayerPositionRotationServer::PACKET_ID) =>
				PlayPlayerPositionRotationServer::deserialize(reader),

			// ???
			_ => return None
		})
	}
}

impl Debug for Packet {
	fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
		match self {
			Self::Handshake(packet) => write!(f, "{:?}", packet),
			Self::StatusRequest(packet) => write!(f, "{:?}", packet),
			Self::StatusResponse(packet) => write!(f, "{:?}", packet),
			Self::StatusPing(packet) => write!(f, "{:?}", packet),
			Self::StatusPong(packet) => write!(f, "{:?}", packet),
			Self::LoginStart(packet) => write!(f, "{:?}", packet),
			Self::LoginCompression(packet) => write!(f, "{:?}", packet),
			Self::LoginSuccess(packet) => write!(f, "{:?}", packet),
			Self::PlayPlayerPositionRotationServer(packet) => write!(f, "{:?}", packet)
		}
	}
}

#[derive(Debug)]
pub struct Handshake {
	pub protocol_version: u32,
	pub address: (String, u16),
	pub next_state: State
}

impl PacketLiterate for Handshake {
	const PACKET_STATE: State = State::Handshake;
	const PACKET_BOUND: Bound = Bound::Server;
	const PACKET_ID: u32 = 0;

	fn serialize(&self, _writer: &mut impl Write) -> Result<()> {
		todo!()
	}

	fn deserialize(reader: &mut impl Read) -> Result<Packet> {
		Ok(Self {
			protocol_version: reader.variable_integer()? as u32,
			address: (reader.string()?, reader.unsigned_short()?),
			next_state: match reader.variable_integer()? {
				1 => State::Status,
				2 => State::Login,
				num => return Err(Error::new(ErrorKind::InvalidData,
					format!("Expected 1 or 2, found {}.", num)))
			}
		}.into())
	}

	fn next_state(&self) -> Option<State> {
		Some(self.next_state)
	}
}

impl Into<Packet> for Handshake {
	fn into(self) -> Packet {
		Packet::Handshake(self)
	}
}

#[derive(Debug)]
pub struct StatusRequest;

impl PacketLiterate for StatusRequest {
	const PACKET_STATE: State = State::Status;
	const PACKET_BOUND: Bound = Bound::Server;
	const PACKET_ID: u32 = 0;

	fn serialize(&self, _writer: &mut impl Write) -> Result<()> {
		todo!()
	}

	fn deserialize(_: &mut impl Read) -> Result<Packet> {
		Ok(Self.into())
	}
}

impl Into<Packet> for StatusRequest {
	fn into(self) -> Packet {
		Packet::StatusRequest(self)
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

impl PacketLiterate for StatusResponse {
	const PACKET_STATE: State = State::Status;
	const PACKET_BOUND: Bound = Bound::Client;
	const PACKET_ID: u32 = 0;

	fn serialize(&self, writer: &mut impl Write) -> Result<()> {
		writer.string(&to_string(self).unwrap())
	}
  fn deserialize(_reader: &mut impl Read) -> Result<Packet> {
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

impl Into<Packet> for StatusResponse {
	fn into(self) -> Packet {
		Packet::StatusResponse(self)
	}
}

#[derive(Debug)]
pub struct StatusPing(pub i64);

impl PacketLiterate for StatusPing {
	const PACKET_STATE: State = State::Status;
	const PACKET_BOUND: Bound = Bound::Server;
	const PACKET_ID: u32 = 1;

	fn serialize(&self, writer: &mut impl Write) -> Result<()> {
		writer.long(self.0)
	}
  fn deserialize(reader: &mut impl Read) -> Result<Packet> {
		Ok(Self(reader.long()?).into())
	}
}

impl Into<Packet> for StatusPing {
	fn into(self) -> Packet {
		Packet::StatusPing(self)
	}
}

#[derive(Debug)]
pub struct StatusPong(pub i64);

impl PacketLiterate for StatusPong {
	const PACKET_STATE: State = State::Status;
	const PACKET_BOUND: Bound = Bound::Client;
	const PACKET_ID: u32 = 1;

	fn serialize(&self, writer: &mut impl Write) -> Result<()> {
		writer.long(self.0)
	}
  fn deserialize(reader: &mut impl Read) -> Result<Packet> {
		Ok(Self(reader.long()?).into())
	}
}

impl Into<Packet> for StatusPong {
	fn into(self) -> Packet {
		Packet::StatusPong(self)
	}
}

#[derive(Debug)]
pub struct LoginStart(pub String);

impl PacketLiterate for LoginStart {
	const PACKET_STATE: State = State::Login;
	const PACKET_BOUND: Bound = Bound::Server;
	const PACKET_ID: u32 = 0;

	fn serialize(&self, _writer: &mut impl Write) -> Result<()> {
		todo!()
	}
  fn deserialize(reader: &mut impl Read) -> Result<Packet> {
		Ok(Self(reader.string()?).into())
	}
}

impl Into<Packet> for LoginStart {
	fn into(self) -> Packet {
		Packet::LoginStart(self)
	}
}

#[derive(Debug)]
pub struct LoginCompression(pub u32);

impl PacketLiterate for LoginCompression {
	const PACKET_STATE: State = State::Login;
	const PACKET_BOUND: Bound = Bound::Client;
	const PACKET_ID: u32 = 3;

	fn serialize(&self, writer: &mut impl Write) -> Result<()> {
		writer.variable_integer(self.0 as i32)
	}

	fn deserialize(_: &mut impl Read) -> Result<Packet> {
		todo!()
	}
}

impl Into<Packet> for LoginCompression {
	fn into(self) -> Packet {
		Packet::LoginCompression(self)
	}
}

#[derive(Debug)]
pub struct LoginSuccess {
	pub uuid: u128,
	pub username: String
}

impl PacketLiterate for LoginSuccess {
	const PACKET_STATE: State = State::Login;
	const PACKET_BOUND: Bound = Bound::Client;
	const PACKET_ID: u32 = 2;

	fn serialize(&self, writer: &mut impl Write) -> Result<()> {
		writer.uuid(self.uuid)?;
		writer.string(&self.username)
	}

	fn deserialize(_reader: &mut impl Read) -> Result<Packet> {
		todo!()
	}

	fn next_state(&self) -> Option<State> {
		Some(State::Play)
	}
}

impl Into<Packet> for LoginSuccess {
	fn into(self) -> Packet {
		Packet::LoginSuccess(self)
	}
}

#[derive(Debug)]
pub struct PlayPlayerPositionRotationServer {
	pub x: f64,
	pub y: f64,
	pub z: f64,
	pub yaw: f32,
	pub pitch: f32,
	pub flags: i8,
	pub teleport_id: i32
}

impl PacketLiterate for PlayPlayerPositionRotationServer {
	const PACKET_STATE: State = State::Play;
	const PACKET_BOUND: Bound = Bound::Client;
	const PACKET_ID: u32 = 52;

	fn serialize(&self, writer: &mut impl Write) -> Result<()> {
		writer.double(self.x)?;
		writer.double(self.y)?;
		writer.double(self.z)?;
		writer.float(self.yaw)?;
		writer.float(self.pitch)?;
		writer.byte(self.flags)?;
		writer.variable_integer(self.teleport_id)
	}

  fn deserialize(_reader: &mut impl Read) -> Result<Packet> {
		todo!()
	}
}

impl Into<Packet> for PlayPlayerPositionRotationServer {
	fn into(self) -> Packet {
		Packet::PlayPlayerPositionRotationServer(self)
	}
}
