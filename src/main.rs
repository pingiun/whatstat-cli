extern crate getopts;
extern crate time;
extern crate regex;
extern crate rustc_serialize;

//mod emoji;

use getopts::Options;
use regex::Regex;
use rustc_serialize::json::{self, ToJson, Json};

use std::env;
use std::io;
use std::io::{Read, Write};
use std::fs::File;
use std::collections::BTreeMap;

const VERSION: &'static str = "0.0.0";

macro_rules! printerr {
    ($($arg:tt)*) => (write!($crate::std::io::stderr(), $($arg)*));
}

macro_rules! printerrln {
    ($fmt:expr) => (printerr!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (printerr!(concat!($fmt, "\n"), $($arg)*));
}

#[derive(RustcEncodable)]
struct Output {
    lines: u64,
    start: Json,
    end: Json,
    analysis: Analysis,
}

#[derive(RustcEncodable)]
struct Analysis {
    msg_per_person: BTreeMap<String, u64>,
    avg_per_person: BTreeMap<String, u64>,
    chr_per_person: BTreeMap<String, u64>,
    most_used_word: BTreeMap<String, u64>,
    //most_used_emoji: BTreeMap<String, u64>,
    msg_per_day: BTreeMap<String, u64>,
}

struct JsonTime(time::Tm);

impl ToJson for JsonTime {
    fn to_json(&self) -> Json {
        Json::String(time::strftime("%FT%TZ", &self.0).ok().unwrap())
    }
}

fn preprocess(input: &str) -> Result<(), io::Error> {
    let mut f = try!(File::open(input));
    let mut o = try!(File::create("tmp"));

    let mut s = String::new();
    try!(f.read_to_string(&mut s));

    let re = Regex::new(r"\n+(\D)").unwrap();
    try!(o.write_all(&re.replace_all(&s, " $1").into_bytes()));
    Ok(())
}

fn analyse(input: &str, out: Option<String>) -> Result<(), io::Error> {
    // Lees de input in een string s
    let mut f = try!(File::open(input));
	let mut s = String::new();
    try!(f.read_to_string(&mut s));

    // De regex die gebruikt wordt om alle benodigde informatie te
    // extraheren
    let berichtreg = Regex::new(r"(?P<datum>\d{4}/\d{2}/\d{2}, \d{2}:\d{2}) - (?:(?P<naam>[\w ]+): (?P<bericht>.*)|(?P<speciaal>.*))").unwrap();
    let woordreg = Regex::new(r"(\b[^\s]+\b)").unwrap();
    //let emojireg = emoji::emojireg();

    // De variabeles voor de informatie die moet worden verzameld
	let mut lines = 0;
    let mut eerste_tijd = None;
    let mut laatste_tijd = None;
    let mut tekens_per_naam: BTreeMap<String, u64> = BTreeMap::new();
    let mut berichten_per_naam: BTreeMap<String, u64> = BTreeMap::new();
    let mut meest_gebruikte_woorden: BTreeMap<String, u64> = BTreeMap::new();
    //let mut meest_gebruikte_emoji: BTreeMap<String, u64> = BTreeMap::new();
    let mut berichten_per_dag: BTreeMap<String, u64> = BTreeMap::new();
    let mut begint_meeste_gesprekken: BTreeMap<String, u64> = BTreeMap::new();

    // Regex de input string s en iterate over de bericht matches
	for cap in berichtreg.captures_iter(&s) {
        // De informatie uit de berichten
        let naam = cap.name("naam").unwrap_or("");
        let bericht = cap.name("bericht").unwrap_or("");
        let datum = cap.name("datum").unwrap_or("");
        let dag = JsonTime(time::strptime(datum, "%Y/%m/%d").ok().unwrap()).to_json().to_string();

        for cap in woordreg.captures_iter(&bericht) {
            let woord = cap.at(1).unwrap_or("");
            if meest_gebruikte_woorden.contains_key(woord) {
                if let Some(x) = meest_gebruikte_woorden.get_mut(woord) {
                    *x += 1;
                }
            } else if naam != "" {
                meest_gebruikte_woorden.insert(woord.to_string(), 1);
            }
        }

        /*printerrln!("{:?}", emojireg.find(bericht));

        for cap in emojireg.captures_iter(&bericht) {
            let emoji = cap.at(0).unwrap_or("");
            printerrln!("{}", cap.len());
            if meest_gebruikte_emoji.contains_key(emoji) {
                if let Some(x) = meest_gebruikte_emoji.get_mut(emoji) {
                    *x += 1;
                }
            } else if emoji != "" {
                meest_gebruikte_emoji.insert(emoji.to_string(), 1);
            }
        }*/

        if eerste_tijd.is_none() {
            eerste_tijd = Some(time::strptime(datum, "%Y/%m/%d, %H:%M"));
        }

        if tekens_per_naam.contains_key(naam) {
            if let Some(x) = tekens_per_naam.get_mut(naam) {
                *x += bericht.len() as u64;
            }
        } else if naam != "" {
            tekens_per_naam.insert(naam.to_string(), bericht.len() as u64);
        }

        if berichten_per_dag.contains_key(&dag) {
            if let Some(x) = berichten_per_dag.get_mut(&dag) {
                *x += 1;
            }
        } else {
            if begint_meeste_gesprekken.contains_key(naam) {
                if let Some(x) = begint_meeste_gesprekken.get_mut(naam) {
                    *x += 1;
                }
            } else {
                begint_meeste_gesprekken.insert(naam.to_string(), 1);
            }
            berichten_per_dag.insert(dag, 1);
        }

        if berichten_per_naam.contains_key(naam) {
            if let Some(x) = berichten_per_naam.get_mut(naam) {
                *x += 1;
            }
        } else if naam != "" {
            berichten_per_naam.insert(naam.to_string(), 1);
        }
		lines += 1;
        laatste_tijd = Some(datum);
	}

    let start_tijd = JsonTime(eerste_tijd.unwrap().ok().unwrap());
    let eind_tijd = JsonTime(time::strptime(laatste_tijd.unwrap(), "%Y/%m/%d, %H:%M").ok().unwrap());

    let mut gemiddelde_per_naam: BTreeMap<String, u64> = BTreeMap::new();

    for (naam, x) in &tekens_per_naam {
        let berichten = berichten_per_naam.get(naam).unwrap();
        gemiddelde_per_naam.insert(naam.to_owned(), x / berichten);
    }

    let mut meest_gebruikte_woorden2: BTreeMap<String, u64> = BTreeMap::new();
    for (woord, x) in meest_gebruikte_woorden {
        if x > 10 {
            meest_gebruikte_woorden2.insert(woord, x);
        }
    }

    let a = Analysis {
        msg_per_person: berichten_per_naam,
        avg_per_person: gemiddelde_per_naam,
        chr_per_person: tekens_per_naam,
        most_used_word: meest_gebruikte_woorden2,
        msg_per_day: berichten_per_dag,
        //most_used_emoji: meest_gebruikte_emoji,
    };

    let o = Output {
        lines: lines,
        start: start_tijd.to_json(),
        end: eind_tijd.to_json(),
        analysis: a,
    };

	println!("{}", json::encode(&o).unwrap());

	Ok(())
}

fn print_version() {
	println!("Using version {}", VERSION);
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    //println!("{:?}", emoji::emojireg().find("😄😈😇"));
	let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("o", "", "set output file name", "NAME");
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
        preprocess(&input).ok();
        printerrln!("It took {} milliseconds for the preprocess to run", (time::now() - start).num_milliseconds());
    }
    {
        let start = time::now();
        analyse("tmp", output).ok();
        printerrln!("It took {} milliseconds for the analysis to run", (time::now() - start).num_milliseconds());
    }
}
