use proc_macro::{Delimiter, TokenStream, TokenTree};

#[derive(Clone, Copy)]
struct SerializerFunction {
	name: &'static str,
	subject: &'static str,
	arguments: &'static [(&'static str, &'static str)],
	type_arguments: &'static [(&'static str, &'static str)],
	result: &'static str
}

const SERIALIZER_FUNCTIONS: &[SerializerFunction] = &[
	SerializerFunction {
		name: "serialize_bool",
		subject: "bool",
		arguments: &[
			("value", "::core::primitive::bool")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_i8",
		subject: "i8",
		arguments: &[
			("value", "::core::primitive::i8")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_i16",
		subject: "i16",
		arguments: &[
			("value", "::core::primitive::i16")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_i32",
		subject: "i32",
		arguments: &[
			("value", "::core::primitive::i32")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_i64",
		subject: "i64",
		arguments: &[
			("value", "::core::primitive::i64")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_u8",
		subject: "u8",
		arguments: &[
			("value", "::core::primitive::u8")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_u16",
		subject: "u16",
		arguments: &[
			("value", "::core::primitive::u16")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_u32",
		subject: "u32",
		arguments: &[
			("value", "::core::primitive::u32")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_u64",
		subject: "u64",
		arguments: &[
			("value", "::core::primitive::u64")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_f32",
		subject: "f32",
		arguments: &[
			("value", "::core::primitive::f32")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_f64",
		subject: "f64",
		arguments: &[
			("value", "::core::primitive::f64")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_char",
		subject: "char",
		arguments: &[
			("value", "::core::primitive::char")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_str",
		subject: "Box<str>",
		arguments: &[
			("value", "&::core::primitive::str")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_bytes",
		subject: "&[u8]",
		arguments: &[
			("value", "&[::core::primitive::u8]")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_none",
		subject: "None",
		arguments: &[],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},

	// Maybe change?
	SerializerFunction {
		name: "serialize_some",
		subject: "Some",
		arguments: &[
			("value", "&T")
		],
		type_arguments: &[
			("T", "?::core::marker::Sized + ::serde::ser::Serialize")
		],
		result: "::core::result::Result<(), self::Error>"
	},

	
	SerializerFunction {
		name: "serialize_unit_struct",
		subject: "",
		arguments: &[
			("name", "&'static ::core::primitive::str")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_unit_variant",
		subject: "",
		arguments: &[
			("name", "&'static ::core::primitive::str"),
			("index", "::core::primitive::u32"),
			("variant", "&'static ::core::primitive::str")
		],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_newtype_struct",
		subject: "",
		arguments: &[
			("name", "&'static ::core::primitive::str"),
			("value", "&T")
		],
		type_arguments: &[
			("T", "?::core::marker::Sized + ::serde::ser::Serialize")
		],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_newtype_variant",
		subject: "",
		arguments: &[
			("name", "&'static ::core::primitive::str"),
			("index", "::core::primitive::u32"),
			("variant", "&'static ::core::primitive::str"),
			("value", "&T")
		],
		type_arguments: &[
			("T", "?::core::marker::Sized + ::serde::ser::Serialize")
		],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_tuple_struct",
		subject: "",
		arguments: &[
			("name", "&'static ::core::primitive::str"),
			("length", "::core::primitive::usize")
		],
		type_arguments: &[],
		result: "::core::result::Result<Self::SerializeTupleStruct, self::Error>" //a
	},
	SerializerFunction {
		name: "serialize_tuple_variant",
		subject: "",
		arguments: &[
			("name", "&'static ::core::primitive::str"),
			("index", "::core::primitive::u32"),
			("variant", "&'static ::core::primitive::str"),
			("length", "::core::primitive::usize")
		],
		type_arguments: &[],
		result: "::core::result::Result<Self::SerializeTupleVariant, self::Error>" //a
	},
	SerializerFunction {
		name: "serialize_struct",
		subject: "",
		arguments: &[
			("name", "&'static ::core::primitive::str"),
			("length", "::core::primitive::usize")
		],
		type_arguments: &[],
		result: "::core::result::Result<Self::SerializeStruct, self::Error>" //a
	},
	SerializerFunction {
		name: "serialize_struct_variant",
		subject: "",
		arguments: &[
			("name", "&'static ::core::primitive::str"),
			("index", "::core::primitive::u32"),
			("variant", "&'static ::core::primitive::str"),
			("length", "::core::primitive::usize")
		],
		type_arguments: &[],
		result: "::core::result::Result<Self::SerializeStructVariant, self::Error>" //a
	},
	SerializerFunction {
		name: "serialize_unit",
		subject: "",
		arguments: &[],
		type_arguments: &[],
		result: "::core::result::Result<(), self::Error>"
	},
	SerializerFunction {
		name: "serialize_tuple",
		subject: "",
		arguments: &[
			("length", "::core::primitive::usize")
		],
		type_arguments: &[],
		result: "::core::result::Result<Self::SerializeTuple, self::Error>"
	},
	SerializerFunction {
		name: "serialize_seq",
		subject: "",
		arguments: &[
			("length", "::core::option::Option<::core::primitive::usize>")
		],
		type_arguments: &[],
		result: "::core::result::Result<Self::SerializeSeq, self::Error>"
	},
	SerializerFunction {
		name: "serialize_map",
		subject: "",
		arguments: &[
			("length", "::core::option::Option<::core::primitive::usize>")
		],
		type_arguments: &[],
		result: "::core::result::Result<Self::SerializeMap, self::Error>"
	}
];

struct Arguments {
	name: Option<String>,
	target: Option<String>
}

#[proc_macro]
pub fn serde_primitive(input: TokenStream) -> TokenStream {
	let arguments = Arguments {name: None, target: None};
	let arguments = input.into_iter().enumerate().fold(arguments, |arguments, token| match token {
		(0, TokenTree::Ident(identifier)) => {
			Arguments {name: Some(identifier.to_string()), ..arguments}
		},
		(1, TokenTree::Group(group))
				if group.delimiter() == Delimiter::Parenthesis => {
			let target = group.stream().into_iter()
				.map(|token| token.to_string())
				.collect::<Vec<_>>().join("");
			Arguments {
				target: Some(target),
				..arguments
			}
		},
		(_, token) => panic!("unexpected token {:?}", token.to_string())
	});

	let name = arguments.name.unwrap();
	let target = arguments.target.unwrap();

	format!(
"#[derive(Debug)]
pub struct {0}(::core::option::Option<{1}>);

impl {0} {{
	pub fn new() -> Self {{
		Self::default()
	}}
}}

impl ::core::default::Default for {0} {{
	fn default() -> Self {{
		Self(::core::option::Option::None)
	}}
}}

impl ::core::ops::Try for {0} {{
	type Ok = {1};
	type Error = ::core::option::NoneError;

	fn into_result(self) -> ::core::result::Result<Self::Ok, Self::Error> {{
		self.0.into_result()
	}}

	fn from_error(value: Self::Error) -> Self {{
		Self(::core::option::Option::from_error(value))
	}}

	fn from_ok(value: Self::Ok) -> Self {{
		Self(::core::option::Option::from_ok(value))
	}}
}}

{2}",
		name, target, make_impl(&target, &name)
	).parse().unwrap()
}

fn make_impl(success: &str, impl_for: &str) -> String {
	let functions = SERIALIZER_FUNCTIONS.iter()
		.map(|function| make_fn(function, function.subject == success))
		.collect::<Vec<_>>().join("");
	format!(
"impl<'r> ::serde::ser::Serializer for &'r mut {} {{
	type Ok = ();
	type Error = self::Error;
	type SerializeTupleStruct = &'r mut BlankImplementations;
	type SerializeTupleVariant = &'r mut BlankImplementations;
	type SerializeStruct = &'r mut BlankImplementations;
	type SerializeStructVariant = &'r mut BlankImplementations;
	type SerializeTuple = &'r mut BlankImplementations;
	type SerializeSeq = &'r mut BlankImplementations;
	type SerializeMap = &'r mut BlankImplementations;

	{}
}}",
		impl_for, functions
	)
}

fn make_fn(function: &SerializerFunction, success: bool) -> String {
	let body = if success {
		"self.0 = ::core::option::Option::Some(value.into()); ::core::result::Result::Ok(())".to_owned()
	} else {
		format!("::core::result::Result::Err(Error::custom(\"Nope.\"))")
	};

	let type_arguments = function.type_arguments.iter()
		.map(|argument| format!("{}: {}", argument.0, argument.1))
		.collect::<Vec<_>>().join(", ");

	let arguments = function.arguments.iter()
		.map(|argument| format!("{}: {}", argument.0, argument.1))
		.collect::<Vec<_>>().join(", ");

	format!(
		"fn {}<{}>(self, {}) -> {} {{\n\t\t{}\n\t}}",
		function.name, type_arguments, arguments, function.result, body
	)
}
