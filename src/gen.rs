//! Handles the generation of the reference files.

use std::{path::PathBuf, fmt::Display, io, fs, error::Error};

use csv::Reader;
use slop_rs::Slop;
use walkdir::WalkDir;

use crate::{
    output,
    paths,
    image_data::ImageData,
    static_file_data::{
        IMAGE_REF_NAME,
        IMAGE_REF_VERSION,
        ALL_LOC_CSV_NAME,
        LOC_REF_NAME, VERSION_KEY,
    },
};

use self::scan_data::ScanData;

mod scan_data;

pub enum InvalidEntry {
    InvalidImage(PathBuf),
}

impl Display for InvalidEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidImage(path) =>
                write!(f, "Couldn't read as image: {}", path.display()),
        }
    }
}

pub fn generate_references(extracted_files: &PathBuf, output: &PathBuf) -> io::Result<()> {
    output::info("ACTION - Generate References");

    let image_dir = paths::push(extracted_files, "Images");

    if !image_dir.is_dir() {
        panic!("`-i` must point to a valid dir with `Images/`");
    }

    if !output.is_dir() {
        panic!("`-o` must point to a valid dir");
    }

    let mut scan_data = ScanData::new();
    output::announce_path("Scanning", &image_dir);

    let mut stdout = io::stdout().lock();

    for entry in WalkDir::new(image_dir.clone()) {
        let entry = entry?;
        let path = entry.path();
        let file_path = path.to_path_buf();

        if file_path.is_dir() {
            continue;
        }

        let sanitized_parent = paths::sanitize_path(file_path.clone(), &image_dir);
        register_item(&mut scan_data, &sanitized_parent, &file_path);

        let joined_count = scan_data.joined_count();

        if joined_count % 100 == 0 {
            output::update_progress(&mut stdout, "images", joined_count)?;
        }
    }

    let joined_count = scan_data.joined_count();
    output::update_progress(&mut stdout, "images", joined_count)?;

    if joined_count > 10_000 {
        println!("\n(Wowza that's a lot of images!)");
    } else {
        println!();
    }

    output::divider("Scan complete.");
    scan_data.print_results();

    let slop = generate_slop(&scan_data);
    write_results(&slop, output);

    if let Err(_) = generate_loc(extracted_files, output) {
        output::warn("`Loc.csv` is not a valid CSV file! Skipping...");
    }

    Ok(())
}

fn register_item(data: &mut ScanData, parent_dir: &PathBuf, path: &PathBuf) {
    let key = String::from(paths::path_buf_to_key_name(parent_dir));

    match ImageData::open(path) {
        Ok(i) => data.push_valid(&key, i),
        Err(e) => {
            data.push_invalid(&key, InvalidEntry::InvalidImage(path.clone()));
            eprintln!("{e}\t{}", path.display());
        },
    }
}

fn generate_slop(data: &ScanData) -> Slop {
    output::divider("Generating SLOP file...");

    let mut slop = Slop::new();
    slop.insert_unchecked(VERSION_KEY.to_string(), IMAGE_REF_VERSION.to_string());
    slop.insert_unchecked("!count".to_string(), data.joined_count().to_string());
    //slop.insert(VERSION_KEY, IMAGE_REF_VERSION.to_string());
    //slop.insert("!count", data.joined_count().to_string());

    for (key, images) in data.image_data.iter() {
        let images: Vec<String> = images.iter().map(|i| i.to_string()).collect();

        slop
            .insert(key.clone(), images)
            .expect("expected parent dir path to be valid");
        //slop.insert(
        //    &key,
        //    images
        //        .iter()
        //        .map(|i| i.to_string())
        //        .collect::<Vec<String>>(),
        //);
    }

    slop
}

fn generate_loc(extracted_files: &PathBuf, output: &PathBuf) -> Result<(), Box<dyn Error>> {
    let loc_path = paths::push(extracted_files, ALL_LOC_CSV_NAME);

    if !loc_path.is_file() {
        return Ok(());
    }

    output::announce_path("Found", &loc_path);

    let mut reader = Reader::from_path(loc_path)?;
    let mut keys = vec![];
    let mut key_count = 0u32;
    let mut stdout = io::stdout().lock();
    println!("Copying translation keys...");

    for record in reader.records() {
        let record = record?;

        if let Some(field) = record.get(0) {
            keys.push(field.to_owned());
            key_count += 1;
            output::update_progress(&mut stdout, "entries", key_count)?;
        }
    }

    output::update_progress(&mut stdout, "entries", key_count)?;
    println!();

    output::divider("Writing keys to disk...");
    fs::write(paths::push(output, LOC_REF_NAME), keys.join("\n"))
        .expect("expected to be able to write to disk");

    Ok(())
}

fn write_results(slop: &Slop, output: &PathBuf) {
    output::divider("Writing SLOP file to disk...");

    slop
        .save(paths::push(output, IMAGE_REF_NAME))
        .expect("expected to be able to write slop to disk");
    //let path = paths::push(output, IMAGE_REF_NAME);
    //fs::write(path, slop.serialize())
    //    .expect("expected to be able to write slop to disk");
}
