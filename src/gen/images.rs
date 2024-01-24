use std::{ffi::OsStr, fmt::Display, io, path::PathBuf};

use slop_rs::Slop;
use walkdir::WalkDir;

use crate::{
    image_data::ImageData,
    output,
    paths,
    static_file_data::{IMAGE_REF_NAME, IMAGE_REF_VERSION, VERSION_KEY},
};

use super::scan_data::ScanData;

type ImageScanData = ScanData<ImageData, InvalidImage>;

struct InvalidImage(PathBuf);

impl Display for InvalidImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}\t:Couldn't read as image.", self.0)
    }
}

pub fn generate_image_ref(image_dir: &PathBuf, refs: &PathBuf) -> io::Result<()> {
    output::divider("Generating image reference...");

    let mut scan_data = ScanData::new("images");
    output::announce_path("Scanning", image_dir);

    let mut stdout = io::stdout().lock();

    for entry in WalkDir::new(image_dir.clone()) {
        let entry = entry?;
        let path = entry.path().to_path_buf();

        if path.is_dir() || path.file_name() == Some(OsStr::new("desktop.ini")) {
            continue;
        }

        let parent = paths::sanitize_path(path.clone(), image_dir);
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

    let slop = generate_slop(&scan_data);
    output::divider("Writing SLOP to disk...");

    slop
        .save(paths::push(refs, IMAGE_REF_NAME))
        .expect("expected to be able to write slop to disk");

    Ok(())
}

fn register_item(data: &mut ImageScanData, parent: &PathBuf, path: &PathBuf) {
    let key = paths::path_buf_to_key_name(parent);

    match ImageData::open(path) {
        Ok(d) => data.push_valid(key, d),
        Err(_e) => data.push_invalid(key, InvalidImage(path.clone())),
    }
}

fn generate_slop(data: &ImageScanData) -> Slop {
    output::divider("Converting to SLOP file...");
    
    let mut slop = Slop::new();
    slop.insert_unchecked(VERSION_KEY.to_string(), IMAGE_REF_VERSION.to_string());
    slop.insert_unchecked("!count".to_string(), data.joined_count().to_string());

    for (key, images) in &data.valid_entries {
        let images: Vec<String> = images
            .iter()
            .map(|i| i.to_string())
            .collect();

        slop
            .insert(key.clone(), images)
            .expect("expected parent dir path to be a valid slop key");
    }

    slop
}
