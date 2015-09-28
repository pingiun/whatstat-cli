extern crate getopts;
extern crate time;
extern crate regex;
extern crate rustc_serialize;

use getopts::Options;

use std::env;
use std::io::Write;
use std::fs;

#[macro_use]
mod utils;
mod lib;

const VERSION: &'static str = "0.1.1";

fn print_version() {
	println!("Using version {}", VERSION);
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
	let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("o", "", "set output file name", "FILE");
    opts.optflag("h", "help", "print this help menu");
	opts.optflag("v", "version", "print the version and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
	if matches.opt_present("v") {
		print_version();
		return;
	}
    let output = matches.opt_str("o");
    let input = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    {
        let start = time::now();
        lib::preprocess(&input).ok();
        printerrln!("It took {} milliseconds for the preprocess to run", (time::now() - start).num_milliseconds()).unwrap();
    }
    {
        let start = time::now();
        lib::analyse("tmp", output).ok();
        fs::remove_file("tmp").ok();
        printerrln!("It took {} milliseconds for the analysis to run", (time::now() - start).num_milliseconds()).unwrap();
    }
}
