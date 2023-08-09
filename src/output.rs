//! Handles printed output.

use std::{
    path::PathBuf,
    io::{self, Write},
    fmt::Display,
};

use ansi_term::{Color, ANSIGenericString, Style};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DASH: ANSIGenericString<'static, str> = Color::Blue.bold().paint("-");
    pub static ref RED_DASH: ANSIGenericString<'static, str> = Color::Red.bold().paint("-");
}

pub fn divider(message: &str) {
    println!("{}", Color::Black.paint(format!("[ ] : {message}")));
}

pub fn info(message: &str) {
    println!("{}", Color::Blue.bold().paint(format!("[i] : {message}")));
}

pub fn warn(message: &str) {
    println!("{}", Color::Yellow.bold().paint(format!("[!] : {message}")));
}

pub fn announce(message: &str, item: &str) {
    divider(&format!("{message} {}...", Color::Green.paint(item)));
}

pub fn announce_path(message: &str, path: &PathBuf) {
    announce(message, path.to_str().unwrap());
    //divider(&format!("{message} {}...", Color::Green.paint(path.to_str().unwrap())));
}

pub fn update_progress(of_what: &str, count: u32) -> io::Result<()> {
    let mut lock = io::stdout().lock();
    write!(lock, "\rFound {} {of_what}...", Color::Yellow.bold().paint(count.to_string()))
}

pub fn bullet_list<T, U>(bullet: T, items: &mut U)
where
    T: Display,
    U: Iterator,
    <U as Iterator>::Item: Display,
{
    for item in items {
        println!("{bullet} {item}");
    }
}

pub fn style<T>(style: &Style, what: &T) -> ANSIGenericString<'static, str>
where
    T: Display,
{
    style.paint(what.to_string())
}
