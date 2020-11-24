use std::{
	io::{Error, ErrorKind, Read as IORead, Result, Write as IOWrite},
	mem::{size_of, transmute}, slice::from_mut as slice_mut
};

pub trait Read {
	fn variable_integer(&mut self) -> Result<i32>;
	fn variable_long(&mut self) -> Result<i64>;

	fn long(&mut self) -> Result<i64>;

	fn unsigned_short(&mut self) -> Result<u16>;

	fn string(&mut self) -> Result<String>;
}

pub macro read_variable_type($target:ty, $unsigned:ty, $name:ident) {
	fn $name(&mut self) -> Result<$target> {
		let mut result = 0;
		let mut reads = 0;
		const TARGET_SIZE: usize = size_of::<$target>();
		const MAX_SIZE: f32 = TARGET_SIZE as f32 + (TARGET_SIZE as f32 + 1.) / 8f32;

		loop {
			let mut read = 0;
			if self.read(slice_mut(&mut read))? == 0 {break Err(Error::new(
				ErrorKind::UnexpectedEof, format!(
					"Unexpected end of file while reading {}.", stringify!($name))))}

			result = result | (read as $unsigned & 0b01111111) << 7 * reads;
			reads = reads + 1;

			if reads > MAX_SIZE.ceil() as usize {break Err(Error::new(
				ErrorKind::InvalidData, format!("Found {} of more than {} bytes",
					stringify!($name), MAX_SIZE.ceil() as usize)))}
			if read & 0b10000000 == 0 {unsafe {break Ok(transmute(result))}}
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

impl<T> Read for T
		where T: IORead {
	read_variable_type!(i32, u32, variable_integer);
	read_variable_type!(i64, u64, variable_long);

	read_primitive_type!(i64, long);

	read_primitive_type!(u16, unsigned_short);

	fn string(&mut self) -> Result<String> {
		let size = self.variable_integer()?;
		let mut buffer = vec![0; size as usize];
		if self.read(&mut buffer)? != buffer.len() {
			return Err(Error::new(ErrorKind::UnexpectedEof,
				"Unexpected end of file while reading string."))
		}

		String::from_utf8(buffer).map_err(|_|
			Error::new(ErrorKind::InvalidData, "String data was not UTF-8."))
	}
}

pub trait Write {
	fn variable_integer(&mut self, value: i32) -> Result<()>;
	fn variable_long(&mut self, value: i64) -> Result<()>;

	fn long(&mut self, value: i64) -> Result<()>;

	fn unsigned_short(&mut self, value: u16) -> Result<()>;
	fn uuid(&mut self, value: u128) -> Result<()>;

	fn string(&mut self, value: &str) -> Result<()>;
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

impl<T> Write for T
		where T: IOWrite {
	write_variable_type!(i32, u32, variable_integer);
	write_variable_type!(i64, u64, variable_long);

	write_primitive_type!(i64, long);

	write_primitive_type!(u16, unsigned_short);
	write_primitive_type!(u128, uuid);

	fn string(&mut self, value: &str) -> Result<()> {
		self.variable_integer(value.as_bytes().len() as i32)?;
		self.write(value.as_bytes())?;
		Ok(())
	}
}
