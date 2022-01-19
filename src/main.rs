#![deny(rust_2018_idioms)]
#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::default_trait_access,
)]

fn help(writer: &mut impl std::io::Write) {
	drop(writer.write_all(b"\
Parses a Turing Complete save file and prints a textual representation of the circuit.

USAGE:
    turing-complete-saves-parser <filename>
"));
}

fn main() {
	let mut args = std::env::args_os();
	drop(args.next().unwrap());
	if let Some(arg1) = args.next() {
		if arg1 == "--help" {
			help(&mut std::io::stdout());
		}
		else {
			let input = std::fs::read(arg1).unwrap();
			let save = <turing_complete_saves_parser::Save<'_> as turing_complete_saves_parser::Parse<'_>>::parse(&mut &input[..]);
			println!("{save:#?}");
		}
	}
	else {
		help(&mut std::io::stderr());
		std::process::exit(1);
	}
}
