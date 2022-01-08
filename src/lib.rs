#![deny(rust_2018_idioms)]
#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::default_trait_access,
	clippy::must_use_candidate,
)]

// Based on https://github.com/Stuffe/save_monger/commit/6f05cf21929aa15c49c365760f5dde72224cd106
// with additional `ComponentKind::DelayBuffer = 95` mapping since upstream hasn't been updated to have that yet.

#[macro_use]
mod parser;
pub use parser::{
	Either,
	Parse,
	Slice, SliceIter,
};

enum_impl_from! {
	#[derive(Clone, Copy, Debug)]
	pub enum ComponentKind: u16 {
		Error = 0,
		Off = 1,
		On = 2,
		Buffer = 3,
		Not = 4,
		And = 5,
		And3 = 6,
		Nand = 7,
		Or = 8,
		Or3 = 9,
		Nor = 10,
		Xor = 11,
		Xnor = 12,
		Counter = 13,
		VirtualCounter = 14,
		QwordCounter = 15,
		VirtualQwordCounter = 16,
		Ram = 17,
		VirtualRam = 18,
		QwordRam = 19,
		VirtualQwordRam = 20,
		Stack = 21,
		VirtualStack = 22,
		Register = 23,
		VirtualRegister = 24,
		RegisterRed = 25,
		VirtualRegisterRed = 26,
		RegisterRedPlus = 27,
		VirtualRegisterRedPlus = 28,
		QwordRegister = 29,
		VirtualQwordRegister = 30,
		ByteSwitch = 31,
		Mux = 32,
		Demux = 33,
		BiggerDemux = 34,
		ByteConstant = 35,
		ByteNot = 36,
		ByteOr = 37,
		ByteAnd = 38,
		ByteXor = 39,
		ByteEqual = 40,
		ByteLessU = 41,
		ByteLessI = 42,
		ByteNeg = 43,
		ByteAdd2 = 44,
		ByteMul2 = 45,
		ByteSplitter = 46,
		ByteMaker = 47,
		QwordSplitter = 48,
		QwordMaker = 49,
		FullAdder = 50,
		BitMemory = 51,
		VirtualBitMemory = 52,
		SRLatch = 53,
		Random = 54,
		Clock = 55,
		WaveformGenerator = 56,
		HttpClient = 57,
		AsciiScreen = 58,
		Keyboard = 59,
		FileInput = 60,
		Halt = 61,
		CircuitCluster = 62,
		Screen = 63,
		Program1 = 64,
		Program1Red = 65,
		Program2 = 66,
		Program3 = 67,
		Program4 = 68,
		LevelGate = 69,
		Input1 = 70,
		Input2 = 71,
		Input3 = 72,
		Input4 = 73,
		Input1BConditions = 74,
		Input1B = 75,
		InputQword = 76,
		Input1BCode = 77,
		Input1_1B = 78,
		Output1 = 79,
		Output1Sum = 80,
		Output1Car = 81,
		Output1Aval = 82,
		Output1Bval = 83,
		Output2 = 84,
		Output3 = 85,
		Output4 = 86,
		Output1B = 87,
		OutputQword = 88,
		Output1_1B = 89,
		OutputCounter = 90,
		InputOutput = 91,
		Custom = 92,
		VirtualCustom = 93,
		QwordProgram = 94,
		DelayBuffer = 95,
	}
}

enum_impl_from! {
	#[derive(Clone, Copy, Debug)]
	pub enum CircuitKind: u8 {
		Bit = 0,
		Byte = 1,
		Qword = 2,
	}
}

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
pub struct Point {
	pub x: i16,
	pub y: i16,
}

impl Parse<'_> for Point {
	fn parse(input: &mut &[u8]) -> Self {
		Point {
			x: i8::parse(input).into(),
			y: i8::parse(input).into(),
		}
	}
}

#[allow(unused)]
#[derive(Debug)]
pub struct Component<'a> {
	pub kind: ComponentKind,
	pub position: Point,
	pub rotation: u8,
	pub permanent_id: u32,
	pub custom_string: &'a str,
	pub custom_id: i64,
	pub program_name: Option<&'a str>,
}

impl<'a> Parse<'a> for Component<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		let kind: ComponentKind = u16::parse(input).into();
		let mut result = Component {
			kind,
			position: Point::parse(input),
			rotation: u8::parse(input),
			permanent_id: u32::parse(input),
			custom_string: <&'a str>::parse(input),
			custom_id: 0,
			program_name: None,
		};
		match kind {
			ComponentKind::Program1 | ComponentKind::Program2 | ComponentKind::Program3 | ComponentKind::Program4 | ComponentKind::QwordProgram =>
				result.program_name = Some(<&'a str>::parse(input)),
			ComponentKind::Custom => result.custom_id = i64::parse(input),
			_ => (),
		}
		result
	}
}

#[allow(unused)]
#[derive(Debug)]
pub struct ParseCircuit<'a> {
	pub permanent_id: u32,
	pub kind: CircuitKind,
	pub color: u8,
	pub comment: &'a str,
	pub path: Either<Slice<'a, Point>, Vec<Point>>,
}

impl<'a> Parse<'a> for ParseCircuit<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		ParseCircuit {
			permanent_id: u32::parse(input),
			kind: u8::parse(input).into(),
			color: u8::parse(input),
			comment: <&'a str>::parse(input),
			path: Either::Left(Slice::parse(input)),
		}
	}
}

#[allow(unused)]
#[derive(Debug)]
pub struct Save<'a> {
	pub version: i64,
	pub nand: u32,
	pub delay: u32,
	pub menu_visible: bool,
	pub clock_speed: u32,
	pub nesting_level: u8,
	pub dependencies: Option<Slice<'a, i64>>,
	pub description: &'a str,
	pub components: Either<Slice<'a, Component<'a>>, Vec<Component<'a>>>,
	pub circuits: Either<Slice<'a, ParseCircuit<'a>>, Vec<ParseCircuit<'a>>>,
}

impl<'a> Parse<'a> for Save<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		let (&version, rest) = input.split_first().unwrap();
		*input = rest;
		match version {
			0 => Save {
				version: i64::parse(input),
				nand: u32::parse(input),
				delay: u32::parse(input),
				menu_visible: bool::parse(input),
				clock_speed: u32::parse(input),
				nesting_level: u8::parse(input),
				dependencies: Some(Slice::parse(input)),
				description: <&'a str>::parse(input),
				components: Either::Left(Slice::parse(input)),
				circuits: Either::Left(Slice::parse(input)),
			},

			b'1' => {
				let mut result = Save {
					version: 0,
					nand: 99999,
					delay: 99999,
					menu_visible: true,
					clock_speed: 100_000,
					nesting_level: 0,
					dependencies: None,
					description: "",
					components: Either::Right(vec![]),
					circuits: Either::Right(vec![]),
				};

				let input = std::str::from_utf8(*input).unwrap();
				let parts: Vec<_> = input.split('|').collect();
				assert!(parts.len() == 4 || parts.len() == 5);

				if !parts[3].is_empty() {
					let mut scores = parts[3].split(',');
					result.nand = scores.next().unwrap().parse().unwrap();
					result.delay = scores.next().unwrap().parse().unwrap();
				}

				if parts.len() == 5 && !parts[4].is_empty() {
					result.version = parts[4].parse().unwrap();
				}

				if !parts[1].is_empty() {
					let mut components = vec![];
					for component_string in parts[1].split(';') {
						let mut component_parts = component_string.split('`');
						let mut component = Component {
							kind: component_parts.next().unwrap().into(),
							position: Point {
								x: component_parts.next().unwrap().parse().unwrap(),
								y: component_parts.next().unwrap().parse().unwrap(),
							},
							rotation: component_parts.next().unwrap().parse().unwrap(),
							permanent_id: component_parts.next().unwrap().parse().unwrap(),
							custom_string: component_parts.next().unwrap(),
							custom_id: 0,
							program_name: None,
						};
						if let ComponentKind::Custom = component.kind {
							component.custom_id = component.custom_string.parse().unwrap();
						}
						assert!(component_parts.next().is_none());
						components.push(component);
					}
					result.components = Either::Right(components);
				}

				if !parts[2].is_empty() {
					let mut circuits = vec![];
					for circuit_string in parts[2].split(';') {
						let mut circuit_parts = circuit_string.split('`');
						let circuit = ParseCircuit {
							permanent_id: circuit_parts.next().unwrap().parse().unwrap(),
							kind: circuit_parts.next().unwrap().parse::<u8>().unwrap().into(),
							color: circuit_parts.next().unwrap().parse().unwrap(),
							comment: circuit_parts.next().unwrap(),
							path: {
								let mut path = vec![];
								let mut x = 0;
								for (i, n) in circuit_parts.next().unwrap().split(',').enumerate() {
								    if i % 2 == 0 {
								        x = n.parse().unwrap();
								    }
								    else {
								        let y = n.parse().unwrap();
								        path.push(Point { x, y });
								    }
								}
								Either::Right(path)
							},
						};
						assert!(circuit_parts.next().is_none());
						circuits.push(circuit);
					}
					result.circuits = Either::Right(circuits);
				}

				result
			},

			_ => unreachable!(),
		}
	}
}
