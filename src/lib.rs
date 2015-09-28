extern crate time;
extern crate regex;
extern crate rustc_serialize;

use regex::Regex;
use rustc_serialize::json::{self, ToJson, Json};

use std::io::{self, Read, Write};
use std::fs::File;
use std::collections::BTreeMap;

enum OutputStream {
    File(File),
    Stdout(io::Stdout),
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
    msg_total: u64,
    media_total: u64,
    title_changes: BTreeMap<String, Vec<String>>,
    starts_most_chats: BTreeMap<String, u64>,
    msg_per_person: BTreeMap<String, u64>,
    media_per_person: BTreeMap<String, u64>,
    avg_per_person: BTreeMap<String, u64>,
    chr_per_person: BTreeMap<String, u64>,
    msg_per_day: BTreeMap<String, u64>,
    most_used_word: BTreeMap<String, u64>,
}

struct JsonTime(time::Tm);

impl ToJson for JsonTime {
    fn to_json(&self) -> Json {
        Json::String(time::strftime("%FT%TZ", &self.0).ok().unwrap())
    }
}

pub fn preprocess(input: &str) -> Result<(), io::Error> {
    let mut f = try!(File::open(input));
    let mut o = try!(File::create("tmp"));

    let mut s = String::new();
    try!(f.read_to_string(&mut s));

    let re = Regex::new(r"\n+(\D)").unwrap();
    try!(o.write_all(&re.replace_all(&s, " $1").into_bytes()));
    Ok(())
}

pub fn analyse(input: &str, out: Option<String>) -> Result<String, io::Error> {
    // Lees de input in een string s
    let mut f = try!(File::open(input));
	let mut s = String::new();
    try!(f.read_to_string(&mut s));

    // De regex die gebruikt wordt om alle benodigde informatie te
    // extraheren
    let berichtreg = Regex::new(r"(?P<datum>\d{4}/\d{2}/\d{2}, \d{2}:\d{2}) - (?:(?P<naam>[\w ]+): (?P<bericht>.*)|(?P<naamspeciaal>.+)(?P<speciaal> changed.+)|(?P<naambeheerder>.+) added (?P<naamjoin>.+)|(?P<naamleft>.+) left)").unwrap();
    let woordreg = Regex::new(r"(\b[^\s]+\b)").unwrap();
    let mediareg = Regex::new(r"<Media omitted>").unwrap();
    let titelreg = Regex::new(r"^changed the subject to “(.+)”").unwrap();

    // De variabeles voor de informatie die moet worden verzameld
	let mut lines = 0;
    let mut berichten = 0;
    let mut media = 0;
    let mut eerste_tijd = None;
    let mut laatste_tijd = None;
    let mut analyse_per_naam: BTreeMap<String, Json> = BTreeMap::new();
    let mut tekens_per_naam: BTreeMap<String, u64> = BTreeMap::new();
    let mut berichten_per_naam: BTreeMap<String, u64> = BTreeMap::new();
    let mut media_per_naam: BTreeMap<String, u64> = BTreeMap::new();
    let mut meest_gebruikte_woorden: BTreeMap<String, u64> = BTreeMap::new();
    let mut berichten_per_dag: BTreeMap<String, u64> = BTreeMap::new();
    let mut begint_meeste_gesprekken: BTreeMap<String, u64> = BTreeMap::new(); //Detecteerd niet daadwerkelijk gesprekken, maar kijkt wie elke dag het gesprek begint.
    let mut titel_verandering: BTreeMap<String, Vec<String>> = BTreeMap::new();

    // Regex de input string s en iterate over de bericht matches
	for cap in berichtreg.captures_iter(&s) {
        // De informatie uit de berichten
        let naam = cap.name("naam").unwrap_or("");
        let bericht = cap.name("bericht").unwrap_or("");
        let naamspeciaal = cap.name("naamspeciaal").unwrap_or("");
        let speciaal = cap.name("speciaal").unwrap_or("");
        let datum = cap.name("datum").unwrap_or("");
        let dag = JsonTime(time::strptime(datum, "%Y/%m/%d").ok().unwrap()).to_json().to_string();

        if mediareg.is_match(bericht) {
            media += 1;
        } else {
            berichten += 1;
        }

        if speciaal != "" {
            if let Some(captures) = titelreg.captures(&speciaal) {
                titel_verandering.insert(dag.to_string(), vec![naamspeciaal.to_string(), captures.at(1).unwrap().to_string()]);
            }
        }

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
        } else if naam != "" && bericht != "" {
            berichten_per_naam.insert(naam.to_string(), 1);
        }

        if media_per_naam.contains_key(naam) {
            if let Some(x) = media_per_naam.get_mut(naam) {
                if mediareg.is_match(bericht){
                    *x += 1;
                }
            }
        } else if naam != "" && mediareg.is_match(bericht) {
            media_per_naam.insert(naam.to_string(), 1);
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
        msg_total: berichten,
        media_total: media,
        title_changes: titel_verandering,
        starts_most_chats: begint_meeste_gesprekken,
        msg_per_person: berichten_per_naam,
        media_per_person: media_per_naam,
        avg_per_person: gemiddelde_per_naam,
        chr_per_person: tekens_per_naam,
        msg_per_day: berichten_per_dag,
        most_used_word: meest_gebruikte_woorden2,
    };

    let o = Output {
        lines: lines,
        start: start_tijd.to_json(),
        end: eind_tijd.to_json(),
        analysis: a,
    };

    let mut output = match out {
        Some(ref x) => OutputStream::File((File::open(x).ok().unwrap())),
        None => OutputStream::Stdout(io::stdout()),
    };

    match output {
	   OutputStream::File(ref mut x) => write!(x, "{}", json::encode(&o).unwrap()).unwrap(),
       OutputStream::Stdout(ref mut x) => write!(x, "{}", json::encode(&o).unwrap()).unwrap(),
    }
	
    Ok(out.unwrap_or("stdout".to_string()))
}