extern crate lada;
extern crate slack;

use lada::Brain;
use slack::{Event, RtmClient};
use slack::api as slack_api;
use std::fs::File;
use std::process;
use std::env;
use std::io::prelude::*;

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

                    if user != self.id && text.contains(&self.name) || text.contains(&self.id) {
                        let _ = cli.sender().send_message(&channel, &self.brain.to_string());

                        let text = text.replace(&self.id, "").replace(&self.name, "");
                        for s in get_sentances(&text) {
                            self.brain.learn(s);
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

    let mut f = File::open("totse.txt").expect("File not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("derp");

    for s in get_sentances(&contents) {
        brain.learn(s.trim());
    }

    let mut client = LadaClient { name: "".to_owned(), id: "".to_owned(), brain: brain };

    let r = RtmClient::login_and_run(&api_key, &mut client);
    match r {
        Ok(_) => {}
        Err(err) => panic!("Error: {}", err),
    }
}

fn get_sentances(contents: &str) -> Vec<&str> {
    let mut sentances = Vec::new();

    for s in contents.split_terminator(|t| { t == '.' || t == '?' || t == '!' }) {
        sentances.push(s.trim());
    }

    sentances
}
