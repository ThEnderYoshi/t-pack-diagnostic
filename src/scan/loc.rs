//! Handles the localization of the `scan` action.

use std::{path::PathBuf, error::Error, fs, cmp::Ordering, io::{self, Write}};

use lazy_static::lazy_static;
use regex::Regex;
use walkdir::WalkDir;

use crate::{
    input,
    output::{self, DASH, RED_DASH},
};

use super::{ValidatorCrawler, ItemValidation};

lazy_static! {
    /// `0`: Entire match, `1`: Locale tag, `2`: File extension
    pub static ref RE_LOC_FILE_NAME: Regex = Regex::new(
        r"^(en-US|de-DE|it-IT|fr-FR|es-ES|ru-RU|zh-Hans|pt-BR|pl-PL)-.*\.(csv|json)$",
    ).unwrap();
}

pub fn crawl_localization(loc_dir: &PathBuf, ref_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let reference: Vec<String> = fs::read_to_string(input::child_path(ref_dir, "loc_keys.txt"))
    .expect("expected `-r` to have the file `loc_keys.txt`")
    .split('\n')
        .map(|i| String::from(i))
        .collect();

    let mut crawler = ValidatorCrawler::new(reference.len() as u32);
    let mut invalid_files: Vec<String> = vec![];

    println!();
    output::announce_path("Scanning", loc_dir);

    for entry in WalkDir::new(loc_dir).max_depth(1) {
        let entry = entry?;
        let path = entry.path().to_path_buf();

        if path.is_dir() {
            continue;
        }
        if let Some(file_name) = path.file_name() {
            let file_name = file_name.to_str()
                .expect(input::EXPECT_UTF8_PATH);

            match get_loc_file_type(file_name) {
                LocFileType::Invalid => {
                    invalid_files.push(String::from(file_name));
                    continue;
                },
                LocFileType::Csv => crawler.crawl(
                    &mut csv::Reader::from_path(path.clone())?.records(),
                    |f| {
                        let record = match f {
                            Ok(record) => record,
                            Err(e) => panic!("{e}"),
                        };
                        Ok(if let Some(key) = record.get(0) {
                            validate_entry(file_name, key, &reference)
                        } else {
                            ItemValidation::Invalid(format!("{file_name}:\t<empty record>"))
                        })
                    },
                )?,
                LocFileType::Json => {
                    io::stdout().flush()?;
                    println!();
                    output::warn("JSON files aren't supported yet.");
                }
            }
        }
    }

    io::stdout().flush()?;
    println!();
    output::divider("Crawl comlpete.");

    crawler.report_findings();
    report_invalid_files(&invalid_files);

    Ok(())
}

enum LocFileType {
    Invalid,
    Csv,
    Json,
}

fn get_loc_file_type(file_name: &str) -> LocFileType {
    if let Some(caps) = RE_LOC_FILE_NAME.captures(file_name) {
        if &caps[2] == "csv" {
            LocFileType::Csv
        } else {
            LocFileType::Json
        }
    } else {
        LocFileType::Invalid
    }
}

fn validate_entry(file_name: &str, key: &str, reference: &Vec<String>) -> ItemValidation {
    let key = String::from(key);

    if reference.iter().any(|i| i == &key) {
        ItemValidation::Valid
    } else if key.chars().next() == Some('#') {
        ItemValidation::Ignored
    } else {
        ItemValidation::Invalid(format!("{file_name}:\t{key}"))
    }
}

fn report_invalid_files(files: &Vec<String>) {
    let count = files.len();
    let dash = if count == 0 { DASH.to_string() } else { RED_DASH.to_string() };

    match count.cmp(&1) {
        Ordering::Less => {
            println!("{dash} No invalid files found!");
            return;
        },
        Ordering::Equal => println!("{dash} This file has an invalid name and was skipped:"),
        Ordering::Greater => println!("{dash} These files have invalid names and were skipped:"),
    }

    output::bullet_list(format!("  {dash}"), &mut files.iter());
}
