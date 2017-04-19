#[macro_use]
extern crate lazy_static;
extern crate regex;


use std::collections::HashMap;
use std::io::prelude::*;
use std::io::{self};
use std::vec::Vec;

use regex::Regex;


#[derive(Debug)]
struct Snapshot {
    time: String,
    processes: Vec<HashMap<String, String>>,
}

fn read_snapshot<R: BufRead>(mut reader: R) -> io::Result<Option<Snapshot>> {
    lazy_static! {
        static ref TOP_START: Regex = Regex::new(r"^top - (.+?) up").unwrap();
        static ref whitespaces: Regex = Regex::new(r"\s+").unwrap();
    }

    let mut buf = String::new();

    // Skip to the start of a block of top's output
    let time_str = {
        let mut x = None;
        while reader.read_line(&mut buf)? > 0 {
            let line = buf.trim().to_owned();
            buf.clear();

            if let Some(caps) = TOP_START.captures(&line) {
                x = Some(caps.get(1).unwrap().as_str().to_owned());
                break;
            }
        }
        if let Some(time_str) = x {
            time_str
        }
        else {
            return Ok(None);
        }
    };

    // Skip to a blank line
    while reader.read_line(&mut buf)? > 0 {
        let line = buf.trim().to_owned();
        buf.clear();

        if line.len() == 0 {
            break;
        }
    }

    // Read a header line of a process list
    reader.read_line(&mut buf).unwrap();
    let line = buf.trim().to_owned();
    buf.clear();
    let col_names: Vec<String> = line.split_whitespace().map(|x| x.to_owned()).collect();

    // Read the process list
    let mut processes: Vec<HashMap<String, String>> = Vec::new();
    while reader.read_line(&mut buf).unwrap() > 0 {
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
    return Ok(Some(snapshot));
}

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    while let Ok(Some(snapshot)) = read_snapshot(&mut stdin) {
        println!("{}", snapshot.time);
    }
}
