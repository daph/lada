extern crate rand;
extern crate rayon;
extern crate bincode;
extern crate slack;
extern crate regex;
#[macro_use] extern crate serde_derive;


pub mod brain;
pub mod client;

pub fn get_sentances(contents: &str) -> Vec<&str> {
    let mut sentances = Vec::new();

    for s in contents.split_terminator(|t| { t == '.' || t == '?' || t == '!' }) {
        sentances.push(s.trim());
    }

    sentances
}
