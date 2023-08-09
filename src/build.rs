//! Handles the `build` action.

use std::{path::PathBuf, error::Error, io, fs, ffi::OsStr};

use walkdir::WalkDir;

use crate::{output, input, slop::Slop, scan};

pub fn build_resource_pack(
    orig_root: &PathBuf,
    copy_root: &PathBuf,
    refs: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    output::info("ACTION - Build Resource Pack");

    if !orig_root.is_dir() {
        panic!("expected `-i` to point to an existing dir");
    }

    prepare_copy_dir(&copy_root)?;
    copy_root_dir(orig_root, copy_root)?;
    copy_images_dir(orig_root, copy_root, refs)?;
    copy_loc_dir(orig_root, copy_root)?;
    copy_music_dir(orig_root, copy_root)?;
    copy_sounds_dir(orig_root, copy_root)?;

    output::divider("Build complete.");
    println!("Consider scanning both versions of the pack to\
        ensure everything was copied properly.");

    Ok(())
}

macro_rules! path_vec {
    () => {
        Vec::<PathBuf>::new()
    };
    ($($path_str:expr),*) => {
        vec![$(PathBuf::from($path_str)),*]
    };
}

fn prepare_copy_dir(copy_root: &PathBuf) -> io::Result<()> {
    output::divider("Preparing output directory...");
    fs::create_dir_all(copy_root)
}

fn copy_root_dir(orig_root: &PathBuf, copy_root: &PathBuf) -> Result<(), Box<dyn Error>> {
    output::announce("Copying", "/");

    if input::child_path(orig_root, "workshop.json").is_file() {
        output::warn("workshop.json detected.\n      \
            Remember to copy-paste it to the result dir.");
    }

    let root_files = path_vec!["icon.png", "pack.json"];
    copy_files_if(
        &orig_root,
        &copy_root,
        false,
        |p| root_files.iter().any(|i| i == p),
    )
}

fn copy_images_dir(
    orig_root: &PathBuf,
    copy_root: &PathBuf,
    refs: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    output::announce("Copying", "/Content/Images");

    let slop = Slop::from_file(input::child_path(refs, "images.slop"))?;
    let orig_images = input::child_path(orig_root, "Content/Images");
    let copy_images = input::child_path(copy_root, "Content/Images");

    copy_files_if(
        &orig_images,
        &copy_images,
        true,
        |p| {
            let path = input::child_path(&orig_images, p);
            scan::images::is_item_valid(&path, &orig_images, &slop)
        },
    )
}

fn copy_loc_dir(
    orig_root: &PathBuf,
    copy_root: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    output::announce("Copying", "/Content/Localization");
    let orig_loc = input::child_path(orig_root, "Content/Localization");
    let copy_loc = input::child_path(copy_root, "Content/Localization");

    copy_files_if(
        &orig_loc,
        &copy_loc,
        false,
        |p| if let Some(path) = p.to_str() {
            scan::loc::RE_LOC_FILE_NAME.is_match(path)
        } else {
            false
        },
    )
}

fn copy_music_dir(orig_root: &PathBuf, copy_root: &PathBuf) -> Result<(), Box<dyn Error>> {
    const EXTENSIONS: [&str; 3] = ["mp3", "ogg", "wav"];
    output::announce("Copying", "/Content/Music");
    let orig_music = input::child_path(orig_root, "Content/Music");
    let copy_music = input::child_path(copy_root, "Content/Music");

    copy_files_if(
        &orig_music,
        &copy_music,
        false,
        |p| EXTENSIONS.iter().any(|e| Some(OsStr::new(e)) == p.extension()),
    )
}

fn copy_sounds_dir(orig_root: &PathBuf, copy_root: &PathBuf) -> Result<(), Box<dyn Error>> {
    output::announce("Copying", "/Content/Sounds");
    let orig_sounds = input::child_path(orig_root, "Content/Sounds");
    let copy_sounds = input::child_path(copy_root, "Content/Sounds");

    copy_files_if(
        &orig_sounds,
        &copy_sounds,
        false,
        |p| Some(OsStr::new("xnb")) == p.extension(),
    )
}

fn copy_files_if<F>(
    from: &PathBuf,
    to: &PathBuf,
    is_recursive: bool,
    should_copy: F,
) -> Result<(), Box<dyn Error>>
where
    F: for <'a> Fn(&'a PathBuf) -> bool,
{
    let mut valid_files = path_vec![];
    let walk_dir = if is_recursive {
        WalkDir::new(from)
    } else {
        WalkDir::new(from).max_depth(1)
    };

    for entry in walk_dir {
        let entry = entry?;
        let path = entry.path().strip_prefix(from)?.to_path_buf();

        if should_copy(&path) {
            valid_files.push(path);
        }
    }

    if valid_files.is_empty() {
        println!("No files in {from:?}!");
    } else {
        fs::create_dir_all(to)?;
        for path in valid_files {
            let orig = input::child_path(from, &path);
            let copy = input::child_path(to, &path);

            if let Some(path) = copy.parent() {
                if !path.is_dir() {
                    fs::create_dir_all(path)?;
                }
            }
            fs::copy(orig, copy)?;
        }
    }

    Ok(())
}
