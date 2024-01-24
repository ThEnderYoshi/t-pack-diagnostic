use std::{
    collections::{HashMap, HashSet}, error::Error, ffi::OsStr, fmt::Display, path::PathBuf
};

use slop_rs::Slop;
use walkdir::{DirEntry, WalkDir};

use crate::{
    output,
    paths,
    scanner::{ItemStatus, Scanner},
    static_file_data::{self, SOUND_REF_NAME, SOUND_REF_VERSION},
};

use super::MSG_BAD_REF_DIR;

/// Shorthand for the data taken from the `sounds.slop` file.
type DataMap = HashMap<String, HashSet<String>>;

pub enum InvalidSound {
    BadName(PathBuf),
    BadExtension(PathBuf),
}

impl Display for InvalidSound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadName(p) => write!(f, "{p:?}\t: Name not found in the reference."),
            Self::BadExtension(p) => {
                write!(f, "{p:?}\t: Sound files must be in the XNB format.")
            }
        }
    }
}

pub fn scan_sounds(sounds_dir: &PathBuf, ref_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let slop = Slop::open(paths::push(ref_dir, SOUND_REF_NAME))
        .expect(MSG_BAD_REF_DIR);

    static_file_data::validate_slop(&slop, SOUND_REF_VERSION);

    let extracted_count: u32 = slop
        .get_string("!count")
        .expect("expected `images.slop` to have a `!count` keyvalue")
        .parse()
        .expect("expected `!count` kv to parse into an unsigned 32 bit integer");

    let mut scanner = Scanner::new("sounds");
    let data = slop_into_sound_data(slop);

    println!();
    output::announce_path("Scanning", sounds_dir);

    scanner.scan(
        WalkDir::new(sounds_dir).into_iter(),
        |f| validate_entry(f, sounds_dir, &data),
    )?;

    println!();
    output::divider("Scan complete.");
    scanner.print_results(extracted_count);
    Ok(())
}

pub fn slop_into_sound_data(slop: Slop) -> DataMap {
    let mut data = HashMap::new();

    for (key, value) in slop {
        if key.starts_with('!') {
            continue;
        }

        let values = value.list().expect("expected a list kv");
        data.insert(key, values.iter().map(String::from).collect());
    }

    data
}

fn validate_entry(f: &walkdir::Result<DirEntry>, sounds_dir: &PathBuf, data: &DataMap)
    -> Result<ItemStatus<InvalidSound>, Box<dyn Error>>
{
    let entry = match f {
        Ok(e) => e,
        Err(e) => panic!("{e}"),
    };

    let path = entry.path().to_path_buf();

    if path.is_dir() || path.file_name() == Some(OsStr::new("desktop.ini")) {
        return Ok(ItemStatus::Ignored);
    }

    let relative_path = path
        .strip_prefix(sounds_dir)
        .expect("expected path to be a child of `Sounds/`")
        .to_path_buf();

    let extension = match path.extension() {
        Some(e) => e,
        None => return Ok(ItemStatus::Invalid(InvalidSound::BadName(relative_path))),
    };

    if extension.to_str() != Some("xnb") {
        return Ok(ItemStatus::Invalid(InvalidSound::BadExtension(relative_path)));
    }

    validate_sound(path, relative_path, sounds_dir, data)
}

pub fn validate_sound(path: PathBuf, relative_path: PathBuf, sounds_dir: &PathBuf, data: &DataMap)
    -> Result<ItemStatus<InvalidSound>, Box<dyn Error>>
{
    let dir = path
        .parent()
        .expect("expected path to have a parent")
        .to_path_buf();

    let dir_key = dir
        .strip_prefix(sounds_dir)
        .expect("expected path to be a child of `Images/`")
        .to_path_buf();

    let dir_key = paths::path_buf_to_key_name(&dir_key);
    let file_name = paths::file_name(&path);

    let data = match data.get(&dir_key) {
        Some(d) => d,
        None => return Ok(ItemStatus::Invalid(InvalidSound::BadName(relative_path))),
    };

    if data.contains(file_name) {
        Ok(ItemStatus::Valid)
    } else {
        Ok(ItemStatus::Invalid(InvalidSound::BadName(relative_path)))
    }
}
