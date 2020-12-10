#![feature(try_trait)]
use serde::ser::{
	Error as SerError,
	Serialize, SerializeTupleStruct, SerializeTupleVariant, SerializeStruct,
	SerializeStructVariant, SerializeTuple, SerializeSeq, SerializeMap
};
use serde_primitives_macro::serde_primitive;
use std::{
	error::Error as STDError,
	fmt::{Display, Formatter, Result as FMTResult},
	result::Result as STDResult
};

pub type Result<T> = STDResult<T, Error>;

#[derive(Debug)]
pub struct Error(String);

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
		write!(f, "{}", self.0)
	}
}

impl STDError for Error {}

impl SerError for Error {
	fn custom<T>(msg: T) -> Self
			where T: Display {
		Self(msg.to_string())
	}
}

pub struct BlankImplementations;

impl<'r> SerializeTupleStruct for &'r mut BlankImplementations {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, _: &T) -> Result<()>
			where T: Serialize + ?Sized {
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}

impl<'r> SerializeTupleVariant for &'r mut BlankImplementations {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, _: &T) -> Result<()>
			where T: Serialize + ?Sized {
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}

impl<'r> SerializeStruct for &'r mut BlankImplementations {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, _: &'static str, _: &T) -> Result<()>
			where T: Serialize + ?Sized {
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}

impl<'r> SerializeStructVariant for &'r mut BlankImplementations {
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, _: &'static str, _: &T) -> Result<()>
			where T: Serialize + ?Sized {
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}

impl<'r> SerializeTuple for &'r mut BlankImplementations {
	type Ok = ();
	type Error = Error;

	fn serialize_element<T>(&mut self, _: &T) -> Result<()>
			where T: Serialize + ?Sized {
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}

impl<'r> SerializeSeq for &'r mut BlankImplementations {
	type Ok = ();
	type Error = Error;

	fn serialize_element<T>(&mut self, _: &T) -> Result<()>
			where T: Serialize + ?Sized {
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}

impl<'r> SerializeMap for &'r mut BlankImplementations {
	type Ok = ();
	type Error = Error;

	fn serialize_key<T>(&mut self, _: &T) -> Result<()>
			where T: Serialize + ?Sized {
		todo!()
	}

	fn serialize_value<T>(&mut self, _: &T) -> Result<()>
			where T: Serialize + ?Sized {
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}

serde_primitive!(StringSerializer(Box<str>));
