use java_intake::nbt::Serializer;
use serde::Serialize;
use std::{collections::HashMap, fs::File};

/// TAG_Compound
#[derive(Serialize)]
struct MyNBT {
	/// TAG_Byte
	tag_byte: i8,

	/// TAG_Short
	tag_short: i16,

	/// TAG_Int
	tag_int: i32,

	/// TAG_Long
	tag_long: i64,

	/// TAG_Float
	tag_float: f32,

	/// TAG_Double
	tag_double: f64,

	/// TAG_Byte_Array
	tag_byte_array: Vec<i8>,

	/// TAG_String
	tag_string: String,

	/// TAG_List
	tag_list: Vec<String>,

	/// TAG_Compound
	tag_compound: HashMap<String, i16>,

	/// TAG_Int_Array
	tag_int_array: Vec<i32>,

	// /// TAG_Long_Array
	// tag_long_array: Vec<i64>
}

fn main() {
	/*let mut file = File::create("./test.nbt")
		.expect("Could not access filesystem.");*/

	let value = MyNBT {
		tag_byte: i8::MAX,
		tag_short: i16::MAX,
		tag_int: i32::MAX,
		tag_long: i64::MAX,
		tag_float: f32::MAX,
		tag_double: f64::MAX,
		tag_byte_array: vec![0, i8::MAX / 2, i8::MAX],
		tag_string: "Hello, world!".to_owned(),
		tag_list: vec!["Hello,".to_owned(), "world!".to_owned()],
		tag_compound: HashMap::new(),
		tag_int_array: vec![0, i32::MAX / 2, i32::MAX],
		// tag_long_array: vec![0, i64::MAX / 2, i64::MAX]
	};

	let mut buffer = Vec::new();
	let mut writer = Serializer::new_compound(&mut buffer, "test");
	value.serialize(&mut writer).expect("Error writing NBT.");
	drop(writer);
	println!("{:?}", buffer);
}
