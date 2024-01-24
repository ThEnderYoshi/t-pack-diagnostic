//! Static data relating to files, such as file names and versions.

use slop_rs::Slop;

/// The type for file versions.
/// Saved as an alias in case the version numbers become too large.
pub type Version = u8;

/// The key of the KV in a [Slop] that represents its version.
pub const VERSION_KEY: &str = "!version";

/// The file name of the image reference file.
pub const IMAGE_REF_NAME: &str = "images.slop";

/// The version of the image reference file.
pub const IMAGE_REF_VERSION: Version = 1;

/// The file name if the CSV file with all the localization entries.
pub const ALL_LOC_CSV_NAME: &str = "Loc.csv";

/// The file name of the text file with only the translation keys.
pub const LOC_REF_NAME: &str = "loc_keys.txt";

/// The file name of the text file with the music file names.
pub const MUSIC_REF_NAME: &str = "music.txt";

/// The file name of the sound reference file.
pub const SOUND_REF_NAME: &str = "sounds.slop";

pub const SOUND_REF_VERSION: Version = 0;

/// The maximum amount of items that can be displayed by lists.
pub const MAX_LIST_SIZE: usize = 100;

/// Asserts the the [Slop]'s `!version` KV matches the provided version.
///
/// ## Panics
///
/// Panics if the assertion fails.
pub fn validate_slop(slop: &Slop, expected_version: Version) {
    let value = slop
        .get(VERSION_KEY)
        .expect("expected slop file to have a `!version` keyvalue");

    let file_version: Version = value
        .string()
        .expect("expected the slop file's `!version` keyvalue to be a string kv")
        .parse()
        .expect("expected the slop file's `!version` kv to be a positive integer");

    if file_version != expected_version {
        panic!("expected file version {expected_version}, but got {file_version}");
    }
}
