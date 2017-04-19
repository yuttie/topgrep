extern crate regex;


use std::io::prelude::*;
use std::io::{self};
use regex::Regex;
use std::collections::HashMap;
use std::vec::Vec;


struct Snapshot {
    time: String,
    processes: Vec<HashMap<String, String>>,
}

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buf = String::new();
    let re = Regex::new(r"^top - (.+?) up").unwrap();
    let whitespaces = Regex::new(r"\s+").unwrap();
    while stdin.read_line(&mut buf).unwrap() > 0 {
        let line = buf.trim().to_owned();
        buf.clear();

        if let Some(caps) = re.captures(&line) {
            let time_str = caps.get(1).unwrap().as_str();
            println!("{}", time_str);

            loop {
                stdin.read_line(&mut buf).unwrap();
                let line = buf.trim().to_owned();
                buf.clear();

                if line == "" {
                    break;
                }
            }

            stdin.read_line(&mut buf).unwrap();
            let line = buf.trim().to_owned();
            buf.clear();
            let col_names: Vec<String> = line.split_whitespace().map(|x| x.to_owned()).collect();

            let mut processes: Vec<HashMap<String, String>> = Vec::new();
            while stdin.read_line(&mut buf).unwrap() > 0 {
                let line = buf.trim().to_owned();
                buf.clear();

                if line.len() == 0 {
                    break;
                }

                let values: Vec<String> = whitespaces.splitn(&line, col_names.len()).map(|x| x.to_owned()).collect();

                let process: HashMap<String, String> = col_names.iter().cloned().zip(values).collect();
                processes.push(process);
            }
            let snapshot = Snapshot {
                time: time_str.to_owned(),
                processes: processes,
            };
        }
    }
}
