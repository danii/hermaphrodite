use self::super::nbt::Serializer as NBTSerializer;
use serde::{de::Deserialize, ser::Serialize};
use std::{
	io::{Error, ErrorKind, Read as IORead, Result, Write as IOWrite},
	mem::{size_of, transmute}, slice::from_mut as slice_mut
};

macro read_variable_type($target:ty, $unsigned:ty, $name:ident) {
	fn $name(&mut self) -> Result<($target, usize)> {
		let mut result = 0;
		let mut reads = 0;
		const TARGET_SIZE: usize = size_of::<$target>();
		const MAX_SIZE: f32 = TARGET_SIZE as f32 + (TARGET_SIZE as f32 + 1.) / 8f32;

		loop {
			let mut read = 0;
			if self.read(slice_mut(&mut read))? == 0 {
				break Err(Error::new(ErrorKind::UnexpectedEof, format!(
					"Unexpected end of file while reading {}.", stringify!($name))))
			}

			result = result | (read as $unsigned & 0b01111111) << 7 * reads;
			reads = reads + 1;

			if reads > MAX_SIZE.ceil() as usize {
				break Err(Error::new(ErrorKind::InvalidData,
					format!("Found {} of more than {} bytes",
						stringify!($name), MAX_SIZE.ceil() as usize)))
			}
			if read & 0b10000000 == 0 {unsafe {break Ok((transmute(result), reads))}}
		}
	}
}

macro read_primitive_type($target:ty, $name:ident) {
	fn $name(&mut self) -> Result<$target> {
		let mut buffer = [0; size_of::<$target>()];
		if self.read(&mut buffer)? != size_of::<$target>() {return Err(
			Error::new(ErrorKind::UnexpectedEof, format!(
				"Unexpected end of file while reading {}.", stringify!($name))))}

		Ok(<$target>::from_be_bytes(buffer))
	}
}

macro write_variable_type($target:ty, $unsigned:ty, $name:ident) {
	fn $name(&mut self, value: $target) -> Result<()> {
		let mut value: $unsigned = unsafe {transmute(value)};

		loop {
			let mut byte = value as u8 & 0b01111111;
			value = value >> 7;

			if value != 0 {
				self.write(slice_mut(&mut (byte | 0b10000000)))?;
			} else {
				self.write(slice_mut(&mut byte))?;
				break Ok(())
			}
		}
	}
}

macro write_primitive_type($target:ty, $name:ident) {
	fn $name(&mut self, value: $target) -> Result<()> {
		self.write(&<$target>::to_be_bytes(value))?;
		Ok(())
	}
}

pub trait Read {
	fn variable_integer(&mut self) -> Result<(i32, usize)>;
	fn variable_long(&mut self) -> Result<(i64, usize)>;

	fn bool(&mut self) -> Result<bool>;
	fn byte(&mut self) -> Result<i8>;
	fn short(&mut self) -> Result<i16>;
	fn int(&mut self) -> Result<i32>;
	fn long(&mut self) -> Result<i64>;

	fn unsigned_byte(&mut self) -> Result<u8>;
	fn unsigned_short(&mut self) -> Result<u16>;
	fn unsigned_int(&mut self) -> Result<u32>;
	fn unsigned_long(&mut self) -> Result<u64>;
	fn uuid(&mut self) -> Result<u128>;

	fn float(&mut self) -> Result<f32>;
	fn double(&mut self) -> Result<f64>;

	fn nbt<'de, T>(&mut self) -> Result<T>
		where T: Deserialize<'de>;
	fn string(&mut self) -> Result<(String, usize)>;
}

impl<R> Read for R
		where R: IORead {
	read_variable_type!(i32, u32, variable_integer);
	read_variable_type!(i64, u64, variable_long);

	fn bool(&mut self) -> Result<bool> {
		Ok(self.unsigned_byte()? == 1)
	}

	read_primitive_type!(i8, byte);
	read_primitive_type!(i16, short);
	read_primitive_type!(i32, int);
	read_primitive_type!(i64, long);

	read_primitive_type!(u8, unsigned_byte);
	read_primitive_type!(u16, unsigned_short);
	read_primitive_type!(u32, unsigned_int);
	read_primitive_type!(u64, unsigned_long);
	read_primitive_type!(u128, uuid);

	read_primitive_type!(f32, float);
	read_primitive_type!(f64, double);

	fn nbt<'de, T>(&mut self) -> Result<T>
			where T: Deserialize<'de> {
		todo!()
	}

	fn string(&mut self) -> Result<(String, usize)> {
		let (size, read) = self.variable_integer()?;
		let mut buffer = vec![0; size as usize];
		if self.read(&mut buffer)? != buffer.len() {
			return Err(Error::new(ErrorKind::UnexpectedEof,
				"Unexpected end of file while reading string."))
		}

		let read = read + buffer.len();
		String::from_utf8(buffer)
			.map_err(|_|
				Error::new(ErrorKind::InvalidData, "String data was not UTF-8."))
			.map(|string| (string, read))
	}
}

pub trait Write {
	fn variable_integer(&mut self, value: i32) -> Result<()>;
	fn variable_long(&mut self, value: i64) -> Result<()>;

	fn bool(&mut self, value: bool) -> Result<()>;
	fn byte(&mut self, value: i8) -> Result<()>;
	fn short(&mut self, value: i16) -> Result<()>;
	fn int(&mut self, value: i32) -> Result<()>;
	fn long(&mut self, value: i64) -> Result<()>;
	
	fn unsigned_byte(&mut self, value: u8) -> Result<()>;
	fn unsigned_short(&mut self, value: u16) -> Result<()>;
	fn unsigned_int(&mut self, value: u32) -> Result<()>;
	fn unsigned_long(&mut self, value: u64) -> Result<()>;
	fn uuid(&mut self, value: u128) -> Result<()>;

	fn float(&mut self, value: f32) -> Result<()>;
	fn double(&mut self, value: f64) -> Result<()>;

	fn nbt<T>(&mut self, value: T, name: &str) -> Result<()>
		where T: Serialize;
	fn string(&mut self, value: &str) -> Result<()>;
}

impl<W> Write for W
		where W: IOWrite {
	write_variable_type!(i32, u32, variable_integer);
	write_variable_type!(i64, u64, variable_long);

	fn bool(&mut self, value: bool) -> Result<()> {
		if value {self.unsigned_byte(1)} else {self.unsigned_byte(0)}
	}

	write_primitive_type!(i8, byte);
	write_primitive_type!(i16, short);
	write_primitive_type!(i32, int);
	write_primitive_type!(i64, long);

	write_primitive_type!(u8, unsigned_byte);
	write_primitive_type!(u16, unsigned_short);
	write_primitive_type!(u32, unsigned_int);
	write_primitive_type!(u64, unsigned_long);
	write_primitive_type!(u128, uuid);

	write_primitive_type!(f32, float);
	write_primitive_type!(f64, double);

	fn nbt<T>(&mut self, value: T, name: &str) -> Result<()>
			where T: Serialize {
		let mut writer = NBTSerializer::new_compound(self, name);
		value.serialize(&mut writer)?;
		Ok(())
	}

	fn string(&mut self, value: &str) -> Result<()> {
		self.variable_integer(value.as_bytes().len() as i32)?;
		self.write(value.as_bytes())?;
		Ok(())
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum State {
	Handshake,
	Status,
	Login,
	Play
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Bound {
	Server,
	Client
}

impl Bound {
	pub fn receiving_bound(&self) -> Self {
		*self
	}

	pub fn sending_bound(&self) -> Self {
		match self {
			Self::Client => Self::Server,
			Self::Server => Self::Client
		}
	}
}
