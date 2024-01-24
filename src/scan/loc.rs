use std::{cmp::Ordering, collections::HashSet, error::Error, fmt::Display, fs, path::PathBuf};

use lazy_static::lazy_static;
use regex::Regex;
use walkdir::WalkDir;

use crate::{
    output::{self, DASH, RED_DASH},
    paths,
    scanner::{ItemStatus, Scanner},
    static_file_data::LOC_REF_NAME,
};

use super::MSG_BAD_REF_DIR;

lazy_static! {
    /// ## Captures
    ///
    /// - `1`: Locale tag
    /// - `2`: File extension
    pub static ref RE_LOC_FILE_NAME: Regex = Regex::new(
        r"^(en-US|de-DE|it-IT|fr-FR|es-ES|ru-RU|zh-Hans|pt-BR|pl-PL)-.*\.(csv|json)$",
    ).unwrap();
}

/// The possible types of localization files.
enum LocFileType {
    Csv,
    Json,
}

impl LocFileType {
    /// Returns the [LocFileType] of the file name, or [None] if the type
    /// is invalid.
    fn from_file_name(file_name: &str) -> Option<Self> {
        //TODO: Handle different locales.

        let caps = RE_LOC_FILE_NAME.captures(file_name)?;

        match &caps[2] {
            "csv" => Some(Self::Csv),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

enum InvalidEntry {
    /// Returned if an empty record was found.
    /// Holds the file name.
    EmptyRecord(String),

    /// Returned if the key was not found in the reference file.
    /// Holds the file name and the key itself.
    BadKey(String, String),
}

impl Display for InvalidEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyRecord(p) => {
                write!(f, "{p}\t: Empty record.")
            }
            Self::BadKey(p, k) => write!(
                f,
                "{p}\t: The key `{k}` was not found in the reference file.",
            ),
        }
    }
}

pub fn scan_localization_files(loc_dir: &PathBuf, ref_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let reference: HashSet<String> = fs::read_to_string(paths::push(ref_dir, LOC_REF_NAME))
        .expect(MSG_BAD_REF_DIR)
        .lines()
        .map(|l| l.to_string())
        .collect();

    let mut scanner = Scanner::new("entries");
    let mut invalid_file_names = vec![];

    println!();
    output::announce_path("Scanning", loc_dir);

    for entry in WalkDir::new(loc_dir).max_depth(1) {
        let entry = entry?;
        let path = entry.path().to_path_buf();

        if path.is_dir() {
            continue;
        }

        let file_name = paths::file_name(&path);

        match LocFileType::from_file_name(file_name) {
            None => {
                invalid_file_names.push(file_name.to_string());
                continue;
            }
            Some(LocFileType::Csv) => scanner.scan(
                csv::Reader::from_path(path.clone())?.records(),
                |r| {
                    let record = match r {
                        Ok(r) => r,
                        Err(e) => panic!("{e}"),
                    };

                    if let Some(key) = record.get(0) {
                        Ok(validate_entry(file_name, key, &reference))
                    } else {
                        Ok(ItemStatus::Invalid(InvalidEntry::EmptyRecord(file_name.to_string())))
                    }
                },
            )?,
            Some(LocFileType::Json) => {
                println!();
                output::warn("JSON translation files are not supported yet.")
            }
        }
    }

    println!();
    output::divider("Scan complete.");

    scanner.print_results(reference.len() as u32);
    print_invalid_files(&invalid_file_names);
    Ok(())
}

fn validate_entry(file_name: &str, key: &str, reference: &HashSet<String>)
    -> ItemStatus<InvalidEntry>
{
    if reference.contains(key) {
        ItemStatus::Valid
    } else if key.chars().next() == Some('#') {
        ItemStatus::Ignored
    } else {
        ItemStatus::Invalid(InvalidEntry::BadKey(file_name.to_string(), key.to_string()))
    }
}

fn print_invalid_files(file_names: &Vec<String>) {
    let count = file_names.len();
    let dash = if count == 0 { DASH.to_string() } else { RED_DASH.to_string() };

    match count.cmp(&1) {
        Ordering::Less => {
            println!("{dash} No invalid files found!");
            return;
        }
        Ordering::Equal => println!("{dash} This file has an invalid name and was skipped:"),
        Ordering::Greater => println!("{dash} These files have invalid names and were skipped:"),
    }

    output::bullet_list(format!("  {dash}"), file_names.iter());
}
