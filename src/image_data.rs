use std::{path::PathBuf, fmt::Display, str::FromStr};

use imagesize::{ImageSize, ImageError};

use crate::paths;

/// Error returned in [ImageData]'s implementation of [FromStr].
#[derive(Debug, PartialEq, Eq)]
pub struct ParseImageDataError;

/// Data about an individual image that is relevant to the reference file.
pub struct ImageData {
    /// The name of the file.
    /// The path leading to the parent directory is stored elsewhere.
    pub file_name: String,

    /// The Image's size.
    pub size: ImageSize,
}

impl ImageData {
    /// Creates an [ImageData] struct from an image file.
    pub fn open(path: &PathBuf) -> Result<Self, ImageError> {
        let file_name = paths::file_name(path).to_string();
        let size = imagesize::size(path)?;

        Ok(Self { file_name, size })
    }

    /// Returns `true` if the image is valid.
    /// 
    /// **Note:** Has to open the image file to check its size.
    /// 
    /// ## Panics
    /// 
    /// Panics if [imagesize] fails to open the image.
    pub fn validate_image(&self, dir: &PathBuf, file_name: &str) -> Result<(), InvalidImage> {
        let path = paths::push(dir, file_name);

        if self.file_name != file_name {
            return Err(InvalidImage::BadName(path));
        }

        let size = imagesize::size(&path)
            .expect(&format!("failed to open {path:?}"));

        if size == self.size {
            Ok(())
        } else {
            Err(InvalidImage::BadSize(path, size, self.size))
        }
    }
}

impl Display for ImageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}x{}", self.file_name, self.size.width, self.size.height)
    }
}

impl FromStr for ImageData {
    type Err = ParseImageDataError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (file_name, size) = s
            .split_once(':')
            .ok_or(ParseImageDataError)?;

        let file_name = file_name.to_string();

        let (width, height) = size
            .split_once('x')
            .ok_or(ParseImageDataError)?;

        let size = ImageSize {
            width: width.parse().map_err(|_| ParseImageDataError)?,
            height: height.parse().map_err(|_| ParseImageDataError)?,
        };

        Ok(Self { file_name, size })
    }
}

/// The possible kinds of invalid images.
/// These aren't fatal; instead, all errors of this type are collected and
/// displayed once the scan ends.
pub enum InvalidImage {
    BadName(PathBuf),
    BadSize(PathBuf, ImageSize, ImageSize),
}

impl Display for InvalidImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadName(p) => {
                let file_name = paths::file_name(p);
                write!(
                    f,
                    "{:?}\t: The name `{file_name}` was not found in the image reference.",
                    display_path_pretty(p),
                )
            }
            Self::BadSize(p, bad_s, good_s) => {
                let (bw, bh) = (bad_s.width, bad_s.height);
                let (gw, gh) = (good_s.width, good_s.height);
                write!(
                    f,
                    "{:?}\t: Wrong image size {bw}\u{00D7}{bh}. (expected {gw}\u{00D7}{gh})",
                    display_path_pretty(p),
                )
            }
        }
    }
}

fn display_path_pretty(path: &PathBuf) -> String {
    path.to_str().unwrap_or("").replace("\\", "/")
}
