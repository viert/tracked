use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum TrackFileError {
  IOError(std::io::Error),
  InvalidMagicNumber,
  InvalidFileLength(usize, usize),
  InsufficientDataLength(String, usize),
  IndexError(usize),
  SequenceError(i64),
  NotFound(String),
}

impl Display for TrackFileError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TrackFileError::IOError(err) => write!(f, "TrackFileError: {err}"),
      TrackFileError::InvalidMagicNumber => write!(f, "Track file corrupted, invalid magic number"),
      TrackFileError::InvalidFileLength(expected, got) => write!(
        f,
        "Invalid track file length: expected {expected}, got {got}"
      ),
      TrackFileError::InsufficientDataLength(ident, size) => {
        write!(f, "Insufficient data length while parsing {ident}: {size}")
      }
      TrackFileError::IndexError(idx) => {
        write!(f, "Invalid index {idx} while reading track file data")
      }
      TrackFileError::NotFound(filename) => {
        write!(f, "Track file {filename} not found")
      }
      TrackFileError::SequenceError(ts) => {
        write!(
          f,
          "Can't append a point ts={ts}, a bigger ts exists in the track"
        )
      }
    }
  }
}

impl Error for TrackFileError {}

impl From<std::io::Error> for TrackFileError {
  fn from(value: std::io::Error) -> Self {
    Self::IOError(value)
  }
}

#[derive(Debug)]
pub enum MetaFileError {
  IOError(std::io::Error),
  InsufficientDataLength(String, usize),
  NotFound(String),
}

impl Display for MetaFileError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      MetaFileError::IOError(err) => write!(f, "MetaFileError: {err}"),
      MetaFileError::InsufficientDataLength(ident, size) => {
        write!(f, "Insufficient data length while parsing {ident}: {size}")
      }
      MetaFileError::NotFound(filename) => {
        write!(f, "Meta file {filename} not found")
      }
    }
  }
}

impl Error for MetaFileError {}

impl From<std::io::Error> for MetaFileError {
  fn from(value: std::io::Error) -> Self {
    Self::IOError(value)
  }
}
