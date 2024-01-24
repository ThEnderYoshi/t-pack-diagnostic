use std::{error::Error, fs, io, path::PathBuf};

use csv::Reader;

use crate::{output, paths, static_file_data::LOC_REF_NAME};

pub fn generate_loc_ref(loc_path: &PathBuf, refs: &PathBuf) -> Result<(), Box<dyn Error>> {
    output::announce_path("Scanning", loc_path);

    let mut reader = Reader::from_path(loc_path)?;
    let mut keys = vec![];
    
    output::divider("Generating localization reference...");

    for record in reader.records() {
        let record = record?;

        if let Some(field) = record.get(0) {
            keys.push(field.to_string());
        }
    }

    output::update_progress(&mut io::stdout().lock(), "entries", keys.len() as u32)?;
    println!();

    output::divider("Writing reference file to disk...");
    fs::write(paths::push(refs, LOC_REF_NAME), keys.join("\n"))?;

    Ok(())
}
