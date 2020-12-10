use self::super::{packet::Packet, types::{Bound, Read, State, Write}};
use std::{
	collections::VecDeque,
	io::{Error, ErrorKind, Read as IORead, Result, Write as IOWrite, copy},
	net::TcpStream, result::Result as STDResult
};

pub struct Socket {
	socket: TcpStream,
	bound: Bound,
	_compression: Option<i32>,

	state: State,
	read_buffer: ReadBuffer
}

impl Socket {
	pub fn new(socket: TcpStream) -> Self {
		socket.set_nonblocking(true).unwrap();

		Self {
			socket,
			bound: Bound::Server,
			_compression: None,
			state: State::Handshake,
			read_buffer: ReadBuffer::new()
		}
	}

	pub fn send(&mut self, packets: Vec<Packet>) -> Result<()> {
		packets.iter().map::<Result<()>, _>(|packet| {
			eprintln!("< {:?}", packet);
			if let Some(state) = packet.next_state() {
				self.state = state
			}

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
				let size = Read::variable_integer(&mut self.read_buffer)?.0 as usize;
				if self.read_buffer.len() < size {Err(Error::new(
					ErrorKind::UnexpectedEof, "Unexpected end of file."))?}

				let (packet_id, read) = Read::variable_integer(&mut self.read_buffer)?;
				let packet = Packet::deserialize(size - read, &mut self.read_buffer,
					self.state, self.bound.receiving_bound(), packet_id as u32);

				match packet.transpose()? {
					Some(packet) => packet,
					None => Err(Error::new(ErrorKind::InvalidData, format!(
						"Bad packet ID {} for STATE {:?}.", packet_id, self.state)))?
				}
			};

			match packet {
				Err(error) => {
					self.read_buffer.mark_as_unread();
					match error.kind() {
						ErrorKind::UnexpectedEof => break Ok(packets),
						_ => break Err((error, packets))
					}
				},
				Ok(packet) => {
					eprintln!("> {:?}", packet);

					self.read_buffer.mark_as_read();
					if let Some(state) = packet.next_state() {self.state = state}
					packets.push(packet);
				}
			}
		}
	}

	pub fn state(&self) -> State {
		self.state
	}
}

/// A readable buffer that retains any data read from it until it is "marked as
/// read", where all previously read data is discarded, or "unread", where the
/// buffer will reread already read data again, from the beginning.
///
/// This grants users the ability to decide that there isn't enough data in the
/// buffer, and to restart reading from the beginning on the next read.
///
/// While this is meant to be used as a buffer, data has to be written manually
/// to it via the Write implementation.
#[derive(Debug)]
struct ReadBuffer(VecDeque<u8>, usize);

impl ReadBuffer {
	fn new() -> Self {
		Self(VecDeque::new(), 0)
	}

	/// Returns the amount of bytes left available for reading. This accounts for
	/// the cursor, meaning it is not the size of the underlying buffer.
	fn len(&self) -> usize {
		self.0.len() - self.1
	}

	/// Marks this buffer as unread, returning the cursor to the beginning.
	fn mark_as_unread(&mut self) {
		self.1 = 0
	}

	/// Marks this buffer as read, discarding all data previously read.
	fn mark_as_read(&mut self) {
		self.0.drain(0..self.1); // Drain the range.
		self.1 = 0 // Set the cursor in accordance with our changes.
	}
}

impl IORead for ReadBuffer {
	fn read(&mut self, mut buffer: &mut [u8]) -> Result<usize> {
		// Read via write.
		let count = buffer.write(&self.0.make_contiguous()[self.1..])?;
		self.1 = self.1 + count;
		Ok(count)
	}
}

impl IOWrite for ReadBuffer {
	fn write(&mut self, buffer: &[u8]) -> Result<usize> {
		self.0.extend(buffer); // Extend the buffer.
		Ok(buffer.len())
	}

	fn flush(&mut self) -> Result<()> {
		Ok(())
	}
}
