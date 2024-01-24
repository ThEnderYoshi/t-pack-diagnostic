use std::{ffi::OsStr, fmt::Display, io, path::PathBuf};

use slop_rs::Slop;
use walkdir::WalkDir;

use crate::{
    output,
    paths,
    static_file_data::{SOUND_REF_NAME, SOUND_REF_VERSION, VERSION_KEY},
};

use super::scan_data::ScanData;

type SoundScanData = ScanData<SoundData, InvalidSound>;

struct SoundData {
    path: String,
}

impl SoundData {
    fn open(mut path: PathBuf) -> Result<Self, InvalidSound> {
        if path.extension() != Some(OsStr::new("wav")) {
            return Err(InvalidSound(path));
        }

        path.set_extension("xnb");
        Ok(Self { path: paths::file_name(&path).to_string() })
    }
}

impl Display for SoundData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}

struct InvalidSound(PathBuf);

impl Display for InvalidSound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}\t:Doesn't have a `.wav` file extension.", self.0)
    }
}

pub fn generate_sound_ref(sound_dir: &PathBuf, refs: &PathBuf) -> io::Result<()> {
    output::divider("Generating sound reference...");

    let mut scan_data = ScanData::new("sounds");
    output::announce_path("Scanning", sound_dir);

    let mut stdout = io::stdout().lock();

    for entry in WalkDir::new(&sound_dir) {
        let entry = entry?;
        let path = entry.path().to_path_buf();

        if path.is_dir() || path.file_name() == Some(OsStr::new("desktop.ini")) {
            continue;
        }

        let parent = paths::sanitize_path(path.clone(), sound_dir);
        register_item(&mut scan_data, &parent, &path);

        let joined_count = scan_data.joined_count() as u32;

        if joined_count % 100 == 0 {
            output::update_progress(&mut stdout, scan_data.item_name, joined_count)?;
        }
    }

    output::update_progress(&mut stdout, scan_data.item_name, scan_data.joined_count() as u32)?;
    println!();
    output::divider("Scan complete.");
    scan_data.print_results();

    let slop = generate_slop(scan_data);
    output::divider("Writing SLOP to disk...");

    slop
        .save(paths::push(refs, SOUND_REF_NAME))
        .expect("expected to be able to write slop to disk");

    Ok(())
}

fn register_item(data: &mut SoundScanData, parent: &PathBuf, path: &PathBuf) {
    let key = paths::path_buf_to_key_name(parent);

    match SoundData::open(path.clone()) {
        Ok(d) => data.push_valid(key, d),
        Err(b) => data.push_invalid(key, b),
    }
}

fn generate_slop(data: SoundScanData) -> Slop {
    output::divider("Converting to SLOP file...");

    let mut slop = Slop::new();
    slop.insert_unchecked(VERSION_KEY.to_string(), SOUND_REF_VERSION.to_string());
    slop.insert_unchecked("!count".to_string(), data.joined_count().to_string());

    for (key, sounds) in data.valid_entries {
        let sounds: Vec<String> = sounds
            .iter()
            .map(|s| s.to_string())
            .collect();

        slop
            .insert(key, sounds)
            .expect("expected parent dir path to be a valid slop key");
    }

    slop
}
