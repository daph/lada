extern crate lada;
extern crate slack;
#[macro_use]
extern crate clap;

use lada::get_sentances;
use lada::brain::Brain;
use lada::client::LadaClient;
use slack::RtmClient;
use std::fs::File;
use std::path::Path;
use std::time::{Duration, Instant};
use std::thread;
use std::io::BufReader;
use std::io::prelude::*;
use clap::{App, Arg};

fn main() {
    let matches = App::new("Lada: A Slack Markov Bot")
        .version(crate_version!())
        .author("David Phillips")
        .about("A little rust program to provide some markov-based hilarity in our otherwise sad lives")
        .arg(Arg::with_name("token")
             .short("t")
             .long("token")
             .help("The slack bot token to connect with")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("seed-file")
             .short("s")
             .long("seed-file")
             .help("The text file to seed the brain with")
             .takes_value(true)
             .default_value("seed.txt"))
        .arg(Arg::with_name("brain-dump")
             .short("b")
             .long("brain-dump")
             .help("The file to dump out the brain for safe keeping")
             .takes_value(true)
             .default_value("brain.dump"))
        .get_matches();

    let api_key = matches.value_of("token").unwrap();
    let seed_file = matches.value_of("seed-file").unwrap();
    let brain_dump = matches.value_of("brain-dump").unwrap();

    let mut brain = Brain::new();

    let instant = Instant::now();
    if Path::new(brain_dump).exists() {
        brain.load(brain_dump);
        let duration  = instant.elapsed();
        let elapsed_secs = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        eprintln!("Took {} seconds to load brain dump", elapsed_secs);
    } else {
        let mut contents = String::new();
        {
            let mut f = BufReader::new(File::open(seed_file).expect("File not found"));
            f.read_to_string(&mut contents).expect("Error reading file");
        }

        for s in get_sentances(&contents) {
            brain.learn(s.trim());
        }

        let duration  = instant.elapsed();
        let elapsed_secs = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        eprintln!("Took {} seconds to load seed text and learn", elapsed_secs);
        brain.save(brain_dump);
    }

    let mut client = LadaClient::new(brain, brain_dump, seed_file);

    loop {
        let r = RtmClient::login_and_run(&api_key, &mut client);
        match r {
            Ok(_) => {},
            Err(err) => {
                eprintln!("Slack error: {:?}", err);
                thread::sleep(Duration::from_millis(2000));
                eprintln!("Retrying...");
            },
        }
    }
}

