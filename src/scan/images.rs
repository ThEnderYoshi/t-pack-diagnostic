//! The part of `scan` that scans images.

use std::{path::PathBuf, error::Error, ffi::OsString, io::{self, Write}};

use walkdir::{WalkDir, DirEntry};

use crate::{slop::Slop, input, slopx, output};

use super::{ValidatorCrawler, ItemValidation};

pub fn crawl_images(images_dir: &PathBuf, reference_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let reference = Slop::from_file(input::child_path(reference_dir, "images.slop"))
        .expect("expected `-r` to have a file called `images.slop`");

    let mut crawler = ValidatorCrawler::new(slopx::parse_string(&reference, "!count")
        .expect("expected `images.slop` to have a `!count` KV"));

    println!();
    output::announce_path("Scanning", images_dir);
    crawler.crawl(
        &mut WalkDir::new(images_dir).into_iter(),
        |f| validate_item(f, images_dir, &reference),
    )?;

    io::stdout().flush()?;
    println!();
    output::divider("Crawl comlpete.");

    crawler.report_findings();
    Ok(())
}

pub fn is_item_valid(path: &PathBuf, images_dir: &PathBuf, reference: &Slop) -> bool {
    if let Some(extension) = path.extension() {
        if extension.to_str() != Some("png") {
            false
        } else {
            let path = path.strip_prefix(images_dir)
                .expect("Expected path to be a child of `content_dir`")
                .to_path_buf();
            is_item_in_reference(&path, reference)
        }
    } else {
        false
    }
}

fn validate_item(
    f: &walkdir::Result<DirEntry>,
    images_dir: &PathBuf,
    reference: &Slop,
) -> Result<ItemValidation, Box<dyn Error>> {
    let entry = match f {
        Ok(entry) => entry,
        Err(err) => panic!("{err}"), //HACK: Yeeeeah you knowwwww
    };
    let path = entry.path().to_path_buf();

    if path.is_dir() || path.file_name() == Some(&OsString::from("desktop.ini")) {
        Ok(ItemValidation::Ignored)
    } else if is_item_valid(&path, images_dir, reference) {
        Ok(ItemValidation::Valid)
    } else {
        Ok(ItemValidation::Invalid(String::from(path.to_str().expect(input::EXPECT_UTF8_PATH))))
    }
}

fn is_item_in_reference(path: &PathBuf, reference: &Slop) -> bool {
    let dir = path.parent()
        .expect("Expected file to have a parent")
        .to_path_buf();
    let dir = &input::path_buf_to_key_name(&dir);
    let path = path.file_name()
        .expect("Expected path to not end in a root.")
        .to_str()
        .expect(input::EXPECT_UTF8_PATH);

    if let Some(list) = reference.get_list(&dir) {
        list.iter().any(|i| i == path)
    } else {
        false
    }
}
