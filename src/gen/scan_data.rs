use std::{collections::HashMap, cmp::Ordering};

use ansi_term::{Color, Style};
use lazy_static::lazy_static;

use crate::image_data::ImageData;

use super::InvalidEntry;

pub struct ScanData {
    pub image_data: HashMap<String, Vec<ImageData>>,
    pub invalid_entries: HashMap<String, InvalidEntry>,
    pub valid_count: u32,
    pub invalid_count: u32,
}

impl ScanData {
    pub fn new() -> Self {
        Self {
            image_data: HashMap::new(),
            invalid_entries: HashMap::new(),
            valid_count: 0,
            invalid_count: 0,
        }
    }

    pub fn push_valid(&mut self, key: &str, item: ImageData) {
        self.valid_count += 1;
        self.image_data.entry(key.to_string())
            .or_insert(vec![])
            .push(item);
    }

    pub fn push_invalid(&mut self, key: &str, item: InvalidEntry) {
        self.invalid_count += 1;
        self.invalid_entries.insert(key.to_string(), item);
    }

    #[inline]
    pub fn joined_count(&self) -> u32 {
        self.valid_count + self.invalid_count
    }

    pub fn print_results(&self) {
        lazy_static! {
            static ref GRAY: Color = Color::Black;
            static ref GREEN: Style = Color::Green.bold();
        }

        println!("Found {} valid images.", GREEN.paint(self.valid_count.to_string()));

        let dash = if self.invalid_count == 0 { Color::Blue } else { Color::Red };
        let dash = dash.bold().paint("-");

        let count =
            Color::Red.bold().paint(self.invalid_count.to_string());
        
        match self.invalid_count.cmp(&1) {
            Ordering::Less => println!("{dash} No invalid items found!"),
            Ordering::Equal => println!("{dash} Found {count} invalid item. \
                It will not be included in the references."),
            Ordering::Greater => println!("{dash} Found {count} invalid items. \
                They will not be included in the references."),
        }

        for (key, value) in self.invalid_entries.iter() {
            println!("  {dash} {key}:\t{value}");
        }
    }
}
