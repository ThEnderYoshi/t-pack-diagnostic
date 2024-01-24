use std::{
    collections::HashSet, error::Error, ffi::OsStr, fmt::Display, fs, io, path::{Path, PathBuf}
};

use walkdir::{DirEntry, WalkDir};

use crate::{
    output,
    paths::{self, EXPECT_UTF8_PATH},
    scanner::{ItemStatus, Scanner},
    static_file_data::MUSIC_REF_NAME,
};

const EXTENSIONS: [&str; 3] = ["mp3", "ogg", "wav"];

pub enum InvalidMusic {
    BadName(PathBuf),
    BadExtension(PathBuf),
}

impl Display for InvalidMusic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadName(p) => write!(f, "{p:?}\t: Name not in the reference."),
            Self::BadExtension(p) => {
                write!(f, "{p:?}\t: Invalid file format. Accepted: {}", EXTENSIONS.join(", "))
            }
        }
    }
}

pub fn scan_music(music_dir: &PathBuf, ref_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let refs = open_music_ref(paths::push(ref_dir, MUSIC_REF_NAME))?;
    let mut scanner = Scanner::new("songs");

    println!();
    output::announce_path("Scanning", music_dir);

    scanner.scan(
        WalkDir::new(music_dir).max_depth(1).into_iter(),
        |f| validate_entry(f, &refs),
    )?;

    println!();
    output::divider("Scan complete.");
    scanner.print_results(refs.len() as u32);
    Ok(())
}

/// Opens the `music.txt` file into a [HashSet].
#[inline]
pub fn open_music_ref<P: AsRef<Path>>(path: P) -> io::Result<HashSet<String>> {
    Ok(
        fs::read_to_string(path)?
            .lines()
            .map(String::from)
            .collect()
    )
}

fn validate_entry(f: &walkdir::Result<DirEntry>, refs: &HashSet<String>)
    -> Result<ItemStatus<InvalidMusic>, Box<dyn Error>>
{
    let entry = match f {
        Ok(e) => e,
        Err(e) => panic!("{e}"),
    };

    let path = entry.path().to_path_buf();

    if path.is_dir() || path.file_name() == Some(OsStr::new("desktop.ini")) {
        return Ok(ItemStatus::Ignored);
    }

    validate_song(path, refs)
}

pub fn validate_song(path: PathBuf, refs: &HashSet<String>) -> Result<ItemStatus<InvalidMusic>, Box<dyn Error>> {
    let extension = match path.extension() {
        Some(e) => e,
        None => return Ok(ItemStatus::Invalid(InvalidMusic::BadExtension(path))),
    };

    let extension = extension.to_str().expect(EXPECT_UTF8_PATH);

    if !EXTENSIONS.iter().any(|e| e == &extension) {
        return Ok(ItemStatus::Invalid(InvalidMusic::BadExtension(path)));
    }

    let file_name = paths::file_name(&path);
    // SAFETY: `extension` is created from the same path as `file_name`.
    let file_name = file_name.strip_suffix(&format!(".{extension}")).unwrap();

    if refs.contains(file_name) {
        Ok(ItemStatus::Valid)
    } else {
        Ok(ItemStatus::Invalid(InvalidMusic::BadName(path)))
    }
}
