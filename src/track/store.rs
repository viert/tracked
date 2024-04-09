use super::{error::TrackFileError, trackfile::TrackFile};
use crate::config::TrackConfig;
use std::{fs, path::PathBuf};

const SUBKEY_LENGTH: usize = 3;
const NESTING_LEVEL: usize = 2;

#[derive(Debug)]
pub struct TrackStore {
  folder: String,
}

impl TrackStore {
  pub fn new(cfg: &TrackConfig) -> Self {
    Self {
      folder: cfg.folder.clone(),
    }
  }

  fn target_directory(&self, track_id: &str) -> PathBuf {
    let mut path = PathBuf::from(&self.folder);
    let hash = md5::compute(track_id);
    let hash = format!("{:x}", hash);

    for i in 0..NESTING_LEVEL {
      let subkey = &hash[i * SUBKEY_LENGTH..(i + 1) * SUBKEY_LENGTH];
      path = path.join(subkey);
    }
    path
  }

  pub fn open_or_create(&self, track_id: &str) -> Result<TrackFile, TrackFileError> {
    let path = self.target_directory(track_id);
    fs::create_dir_all(&path)?;
    let path = path.join(format!("{track_id}.bin"));
    TrackFile::new(path)
  }

  pub fn open(&self, track_id: &str) -> Result<TrackFile, TrackFileError> {
    let path = self.target_directory(track_id);
    let path = path.join(format!("{track_id}.bin"));
    TrackFile::open(path)
  }
}
