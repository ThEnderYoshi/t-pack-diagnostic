//! Handles the printed output.

use std::{path::PathBuf, io::{StdoutLock, self, Write}, fmt::Display};

use ansi_term::{Color, Style, ANSIGenericString};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DASH: ANSIGenericString<'static, str> = Color::Blue.bold().paint("-");
    pub static ref RED_DASH: ANSIGenericString<'static, str> = Color::Red.bold().paint("-");
}

#[inline]
pub fn divider(message: &str) {
    println!("{}", Color::Black.paint(format!("[ ] : {message}")));
}

#[inline]
pub fn info(message: &str) {
    println!("{}", Color::Blue.bold().paint(format!("[i] : {message}")));
}

#[inline]
pub fn warn(message: &str) {
    println!("{}", Color::Yellow.bold().paint(format!("[!] : {message}")));
}

#[inline]
pub fn announce(message: &str, item: &str) {
    divider(&format!("{message} {}...", Color::Green.paint(item)));
}

#[inline]
pub fn announce_path(message: &str, path: &PathBuf) {
    announce(message, path.to_str().expect("expected path to be a valid utf-8 str"));
}

#[inline]
pub fn update_progress<'a>(lock: &mut StdoutLock<'a>, what: &str, count: u32) -> io::Result<()> {
    lazy_static! {
        static ref YELLOW: Style = Color::Yellow.bold();
    }

    write!(lock, "\rFound {} {what}...", YELLOW.paint(count.to_string()))
}

/// Prints the items of the provided [Iterator] on separate lines, each preceded
/// by `bullet`.
#[inline]
pub fn bullet_list<B, I>(bullet: B, items: I)
where
    B: Display,
    I: Iterator,
    <I as Iterator>::Item: Display,
{
    for item in items {
        println!("{bullet} {item}");
    }
}
