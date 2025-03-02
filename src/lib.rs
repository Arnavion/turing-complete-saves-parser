#[macro_use]
mod parser;
pub use parser::{
	Either,
	Encode,
	Parse,
	Slice, SliceIter,
};

pub mod v6;
pub mod v7;
pub mod v8;
pub mod v9;
pub mod v10;

#[derive(Debug)]
pub enum CircuitData {
	V6(Vec<u8>),
	V7(Vec<u8>),
	V8(Vec<u8>),
	V9(Vec<u8>),
	V10(Vec<u8>),
}

impl<'a> Parse<'a> for CircuitData {
	fn parse(input: &mut &'a [u8]) -> Self {
		let (&version, rest) = input.split_first().unwrap();
		*input = rest;
		match version {
			6 => {
				let input = snap::raw::Decoder::new().decompress_vec(input).unwrap();
				Self::V6(input)
			},

			7 => {
				let input = snap::raw::Decoder::new().decompress_vec(input).unwrap();
				Self::V7(input)
			},

			8 => {
				let input = snap::raw::Decoder::new().decompress_vec(input).unwrap();
				Self::V8(input)
			},

			9 => {
				let input = snap::raw::Decoder::new().decompress_vec(input).unwrap();
				Self::V9(input)
			},

			10 => {
				let input = snap::raw::Decoder::new().decompress_vec(input).unwrap();
				Self::V10(input)
			},

			version => panic!("version {version} unsupported"),
		}
	}
}
