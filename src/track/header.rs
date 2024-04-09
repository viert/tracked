use chrono::Utc;

use super::error::TrackFileError;

const HEADER_MAGIC_NUMBER: u64 = 0xfb9cfc9b116a158e;
const HEADER_VERSION: u64 = 1;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Header {
  pub magic: u64,
  pub version: u64,
  pub updated_at: u64,
  pub count: u64,
}

impl Header {
  pub fn new() -> Result<Self, TrackFileError> {
    Ok(Self {
      magic: HEADER_MAGIC_NUMBER,
      version: HEADER_VERSION,
      updated_at: Utc::now().timestamp_millis() as u64,
      count: 0,
    })
  }

  pub fn check_magic(&self) -> bool {
    self.magic == HEADER_MAGIC_NUMBER
  }

  pub fn version(&self) -> u64 {
    self.version
  }

  pub fn timestamp(&self) -> u64 {
    self.updated_at
  }

  pub fn count(&self) -> u64 {
    self.count
  }

  pub fn touch(&mut self) {
    self.updated_at = Utc::now().timestamp_millis() as u64;
  }

  pub fn inc(&mut self) {
    self.count += 1;
    self.touch();
  }
}
