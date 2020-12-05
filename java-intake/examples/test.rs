use java_intake::nbt::Serializer;
use serde::Serialize;
use std::fs::File;

#[derive(Serialize)]
struct MyNBT {
	lol: (&'static str, &'static str, u8),
	other_value: u8
}

fn main() {
	let mut file = File::create("./test.nbt")
		.expect("Could not access filesystem.");

	let value = MyNBT {
		lol: ("cock", "penis", 4),
		other_value: 255
	};

	let mut writer = Serializer::new_compound(&mut file, "test");
	value.serialize(&mut writer).expect("Error writing NBT.");
}
