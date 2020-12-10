use self::super::{packet::Packet, types::{Bound, Read, State, Write}};
use std::{
	io::{Error, ErrorKind, Read as IORead, Result, Write as IOWrite, copy},
	mem::swap,
	net::TcpStream, result::Result as STDResult,
	slice::from_raw_parts_mut as slice_from_raw_parts_mut
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
				if self.read_buffer.len() > size {Err(Error::new(
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
struct ReadBuffer(Box<[u8]>, usize);

impl ReadBuffer {
	fn new() -> Self {
		Self(Box::new([]), 0)
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
		// Take ownership for a hot minute, so we can avoid copying. All allocation
		// here will be done for us by Box, a technique I'm frankly really happy
		// about.
		let mut local: Box<[u8]> = Box::new([]);
		swap(&mut self.0, &mut local);

		// Get a raw pointer to the box.
		let local = Box::into_raw(local);
		let mut local = unsafe {
			// Here we effectively split the box into two around the cursor, dropping
			// the first half.

			// SAFETY: We just obtained this pointer from a box, so we know it's from
			// the heap. Using slice_from_raw_parts_mut simply resizes the box, and
			// we ensure the invariant that cursor 1 is never larger than the buffer.
			// (Infact, if the cursor is larger than the buffer, local_len should
			// panic first due to subtraction underflow.)
			let local_offset = (local as *mut u8).offset(self.1 as isize);
			let local_len = local.len() - self.1;

			// First half. Dropped.
			Box::from_raw(slice_from_raw_parts_mut(local as *mut u8, self.1));
			// Second half.
			Box::from_raw(slice_from_raw_parts_mut(local_offset, local_len))
		};

		// Give ownership back.
		swap(&mut self.0, &mut local);
		self.1 = 0 // Set the cursor in accordance with our changes.
	}
}

impl IORead for ReadBuffer {
	fn read(&mut self, mut buffer: &mut [u8]) -> Result<usize> {
		// Cool trick here, read via write.
		let count = buffer.write(&self.0[self.1..])?;
		self.1 = self.1 + count;
		Ok(count)
	}
}

impl IOWrite for ReadBuffer {
	fn write(&mut self, buffer: &[u8]) -> Result<usize> {
		// Take ownership for a hot minute, so we can avoid copying. All allocation
		// here will be done for us by a Vec.
		let mut local: Box<[u8]> = Box::new([]); // *Should* not allocate.
		swap(&mut self.0, &mut local);

		let mut local = local.into_vec();
		// extend_from_slice does a good enough job of reserving, via size hints.
		local.extend_from_slice(buffer);

		// Give ownership back.
		let mut local = local.into_boxed_slice();
		swap(&mut self.0, &mut local);
		Ok(buffer.len())
	}

	fn flush(&mut self) -> Result<()> {
		Ok(())
	}
}
