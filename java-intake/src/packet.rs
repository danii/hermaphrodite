use self::super::{
	nbt::Serializer as NBTSerializer,
	types::{Bound, Read, State, Write}
};
use serde::{ser::{SerializeMap, SerializeSeq, Serializer}, Serialize};
use serde_json::to_string;
use std::{
	io::{Error, ErrorKind, Result},
	fmt::{Debug, Formatter, Result as FMTResult},
	result::Result as STDResult
};

pub trait PacketLiterate: Clone + Debug + Sized {
	const PACKET_STATE: State;
	const PACKET_BOUND: Bound;
	const PACKET_ID: u32;

	fn serialize(&self, writer: &mut impl Write) -> Result<()>;
	fn deserialize(len: usize, reader: &mut impl Read) -> Result<Packet>;

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
	PlayTeleportConfirm(PlayTeleportConfirm),
	PlayClientSettings(PlayClientSettings),
	PlayPluginMessageClient(PlayPluginMessageClient),
	PlayPlayerPositionRotationClient(PlayPlayerPositionRotationClient),
	PlayJoinGame(PlayJoinGame),
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
			Self::PlayTeleportConfirm(_) =>
				PlayTeleportConfirm::$constant,
			Self::PlayClientSettings(_) =>
				PlayClientSettings::$constant,
			Self::PlayPluginMessageClient(_) =>
				PlayPluginMessageClient::$constant,
			Self::PlayPlayerPositionRotationClient(_) =>
				PlayPlayerPositionRotationClient::$constant,
			Self::PlayJoinGame(_) =>
				PlayJoinGame::$constant,
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
			Self::PlayTeleportConfirm(packet) =>
				$traitt::$name(packet, $($arg),*),
			Self::PlayClientSettings(packet) =>
				$traitt::$name(packet, $($arg),*),
			Self::PlayPluginMessageClient(packet) =>
				$traitt::$name(packet, $($arg),*),
			Self::PlayPlayerPositionRotationClient(packet) =>
				$traitt::$name(packet, $($arg),*),
			Self::PlayJoinGame(packet) =>
				$traitt::$name(packet, $($arg),*),
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

	pub fn deserialize(len: usize, reader: &mut impl Read, state: State, bound: Bound, id: u32) -> Option<Result<Packet>> {
		Some(match (state, bound, id) {
			// Handshake
			(Handshake::PACKET_STATE, Handshake::PACKET_BOUND, Handshake::PACKET_ID) =>
				Handshake::deserialize(len, reader),

			// Status
			(StatusRequest::PACKET_STATE, StatusRequest::PACKET_BOUND, StatusRequest::PACKET_ID) =>
				StatusRequest::deserialize(len, reader),
			(StatusResponse::PACKET_STATE, StatusResponse::PACKET_BOUND, StatusResponse::PACKET_ID) =>
				StatusResponse::deserialize(len, reader),
			(StatusPing::PACKET_STATE, StatusPing::PACKET_BOUND, StatusPing::PACKET_ID) =>
				StatusPing::deserialize(len, reader),
			(StatusPong::PACKET_STATE, StatusPong::PACKET_BOUND, StatusPong::PACKET_ID) =>
				StatusPong::deserialize(len, reader),

			// Login
			(LoginStart::PACKET_STATE, LoginStart::PACKET_BOUND, LoginStart::PACKET_ID) =>
				LoginStart::deserialize(len, reader),
			(LoginCompression::PACKET_STATE, LoginCompression::PACKET_BOUND, LoginCompression::PACKET_ID) =>
				LoginCompression::deserialize(len, reader),
			(LoginSuccess::PACKET_STATE, LoginSuccess::PACKET_BOUND, LoginSuccess::PACKET_ID) =>
				LoginSuccess::deserialize(len, reader),

			// Play
			(PlayTeleportConfirm::PACKET_STATE, PlayTeleportConfirm::PACKET_BOUND, PlayTeleportConfirm::PACKET_ID) =>
				PlayTeleportConfirm::deserialize(len, reader),
			(PlayClientSettings::PACKET_STATE, PlayClientSettings::PACKET_BOUND, PlayClientSettings::PACKET_ID) =>
				PlayClientSettings::deserialize(len, reader),
			(PlayPluginMessageClient::PACKET_STATE, PlayPluginMessageClient::PACKET_BOUND, PlayPluginMessageClient::PACKET_ID) =>
				PlayPluginMessageClient::deserialize(len, reader),
			(PlayPlayerPositionRotationClient::PACKET_STATE, PlayPlayerPositionRotationClient::PACKET_BOUND, PlayPlayerPositionRotationClient::PACKET_ID) =>
				PlayPlayerPositionRotationClient::deserialize(len, reader),
			(PlayJoinGame::PACKET_STATE, PlayJoinGame::PACKET_BOUND, PlayJoinGame::PACKET_ID) =>
				PlayJoinGame::deserialize(len, reader),
			(PlayPlayerPositionRotationServer::PACKET_STATE, PlayPlayerPositionRotationServer::PACKET_BOUND, PlayPlayerPositionRotationServer::PACKET_ID) =>
				PlayPlayerPositionRotationServer::deserialize(len, reader),

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
			Self::PlayTeleportConfirm(packet) => write!(f, "{:?}", packet),
			Self::PlayClientSettings(packet) => write!(f, "{:?}", packet),
			Self::PlayPluginMessageClient(packet) => write!(f, "{:?}", packet),
			Self::PlayPlayerPositionRotationClient(packet) => write!(f, "{:?}", packet),
			Self::PlayJoinGame(packet) => write!(f, "{:?}", packet),
			Self::PlayPlayerPositionRotationServer(packet) => write!(f, "{:?}", packet)
		}
	}
}

#[derive(Clone, Debug)]
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

	fn deserialize(_len: usize, reader: &mut impl Read) -> Result<Packet> {
		Ok(Self {
			protocol_version: reader.variable_integer()?.0 as u32,
			address: (reader.string()?.0, reader.unsigned_short()?),
			next_state: match reader.variable_integer()?.0 {
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

#[derive(Clone, Debug)]
pub struct StatusRequest;

impl PacketLiterate for StatusRequest {
	const PACKET_STATE: State = State::Status;
	const PACKET_BOUND: Bound = Bound::Server;
	const PACKET_ID: u32 = 0;

	fn serialize(&self, _writer: &mut impl Write) -> Result<()> {
		todo!()
	}

	fn deserialize(_: usize, _: &mut impl Read) -> Result<Packet> {
		Ok(Self.into())
	}
}

impl Into<Packet> for StatusRequest {
	fn into(self) -> Packet {
		Packet::StatusRequest(self)
	}
}

#[derive(Clone, Debug)]
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
	fn deserialize(_: usize, _: &mut impl Read) -> Result<Packet> {
		todo!()
	}
}

impl Serialize for StatusResponse {
	fn serialize<S>(&self, serializer: S) -> STDResult<S::Ok, S::Error>
			where S: Serializer {
		struct Protocol<'r>(&'r StatusResponse);

		impl<'r> Serialize for Protocol<'r> {
			fn serialize<S>(&self, serializer: S) -> STDResult<S::Ok, S::Error>
					where S: Serializer {
				let mut map = serializer.serialize_map(Some(2))?;
				map.serialize_entry("name", &self.0.protocol_name)?;
				map.serialize_entry("protocol", &self.0.protocol_version)?;
				map.end()
			}
		}

		struct Players<'r>(&'r StatusResponse);

		impl<'r> Serialize for Players<'r> {
			fn serialize<S>(&self, serializer: S) -> STDResult<S::Ok, S::Error>
					where S: Serializer {
				let mut map = serializer.serialize_map(Some(3))?;
				map.serialize_entry("max", &self.0.players_max)?;
				map.serialize_entry("online", &self.0.players_online)?;
				map.serialize_entry("sample", &self.0.players_sample)?;
				map.end()
			}
		}

		struct MessageOfTheDay<'r>(&'r StatusResponse);

		impl<'r> Serialize for MessageOfTheDay<'r> {
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

#[derive(Clone, Debug)]
pub struct StatusPing(pub i64);

impl PacketLiterate for StatusPing {
	const PACKET_STATE: State = State::Status;
	const PACKET_BOUND: Bound = Bound::Server;
	const PACKET_ID: u32 = 1;

	fn serialize(&self, writer: &mut impl Write) -> Result<()> {
		writer.long(self.0)
	}

	fn deserialize(len: usize, reader: &mut impl Read) -> Result<Packet> {
		Ok(Self(reader.long()?).into())
	}
}

impl Into<Packet> for StatusPing {
	fn into(self) -> Packet {
		Packet::StatusPing(self)
	}
}

#[derive(Clone, Debug)]
pub struct StatusPong(pub i64);

impl PacketLiterate for StatusPong {
	const PACKET_STATE: State = State::Status;
	const PACKET_BOUND: Bound = Bound::Client;
	const PACKET_ID: u32 = 1;

	fn serialize(&self, writer: &mut impl Write) -> Result<()> {
		writer.long(self.0)
	}

	fn deserialize(len: usize, reader: &mut impl Read) -> Result<Packet> {
		Ok(Self(reader.long()?).into())
	}
}

impl Into<Packet> for StatusPong {
	fn into(self) -> Packet {
		Packet::StatusPong(self)
	}
}

#[derive(Clone, Debug)]
pub struct LoginStart(pub String);

impl PacketLiterate for LoginStart {
	const PACKET_STATE: State = State::Login;
	const PACKET_BOUND: Bound = Bound::Server;
	const PACKET_ID: u32 = 0;

	fn serialize(&self, _writer: &mut impl Write) -> Result<()> {
		todo!()
	}

	fn deserialize(len: usize, reader: &mut impl Read) -> Result<Packet> {
		Ok(Self(reader.string()?.0).into())
	}
}

impl Into<Packet> for LoginStart {
	fn into(self) -> Packet {
		Packet::LoginStart(self)
	}
}

#[derive(Clone, Debug)]
pub struct LoginCompression(pub u32);

impl PacketLiterate for LoginCompression {
	const PACKET_STATE: State = State::Login;
	const PACKET_BOUND: Bound = Bound::Client;
	const PACKET_ID: u32 = 3;

	fn serialize(&self, writer: &mut impl Write) -> Result<()> {
		writer.variable_integer(self.0 as i32)
	}

	fn deserialize(_: usize, _: &mut impl Read) -> Result<Packet> {
		todo!()
	}
}

impl Into<Packet> for LoginCompression {
	fn into(self) -> Packet {
		Packet::LoginCompression(self)
	}
}

#[derive(Clone, Debug)]
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

	fn deserialize(_: usize, _: &mut impl Read) -> Result<Packet> {
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

#[derive(Clone, Debug)]
pub struct PlayTeleportConfirm(pub u32);

impl PacketLiterate for PlayTeleportConfirm {
	const PACKET_STATE: State = State::Play;
	const PACKET_BOUND: Bound = Bound::Server;
	const PACKET_ID: u32 = 0;

	fn serialize(&self, writer: &mut impl Write) -> Result<()> {
		todo!()
	}

	fn deserialize(len: usize, reader: &mut impl Read) -> Result<Packet> {
		Ok(Self(reader.variable_integer()?.0 as u32).into())
	}
}

impl Into<Packet> for PlayTeleportConfirm {
	fn into(self) -> Packet {
		Packet::PlayTeleportConfirm(self)
	}
}

#[derive(Clone, Debug)]
pub struct PlayClientSettings {
	pub locale: String,
	pub view_distance: u8,
	pub chat_mode: u8,
	pub chat_color: bool,
	pub skin_mask: u8,
	pub primary_hand: u8
}

impl PacketLiterate for PlayClientSettings {
	const PACKET_STATE: State = State::Play;
	const PACKET_BOUND: Bound = Bound::Server;
	const PACKET_ID: u32 = 5;

	fn serialize(&self, _writer: &mut impl Write) -> Result<()> {
		todo!()
	}

	fn deserialize(len: usize, reader: &mut impl Read) -> Result<Packet> {
		Ok(Self {
			locale: reader.string()?.0,
			view_distance: reader.byte()? as u8,
			chat_mode: reader.byte()? as u8,
			chat_color: reader.bool()?,
			skin_mask: reader.byte()? as u8,
			primary_hand: reader.variable_integer()?.0 as u8
		}.into())
	}
}

impl Into<Packet> for PlayClientSettings {
	fn into(self) -> Packet {
		Packet::PlayClientSettings(self)
	}
}

#[derive(Clone, Debug)]
pub struct PlayPluginMessageClient {
	pub channel: String,
	pub data: Vec<u8>
}

impl PacketLiterate for PlayPluginMessageClient {
	const PACKET_STATE: State = State::Play;
	const PACKET_BOUND: Bound = Bound::Server;
	const PACKET_ID: u32 = 11;

	fn serialize(&self, writer: &mut impl Write) -> Result<()> {
		todo!()
	}

	fn deserialize(len: usize, reader: &mut impl Read) -> Result<Packet> {
		let (channel, channel_read) = reader.string()?;

		Ok(Self {
			channel,
			data: (0..len - channel_read)
				.map(|_| reader.byte().map(|data| data as u8))
				.collect::<Result<Vec<_>>>()?
		}.into())
	}
}

impl Into<Packet> for PlayPluginMessageClient {
	fn into(self) -> Packet {
		Packet::PlayPluginMessageClient(self)
	}
}

#[derive(Clone, Debug)]
pub struct PlayPlayerPositionRotationClient {
	pub x: f64,
	pub y_feet: f64,
	pub z: f64,
	pub yaw: f32,
	pub pitch: f32,
	pub grounded: bool
}

impl PacketLiterate for PlayPlayerPositionRotationClient {
	const PACKET_STATE: State = State::Play;
	const PACKET_BOUND: Bound = Bound::Server;
	const PACKET_ID: u32 = 19;

	fn serialize(&self, writer: &mut impl Write) -> Result<()> {
		todo!()
	}

	fn deserialize(_len: usize, reader: &mut impl Read) -> Result<Packet> {
		Ok(Self {
			x: reader.double()?,
			y_feet: reader.double()?,
			z: reader.double()?,
			yaw: reader.float()?,
			pitch: reader.float()?,
			grounded: reader.bool()?
		}.into())
	}
}

impl Into<Packet> for PlayPlayerPositionRotationClient {
	fn into(self) -> Packet {
		Packet::PlayPlayerPositionRotationClient(self)
	}
}

#[derive(Clone, Debug)]
pub struct PlayJoinGame {
	pub entity_id: u32,

	pub gamemode_current: u8,
	pub gamemode_previous: u8,
	pub gamemode_hardcore: bool,
	pub view_distance: u32,
	pub reduced_debug: bool,
	pub respawn_screen: bool,

	pub world_list: Vec<String>,
	pub world_name: String,
	pub seed_hashed: u64,
	pub world_debug: bool,
	pub world_flat: bool,

	pub dimension: Dimension,
	pub dimension_codec: DimensionCodec
}

impl PacketLiterate for PlayJoinGame {
	const PACKET_STATE: State = State::Play;
	const PACKET_BOUND: Bound = Bound::Client;
	const PACKET_ID: u32 = 36;

	fn serialize(&self, writer: &mut impl Write) -> Result<()> {
		writer.int(self.entity_id as i32)?;
		writer.bool(self.gamemode_hardcore)?;
		writer.unsigned_byte(self.gamemode_current)?;
		writer.byte(self.gamemode_previous as i8)?;
		writer.variable_integer(self.world_list.len() as i32)?;
		self.world_list.iter().try_for_each(|world| writer.string(world))?;
		writer.nbt(&self.dimension_codec, "")?;
		writer.nbt(&self.dimension, "")?;
		writer.string(&self.world_name)?;
		writer.long(self.seed_hashed as i64)?;
		writer.variable_integer(0)?;
		writer.variable_integer(self.view_distance as i32)?;
		writer.bool(self.reduced_debug)?;
		writer.bool(self.respawn_screen)?;
		writer.bool(self.world_debug)?;
		writer.bool(self.world_flat)?;
		Ok(())
	}

	fn deserialize(_: usize, _: &mut impl Read) -> Result<Packet> {
		todo!()
	}
}

impl Into<Packet> for PlayJoinGame {
	fn into(self) -> Packet {
		Packet::PlayJoinGame(self)
	}
}

#[derive(Clone, Debug)]
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

	fn deserialize(_: usize, _: &mut impl Read) -> Result<Packet> {
		todo!()
	}
}

impl Into<Packet> for PlayPlayerPositionRotationServer {
	fn into(self) -> Packet {
		Packet::PlayPlayerPositionRotationServer(self)
	}
}

#[derive(Clone, Debug, Serialize)]
pub struct Dimension {
	#[serde(rename = "respawn_anchor_works")]
	pub work_anchor: bool,
	#[serde(rename = "bed_works")]
	pub work_bed: bool,
	#[serde(rename = "piglin_safe")]
	pub work_piglin: bool,
	#[serde(rename = "has_raids")]
	pub work_raids: bool,
	#[serde(rename = "has_skylight")]
	pub work_skylight: bool,

	#[serde(rename = "infiniburn")]
	pub category_infiniburn: String,
	#[serde(rename = "effects")]
	pub category_effects: String,

	#[serde(rename = "ambient_light")]
	pub light: f32,
	#[serde(rename = "logical_height")]
	pub height: u32,
	#[serde(rename = "coordinate_scale")]
	pub scale: f64,
	pub natural: bool,
	#[serde(rename = "has_ceiling")]
	pub ceiling: bool,
	pub ultrawarm: bool
}

#[derive(Clone, Debug)]
pub struct Biome {
	pub precipitation: String,
	pub depth: f32,
	pub temperature: f32,
	pub scale: f32,
	pub downfall: f32,
	pub category: String,

	pub color_sky: u32,
	pub color_water: u32,
	pub color_fog: u32,
	pub color_water_fog: u32,

	pub mood_tick_delay: u32,
	pub mood_offset: f64,
	pub mood_sound: String,
	pub mood_block_search_extent: u32
}

impl Serialize for Biome {
	fn serialize<S>(&self, serializer: S) -> STDResult<S::Ok, S::Error>
			where S: Serializer {
		struct Effects<'r>(&'r Biome);

		impl<'r> Serialize for Effects<'r> {
			fn serialize<S>(&self, serializer: S) -> STDResult<S::Ok, S::Error>
					where S: Serializer {
				let mut map = serializer.serialize_map(Some(5))?;
				map.serialize_entry("sky_color", &self.0.color_sky)?;
				map.serialize_entry("water_fog_color", &self.0.color_water_fog)?;
				map.serialize_entry("fog_color", &self.0.color_fog)?;
				map.serialize_entry("water_color", &self.0.color_water)?;
				map.serialize_entry("mood_sound", &MoodSound(&self.0))?;
				map.end()
			}
		}

		struct MoodSound<'r>(&'r Biome);

		impl<'r> Serialize for MoodSound<'r> {
			fn serialize<S>(&self, serializer: S) -> STDResult<S::Ok, S::Error>
					where S: Serializer {
				let mut map = serializer.serialize_map(Some(4))?;
				map.serialize_entry("tick_delay", &self.0.mood_tick_delay)?;
				map.serialize_entry("offset", &self.0.mood_offset)?;
				map.serialize_entry("sound", &self.0.mood_sound)?;
				map.serialize_entry("block_search_extent",
					&self.0.mood_block_search_extent)?;
				map.end()
			}
		}

		let mut map = serializer.serialize_map(Some(7))?;
		map.serialize_entry("precipitation", &self.precipitation)?;
		map.serialize_entry("depth", &self.depth)?;
		map.serialize_entry("temperature", &self.temperature)?;
		map.serialize_entry("scale", &self.scale)?;
		map.serialize_entry("downfall", &self.downfall)?;
		map.serialize_entry("category", &self.category)?;
		map.serialize_entry("effects", &Effects(&self))?;
		map.end()
	}
}

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct DimensionCodec {
	pub dimensions: HashMap<String, Dimension>,
	pub biomes: HashMap<String, Biome>
}

impl Serialize for DimensionCodec {
	fn serialize<S>(&self, serializer: S) -> STDResult<S::Ok, S::Error>
			where S: Serializer {
		struct Entry<'r, V>(u32, &'r str, &'r V)
			where V: Serialize;

		impl<'r, V> Serialize for Entry<'r, V>
				where V: Serialize {
			fn serialize<S>(&self, serializer: S) -> STDResult<S::Ok, S::Error>
					where S: Serializer {
				let mut map = serializer.serialize_map(Some(3))?;
				map.serialize_entry("name", self.1)?;
				map.serialize_entry("id", &self.0)?;
				map.serialize_entry("element", self.2)?;
				map.end()
			}
		}

		struct Entries<'r, V>(&'r HashMap<String, V>)
			where V: Serialize;

		impl<'r, V> Serialize for Entries<'r, V>
				where V: Serialize {
			fn serialize<S>(&self, serializer: S) -> STDResult<S::Ok, S::Error>
					where S: Serializer {
				let mut list = serializer.serialize_seq(Some(self.0.len()))?;
				self.0.iter().enumerate().try_for_each(|(index, (key, value))|
					list.serialize_element(&Entry(index as u32, key, value)))?;
				list.end()
			}
		}

		struct Category<'r, V>(&'r str, &'r HashMap<String, V>)
			where V: Serialize;

		impl<'r, V> Serialize for Category<'r, V>
				where V: Serialize {
			fn serialize<S>(&self, serializer: S) -> STDResult<S::Ok, S::Error>
					where S: Serializer {
				let mut map = serializer.serialize_map(Some(2))?;
				map.serialize_entry("type", self.0)?;
				map.serialize_entry("value", &Entries(&self.1))?;
				map.end()
			}
		}

		const DIMENSION: &str = "minecraft:dimension_type";
		const BIOME: &str = "minecraft:worldgen/biome";

		let mut map = serializer.serialize_map(Some(2))?;
		map.serialize_entry(DIMENSION, &Category(DIMENSION, &self.dimensions))?;
		map.serialize_entry(BIOME, &Category(BIOME, &self.biomes))?;
		map.end()
	}
}
