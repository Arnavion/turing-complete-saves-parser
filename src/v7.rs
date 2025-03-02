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
	pub clock_speed: u64,
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
		let menu_visible = <_>::parse(input);
		let clock_speed = <_>::parse(input);
		let dependencies = Slice::parse_with_length_prefix(input);
		let description = <_>::parse(input);
		let camera_position = <_>::parse(input);
		let synced = u8::parse(input).into();
		_ = u16::parse(input);
		let player_data = Slice::parse_with_length_prefix(input);
		let hub_description = <_>::parse(input);
		let components = Slice::parse_with_length_prefix(input);
		let wires = Slice::parse_with_length_prefix(input);

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
			let wire = wire.as_inner_ref();
			let WireSegments::Segments(segments) = &wire.segments else { continue; };

			let start = wire.start;
			let mut end = start;
			for segment in segments {
				let segment = segment.as_inner_ref();
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

			if let Some(previous_wire) = wires.insert((start.min(end), start.max(end)), format!("{wire:?}")) {
				println!("{wire:?} overlaps with {previous_wire}");
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
	pub settings: Slice<'a, u16, u64>,
	pub buffer_size: i64,
	pub ui_order: i16,
	pub word_size: i64,
	pub discarded: i64,
	pub custom_data: Option<CustomCompData<'a>>,
	pub assembler_data: Option<AssemblerInfo<'a>>,
}

impl<'a> Parse<'a> for Component<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		let kind = <_>::parse(input);
		Self {
			kind,
			position: <_>::parse(input),
			rotation: <_>::parse(input),
			permanent_id: <_>::parse(input),
			custom_string: <_>::parse(input),
			settings: Slice::parse_with_length_prefix(input),
			buffer_size: <_>::parse(input),
			ui_order: <_>::parse(input),
			word_size: <_>::parse(input),
			discarded: <_>::parse(input),
			custom_data: matches!(kind, ComponentKind::Custom).then(|| <_>::parse(input)),
			assembler_data: matches!(kind, ComponentKind::Assembler | ComponentKind::ProbeMemoryBit | ComponentKind::ImmProbeMemoryBit | ComponentKind::ProbeMemoryWord | ComponentKind::StaticValue | ComponentKind::Console | ComponentKind::PixelScreen).then(|| <_>::parse(input)),
		}
	}
}

enum_impl_from! {
	#[derive(Clone, Copy, Debug)]
	pub enum ComponentKind: u16 {
		None = 0,
		Off = 1,
		On = 2,
		NotBit = 3,
		AndBit = 4,
		And3Bit = 5,
		NandBit = 6,
		OrBit = 7,
		Or3Bit = 8,
		NorBit = 9,
		XorBit = 10,
		XnorBit = 11,
		SwitchBit = 12,
		DelayLineBit = 13,
		RegisterBit = 14,
		FullAdder = 15,
		MakerBit8 = 16,
		SplitterBit8 = 17,
		NotWord = 18,
		OrWord = 19,
		AndWord = 20,
		NandWord = 21,
		NorWord = 22,
		XorWord = 23,
		XnorWord = 24,
		SwitchWord = 25,
		Equal = 26,
		LessU = 27,
		LessS = 28,
		Neg = 29,
		Add = 30,
		Mul = 31,
		Div = 32,
		Lsl = 33,
		Lsr = 34,
		Rol = 35,
		Ror = 36,
		Asr = 37,
		Counter = 38,
		RegisterWord = 39,
		ImmRegisterWord = 40,
		ImmDelayLineBit = 41,
		Mux = 42,
		Decoder1 = 43,
		Decoder2 = 44,
		Decoder3 = 45,
		Constant = 46,
		SplitterWord2 = 47,
		MakerWord2 = 48,
		FrontPanel = 49,
		Assembler = 50,
		Ssd = 51,
		Ram = 52,
		RamLatency = 53,
		RamFast = 54,
		DelayLineWord = 55,
		RamDualLoad = 56,
		FileLoader = 57,
		CcLevelOutput = 58,
		LevelGate = 59,
		LevelInput1 = 60,
		LevelInputWord = 61,
		LevelInputSwitched = 62,
		LevelInput2Pin = 63,
		LevelInput3Pin = 64,
		LevelInput4Pin = 65,
		LevelInputCustom = 66,
		LevelInputArch = 67,
		LevelOutput1 = 68,
		LevelOutputWord = 69,
		LevelOutputSwitched = 70,
		LevelOutput1Sum = 71,
		LevelOutput1Car = 72,
		LevelOutput2Pin = 73,
		LevelOutput3Pin = 74,
		LevelOutput4Pin = 75,
		LevelOutputArch = 76,
		LevelOutputCounter = 77,
		Custom = 78,
		CcInput = 79,
		CcInputBuffer = 80,
		CcOutput = 81,
		ProbeMemoryBit = 82,
		ProbeMemoryWord = 83,
		ProbeWireBit = 84,
		ProbeWireWord = 85,
		ConfigDelay = 86,
		Halt = 87,
		Console = 88,
		SegmentDisplay = 89,
		StaticValue = 90,
		PixelScreen = 91,
		Time = 92,
		Keyboard = 93,
		StaticEval = 94,
		VerilogInput = 95,
		VerilogOutput = 96,
		MakerWord4 = 97,
		MakerWord8 = 98,
		SplitterWord4 = 99,
		SplitterWord8 = 100,
		StaticIndexer = 101,
		ImmProbeMemoryBit = 102,
		ImmDelayLineWord = 103,
		Inc = 104,
		CcLevelInputCustom = 105,
		CcLevelInput = 106,
		ImmRegisterBit = 107,
		Mod = 108,
		SplitterBit2 = 109,
		SplitterBit4 = 110,
		MakerBit2 = 111,
		MakerBit4 = 112,
		ImmProbeMemoryWord = 113,
		Concatenator2 = 114,
		Concatenator4 = 115,
		Concatenator8 = 116,
		StaticIndexerConfig = 117,
		Rom = 118,
	}
}

#[derive(Clone, Debug)]
pub struct CustomCompData<'a> {
	pub id: i64,
	pub static_states: Slice<'a, u16, (i64, i64)>,
	pub linked_word_sizes: Slice<'a, u16, (i64, i64)>,
}

impl<'a> Parse<'a> for CustomCompData<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		Self {
			id: <_>::parse(input),
			static_states: Slice::parse_with_length_prefix(input),
			linked_word_sizes: Slice::parse_with_length_prefix(input),
		}
	}
}

#[derive(Clone, Debug)]
pub struct AssemblerInfo<'a> {
	pub programs: Slice<'a, u16, (&'a str, &'a str)>,
	pub watched_components: Slice<'a, u16, WatchedComponent<'a>>,
}

impl<'a> Parse<'a> for AssemblerInfo<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		Self {
			programs: Slice::parse_with_length_prefix(input),
			watched_components: Slice::parse_with_length_prefix(input),
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct WatchedComponent<'a> {
	pub permanent_id: i64,
	pub inner_id: i64,
	pub name: &'a str,
}

impl<'a> Parse<'a> for WatchedComponent<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		Self {
			permanent_id: <_>::parse(input),
			inner_id: <_>::parse(input),
			name: <_>::parse(input),
		}
	}
}

#[derive(Clone, Debug)]
pub struct Wire<'a> {
	pub color: u8,
	pub comment: &'a str,
	pub start: Point,
	pub segments: WireSegments<'a>,
}

impl<'a> Parse<'a> for Wire<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		Self {
			color: <_>::parse(input),
			comment: <_>::parse(input),
			start: <_>::parse(input),
			segments: <_>::parse(input),
		}
	}
}

#[derive(Clone, Debug)]
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
			let mut segments = &input[..input.iter().position(|&b| b & 0x1f == 0).unwrap()];
			*input = &input[(segments.len() + 1)..];
			Self::Segments(Slice::parse_until_end(&mut segments))
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
