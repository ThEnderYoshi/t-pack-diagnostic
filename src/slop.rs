//! Implementation of Sans' Lovely prOPerties.

use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader},
    path::Path,
    fs::File,
};

use lazy_static::lazy_static;
use regex::Regex;

pub enum SlopValue {
    String(String),
    List(Vec<String>),
}

pub struct Slop {
    items: HashMap<String, SlopValue>,
}

impl Slop {
    pub fn new() -> Self {
        Self { items: HashMap::new() }
    }

    pub fn from_map(items: HashMap<String, SlopValue>) -> Self {
        Self { items }
    }

    pub fn from_lines(lines: Vec<String>) -> Self {
        let mut slop = Self::new();
        slop.parse_lines(lines);
        slop
    }

    pub fn from_file<P>(file_path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(Self::from_lines(read_lines(file_path)?))
    }

    pub fn serialize(&self) -> String {
        let mut result = String::new();

        for (key, value) in self.items.iter() {
            match value {
                SlopValue::String(string)
                    => result.push_str(&format!("{key}={string}\n")),
                SlopValue::List(list)
                    => result.push_str(&format!(
                        "{key}{{\n    {}\n}}\n",
                        list.join("\n    "),
                    )),
            }
        }

        result
    }

    // Commented out while unused.
    //pub fn contains_key(&self, key: &str) -> bool {
    //    self.items.contains_key(key)
    //}

    pub fn get(&self, key: &str) -> Option<&SlopValue> {
        let key = String::from(key);
        self.items.get(&key)
    }

    pub fn get_list(&self, key: &str) -> Option<&Vec<String>> {
        if let Some(SlopValue::List(list)) = self.get(key) {
            Some(list)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: &str, value: SlopValue) {
        self.items.insert(String::from(key), value);
    }

    pub fn insert_str(&mut self, key: &str, value: &str) {
        self.insert(key, SlopValue::String(String::from(value)));
    }

    pub fn parse_lines(&mut self, lines: Vec<String>) {
        let mut skip_lines: usize = 0;

        for i in 0..lines.len() {
            if skip_lines > 0 {
                skip_lines -= 1;
                continue;
            }

            let line = clean_up_line(&lines[i]);
            if line.is_empty() || line.chars().next() == Some('#') {
                continue;
            }

            if let Some((key, value)) = parse_string_kv(&line) {
                self.items.insert(key, value);
            } else if let Some((key, value, skip)) = parse_list_kv(&lines, i) {
                self.items.insert(key, value);
                skip_lines = skip;
            } else {
                panic!("Malformed SLOP: Line {} is invalid: `{line}`", i + 1);
            }
        }
    }
}

fn clean_up_line(line: &str) -> String {
    String::from(line.trim_start())
}

fn parse_string_kv(line: &str) -> Option<(String, SlopValue)> {
    lazy_static! {
        static ref RE_STRING_KV: Regex = Regex::new(r"^([^=\}]*)=(.*)$").unwrap();
    }

    if RE_STRING_KV.is_match(line) {
        let captures = RE_STRING_KV.captures(line).unwrap();
        Some((
            String::from(&captures[1].to_owned()),
            SlopValue::String(captures[2].to_owned()),
        ))
    } else {
        None
    }
}

fn parse_list_kv(lines: &Vec<String>, start_index: usize) -> Option<(String, SlopValue, usize)> {
    lazy_static! {
        static ref RE_LIST_KV_START: Regex = Regex::new(r"^([^=\}]*)\{\s*$").unwrap();
        static ref RE_LIST_KV_END: Regex = Regex::new(r"^\}\s*$").unwrap();
    }
    let start_line = &lines[start_index];
    if RE_LIST_KV_START.is_match(start_line) {
        let mut values: Vec<String> = vec![];

        for i in start_index..lines.len() {
            let line = clean_up_line(&lines[i]);
            if RE_LIST_KV_END.is_match(&line) {
                let captures = RE_LIST_KV_START.captures(start_line).unwrap();
                return Some((
                    String::from(&captures[1]),
                    SlopValue::List(values),
                    i - start_index,
                ));
            }
            values.push(line);
        }
    }
    None
}

fn read_lines<P>(file_path: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    BufReader::new(File::open(file_path)?).lines().collect()
}
