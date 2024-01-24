use std::{cmp::Ordering, error::Error, fmt::Display, io};

use ansi_term::{Color, Style, ANSIStrings};
use lazy_static::lazy_static;

use crate::{
    output::{self, DASH, RED_DASH},
    static_file_data::MAX_LIST_SIZE,
};

/// The possible responses a validator function can return, other than errors.
pub enum ItemStatus<B> {
    Valid,
    Invalid(B),
    Ignored,
}

impl<B> ItemStatus<B> {
    /// Returns `true` if the item is an [ItemStatus::Valid].
    #[inline]
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Valid => true,
            _ => false,
        }
    }
}

/// Iterates through something, validating each item.
/// Valid items increase `valid_count` and invalid items are added
/// to `invalid_items`.
/// 
/// `B` is the type of the invalid items.
pub struct Scanner<B> {
    pub item_name: &'static str,
    pub valid_count: u32,
    pub invalid_items: Vec<B>,
}

impl<B: Display> Scanner<B> {
    pub fn new(item_name: &'static str) -> Self {
        Self { item_name, valid_count: 0, invalid_items: vec![] }
    }

    pub fn scan<I, F>(&mut self, iter: I, validator: F) -> Result<(), Box<dyn Error>>
    where
        I: Iterator,
        F: Fn(&<I as Iterator>::Item) -> Result<ItemStatus<B>, Box<dyn Error>>,
    {
        let mut stdout = io::stdout().lock();

        for item in iter {
            match validator(&item)? {
                ItemStatus::Ignored => continue,
                ItemStatus::Valid => self.valid_count += 1,
                ItemStatus::Invalid(b) => self.invalid_items.push(b),
            }

            let joined_count = self.joined_count();

            if joined_count % 100 == 0 {
                output::update_progress(&mut stdout, self.item_name, joined_count)?;
            }
        }

        output::update_progress(&mut stdout, self.item_name, self.joined_count())?;
        Ok(())
    }

    pub fn print_results(&self, extracted_count: u32) {
        lazy_static! {
            static ref GRAY: Color = Color::Black;
            static ref GREEN: Style = Color::Green.bold();
        }

        let valid = GREEN.paint(self.valid_count.to_string());
        let total = GREEN.paint(extracted_count.to_string());

        let total_percent = [
            GRAY.paint("("),
            GREEN.paint(format!("{:.2}", self.get_percent(extracted_count))),
            GRAY.paint("% of the way!)"),
        ];

        let milestone_percent = format!(
            "{}% of the way to the next 1000!",
            GREEN.paint(&format!("{:.1}", self.get_milestone())),
        );

        println!("Found {valid}/{total} items. {}", ANSIStrings(&total_percent));
        output::bullet_list(DASH.to_string(), vec![milestone_percent].iter());

        self.print_invalid_items();
    }

    fn print_invalid_items(&self) {
        let invalid_count = self.invalid_items.len();

        let dash = if invalid_count == 0 {
            DASH.to_string()
        } else {
            RED_DASH.to_string()
        };

        let count = Color::Red.bold().paint(invalid_count.to_string());

        match invalid_count.cmp(&1) {
            Ordering::Less => {
                println!("{dash} No invalid items found!");
                return;
            },
            Ordering::Equal => println!("{dash} Found {count} invalid item."),
            Ordering::Greater => println!("{dash} Found {count} invalid items."),
        }

        let iter = self.invalid_items
            .iter()
            //.map(|i| i.replace("\\", "/"))
            .take(MAX_LIST_SIZE);

        output::bullet_list(format!("  {dash}"), iter);

        if invalid_count > MAX_LIST_SIZE {
            println!("  {dash} ... and {} more.", invalid_count - 100);
        }
    }

    #[inline]
    fn get_percent(&self, extracted_count: u32) -> f32 {
        (self.valid_count as f32) / (extracted_count as f32) * 100.0
    }

    #[inline]
    fn get_milestone(&self) -> f32 {
        (self.valid_count % 1000) as f32 * 0.1
    }

    #[inline]
    fn joined_count(&self) -> u32 {
        self.valid_count + (self.invalid_items.len() as u32)
    }
}
