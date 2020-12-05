use self::super::{packet::Packet, types::{Bound, Read, State, Write}};
use std::{
	io::{Error, ErrorKind, Read as IORead, Result, Write as IOWrite, copy},
	net::TcpStream, result::Result as STDResult
};

pub struct Socket {
	socket: TcpStream,
	bound: Bound,
	compression: Option<i32>,

	state: State,
	read_buffer: ReadBuffer
}

impl Socket {
	pub fn new(socket: TcpStream) -> Self {
		socket.set_nonblocking(true).unwrap();

		Self {
			socket,
			bound: Bound::Server,
			compression: None,
			state: State::Handshake,
			read_buffer: ReadBuffer::new()
		}
	}

	pub fn send(&mut self, packets: Vec<Packet>) -> Result<()> {
		packets.iter().map::<Result<()>, _>(|packet| {
			println!("{:?}", packet);
			let mut header = Vec::new();
			let mut bytes = Vec::new();

			bytes.variable_integer(packet.packet_id() as i32)?;
			packet.serialize(&mut bytes)?;

			header.variable_integer(bytes.len() as i32)?;
			header.extend(bytes);
			self.socket.write(&header)?;

			Ok(())
		}).collect::<Result<()>>()
	}

	pub fn recv(&mut self)
			-> STDResult<Vec<Packet>, (Error, Vec<Packet>)> {
		match copy(&mut self.socket, &mut self.read_buffer) {
			Ok(_) => return Ok(vec![]), // Socket closed...
			Err(error) => match error.kind() {
				ErrorKind::WouldBlock => {}, // What we want! Continue.
				_ => return Err((error, vec![])) // An error occurred...
			}
		}

		let mut packets = Vec::new();
		loop {
			let packet: Result<Packet> = try {
				let size = Read::variable_integer(&mut self.read_buffer)? as usize;
				if self.read_buffer.unread() > size {Err(Error::new(
					ErrorKind::UnexpectedEof, "Unexpected end of file."))?}

				let packet_id = Read::variable_integer(&mut self.read_buffer)? as u32;
				match Packet::deserialize(&mut self.read_buffer, self.state,
						self.bound.receiving_bound(), packet_id).transpose()? {
					Some(packet) => packet,
					None => Err(Error::new(
						ErrorKind::InvalidData, "Bad packet ID and state combo."))?
				}
			};

			match packet {
				Err(error) => {
					self.read_buffer.undo();
					match error.kind() {
						ErrorKind::UnexpectedEof => break Ok(packets),
						_ => break Err((error, packets))
					}
				},
				Ok(packet) => {
					self.read_buffer.apply();
					if let Some(state) = packet.next_state() {self.state = state}
					packets.push(packet);
				}
			}
		}
	}
}

#[derive(Debug)]
struct ReadBuffer(Box<[u8]>, usize);

impl ReadBuffer {
	fn new() -> Self {
		Self(Box::new([]), 0)
	}

	fn unread(&self) -> usize {
		self.0.len() - self.1
	}

	fn undo(&mut self) {
		self.1 = 0
	}

	fn apply(&mut self) {
		self.0 = self.0[self.1..].into();
		self.1 = 0
	}
}

impl IORead for ReadBuffer {
	fn read(&mut self, mut buffer: &mut [u8]) -> Result<usize> {
		let count = buffer.write(&self.0[self.1..])?;
		self.1 = self.1 + count;
		Ok(count)
	}
}

impl IOWrite for ReadBuffer {
	fn write(&mut self, buffer: &[u8]) -> Result<usize> {
		let mut vec = self.0.to_vec();
		vec.extend_from_slice(buffer);
		self.0 = vec.into_boxed_slice();
		Ok(buffer.len())
	}

	fn flush(&mut self) -> Result<()> {
		Ok(())
	}
}
