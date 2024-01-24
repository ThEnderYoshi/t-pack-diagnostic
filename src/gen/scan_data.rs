use std::{cmp::Ordering, collections::HashMap, fmt::Display};

use ansi_term::{Color, Style};
use lazy_static::lazy_static;

use crate::output::{self, DASH, RED_DASH};

/// Data collected by the reference generators.
/// 
/// `D` is the data of each individual item and `B` is data about invalid items.
pub struct ScanData<D, B: Display> {
    pub item_name: &'static str,
    pub valid_entries: HashMap<String, Vec<D>>,
    pub invalid_entries: HashMap<String, B>,
    valid_count: usize,
}

impl<D, B: Display> ScanData<D, B> {
    pub fn new(item_name: &'static str) -> Self {
        Self {
            item_name,
            valid_entries: HashMap::new(),
            invalid_entries: HashMap::new(),
            valid_count: 0,
        }
    }

    pub fn push_valid(&mut self, key: String, entry: D) {
        self.valid_count += 1;
        self.valid_entries
            .entry(key.to_string())
            .or_insert(vec![])
            .push(entry);
    }

    pub fn push_invalid(&mut self, key: String, entry: B) {
        self.invalid_entries.insert(key, entry);
    }

    #[inline(always)]
    pub fn valid_count(&self) -> usize {
        self.valid_count
    }

    #[inline(always)]
    pub fn invalid_count(&self) -> usize {
        self.invalid_entries.len()
    }

    #[inline(always)]
    pub fn joined_count(&self) -> usize {
        self.valid_count() + self.invalid_count()
    }

    pub fn print_results(&self) {
        lazy_static! {
            static ref GREEN: Style = Color::Green.bold();
        }

        println!("Found {} valid {}.", GREEN.paint(self.valid_count().to_string()), self.item_name);

        let count = self.invalid_count();
        let dash = if count == 0 { DASH.to_string() } else { RED_DASH.to_string() };

        let count =
            Color::Red.bold().paint(count.to_string());

        match self.invalid_count().cmp(&1) {
            Ordering::Less => {
                println!("{dash} No invalid items found!");
                return;
            }
            Ordering::Equal => println!(
                "{dash} Fount {count} invalid item:\n  \
                It will not be included in the reference file.",
            ),
            Ordering::Greater => println!(
                "{dash} Fount {count} invalid items:\n  \
                They will not be included in the reference file.",
            ),
        }

        output::bullet_list(
            dash,
            self.invalid_entries
                .iter()
                .map(|(k, v)| format!("{k}\t: {v}")),
        );
    }
}
