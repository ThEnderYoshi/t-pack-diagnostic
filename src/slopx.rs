//! SLOP Xtended functions.

use std::{str::FromStr, fmt::Debug};

use crate::slop::{Slop, SlopValue};

pub fn parse_string<T>(slop: &Slop, key: &str) -> Option<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    if let Some(value) = slop.get(key) {
        if let SlopValue::String(string) = value {
            Some(string.parse()
                .expect(&format!("Expected `{key}={string}` to parse to a specific value")))
        } else {
            panic!("Expected `{key}` to hold a string");
        }
    } else {
        None
    }
}
