fn main() {
	let mut args = std::env::args_os();
	let argv0 = args.next().unwrap_or_else(|| env!("CARGO_BIN_NAME").into());
	let path = parse_args(args, &argv0);

	let input = std::fs::read(&path).unwrap();
	let circuit_data = <turing_complete_saves_parser::CircuitData as turing_complete_saves_parser::Parse<'_>>::parse(&mut &input[..]);
	#[allow(clippy::match_same_arms)]
	match circuit_data {
		turing_complete_saves_parser::CircuitData::V6(input) =>
			_ = <turing_complete_saves_parser::v6::CircuitData<'_> as turing_complete_saves_parser::Parse<'_>>::parse(&mut &input[..]),
			// println!("{:#?}", <turing_complete_saves_parser::v6::CircuitData<'_> as turing_complete_saves_parser::Parse<'_>>::parse(&mut &input[..])),

		turing_complete_saves_parser::CircuitData::V7(input) =>
			_ = <turing_complete_saves_parser::v7::CircuitData<'_> as turing_complete_saves_parser::Parse<'_>>::parse(&mut &input[..]),
			// println!("{:#?}", <turing_complete_saves_parser::v7::CircuitData<'_> as turing_complete_saves_parser::Parse<'_>>::parse(&mut &input[..])),

		turing_complete_saves_parser::CircuitData::V8(input) =>
			_ = <turing_complete_saves_parser::v8::CircuitData<'_> as turing_complete_saves_parser::Parse<'_>>::parse(&mut &input[..]),
			// println!("{:#?}", <turing_complete_saves_parser::v8::CircuitData<'_> as turing_complete_saves_parser::Parse<'_>>::parse(&mut &input[..])),

		turing_complete_saves_parser::CircuitData::V9(input) =>
			_ = <turing_complete_saves_parser::v9::CircuitData<'_> as turing_complete_saves_parser::Parse<'_>>::parse(&mut &input[..]),
			// println!("{:#?}", <turing_complete_saves_parser::v9::CircuitData<'_> as turing_complete_saves_parser::Parse<'_>>::parse(&mut &input[..])),

		turing_complete_saves_parser::CircuitData::V10(input) =>
			_ = <turing_complete_saves_parser::v10::CircuitData<'_> as turing_complete_saves_parser::Parse<'_>>::parse(&mut &input[..]),
			// println!("{:#?}", <turing_complete_saves_parser::v10::CircuitData<'_> as turing_complete_saves_parser::Parse<'_>>::parse(&mut &input[..])),
	}
}

fn parse_args(mut args: impl Iterator<Item = std::ffi::OsString>, argv0: &std::ffi::OsStr) -> std::path::PathBuf {
	let mut path = None;

	for opt in &mut args {
		match opt.to_str() {
			Some("--help") => {
				write_usage(std::io::stdout(), argv0);
				std::process::exit(0);
			},

			Some("--") => {
				path = args.next();
				break;
			},

			_ if path.is_none() => path = Some(opt),

			_ => write_usage_and_crash(argv0),
		}
	}

	let None = args.next() else { write_usage_and_crash(argv0); };

	let Some(path) = path else { write_usage_and_crash(argv0); };
	path.into()
}

fn write_usage_and_crash(argv0: &std::ffi::OsStr) -> ! {
	write_usage(std::io::stderr(), argv0);
	std::process::exit(1);
}

fn write_usage(mut w: impl std::io::Write, argv0: &std::ffi::OsStr) {
	_ = writeln!(w, "Usage: {} <circuit.data>", argv0.to_string_lossy());
}
