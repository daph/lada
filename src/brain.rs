use std::fmt;
use std::hash::BuildHasherDefault;
use std::collections::HashMap;
use twox_hash::XxHash;
use std::fs::OpenOptions;
use std::io::{BufWriter, BufReader};
use rand::{Rng, thread_rng};
use rand::distributions::Uniform;
use bincode::{serialize_into, deserialize_from};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Brain {
    brain_map: HashMap<(String, String), Vec<String>, BuildHasherDefault<XxHash>>
}

impl Brain {
    pub fn new() -> Brain {
        Brain { brain_map: Default::default() }
    }

    pub fn learn(&mut self, new_sentence: &str) {
        let mut sentence = "<START> ".to_owned();
        sentence.push_str(new_sentence.trim());
        let w1list = sentence.split_whitespace();
        let w2list = sentence.split_whitespace().skip(1);
        let mut tuples = w1list.zip(w2list).peekable();

        while let Some((w1, w2)) = tuples.next() {
            let w1 = w1.to_owned();
            let w2 = w2.to_owned();
            let words = self.brain_map.entry((w1, w2)).or_insert(vec![]);
            match tuples.peek() {
                Some(&(_, w3)) => {
                    words.push(w3.to_owned());
                }
                None => {
                    words.push("<STOP>".to_owned());
                }
            }
        }
    }

    pub fn make_sentence(&self, max_length: u32, context: &str) -> String {
        let mut rng = thread_rng();
        let mut sentence = String::new();
        let mut word_tuple = match self.process_context(context) {
            Some(ws) => {
                sentence.push_str(&ws.0);
                sentence.push_str(" ");
                sentence.push_str(&ws.1);
                ws
            }
            None => {
                let starts = self.get_starts();
                let range = Uniform::new(0, starts.len());
                let pick = rng.sample(range);
                let ws = starts[pick].clone();
                sentence.push_str(&ws.1);
                ws
            }
        };


        for _ in 0..max_length {
            if let Some(next) = self.brain_map.get(&word_tuple) {
                let range = Uniform::new(0, next.len());
                let pick = rng.sample(range);

                if next[pick].contains("<STOP>") {
                    break;
                }
                sentence.push_str(" ");
                sentence.push_str(&next[pick]);

                word_tuple = (word_tuple.1, next[pick].to_owned());
            }
            else {
                break;
            }
        }

        sentence
    }

    pub fn save(&self, file: &str) {
        let mut f = BufWriter::new(OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file)
            .unwrap());
        serialize_into(&mut f, &self.brain_map).unwrap();
    }

    pub fn load(&mut self, file: &str) {
        let mut f = BufReader::new(OpenOptions::new()
            .read(true)
            .open(file)
            .unwrap());
         self.brain_map = deserialize_from(&mut f).unwrap();
    }

    fn get_starts(&self) -> Vec<&(String, String)> {
        self.brain_map
            .iter()
            .filter_map(|(k, _)| {
                if k.0 == "<START>" {
                    Some(k)
                } else {
                    None
                }}).collect()
    }

    fn process_context(&self, context: &str) -> Option<(String, String)> {
        let seed = context.split_whitespace().take(2).collect::<Vec<&str>>();
        if seed.len() > 1 {
            let seed_tuple = (seed[0].to_owned(), seed[1].to_owned());
            match self.brain_map.get(&seed_tuple) {
                Some(_) => Some(seed_tuple),
                None => None
            }
        } else {
            None
        }
    }
}

impl fmt::Display for Brain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.make_sentence(300, ""))
    }
}
