extern crate lada;
extern crate slack;
extern crate regex;

use lada::Brain;
use slack::{Event, RtmClient};
use slack::api as slack_api;
use regex::Regex;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::process;
use std::env;
use std::time::{Duration, Instant};
use std::thread;
use std::io::prelude::*;

static SEED_FILE: &'static str = "corpus.txt";
static BRAIN_DUMP: &'static str = "brain.dump";

struct LadaClient {
    name: String,
    id: String,
    brain: lada::Brain,
}

impl slack::EventHandler for LadaClient {
    fn on_event(&mut self, cli: &RtmClient, event: Event) {
        if let Event::Message(msg) = event {
            match *msg {
                slack_api::Message::Standard(my_msg) => {
                    let user = my_msg.user.unwrap_or("".to_owned());
                    let text = my_msg.text.unwrap_or("".to_owned());
                    let channel = my_msg.channel.unwrap_or("".to_owned());

                    let re_name = Regex::new(&format!("(?i){}", self.name)).unwrap();
                    let re_id = Regex::new(&format!("(?i){}", self.id)).unwrap();

                    if user != self.id && re_name.is_match(&text) || re_id.is_match(&text) {
                        if text.contains("getget10") {
                            for _ in 0..10 {
                                let _ = cli.sender()
                                    .send_message(&channel, &self.brain.make_sentance(300, ""));
                            }
                        } else {
                            let text = re_name.replace_all(&re_id.replace_all(&text, ""), "")
                                .split_whitespace()
                                .collect::<Vec<&str>>()
                                .join(" ")
                                .to_owned();
                            let _ = cli.sender().send_message(&channel, &self.brain.make_sentance(300, &text));
                            for s in get_sentances(&text) {
                                self.brain.learn(s);
                            }
                            self.brain.save(BRAIN_DUMP);

                            if !text.is_empty() {
                                match OpenOptions::new().append(true).open(SEED_FILE).as_mut() {
                                    Ok(f) => {
                                        writeln!(f, "{}", &text.trim()).unwrap_or_else(|e| {
                                            eprintln!("Couldn't appened to seed file: {:?}", e);
                                        });
                                    },
                                    Err(e) => eprintln!("Couldn't open seed file as appened {:?}", e)
                                }
                            }
                        }
                    }
                },

                _ => ()
            }
        }
    }

    fn on_close(&mut self, _cli: &RtmClient) {
        eprintln!("Slack channel closed!");
        process::exit(1);
    }

    fn on_connect(&mut self, cli: &RtmClient) {
        if let Some(slf) = cli.start_response().slf.as_ref() {
            let slf = slf.clone();
            self.name = slf.name.expect("No username found in start response!");
            self.id = slf.id.expect("No user id found in start response!");
            self.id = format!("<@{}>", self.id);
        }
    }
}

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

    let mut client = LadaClient { name: "".to_owned(), id: "".to_owned(), brain: brain };

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

fn get_sentances(contents: &str) -> Vec<&str> {
    let mut sentances = Vec::new();

    for s in contents.split_terminator(|t| { t == '.' || t == '?' || t == '!' }) {
        sentances.push(s.trim());
    }

    sentances
}
