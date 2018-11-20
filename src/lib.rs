extern crate rand;
extern crate bincode;
extern crate slack;
extern crate regex;
extern crate twox_hash;
#[macro_use] extern crate serde_derive;


pub mod brain;
pub mod client;

pub fn get_sentences(contents: &str) -> Vec<&str> {
    let mut sentences = Vec::new();

    for s in contents.split_terminator(|t| { t == '.' || t == '?' || t == '!' }) {
        sentences.push(s.trim());
    }

    sentences
}
