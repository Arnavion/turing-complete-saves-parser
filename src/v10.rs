use crate::parser::{
	Encode,
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

impl CircuitData<'_> {
	pub fn encode_final(&self) -> Vec<u8> {
		let mut raw = vec![];
		self.encode(&mut raw);
		let mut raw = snap::raw::Encoder::new().compress_vec(&raw).unwrap();
		raw.insert(0, 10);
		raw
	}
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

		for component in &result.components {
			let component = component.as_inner_ref();

			#[allow(clippy::manual_assert)]
			if matches!(component.kind, ComponentKind::StaticIndexer) && component.word_size <= 0 {
				panic!("{component:?}");
			}

			#[allow(clippy::manual_assert)]
			if matches!(component.kind, ComponentKind::MakerWord2 | ComponentKind::MakerWord4 | ComponentKind::MakerWord8) {
				panic!("{component:?}");
			}
		}

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

impl Encode for CircuitData<'_> {
	fn encode(&self, out: &mut Vec<u8>) {
		self.custom_id.encode(out);
		self.hub_id.encode(out);
		self.gate.encode(out);
		self.delay.encode(out);
		self.menu_visible.encode(out);
		self.clock_speed.encode(out);
		self.dependencies.encode_with_length_prefix(out);
		self.description.encode(out);
		self.camera_position.encode(out);
		self.synced.encode(out);
		0_u16.encode(out);
		self.player_data.encode_with_length_prefix(out);
		self.hub_description.encode(out);
		self.components.encode_with_length_prefix(out);
		self.wires.encode_with_length_prefix(out);
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

impl Encode for Point {
	fn encode(&self, out: &mut Vec<u8>) {
		self.x.encode(out);
		self.y.encode(out);
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
	pub linked_components: Slice<'a, u16, LinkedComponent<'a>>,
	pub selected_programs: AssemblerInfo<'a>,
	pub custom_data: Option<CustomCompData<'a>>,
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
			linked_components: Slice::parse_with_length_prefix(input),
			selected_programs: <_>::parse(input),
			custom_data: matches!(kind, ComponentKind::Custom).then(|| <_>::parse(input)),
		}
	}
}

impl Encode for Component<'_> {
	fn encode(&self, out: &mut Vec<u8>) {
		self.kind.encode(out);
		self.position.encode(out);
		self.rotation.encode(out);
		self.permanent_id.encode(out);
		self.custom_string.encode(out);
		self.settings.encode_with_length_prefix(out);
		self.buffer_size.encode(out);
		self.ui_order.encode(out);
		self.word_size.encode(out);
		self.linked_components.encode_with_length_prefix(out);
		self.selected_programs.encode(out);
		self.custom_data.encode(out);
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
		Clz = 49,
		RegisterWordConfig = 50,
		Ssd = 51,
		Deleted3 = 52,
		RamLatency = 53,
		LoadPort = 54,
		DelayLineWord = 55,
		StorePort = 56,
		Ctz = 57,
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
		RamFast = 118,
		DelayLineWordConfig = 119,
		Deleted1 = 120,
		Deleted2 = 121,
		ImmStaticValue = 122,
	}
}

#[derive(Clone, Copy, Debug)]
pub struct LinkedComponent<'a> {
	pub permanent_id: i64,
	pub inner_id: i64,
	pub name: &'a str,
	pub offset: i64,
}

impl<'a> Parse<'a> for LinkedComponent<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		Self {
			permanent_id: <_>::parse(input),
			inner_id: <_>::parse(input),
			name: <_>::parse(input),
			offset: <_>::parse(input),
		}
	}
}

impl Encode for LinkedComponent<'_> {
	fn encode(&self, out: &mut Vec<u8>) {
		self.permanent_id.encode(out);
		self.inner_id.encode(out);
		self.name.encode(out);
		self.offset.encode(out);
	}
}

#[derive(Clone, Debug)]
pub struct AssemblerInfo<'a> {
	pub programs: Slice<'a, u16, (&'a str, &'a str)>,
}

impl<'a> Parse<'a> for AssemblerInfo<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		Self {
			programs: Slice::parse_with_length_prefix(input),
		}
	}
}

impl Encode for AssemblerInfo<'_> {
	fn encode(&self, out: &mut Vec<u8>) {
		self.programs.encode_with_length_prefix(out);
	}
}

#[derive(Clone, Debug)]
pub struct CustomCompData<'a> {
	pub id: i64,
	pub static_states: Slice<'a, u16, (i64, i64)>,
}

impl<'a> Parse<'a> for CustomCompData<'a> {
	fn parse(input: &mut &'a [u8]) -> Self {
		Self {
			id: <_>::parse(input),
			static_states: Slice::parse_with_length_prefix(input),
		}
	}
}

impl Encode for CustomCompData<'_> {
	fn encode(&self, out: &mut Vec<u8>) {
		self.id.encode(out);
		self.static_states.encode_with_length_prefix(out);
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

impl Encode for Wire<'_> {
	fn encode(&self, out: &mut Vec<u8>) {
		self.color.encode(out);
		self.comment.encode(out);
		self.start.encode(out);
		self.segments.encode(out);
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
			let segments_end_pos = input.iter().position(|&b| b & 0x1f == 0).unwrap() + 1;
			let mut segments;
			(segments, *input) = input.split_at(segments_end_pos);
			Self::Segments(Slice::parse_until_end(&mut segments))
		}
	}
}

impl Encode for WireSegments<'_> {
	fn encode(&self, out: &mut Vec<u8>) {
		match self {
			Self::TeleWireEnd(point) => {
				out.push(0x20);
				point.encode(out);
			},

			Self::Segments(segments) => segments.encode_without_length_prefix(out),
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

impl Encode for WireSegment {
	fn encode(&self, out: &mut Vec<u8>) {
		let length = self.length & 0x1f;
		assert_eq!(self.length, length);
		let direction = u8::from(self.direction);
		let ws = length | (direction << 5);
		ws.encode(out);
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
