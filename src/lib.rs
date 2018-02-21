extern crate rand;
extern crate rayon;

use std::fmt;
use std::collections::HashMap;
use rand::{thread_rng, Rng};
use rayon::prelude::*;

#[derive(Debug)]
pub struct Brain {
    brain_map: HashMap<(String, String), Vec<String>>
}

impl Brain {
    pub fn new() -> Brain {
        Brain { brain_map: HashMap::new() }
    }

    pub fn learn(&mut self, new_sentance: &str) {
        let mut sentance = "<START> ".to_owned();
        sentance.push_str(new_sentance.trim());
        let w1list = sentance.split_whitespace();
        let w2list = sentance.split_whitespace().skip(1);
        let mut tuples = w1list.zip(w2list).peekable();

        while let Some((w1, w2)) = tuples.next() {
            let w1 = w1.to_owned();
            let w2 = w2.to_owned();
            match tuples.peek() {
                Some(&(_, w3)) => {
                    let words = self.brain_map.entry((w1, w2)).or_insert(vec![]);
                    words.push(w3.to_owned());
                }
                None => {
                    let w3 = vec!["<STOP>".to_owned()];
                    self.brain_map.insert((w1, w2), w3);
                }
            }
        }
    }

    fn make_sentance(&self, max_length: u32) -> String {
        let mut rng = thread_rng();
        let starts = self.get_starts();
        let pick = rng.gen_range(0, starts.len());
        let mut word_tuple = starts[pick].clone();

        let mut sentance = String::new();
        sentance.push_str(&word_tuple.1);

        for _ in 0..max_length {
            if let Some(next) = self.brain_map.get(&word_tuple) {
                let pick = rng.gen_range(0, next.len());

                if next[pick].contains("<STOP>") {
                    break;
                }
                sentance.push_str(" ");
                sentance.push_str(&next[pick]);

                word_tuple = (word_tuple.1, next[pick].to_owned());
            }
            else {
                break;
            }
        }

        sentance
    }

    fn get_starts(&self) -> Vec<&(String, String)> {
        self.brain_map
            .par_iter()
            .filter_map(|(k, _)| {
                if k.0 == "<START>" {
                    Some(k)
                } else {
                    None
                }}).collect()
    }
}

impl fmt::Display for Brain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.make_sentance(300))
    }
}
