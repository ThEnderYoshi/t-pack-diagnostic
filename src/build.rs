//! Handles the creation of a copy of a Resource Pack, optimized for publishing.

use std::{collections::HashSet, error::Error, fs, io, path::PathBuf};

use slop_rs::Slop;
use walkdir::WalkDir;

use crate::{
    output,
    paths::{self, EXPECT_UTF8_PATH},
    scan::{images, loc, sounds},
    static_file_data::{self, IMAGE_REF_NAME, IMAGE_REF_VERSION, SOUND_REF_NAME},
};

macro_rules! path_vec {
    () => {
        Vec::<PathBuf>::new()
    };
    ($($strs:expr),*) => {
        vec![$(PathBuf::from($path_str)),*]
    };
}

pub fn build_resource_pack(orig: &PathBuf, target: &PathBuf, refs: &PathBuf)
    -> Result<(), Box<dyn Error>>
{
    output::info("ACTION - Build Resource Pack");

    if !orig.is_dir() {
        panic!("expected `-i` to point to an existing dir");
    }

    prepare_target(&target)?;
    build_root(orig, target)?;
    build_images(orig, target, refs)?;
    build_loc(orig, target)?;
    build_music(orig, target)?;
    build_sounds(orig, target, refs)?;

    output::divider("Build complete");
    output::info("Consider scanning both versions of the pack");
    output::info("to ensure everything was copied properly.");

    Ok(())
}

fn prepare_target(target: &PathBuf) -> io::Result<()> {
    output::divider("Preparing output directory...");
    fs::create_dir_all(target)
}

fn build_root(orig: &PathBuf, target: &PathBuf) -> Result<(), Box<dyn Error>> {
    output::announce("Building", "/");

    if paths::push(orig, "workshop.json").is_file() {
        output::warn("`workshop.json` detected.");
        output::warn("Remember to copy it into the new version.");
    }

    let root_files: HashSet<PathBuf> = HashSet::from(["icon.png".into(), "pack.json".into()]);
    copy_files_if(&orig, &target, false, |p| root_files.contains(p))
}

fn build_images(orig: &PathBuf, target: &PathBuf, refs: &PathBuf) -> Result<(), Box<dyn Error>> {
    output::announce("Building", "/Content/Images");

    let slop = Slop::open(paths::push(refs, IMAGE_REF_NAME))?;
    static_file_data::validate_slop(&slop, IMAGE_REF_VERSION);
    let data = images::slop_into_image_data(slop);

    let orig = paths::push(orig, "Content/Images");
    let target = paths::push(target, "Content/Images");

    copy_files_if(&orig, &target, true, |p| {
        let path = paths::push(&orig, p);
        let result =
            images::validate_image(path, PathBuf::new(), &orig, &data);
        
        if result.is_err() {
            return false;
        }

        result.unwrap().is_valid()
    })
}

fn build_loc(orig: &PathBuf, target: &PathBuf) -> Result<(), Box<dyn Error>> {
    output::announce("Building", "/Content/Localization");
    let orig = paths::push(orig, "Content/Localization");
    let target = paths::push(target, "Content/Localization");

    copy_files_if(&orig, &target, false, |p| if let Some(path) = p.to_str() {
        loc::RE_LOC_FILE_NAME.is_match(path)
    } else {
        false
    })
}

fn build_music(orig: &PathBuf, target: &PathBuf) -> Result<(), Box<dyn Error>> {
    let extensions: HashSet<&str> = HashSet::from(["mp3", "ogg", "wav"]);

    output::announce("Building", "/Content/Music");
    let orig = paths::push(orig, "Content/Music");
    let target = paths::push(target, "Content/Music");

    copy_files_if(&orig, &target, false, |p| {
        let extension = paths::extension(p);
        extensions.contains(extension)
    })
}

fn build_sounds(orig: &PathBuf, target: &PathBuf, refs: &PathBuf) -> Result<(), Box<dyn Error>> {
    output::announce("Building", "/Content/Sounds");
    let orig = paths::push(orig, "Content/Sounds");
    let target = paths::push(target, "Content/Sounds");

    let slop = Slop::open(paths::push(refs, SOUND_REF_NAME))?;
    let data = sounds::slop_into_sound_data(slop);

    copy_files_if(&orig, &target, true, |p| {
        let path = paths::push(&orig, p);
        let result =
            sounds::validate_sound(path, PathBuf::new(), &orig, &data);

        match result {
            Ok(s) => s.is_valid(),
            Err(_) => false,
        }
    })
}

fn copy_files_if<F>(from: &PathBuf, to: &PathBuf, recursive: bool, should_copy: F)
    -> Result<(), Box<dyn Error>>
where
    F: for<'a> Fn(&'a PathBuf) -> bool,
{
    let walk_dir = if recursive {
        WalkDir::new(from)
    } else {
        WalkDir::new(from).max_depth(1)
    };

    let mut valid_files = path_vec![];

    for entry in walk_dir {
        let entry = entry?;
        let path = entry.path().strip_prefix(from)?.to_path_buf();

        if path.is_dir() {
            continue;
        }

        if path.to_str().expect(EXPECT_UTF8_PATH).is_empty() {
            continue;
        }

        if should_copy(&path) {
            valid_files.push(path);
        }
    }

    if valid_files.is_empty() {
        output::info(&format!("No files in `{from:?}`."));
        return Ok(());
    }

    fs::create_dir_all(to)?;

    for path in valid_files {
        let orig = paths::push(from, &path);
        let target = paths::push(to, &path);

        if let Some(dir) = target.parent() {
            if !dir.is_dir() {
                fs::create_dir_all(dir)?;
            }
        }

        fs::copy(orig, target)?;
    }

    Ok(())
}
