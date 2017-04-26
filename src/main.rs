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


#[derive(Debug, Clone, Copy)]
struct ProcessIterator<'a> {
    snapshot: &'a Snapshot,
    index: usize,
}

impl<'a> Iterator for ProcessIterator<'a> {
    type Item = Process<'a>;

    fn next(&mut self) -> Option<Process<'a>> {
        if self.index < self.snapshot.nrows {
            let i = self.index;
            self.index += 1;

            Some(Process {
                snapshot: self.snapshot,
                index: i,
            })
        }
        else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Process<'a> {
    snapshot: &'a Snapshot,
    index: usize,
}

impl<'a> Process<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        if let Some(v) = self.snapshot.table.get(key) {
            Some(&v[self.index])
        }
        else {
            None
        }
    }
}

#[derive(Debug)]
struct Snapshot {
    time: String,
    nrows: usize,
    table: HashMap<String, Vec<String>>,
}

impl Snapshot {
    fn iter(&self) -> ProcessIterator {
        ProcessIterator {
            snapshot: self,
            index: 0,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Query {
    PID(u32),
    Command(String),
}

impl Query {
    fn is_match(&self, p: Process) -> bool {
        match self {
            &Query::PID(pid)             => p.get("PID").unwrap().parse::<u32>().unwrap() == pid,
            &Query::Command(ref command) => &p.get("COMMAND").unwrap() == command,
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
    let mut table: HashMap<String, Vec<String>> = HashMap::new();
    for col_name in &col_names {
        table.entry(col_name.to_owned()).or_insert(Vec::new());
    }
    let mut nrows = 0;
    while reader.read_line(&mut buf).unwrap() > 0 {
        let line = buf.trim().to_owned();
        buf.clear();

        if line.len() == 0 {
            break;
        }

        let values: Vec<String> = WHITESPACES.splitn(&line, col_names.len()).map(|x| x.to_owned()).collect();

        if values.len() == col_names.len() {
            for (col_name, value) in col_names.iter().zip(values) {
                let col = table.get_mut(col_name).unwrap();
                col.push(value);
            }
            nrows += 1;
        }
    }

    let snapshot = Snapshot {
        time: time_str,
        nrows: nrows,
        table: table,
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
                current_time = snapshot.time.clone();
                records.clear();
            }
            for query in &queries {
                let mut sum: f64 = 0.0;
                for p in snapshot.iter() {
                    if query.is_match(p) {
                        sum += p.get("%CPU").unwrap().parse::<f64>().unwrap();
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
                for p in snapshot.iter() {
                    if query.is_match(p) {
                        sum += p.get("%CPU").unwrap().parse::<f64>().unwrap();
                    }
                }
                println!("{}\t{}\t{}", snapshot.time, query, sum);
            }
        }
    }
}
