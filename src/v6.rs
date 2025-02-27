use crate::parser::{
	Parse,
	Slice,
};

#[derive(Clone, Debug)]
pub struct CircuitData<'a> {
	pub custom_id: i64,
	pub hub_id: u32,
	pub gate: i64,
	pub delay: i64,
	pub menu_visible: bool,
	pub clock_speed: u32,
	pub dependencies: Slice<'a, u16, i64>,
	pub description: &'a str,
	pub camera_position: Point,
	pub synced: SyncState,
	pub player_data: Slice<'a, u16, u8>,
	pub hub_description: &'a str,
	pub components: Slice<'a, u64, Component<'a>>,
	pub wires: Slice<'a, u64, Wire<'a>>,
}

impl<'a> Parse<'a> for CircuitData<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		let custom_id = <_>::parse(input);
		let hub_id = <_>::parse(input);
		let gate = <_>::parse(input);
		let delay = <_>::parse(input);
		let menu_visible = u8::parse(input) != 0;
		let clock_speed = <_>::parse(input);
		let dependencies = <_>::parse(input);
		let description = <_>::parse(input);
		let camera_position = <_>::parse(input);
		let synced = u8::parse(input).into();
		_ = u8::parse(input);
		_ = u16::parse(input);
		let player_data = <_>::parse(input);
		let hub_description = <_>::parse(input);
		let components = <_>::parse(input);
		let wires = <_>::parse(input);

		let result = Self {
			custom_id,
			hub_id,
			gate,
			delay,
			menu_visible,
			clock_speed,
			dependencies,
			description,
			camera_position,
			synced,
			player_data,
			hub_description,
			components,
			wires,
		};

		let mut wires: std::collections::BTreeMap<_, _> = Default::default();
		let mut found_dupes = false;
		for wire in &result.wires {
			let WireSegments::Segments(segments) = &wire.segments else { continue; };

			let start = wire.start;
			let mut end = start;
			for segment in segments {
				let len = i16::from(segment.length);
				match segment.direction {
					WireDirection::Right => end.x += len,
					WireDirection::DownRight => {end.x += len; end.y += len; },
					WireDirection::Down => end.y += len,
					WireDirection::DownLeft => { end.x -= len; end.y += len; },
					WireDirection::Left => end.x -= len,
					WireDirection::UpLeft => { end.x -= len; end.y -= len; },
					WireDirection::Up => end.y -= len,
					WireDirection::UpRight => { end.x += len; end.y -= len; },
				}
			}

			if let Some(previous_wire) = wires.insert((start.min(end), start.max(end)), wire) {
				println!("{wire:?} overlaps with {previous_wire:?}");
				found_dupes = true;
			}
		}
		assert!(!found_dupes);

		result
	}
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Point {
	pub x: i16,
	pub y: i16,
}

impl Parse<'_> for Point {
	fn parse(input: &mut &[u8]) -> Self {
		Self {
			x: i16::parse(input),
			y: i16::parse(input),
		}
	}
}

enum_impl_from! {
	#[derive(Clone, Copy, Debug)]
	pub enum SyncState: u8 {
		Unsynced = 0,
		Synced = 1,
		ChangedAfterSync = 2,
	}
}

#[derive(Clone, Debug)]
pub struct Component<'a> {
	pub kind: ComponentKind,
	pub position: Point,
	pub rotation: u8,
	pub permanent_id: u64,
	pub custom_string: &'a str,
	pub settings: [u64; 2],
	pub ui_order: i16,
	pub custom_data: Option<CustomCompData>,
	pub assembler_data: Option<AssemblerInfo<'a>>,
}

impl<'a> Parse<'a> for Component<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		let kind = u16::parse(input).into();
		Self {
			kind,
			position: <_>::parse(input),
			rotation: <_>::parse(input),
			permanent_id: <_>::parse(input),
			custom_string: <_>::parse(input),
			settings: [
				<_>::parse(input),
				<_>::parse(input),
			],
			ui_order: <_>::parse(input),
			custom_data: matches!(kind, ComponentKind::Custom).then(|| <_>::parse(input)),
			assembler_data: matches!(kind, ComponentKind::Program | ComponentKind::Program81 | ComponentKind::Program84).then(|| <_>::parse(input)),
		}
	}
}

enum_impl_from! {
	#[derive(Clone, Copy, Debug)]
	pub enum ComponentKind: u16 {
		Error = 0,
		Off = 1,
		On = 2,
		Buffer1 = 3,
		Not = 4,
		And = 5,
		And3 = 6,
		Nand = 7,
		Or = 8,
		Or3 = 9,
		Nor = 10,
		Xor = 11,
		Xnor = 12,
		Counter8 = 13,
		VirtualCounter8 = 14,
		Counter64 = 15,
		VirtualCounter64 = 16,
		Ram8 = 17,
		VirtualRam8 = 18,
		Deleted0 = 19,
		Deleted1 = 20,
		Deleted17 = 21,
		Deleted18 = 22,
		Register8 = 23,
		VirtualRegister8 = 24,
		Register8red = 25,
		VirtualRegister8red = 26,
		Register8redPlus = 27,
		VirtualRegister8redPlus = 28,
		Register64 = 29,
		VirtualRegister64 = 30,
		Switch8 = 31,
		Mux8 = 32,
		ComDecoder1 = 33,
		ComDecoder3 = 34,
		Constant8 = 35,
		Not8 = 36,
		Or8 = 37,
		And8 = 38,
		Xor8 = 39,
		ComEqual8 = 40,
		Deleted2 = 41,
		Deleted3 = 42,
		Neg8 = 43,
		Add8 = 44,
		Mul8 = 45,
		Splitter8 = 46,
		Maker8 = 47,
		Splitter64 = 48,
		Maker64 = 49,
		ComFullAdder = 50,
		ComBitMemory = 51,
		VirtualcomBitMemory = 52,
		Deleted10 = 53,
		ComDecoder2 = 54,
		ComTime = 55,
		NoteSound = 56,
		Deleted4 = 57,
		Deleted5 = 58,
		Keyboard = 59,
		ComFileLoader = 60,
		Halt = 61,
		WireCluster = 62,
		LevelScreen = 63,
		Program81 = 64,
		Program81red = 65,
		Deleted6 = 66,
		Deleted7 = 67,
		Program84 = 68,
		ComLevelGate = 69,
		Input1 = 70,
		ComLevelInput2Pin = 71,
		ComLevelInput3Pin = 72,
		ComLevelInput4Pin = 73,
		LevelInputConditions = 74,
		Input8 = 75,
		Input64 = 76,
		LevelInputCode = 77,
		ComLevelInputArch = 78,
		Output1 = 79,
		ComLevelOutput1Sum = 80,
		ComLevelOutput1Car = 81,
		Deleted8 = 82,
		Deleted9 = 83,
		ComLevelOutput2Pin = 84,
		ComLevelOutput3Pin = 85,
		ComLevelOutput4Pin = 86,
		Output8 = 87,
		Output64 = 88,
		ComLevelOutputArch = 89,
		ComLevelOutputCounter = 90,
		Deleted11 = 91,
		Custom = 92,
		VirtualCustom = 93,
		Program = 94,
		DelayLine1 = 95,
		VirtualDelayLine1 = 96,
		Console = 97,
		Shl8 = 98,
		Shr8 = 99,
		Constant64 = 100,
		Not64 = 101,
		Or64 = 102,
		And64 = 103,
		Xor64 = 104,
		Neg64 = 105,
		Add64 = 106,
		Mul64 = 107,
		ComEqual64 = 108,
		ComLessU64 = 109,
		ComLessS64 = 110,
		Shl64 = 111,
		Shr64 = 112,
		Mux64 = 113,
		Switch64 = 114,
		ComProbeMemoryBit = 115,
		ComProbeMemoryWord = 116,
		AndOrLatch = 117,
		NandNandLatch = 118,
		NorNorLatch = 119,
		ComLessU8 = 120,
		ComLessS8 = 121,
		DotMatrixDisplay = 122,
		ComSegmentDisplay = 123,
		Input16 = 124,
		Input32 = 125,
		Output16 = 126,
		Output32 = 127,
		Deleted12 = 128,
		Deleted13 = 129,
		Deleted14 = 130,
		Deleted15 = 131,
		Deleted16 = 132,
		Buffer8 = 133,
		Buffer16 = 134,
		Buffer32 = 135,
		Buffer64 = 136,
		ComProbeWireBit = 137,
		ComProbeWireWord = 138,
		Switch1 = 139,
		Output1z = 140,
		Output8z = 141,
		Output16z = 142,
		Output32z = 143,
		Output64z = 144,
		Constant16 = 145,
		Not16 = 146,
		Or16 = 147,
		And16 = 148,
		Xor16 = 149,
		Neg16 = 150,
		Add16 = 151,
		Mul16 = 152,
		ComEqual16 = 153,
		ComLessU16 = 154,
		ComLessS16 = 155,
		Shl16 = 156,
		Shr16 = 157,
		Mux16 = 158,
		Switch16 = 159,
		Splitter16 = 160,
		Maker16 = 161,
		Register16 = 162,
		VirtualRegister16 = 163,
		Counter16 = 164,
		VirtualCounter16 = 165,
		Constant32 = 166,
		Not32 = 167,
		Or32 = 168,
		And32 = 169,
		Xor32 = 170,
		Neg32 = 171,
		Add32 = 172,
		Mul32 = 173,
		ComEqual32 = 174,
		ComLessU32 = 175,
		ComLessS32 = 176,
		Shl32 = 177,
		Shr32 = 178,
		Mux32 = 179,
		Switch32 = 180,
		Splitter32 = 181,
		Maker32 = 182,
		Register32 = 183,
		VirtualRegister32 = 184,
		Counter32 = 185,
		VirtualCounter32 = 186,
		LevelOutput8z = 187,
		Nand8 = 188,
		Nor8 = 189,
		Xnor8 = 190,
		Nand16 = 191,
		Nor16 = 192,
		Xnor16 = 193,
		Nand32 = 194,
		Nor32 = 195,
		Xnor32 = 196,
		Nand64 = 197,
		Nor64 = 198,
		Xnor64 = 199,
		Ram = 200,
		VirtualRam = 201,
		ComRamLatency = 202,
		VirtualcomRamLatency = 203,
		ComRamFast = 204,
		VirtualcomRamFast = 205,
		Rom = 206,
		VirtualRom = 207,
		SolutionRom = 208,
		VirtualSolutionRom = 209,
		DelayLine8 = 210,
		VirtualDelayLine8 = 211,
		DelayLine16 = 212,
		VirtualDelayLine16 = 213,
		DelayLine32 = 214,
		VirtualDelayLine32 = 215,
		DelayLine64 = 216,
		VirtualDelayLine64 = 217,
		ComRamDualLoad = 218,
		VirtualcomRamDualLoad = 219,
		Hdd = 220,
		VirtualHdd = 221,
		Network = 222,
		Rol8 = 223,
		Rol16 = 224,
		Rol32 = 225,
		Rol64 = 226,
		Ror8 = 227,
		Ror16 = 228,
		Ror32 = 229,
		Ror64 = 230,
		IndexerBit = 231,
		IndexerByte = 232,
		DivMod8 = 233,
		DivMod16 = 234,
		DivMod32 = 235,
		DivMod64 = 236,
		SpriteDisplay = 237,
		ComConfigDelay = 238,
		Clock = 239,
		ComLevelInput1 = 240,
		LevelInput8 = 241,
		ComLevelOutput1 = 242,
		LevelOutput8 = 243,
		Ashr8 = 244,
		Ashr16 = 245,
		Ashr32 = 246,
		Ashr64 = 247,
		Bidirectional1 = 248,
		VirtualBidirectional1 = 249,
		Bidirectional8 = 250,
		VirtualBidirectional8 = 251,
		Bidirectional16 = 252,
		VirtualBidirectional16 = 253,
		Bidirectional32 = 254,
		VirtualBidirectional32 = 255,
		Bidirectional64 = 256,
		VirtualBidirectional64 = 257,
	}
}

#[derive(Clone, Copy, Debug)]
pub struct CustomCompData {
	pub id: i64,
	pub custom_nudge: Point,
}

impl Parse<'_> for CustomCompData {
	fn parse(input: &mut &[u8]) -> Self {
		Self {
			id: <_>::parse(input),
			custom_nudge: <_>::parse(input),
		}
	}
}

#[derive(Clone, Debug)]
pub struct AssemblerInfo<'a> {
	pub programs: Slice<'a, u16, (i64, &'a str)>,
}

impl<'a> Parse<'a> for AssemblerInfo<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		Self {
			programs: <_>::parse(input),
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct Wire<'a> {
	pub width: u8,
	pub color: u8,
	pub comment: &'a str,
	pub start: Point,
	pub segments: WireSegments<'a>,
}

impl<'a> Parse<'a> for Wire<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		Self {
			width: <_>::parse(input),
			color: <_>::parse(input),
			comment: <_>::parse(input),
			start: <_>::parse(input),
			segments: <_>::parse(input),
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub enum WireSegments<'a> {
	TeleWireEnd(Point),
	Segments(Slice<'a, u64, WireSegment>),
}

impl<'a> Parse<'a> for WireSegments<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		if input[0] == 0x20 {
			_ = u8::parse(input);
			Self::TeleWireEnd(<_>::parse(input))
		}
		else {
			#[allow(clippy::verbose_bit_mask)]
			let segments = &input[..input.iter().position(|&b| b & 0x1f == 0).unwrap()];
			*input = &input[(segments.len() + 1)..];
			Self::Segments(Slice::until_end(segments))
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct WireSegment {
	pub length: u8,
	pub direction: WireDirection,
}

impl<'a> Parse<'a> for WireSegment {
	fn parse(input: &mut &'a [u8]) -> Self {
		let ws = u8::parse(input);
		let length = ws & 0x1f;
		let direction = WireDirection::from(ws >> 5);
		Self {
			length,
			direction,
		}
	}
}

enum_impl_from! {
	#[derive(Clone, Copy, Debug)]
	pub enum WireDirection: u8 {
		Right = 0,
		DownRight = 1,
		Down = 2,
		DownLeft = 3,
		Left = 4,
		UpLeft = 5,
		Up = 6,
		UpRight = 7,
	}
}
