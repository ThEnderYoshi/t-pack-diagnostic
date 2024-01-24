//! Handles the scanning and actual 'diagnostic' of a Resource Pack.

use std::{path::PathBuf, error::Error};

use crate::{output, paths};

pub mod images;
pub mod loc;
pub mod music;
pub mod sounds;

const MSG_BAD_REF_DIR: &str = "expected `-r` to be the dir with the reference files";

pub fn scan_resource_pack(root_dir: &PathBuf, ref_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    output::divider("ACTION - Scan Directory");

    if !ref_dir.is_dir() {
        panic!("{MSG_BAD_REF_DIR}");
    }

    let images_dir = paths::push(root_dir, "Content/Images/");

    if images_dir.is_dir() {
        images::scan_images(&images_dir, ref_dir)?;
    }

    let loc_dir = paths::push(root_dir, "Content/Localization/");

    if loc_dir.is_dir() {
        loc::scan_localization_files(&loc_dir, ref_dir)?;
    }

    let music_dir = paths::push(root_dir, "Content/Music/");

    if music_dir.is_dir() {
        music::scan_music(&music_dir, ref_dir)?;
    }

    let sounds_dir = paths::push(root_dir, "Content/Sounds/");

    if sounds_dir.is_dir() {
        sounds::scan_sounds(&sounds_dir, ref_dir)?;
    }

    Ok(())
}
