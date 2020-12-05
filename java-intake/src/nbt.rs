use serde::ser::{
	Error as SerError, Serialize, Serializer as SerDeSerializer,
	SerializeTupleStruct, SerializeTupleVariant, SerializeStruct,
	SerializeStructVariant, SerializeTuple, SerializeSeq, SerializeMap
};
use std::{
	error::Error as STDError,
	fmt::{Display, Formatter, Result as FMTResult},
	io::{Error as IOError, Result as IOResult, Write},
	result::Result as STDResult
};

type Result<T> = STDResult<T, Error>;

pub struct Serializer<'n, W>
		where W: Write {
	writer: W,
	action: WriteAction<'n>
}

impl<'n, W> Serializer<'n, W>
		where W: Write {
	pub fn new_raw(writer: W) -> Self {
		Self {writer, action: WriteAction::None}
	}

	pub fn new_compound(writer: W, name: &'n str) -> Self {
		Self {writer, action: WriteAction::Compound(name)}
	}

	pub fn new_compound_unnamed(writer: W) -> Self {
		Self::new_compound(writer, "")
	}

	fn write_header(&mut self, tag_type: u8) -> Result<()> {
		match &mut self.action {
			WriteAction::Compound(name) => {
				let name = *name;
				self.writer.write(&[tag_type])?;
				self.action = WriteAction::None;
				self.serialize_str(name)?;
			},
			WriteAction::List(list_type, length) => match list_type {
				0 => {
					self.writer.write(&[tag_type])?;
					self.writer.write(&length.to_be_bytes())?;
					*list_type = tag_type
				},
				value if *value == tag_type => (),
				_ => {
					return Err(Error::Custom("list diff types".to_owned().into_boxed_str()))
				}
			},
			WriteAction::DynamicList(list_type, ..) => match list_type {
				0 => {
					self.writer.write(&[tag_type])?;
					*list_type = tag_type
				},
				value if *value == tag_type => (),
				_ => {
					return Err(Error::Custom("list diff types".to_owned().into_boxed_str()))
				}
			},
			_ => {}
		}

		Ok(())
	}

	fn compound_set(&mut self, name: &'n str) -> Result<()> {
		self.action = WriteAction::Compound(name);
		Ok(())
	}

	fn compound_end(&mut self) -> Result<()> {
		self.writer.write(&[0])?;
		Ok(())
	}

	fn list_set_optional_length(&mut self, length: Option<usize>) -> Result<()> {
		match length {
			Some(length) => self.list_set_length(length)?,
			None => self.action = WriteAction::DynamicList(0, 0, Vec::new())
		}

		Ok(())
	}

	fn list_set_length(&mut self, length: usize) -> Result<()> {
		if length > u32::MAX as usize {
			return Err(Error::Custom("Oops".to_owned().into_boxed_str()))
		}

		self.action = WriteAction::List(0, length as u32);
		Ok(())
	}

	fn list_increment_length(&mut self) -> Result<()> {
		match &mut self.action {
			WriteAction::List(..) => (),
			WriteAction::DynamicList(_, length, ..) => *length = *length + 1,
			_ => return Err(Error::Custom("bruh".to_owned().into_boxed_str()))
		}

		Ok(())
	}
}

impl<'n, W> Write for Serializer<'n, W>
		where W: Write {
	fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
		if let WriteAction::DynamicList(.., buffer) = &mut self.action {
			buffer.write(buf)
		} else {
			self.writer.write(buf)
		}
	}

	fn flush(&mut self) -> IOResult<()> {
		if let WriteAction::DynamicList(.., buffer) = &mut self.action {
			buffer.flush()
		} else {
			self.writer.flush()
		}
	}
}

enum WriteAction<'n> {
	None,
	Compound(&'n str),
	List(u8, u32),
	DynamicList(u8, u32, Vec<u8>)
}

impl<'r, 'n, W> SerDeSerializer for &'r mut Serializer<'n, W>
		where W: Write {
	type Ok = ();
	type Error = Error;

	type SerializeTupleStruct = Self;
	type SerializeTupleVariant = Self;
	type SerializeStruct = Self;
	type SerializeStructVariant = Self;
	type SerializeTuple = Self;
	type SerializeSeq = Self;
	type SerializeMap = Self;

	fn serialize_bool(self, value: bool) -> Result<()> {
		self.serialize_i8(if value {1} else {0})
	}

	fn serialize_i8(self, value: i8) -> Result<()> {
		self.write_header(1)?; // header
		self.write(&value.to_be_bytes())?; // payload
		Ok(())
	}

	fn serialize_i16(self, value: i16) -> Result<()> {
		self.write_header(2)?; // header
		self.write(&value.to_be_bytes())?; // payload
		Ok(())
	}

	fn serialize_i32(self, value: i32) -> Result<()> {
		self.write_header(3)?; // header
		self.write(&value.to_be_bytes())?; // payload
		Ok(())
	}

	fn serialize_i64(self, value: i64) -> Result<()> {
		self.write_header(4)?; // header
		self.write(&value.to_be_bytes())?; // payload
		Ok(())
	}

	fn serialize_u8(self, value: u8) -> Result<()> {
		self.write_header(1)?; // header
		self.write(&value.to_be_bytes())?; // payload
		Ok(())
	}

	fn serialize_u16(self, value: u16) -> Result<()> {
		self.write_header(2)?; // header
		self.write(&value.to_be_bytes())?; // payload
		Ok(())
	}

	fn serialize_u32(self, value: u32) -> Result<()> {
		self.write_header(3)?; // header
		self.write(&value.to_be_bytes())?; // payload
		Ok(())
	}

	fn serialize_u64(self, value: u64) -> Result<()> {
		self.write_header(4)?; // header
		self.write(&value.to_be_bytes())?; // payload
		Ok(())
	}

	fn serialize_f32(self, value: f32) -> Result<()> {
		self.write_header(5)?; // header
		self.write(&value.to_be_bytes())?; // payload
		Ok(())
	}

	fn serialize_f64(self, value: f64) -> Result<()> {
		self.write_header(6)?; // header
		self.write(&value.to_be_bytes())?; // payload
		Ok(())
	}

	fn serialize_char(self, value: char) -> Result<()> {
		todo!()
	}

	// Return error when value is too many bytes?
	fn serialize_str(self, value: &str) -> Result<()> {
		self.write_header(8)?; // header
		let bytes = value.as_bytes();
		self.write(&(bytes.len() as i16).to_be_bytes())?; // TAG_Short length
		self.write(bytes)?; // payload
		Ok(())
	}

	fn serialize_bytes(self, value: &[u8]) -> Result<()> {
		todo!()
	}

	fn serialize_none(self) -> Result<()> {
		todo!()
	}

	fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
			where T: Serialize {
		todo!()
	}

	fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
		todo!()
	}

	fn serialize_unit_variant(self, name: &'static str, index: u32, variant: &'static str) -> Result<()> {
		todo!()
	}

	fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<()>
			where T: Serialize {
		todo!()
	}

	fn serialize_newtype_variant<T: ?Sized>(self, name: &'static str, index: u32, variant: &'static str, value: &T) -> Result<()>
			where T: Serialize {
		todo!()
	}

	fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct> {
		todo!()
	}

	fn serialize_tuple_variant(self, name: &'static str, index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant> {
		todo!()
	}

	fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self> {
		self.write_header(10)?;
		Ok(self)
	}

	fn serialize_struct_variant(self, name: &'static str, index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeStructVariant> {
		todo!()
	}

	fn serialize_unit(self) -> Result<()> {
		todo!()
	}

	fn serialize_tuple(self, length: usize) -> Result<Self> {
		self.write_header(9)?;
		self.list_set_length(length)?;
		Ok(self)
	}

	fn serialize_seq(self, length: Option<usize>) -> Result<Self> {
		self.write_header(9)?;
		self.list_set_optional_length(length)?;
		Ok(self)
	}

	fn serialize_map(self, _: Option<usize>) -> Result<Self> {
		Ok(self)
	}
}

impl<'r, 'n, W> SerializeTupleStruct for &'r mut Serializer<'n, W>
		where W: Write {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, value: &T) -> Result<()>
			where T: Serialize + ?Sized {
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}

impl<'r, 'n, W> SerializeTupleVariant for &'r mut Serializer<'n, W>
		where W: Write {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, value: &T) -> Result<()>
			where T: Serialize + ?Sized {
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}

impl<'r, 'n, W> SerializeStruct for &'r mut Serializer<'n, W>
		where W: Write {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
			where T: Serialize + ?Sized {
		self.compound_set(key)?;
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		self.compound_end()
	}
}

impl<'r, 'n, W> SerializeStructVariant for &'r mut Serializer<'n, W>
		where W: Write {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
			where T: Serialize + ?Sized {
		self.compound_set(key)?;
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		self.compound_end()
	}
}

impl<'r, 'n, W> SerializeTuple for &'r mut Serializer<'n, W>
		where W: Write {
	type Ok = ();
	type Error = Error;

	fn serialize_element<T>(&mut self, value: &T) -> Result<()>
			where T: Serialize + ?Sized {
		self.list_increment_length()?;
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'r, 'n, W> SerializeSeq for &'r mut Serializer<'n, W>
		where W: Write {
	type Ok = ();
	type Error = Error;

	fn serialize_element<T>(&mut self, value: &T) -> Result<()>
			where T: Serialize + ?Sized {
		self.list_increment_length()?;
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'r, 'n, W> SerializeMap for &'r mut Serializer<'n, W>
		where W: Write {
	type Ok = ();
	type Error = Error;

	fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
	where
					T: Serialize {
			todo!()
	}

	fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
	where
					T: Serialize {
			todo!()
	}

	fn end(self) -> Result<()> {
		self.compound_end()
	}
}

#[derive(Debug)]
pub enum Error {
	IO(IOError),
	Custom(Box<str>)
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
		match self {
			Self::IO(error) => write!(f, "{}", error),
			Self::Custom(error) => write!(f, "{}", error)
		}
	}
}

impl STDError for Error {}

impl SerError for Error {
	fn custom<M>(msg: M) -> Self
			where M: Display {
		Self::Custom(msg.to_string().into_boxed_str())
	}
}

impl From<IOError> for Error {
	fn from(error: IOError) -> Self {
		Self::IO(error)
	}
}
