//! Handles the generation of the reference files.

use std::{error::Error, path::PathBuf};

use crate::{output, paths, static_file_data::ALL_LOC_CSV_NAME};

mod images;
mod loc;
mod music;
mod scan_data;
mod sounds;

pub fn generate_references(extracted: &PathBuf, refs: &PathBuf) -> Result<(), Box<dyn Error>> {
    assert!(refs.is_dir(), "`-o` ({refs:?}) must point to a valid dir");
    output::info("ACTION - Generate References");

    let images_dir = paths::push(extracted, "Images");

    if images_dir.is_dir() {
        images::generate_image_ref(&images_dir, refs)?;
    }

    let loc_file = paths::push(extracted, ALL_LOC_CSV_NAME);

    if loc_file.is_file() {
        loc::generate_loc_ref(&loc_file, refs)?;
    }

    music::generate_music_ref(extracted, refs)?;

    let sound_dir = paths::push(extracted, "Sounds");

    if sound_dir.is_dir() {
        sounds::generate_sound_ref(&sound_dir, refs)?;
    }

    Ok(())
}
