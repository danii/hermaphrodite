use serde::ser::{
	Error as SerError, Serialize, Serializer as SerDeSerializer,
	SerializeTupleStruct, SerializeTupleVariant, SerializeStruct,
	SerializeStructVariant, SerializeTuple, SerializeSeq, SerializeMap
};
use serde_primitives::StringSerializer;
use std::{
	error::Error as STDError,
	fmt::{Display, Formatter, Result as FMTResult},
	io::{Error as IOError, ErrorKind as IOErrorKind, Result as IOResult, Write},
	mem::swap,
	option::NoneError,
	result::Result as STDResult
};

type Result<T> = STDResult<T, Error>;

enum WriteAction {
	None,
	Named(Box<str>),
	List(Option<Box<str>>, u8, u32),
	DynamicList(Option<Box<str>>, u8, u32, Vec<u8>)
}

pub struct Serializer<W>
		where W: Write {
	writer: W,
	action: WriteAction
}

impl <W> Serializer<W>
		where W: Write {
	#[inline]
	pub fn new_raw(writer: W) -> Self {
		Self {writer, action: WriteAction::None}
	}

	#[inline]
	pub fn new_compound(writer: W, name: impl AsRef<str>) -> Self {
		Self {writer, action: WriteAction::Named(name.as_ref().into())}
	}

	#[inline]
	pub fn new_compound_unnamed(writer: W) -> Self {
		Self::new_compound(writer, "")
	}

	#[inline]
	fn write_header(&mut self, tag_type: u8) -> Result<()> {
		match &mut self.action {
			WriteAction::Named(name) => {
				let name = name.clone();
				self.writer.write(&[tag_type])?;
				self.action = WriteAction::None;
				self.serialize_str(&name)?;
			},
			WriteAction::List(compound, list_type, len) => match list_type {
				// If this is the first value of the list (represented by the stored
				// type ID being TAG_End), then we must write the type ID of the types
				// we're storing.
				0 => {
					// Write the tag_type to this action to check against in the future,
					// as lists can only store one type, like a vector.
					*list_type = tag_type;

					// Borrow checker stuff.
					let len = len.to_be_bytes();

					// If compound is Some, that means this is a named list, and we
					// skipped writing the type and name information last time, so we
					// could gather more data to perform some optimizations.
					if let Some(name) = compound.clone() {
						// If we're about to write the named tag data, we shouldn't do it
						// again.
						*compound = None;
						let mut default = false;

						// That optimization is using the array tags, which includes the
						// type ID of types we'll be writing in the array's tag type, saving
						// us one byte. Small on it's own, big in the long run!
						self.writer.write(match tag_type {
							1 => &[7], // TAG_List of TAG_Byte? Use TAG_Byte_Array instead.
							3 => &[11], // TAG_List of TAG_Int? Use TAG_Int_Array instead.
							4 => &[12], // TAG_List of TAG_Long?? Use TAG_Long_Array instead.
							_ => {
								default = true;
								&[9] // Otherwise, revert to using TAG_List.
							}
						})?;

						// Write this named tag's name, without the write action set.
						let mut action = WriteAction::None;
						swap(&mut self.action, &mut action); // Swap to.
						self.serialize_str(&name)?;
						self.action = action; // Swap back.

						if default {
							// Write the tag type of the items in this list, if we're not
							// using a specialized array.
							// do this on dyn!
							self.writer.write(&[tag_type])?;
						}
					} else {
						// Otherwise, we are not writing a named list, rather a normal list,
						// meaning we don't have to write a type, other than the types of
						// values we'll be storing.
						self.writer.write(&[tag_type])?;
					}

					// Write this list's length. Thankfully, the size of the length field
					// is the same regardless if this is a type specific array.
					self.writer.write(&len)?;
				},

				// If the type ID we have stored is the same type ID as we're about to
				// write, we don't have to do anything, and can let the value serialzie.
				_ if *list_type == tag_type => (),

				// If it isn't, then we must throw an error.
				_ => return Err(Error::Custom("list diff types".to_owned().into_boxed_str()))
			},
			WriteAction::DynamicList(compound, list_type, ..) => match list_type {
				0 => {
					// Same optimization as in WriteAction::List.
					*list_type = tag_type;

					if let Some(name) = compound.clone() {
						*compound = None;
						let default = &[9, tag_type];

						self.writer.write(match tag_type {
							1 => &[7], // TAG_List of TAG_Byte? Use TAG_Byte_Array instead.
							3 => &[11], // TAG_List of TAG_Int? Use TAG_Int_Array instead.
							4 => &[12], // TAG_List of TAG_Long?? Use TAG_Long_Array instead.
							_ => default // Otherwise, revert to using TAG_List.
						})?;

						let mut action = WriteAction::None;
						swap(&mut self.action, &mut action); // Swap to.
						self.serialize_str(&name)?;
						self.action = action; // Swap back.
					} else {
						self.writer.write(&[tag_type])?;
					}

					// This time, don't write the length, as we do not know it now.
					// Instead, we will be writing into a temporary buffer, computing
					// the length as we go until the list is ended, then we flush the
					// buffer into the writer, with the computed length first of course.
				},

				// Same things as WriteAction::List.
				_ if *list_type == tag_type => (),
				_ => return Err(Error::Custom("list diff types2".to_owned().into_boxed_str()))
			},
			_ => {}
		}

		Ok(())
	}

	#[inline]
	fn named_start(&mut self, name: impl AsRef<str>) -> Result<()> {
		self.action = WriteAction::Named(name.as_ref().into());
		Ok(())
	}

	#[inline]
	fn named_end(&mut self) -> Result<()> {
		self.writer.write(&[0])?;
		Ok(())
	}

	/// This handles calling write_header for us.
	#[inline]
	fn list_start(&mut self, len: impl Into<Option<usize>>)
			-> Result<()> {
		// I expect this to be optimized away when length is passed a usize.
		match len.into() {
			Some(len) => {
				// If we know the length of this list, first make sure it's in range
				// of a u32, then set the write action to a list.
				if len > u32::MAX as usize {
					return Err(Error::Custom("Oops".to_owned().into_boxed_str()))
				}

				// Be sure to use the optimized named tag code for lists.
				let compound = match &self.action {
					WriteAction::Named(name) => Some(name.clone()),
					_ => {
						self.write_header(9)?;
						None
					}
				};

				self.action = WriteAction::List(compound, 0, len as u32)
			},
			None => {
				// Otherwise, we have to use a buffer... Once again, be sure to use the
				// optimized tag code for lists.
				let compound = match &self.action {
					WriteAction::Named(name) => Some(name.clone()),
					_ => {
						self.write_header(9)?;
						None
					}
				};

				self.action = WriteAction::DynamicList(compound, 0, 0, Vec::new())
			}
		}

		Ok(())
	}

	/*
	fn list_set_length(&mut self, length: usize) -> Result<()> {
		if length > u32::MAX as usize {
			return Err(Error::Custom("Oops".to_owned().into_boxed_str()))
		}

		self.action = WriteAction::List(0, length as u32);
		Ok(())
	}*/

	fn list_increment_length(&mut self) -> Result<()> {
		match &mut self.action {
			WriteAction::List(..) => (),
			WriteAction::DynamicList(_, len, ..) => *len = *len + 1,
			_ => return Err(Error::Custom("bruh".to_owned().into_boxed_str()))
		}

		Ok(())
	}
}

impl <W> Write for Serializer <W>
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

impl<'r, W> SerDeSerializer for &'r mut Serializer <W>
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

	fn serialize_char(self, _value: char) -> Result<()> {
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

	fn serialize_bytes(self, _value: &[u8]) -> Result<()> {
		todo!()
	}

	fn serialize_none(self) -> Result<()> {
		todo!()
	}

	fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<()>
			where T: Serialize {
		todo!()
	}

	fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
		todo!()
	}

	fn serialize_unit_variant(self, _name: &'static str, _index: u32, _variant: &'static str) -> Result<()> {
		todo!()
	}

	fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, _value: &T) -> Result<()>
			where T: Serialize {
		todo!()
	}

	fn serialize_newtype_variant<T: ?Sized>(self, _name: &'static str, _index: u32, _variant: &'static str, _value: &T) -> Result<()>
			where T: Serialize {
		todo!()
	}

	fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct> {
		self.list_start(len)?;
		Ok(self)
	}

	fn serialize_tuple_variant(self, _name: &'static str, _index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant> {
		todo!()
	}

	fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self> {
		self.write_header(10)?;
		Ok(self)
	}

	fn serialize_struct_variant(self, _name: &'static str, _index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant> {
		todo!()
	}

	fn serialize_unit(self) -> Result<()> {
		todo!()
	}

	fn serialize_tuple(self, len: usize) -> Result<Self> {
		self.list_start(len)?;
		Ok(self)
	}

	fn serialize_seq(self, len: Option<usize>) -> Result<Self> {
		self.list_start(len)?;
		Ok(self)
	}

	fn serialize_map(self, _: Option<usize>) -> Result<Self> {
		self.write_header(10)?;
		Ok(self)
	}
}

impl<'r, W> SerializeTupleStruct for &'r mut Serializer <W>
		where W: Write {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, value: &T) -> Result<()>
			where T: Serialize + ?Sized {
		self.list_increment_length()?;
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'r, W> SerializeTupleVariant for &'r mut Serializer <W>
		where W: Write {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, value: &T) -> Result<()>
			where T: Serialize + ?Sized {
		self.list_increment_length()?;
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'r, W> SerializeStruct for &'r mut Serializer <W>
		where W: Write {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
			where T: Serialize + ?Sized {
		self.named_start(key)?;
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		self.named_end()
	}
}

impl<'r, W> SerializeStructVariant for &'r mut Serializer <W>
		where W: Write {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
			where T: Serialize + ?Sized {
		self.named_start(key)?;
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		self.named_end()
	}
}

impl<'r, W> SerializeTuple for &'r mut Serializer <W>
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

impl<'r, W> SerializeSeq for &'r mut Serializer <W>
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

impl<'r, W> SerializeMap for &'r mut Serializer <W>
		where W: Write {
	type Ok = ();
	type Error = Error;

	fn serialize_key<T>(&mut self, key: &T) -> Result<()>
			where T: Serialize + ?Sized {
		let mut serializer = StringSerializer::new();
		key.serialize(&mut serializer).unwrap();
		self.named_start(&serializer?)?;
		Ok(())
	}

	fn serialize_value<T>(&mut self, value: &T) -> Result<()>
			where T: Serialize + ?Sized {
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		self.named_end()
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

impl From<Error> for IOError {
	fn from(error: Error) -> Self {
		match error {
			Error::IO(error) =>
				error,
			Error::Custom(..) =>
				IOError::new(IOErrorKind::InvalidData, error)
		}
	}
}

impl From<IOError> for Error {
	fn from(error: IOError) -> Self {
		Self::IO(error)
	}
}

impl From<NoneError> for Error {
	fn from(_: NoneError) -> Self {
		Self::Custom("A depended on serialization failed".into())
	}
}
