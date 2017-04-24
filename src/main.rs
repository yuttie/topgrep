#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate regex;


use std::fmt;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::{self};
use std::vec::Vec;

use clap::{Arg, App, AppSettings};
use regex::Regex;


type ProcessInfo = HashMap<String, String>;

#[derive(Debug)]
struct Snapshot {
    time: String,
    processes: Vec<ProcessInfo>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Query {
    PID(u32),
    Command(String),
}

impl Query {
    fn is_match(&self, p: &ProcessInfo) -> bool {
        match self {
            &Query::PID(pid)             => p["PID"].parse::<u32>().unwrap() == pid,
            &Query::Command(ref command) => &p["COMMAND"] == command,
        }
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Query::PID(pid)             => write!(f, "{}", pid),
            &Query::Command(ref command) => write!(f, "{}", command),
        }
    }
}

fn read_snapshot<R: BufRead>(mut reader: R) -> io::Result<Option<Snapshot>> {
    lazy_static! {
        static ref TOP_START: Regex = Regex::new(r"^top - (.+?) up").unwrap();
        static ref WHITESPACES: Regex = Regex::new(r"\s+").unwrap();
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
        let line_is_empty = buf.trim().len() == 0;
        buf.clear();

        if line_is_empty {
            break;
        }
    }

    // Read a header line of a process list
    reader.read_line(&mut buf).unwrap();
    let col_names: Vec<String> = buf.trim().split_whitespace().map(|x| x.to_owned()).collect();
    buf.clear();

    // Read the process list
    let mut processes: Vec<ProcessInfo> = Vec::new();
    while reader.read_line(&mut buf).unwrap() > 0 {
        let line = buf.trim().to_owned();
        buf.clear();

        if line.len() == 0 {
            break;
        }

        let values: Vec<String> = WHITESPACES.splitn(&line, col_names.len()).map(|x| x.to_owned()).collect();

        if values.len() == col_names.len() {
            let process: ProcessInfo = col_names.iter().cloned().zip(values).collect();
            processes.push(process);
        }
    }

    let snapshot = Snapshot {
        time: time_str.to_owned(),
        processes: processes,
    };
    return Ok(Some(snapshot));
}

fn main() {
    let matches = App::new("topgrep")
        .author("Yuta Taniguchi <yuta.taniguchi.y.t@gmail.com>")
        .arg(Arg::with_name("pid")
             .long("pid")
             .takes_value(true)
             .value_name("PID")
             .multiple(true)
             .number_of_values(1))
        .arg(Arg::with_name("command")
             .long("command")
             .takes_value(true)
             .value_name("COMMAND")
             .multiple(true)
             .number_of_values(1))
        .arg(Arg::with_name("fold")
             .long("fold"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut queries: Vec<Query> = Vec::new();
    for pid in values_t!(matches, "pid", u32).unwrap_or(Vec::new()) {
        queries.push(Query::PID(pid));
    }
    for command in values_t!(matches, "command", String).unwrap_or(Vec::new()) {
        queries.push(Query::Command(command));
    }
    if matches.is_present("fold") {
        let mut records: HashMap<Query, (usize, f64)> = HashMap::new();
        let mut current_time: String = String::new();
        while let Ok(Some(snapshot)) = read_snapshot(&mut stdin) {
            if snapshot.time != current_time {
                for (query, &(n, sum)) in records.iter() {
                    println!("{}\t{}\t{}", current_time, query, sum / n as f64);
                }
                current_time = snapshot.time;
                records.clear();
            }
            for query in &queries {
                let mut sum: f64 = 0.0;
                for p in &snapshot.processes {
                    if query.is_match(p) {
                        sum += p["%CPU"].parse::<f64>().unwrap();
                    }
                }
                let accum = records.entry(query.clone()).or_insert((0, 0.0));
                accum.0 += 1;
                accum.1 += sum;
            }
        }
    }
    else {
        while let Ok(Some(snapshot)) = read_snapshot(&mut stdin) {
            for query in &queries {
                let mut sum: f64 = 0.0;
                for p in &snapshot.processes {
                    if query.is_match(p) {
                        sum += p["%CPU"].parse::<f64>().unwrap();
                    }
                }
                println!("{}\t{}\t{}", snapshot.time, query, sum);
            }
        }
    }
}
