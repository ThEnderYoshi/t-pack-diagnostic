//! Handles the `scan` action.

pub mod images;
pub mod loc;

use std::{path::PathBuf, cmp::Ordering, error::Error};

use ansi_term::{Color, Style, ANSIStrings};
use lazy_static::lazy_static;

use crate::{output::{self, DASH, RED_DASH}, input};

pub fn scan_directory(root_dir: &PathBuf, ref_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    output::divider("ACTION - Scan Directory");

    if !ref_dir.is_dir() {
        panic!("expected `-r` to be a valid dir - you must run `gen` first");
    }

    let images_dir = input::child_path(root_dir, "Content/Images");
    if images_dir.is_dir() {
        images::crawl_images(&images_dir, ref_dir)?;
    }

    let loc_dir = input::child_path(root_dir, "Content/Localization");
    if loc_dir.is_dir() {
        loc::crawl_localization(&loc_dir, ref_dir)?;
    }

    Ok(())
}

enum ItemValidation {
    Valid,
    Invalid(String),
    Ignored,
}

struct ValidatorCrawler<> {
    total_count: u32,
    valid_count: u32,
    invalid_count: u32,
    invalid_items: Vec<String>,
}

impl ValidatorCrawler {
    fn new(total_count: u32) -> Self {
        Self {
            total_count,
            valid_count: 0,
            invalid_count: 0,
            invalid_items: vec![],
        }
    }

    fn crawl<I, F>(&mut self, iter: &mut I, is_item_valid: F) -> Result<(), Box<dyn Error>>
    where
        I: Iterator,
        F: Fn(&<I as Iterator>::Item) -> Result<ItemValidation, Box<dyn Error>>,
    {
        for item in iter {
            match is_item_valid(&item)? {
                ItemValidation::Ignored => continue,
                ItemValidation::Valid => self.valid_count += 1,
                ItemValidation::Invalid(item) => {
                    self.invalid_count += 1;
                    self.invalid_items.push(item);
                }
            }
            let joined_count = self.joined_count();
            if joined_count % 100 == 0 {
                output::update_progress("items", joined_count)?;
            }
        }

        output::update_progress("items", self.joined_count())?;
        Ok(())
    }

    fn report_findings(&self) {
        lazy_static! {
            static ref GRAY: Color = Color::Black;
            static ref GREEN: Style = Color::Green.bold();
        }

        let valid = output::style(&GREEN, &self.valid_count);
        let total = output::style(&GREEN, &self.total_count);
        let percent = [
            GRAY.paint("("),
            GREEN.paint(format!("{:.2}", self.get_percent())),
            GRAY.paint("% of the way!)"),
        ];
        let milestone = format!(
            "{}% to the next 1000!",
            GREEN.paint(&format!("{:.1}", self.get_milestone())),
        );

        println!("Found {valid}/{total} items. {}", ANSIStrings(&percent));
        output::bullet_list(DASH.to_string(), &mut vec![milestone].iter());

        self.report_invalid_items()
    }

    fn report_invalid_items(&self) {
        let dash = if self.invalid_count == 0 {
            DASH.to_string()
        } else {
            RED_DASH.to_string()
        };

        let count =
            output::style(&Color::Red.bold(), &self.invalid_count);

        match self.invalid_count.cmp(&1) {
            Ordering::Less => {
                println!("{dash} No invalid items found!");
                return;
            },
            Ordering::Equal => println!("{dash} Found {count} invalid item."),
            Ordering::Greater => println!("{dash} Found {count} invalid items."),
        }

        output::bullet_list(
            format!("  {dash}"),
            &mut self.invalid_items.iter().map(|i| i.replace("\\", "/")),
        );
    }

    fn get_percent(&self) -> f32 {
        self.valid_count as f32 / self.total_count as f32 * 100.0
    }

    fn get_milestone(&self) -> f32 {
        (self.valid_count % 1000) as f32 * 0.1
    }

    fn joined_count(&self) -> u32 {
        self.valid_count + self.invalid_count
    }
}
