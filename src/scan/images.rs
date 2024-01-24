use std::{collections::HashMap, error::Error, ffi::OsString, path::PathBuf};

use lazy_static::lazy_static;
use slop_rs::Slop;
use walkdir::{WalkDir, DirEntry};

use crate::{
    image_data::{ImageData, InvalidImage},
    output,
    paths,
    scanner::{Scanner, ItemStatus},
    static_file_data::{self, IMAGE_REF_NAME, IMAGE_REF_VERSION},
};

use super::MSG_BAD_REF_DIR;

lazy_static! {
    static ref DESKTOP_INI: OsString = OsString::from("desktop.ini");
}

/// Shorthand for the data taken from the `images.slop` file.
pub type DataMap = HashMap<String, Vec<ImageData>>;

/// Scans through images in the `<pack>/Content/Images/` directory and prints
/// its findings.
pub fn scan_images(images_dir: &PathBuf, ref_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let slop = Slop::open(paths::push(ref_dir, IMAGE_REF_NAME))
        .expect(MSG_BAD_REF_DIR);

    static_file_data::validate_slop(&slop, IMAGE_REF_VERSION);

    let extracted_count: u32 = slop
        .get("!count")
        .expect("expected `images.slop` to have a `!count` keyvalue")
        .parse_into()
        .expect("expected `!count` kv be a string")
        .expect("expected `!count` kv to parse into an unsigned 32 bit integer");

    let mut scanner = Scanner::new("images");
    let data = slop_into_image_data(slop);

    println!();
    output::announce_path("Scanning ", images_dir);

    scanner.scan(
        WalkDir::new(images_dir).into_iter(),
        |f| validate_entry(f, images_dir, &data),
    )?;

    println!();
    output::divider("Scan complete.");
    scanner.print_results(extracted_count);
    Ok(())
}

/// Converts the slop into an equivalent [HashMap] that holds [ImageData] items.
pub fn slop_into_image_data(slop: Slop) -> DataMap {
    let mut data = HashMap::new();

    for (key, value) in slop {
        if key.starts_with('!') {
            continue;
        }

        let values = value.list().expect("expected a list kv");
        let values = values.iter().map(
            |f| f.parse().expect("expected a valid image data string"),
        );

        data.insert(key, values.collect());
    }

    data
}

fn validate_entry(f: &walkdir::Result<DirEntry>, images_dir: &PathBuf, data: &DataMap)
    -> Result<ItemStatus<InvalidImage>, Box<dyn Error>>
{
    //HACK: Rust doesn't like it if I use anything other than match here.
    let entry = match f {
        Ok(e) => e,
        Err(e) => panic!("{e}"),
    };

    let path = entry.path().to_path_buf();

    // First we need to make sure the entry is an image in the first place.

    if path.is_dir() || path.file_name() == Some(&DESKTOP_INI) {
        return Ok(ItemStatus::Ignored);
    }

    let relative_path = path
        .strip_prefix(images_dir)
        .expect("expected path to be a child of `Images/`")
        .to_path_buf();

    let extension = match path.extension() {
        Some(e) => e,
        None => return Ok(ItemStatus::Invalid(InvalidImage::BadName(relative_path))),
    };

    if extension.to_str() != Some("png") {
        return Ok(ItemStatus::Invalid(InvalidImage::BadName(relative_path)));
    }

    // Now that we know the entry is an image, let's properly validate it.
    validate_image(path, relative_path, images_dir, data)
}

pub fn validate_image(path: PathBuf, relative_path: PathBuf, images_dir: &PathBuf, data: &DataMap)
    -> Result<ItemStatus<InvalidImage>, Box<dyn Error>>
{
    let dir = path
        .parent()
        .expect("expected path to have a parent")
        .to_path_buf();

    let dir_key = dir
        .strip_prefix(images_dir)
        .expect("expected path to be a child of `Images/`")
        .to_path_buf();

    let dir_key = paths::path_buf_to_key_name(&dir_key);
    let file_name = paths::file_name(&path);

    let data = match data.get(&dir_key) {
        Some(d) => d,
        None => return Ok(ItemStatus::Invalid(InvalidImage::BadName(relative_path))),
    };

    for data in data {
        match data.validate_image(&dir, file_name) {
            Ok(_) => return Ok(ItemStatus::Valid),
            Err(InvalidImage::BadName(_)) => continue,
            Err(InvalidImage::BadSize(_, b_size, g_size)) => {
                return Ok(ItemStatus::Invalid(InvalidImage::BadSize(relative_path, b_size, g_size)))
            }
        }
    }

    Ok(ItemStatus::Invalid(InvalidImage::BadName(relative_path)))
}
