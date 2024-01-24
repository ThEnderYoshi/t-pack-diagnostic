use std::{fs, io, path::PathBuf};

use lazy_static::lazy_static;
use regex::Regex;
use walkdir::WalkDir;

use crate::{output, paths, static_file_data::MUSIC_REF_NAME};

pub fn generate_music_ref(root_dir: &PathBuf, refs: &PathBuf) -> io::Result<()> {
    lazy_static! {
        /// Matches the file names of music files.
        /// 
        /// Captures:
        /// - `1`: The song's ID.
        static ref RE_MUSIC_FILE_NAME: Regex = Regex::new(r"([0-9]{2,}).*?\.wav").unwrap();
    }

    output::announce_path("Scanning music files in", root_dir);

    let mut stdout = io::stdout().lock();

    let mut ids: Vec<usize> = vec![];
    let mut id_count = 0;

    for entry in WalkDir::new(root_dir).max_depth(1) {
        let entry = entry?;
        let path = entry.path().to_path_buf();

        if path.is_dir() {
            continue;
        }

        let file_name = paths::file_name(&path);
        
        let caps = match RE_MUSIC_FILE_NAME.captures(file_name) {
            Some(cs) => cs,
            None => continue,
        };

        // SAFETY: The pattern includes group 1.
        ids.push(caps[1].parse().expect("expected a valid uint str"));
        id_count += 1;

        if id_count % 10 == 0 {
            output::update_progress(&mut stdout, "files", id_count)?;
        }
    }

    output::update_progress(&mut stdout, "files", id_count)?;
    println!();

    output::divider("Scan complete.");
    output::divider("Writing reference file to disk...");

    let ids = ids
        .iter()
        .fold(String::new(), |mut acc, id| {
            acc.push_str(&format!("Music_{id}\n"));
            acc
        });

    fs::write(paths::push(refs, MUSIC_REF_NAME), ids)?;
    Ok(())
}
