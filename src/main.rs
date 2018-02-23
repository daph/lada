extern crate lada;
extern crate slack;

use lada::get_sentances;
use lada::brain::Brain;
use lada::client::LadaClient;
use slack::RtmClient;
use std::fs::File;
use std::path::Path;
use std::env;
use std::time::{Duration, Instant};
use std::thread;
use std::io::prelude::*;

static SEED_FILE: &'static str = "corpus.txt";
static BRAIN_DUMP: &'static str = "brain.dump";

fn main() {
    let args: Vec<String> = env::args().collect();
    let api_key = match args.len() {
        0 | 1 => panic!("No api-key in args! Usage: cargo run -- <api-key>"),
        x => args[x - 1].clone(),
    };

    let mut brain = Brain::new();

    if Path::new(BRAIN_DUMP).exists() {
        brain.load(BRAIN_DUMP);
    } else {
        let instant = Instant::now();
        let mut contents = String::new();
        {
            let mut f = File::open(SEED_FILE).expect("File not found");
            f.read_to_string(&mut contents).expect("derp");
        }

        for s in get_sentances(&contents) {
            brain.learn(s.trim());
        }

        let duration  = instant.elapsed();
        let elapsed_secs = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        eprintln!("Took {} seconds to load file and learn", elapsed_secs);
        brain.save(BRAIN_DUMP);
    }

    let mut client = LadaClient {
        name: "".to_owned(),
        id: "".to_owned(),
        seed_file: SEED_FILE.to_owned(),
        dump_file: BRAIN_DUMP.to_owned(),
        brain: brain,
    };

    let mut retries = 3;
    loop {
        let r = RtmClient::login_and_run(&api_key, &mut client);
        match r {
            Ok(_) => {},
            Err(err) => {
                if retries <= 0 {
                    panic!("No more retries left!")
                }
                eprintln!("Slack error: {:?}", err);
                retries -= 1;
                thread::sleep(Duration::from_millis(2000));
                eprintln!("Retrying... (retries left: {})", retries);
            },
        }
    }
}

