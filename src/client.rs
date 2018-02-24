use brain::Brain;
use slack::{Event, EventHandler, RtmClient};
use slack::api as slack_api;
use regex::Regex;
use std::fs::OpenOptions;
use std::process;
use std::io::prelude::*;
use get_sentances;

pub struct LadaClient {
    name: String,
    id: String,
    dump_file: String,
    seed_file: String,
    brain: Brain,
}

impl LadaClient {
    pub fn new(brain: Brain, dump_file: &str, seed_file: &str) -> LadaClient {
        LadaClient {
            name: "".to_owned(),
            id: "".to_owned(),
            dump_file: dump_file.to_owned(),
            seed_file: seed_file.to_owned(),
            brain: brain
        }
    }
}

impl EventHandler for LadaClient {
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
                            self.brain.save(&self.dump_file);

                            if !text.is_empty() {
                                match OpenOptions::new().append(true).open(&self.seed_file).as_mut() {
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
