//! Handles the `gen` action.

use std::{
    path::{PathBuf, Path},
    io::{self, Write},
    collections::HashMap, fs, error::Error,
};

use csv::Reader;
use walkdir::WalkDir;

use crate::{
    output,
    slop::{SlopValue, Slop},
    input,
};

const IMAGES_SLOP_FILE_NAME: &str = "images.slop";
const LOC_KEYS_FILE_NAME: &str = "loc_keys.txt";

pub fn generate_references(extracted_files: &PathBuf, generated_refs: &PathBuf) -> io::Result<()> {
    output::info("ACTION - Generate References");
    let mut extracted_images = extracted_files.clone();
    extracted_images.push("Images");

    if !extracted_images.is_dir() {
        println!("{extracted_files:?}");
        panic!("`-i` must point to a valid directory with an `/Images` dir");
    }
    if !generated_refs.is_dir() {
        panic!("`-o` must point to a valid directory");
    }

    let mut image_paths: HashMap<String, SlopValue> = HashMap::new();
    let mut image_count: u32 = 0;

    output::announce_path("Crawling through", &extracted_images);

    for entry in WalkDir::new(extracted_images.clone()) {
        let entry = entry?;
        let path = entry.path();
        let path_dir = path.to_path_buf();

        if path_dir.is_dir() {
            continue;
        }

        let file_name = input::file_name(&path_dir);
        let path_dir = input::sanitize_path(&mut path_dir.clone(), &extracted_images);
        let path_dir = input::path_buf_to_key_name(&path_dir);
        register_item(&mut image_paths, &path_dir, file_name);

        image_count += 1;
        if image_count % 100 == 0 {
            output::update_progress("images", image_count)?;
        }
    }

    output::update_progress("images", image_count)?;
    io::stdout().flush()?;
    if image_count > 10000 {
        println!("\n(Wowza that's a lot of images!)");
    } else {
        println!();
    }
    output::divider("Crawl complete.");

    let slop = generate_slop(image_paths, image_count);
    write_findings(&slop, generated_refs);

    if let Err(_) = generate_loc(extracted_files, generated_refs) {
        output::warn("Loc.csv is not a valid CSV file! Skipping...");
    }

    Ok(())
}


fn register_item(images: &mut HashMap<String, SlopValue>, key: &str, item: &str) {
    let key = String::from(key);
    let item = String::from(item);
    if let Some(SlopValue::List(list)) = images.get(&key) {
        let mut list = list.clone();
        list.push(item);
        images.insert(key, SlopValue::List(list));
    } else {
        images.insert(key, SlopValue::List(vec![item]));
    }
}

fn generate_slop(image_paths: HashMap<String, SlopValue>, image_count: u32) -> Slop {
    output::divider("Generating SLOP file...");

    let mut slop = Slop::from_map(image_paths);
    slop.insert_str("!count", &image_count.to_string());
    slop
}

fn generate_loc(extracted_files: &PathBuf, output_dir: &PathBuf) -> Result<(), Box<dyn Error>> {

    let mut loc_path = extracted_files.clone();
    loc_path.push("Loc.csv");

    if loc_path.is_file() {
        output::announce_path("Found", &loc_path);

        let mut reader = Reader::from_path(loc_path)?;
        let mut keys: Vec<String> = vec![];
        let mut key_count: u32 = 0;
        println!("Copying translation keys...");

        for record in reader.records() {
            let record = record?;
            if let Some(field) = record.get(0) {
                keys.push(field.to_owned());
                key_count += 1;
                if key_count % 100 == 0 {
                    output::update_progress("entries", key_count)?;
                }
            }
        }
        output::update_progress("entries", key_count)?;
        io::stdout().flush()?;
        println!();

        output::divider("Writing keys to disk...");
        fs::write(input::child_path(output_dir, LOC_KEYS_FILE_NAME), keys.join("\n"))
            .expect("Expected to be able to write to disk");
    }

    Ok(())
}

fn write_findings(slop: &Slop, output_dir: &PathBuf) {
    output::divider("Writing findings to disk...");

    let mut file_path = output_dir.clone();
    file_path.push(Path::new(IMAGES_SLOP_FILE_NAME));
    fs::write(file_path, slop.serialize())
        .expect("Expected to be able to write to disk");
}
