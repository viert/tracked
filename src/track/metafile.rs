use std::{
  fmt::Display,
  fs::{File, OpenOptions},
  io::Write,
  mem::size_of,
  os::unix::fs::FileExt,
  path::Path,
  ptr::slice_from_raw_parts,
};

use chrono::Utc;

use super::error::MetaFileError;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct MetaBlock {
  pub track_count: u64,
  pub point_count: u64,
  pub updated_at: u64,
}

#[derive(Debug)]
pub struct MetaFile {
  file: File,
}

#[allow(clippy::size_of_in_element_count)]
fn to_raw<T: Sized>(obj: &T) -> Vec<u8> {
  let slice = slice_from_raw_parts(obj, size_of::<T>()) as *const [u8];
  let slice = unsafe { &*slice };
  slice.into()
}

fn from_raw<T: Sized + Clone, I: AsRef<str> + Display>(
  data: &[u8],
  ident: I,
) -> std::result::Result<T, MetaFileError> {
  if data.len() < size_of::<T>() {
    Err(MetaFileError::InsufficientDataLength(
      ident.to_string(),
      data.len(),
    ))
  } else {
    let slice = data as *const [u8] as *const T;
    let tp = unsafe { &*slice };
    Ok(tp.clone())
  }
}

impl MetaFile {
  pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, MetaFileError> {
    let path = path.as_ref().to_path_buf();
    let res = OpenOptions::new().write(true).read(true).open(&path);
    match res {
      Ok(file) => {
        let mf = Self { file };
        Ok(mf)
      }
      Err(err) => match err.kind() {
        std::io::ErrorKind::NotFound => {
          let path = path.to_string_lossy().to_string();
          Err(MetaFileError::NotFound(path))
        }
        _ => Err(err.into()),
      },
    }
  }

  pub fn create<P: AsRef<Path>>(path: P) -> Result<Self, MetaFileError> {
    let mut file = OpenOptions::new()
      .create(true)
      .write(true)
      .read(true)
      .open(&path)?;
    let block = MetaBlock {
      track_count: 0,
      point_count: 0,
      updated_at: Utc::now().timestamp_millis() as u64,
    };
    let raw_block = to_raw(&block);
    file.write_all(&raw_block)?;
    Ok(Self { file })
  }

  fn make_buf(&self) -> Vec<u8> {
    let size = size_of::<MetaBlock>();
    let buf = vec![0; size];
    buf
  }

  pub fn read_block(&mut self) -> Result<MetaBlock, MetaFileError> {
    let mut buf = self.make_buf();
    self.file.read_at(&mut buf, 0)?;
    from_raw(&buf, "metablock")
  }

  pub fn write_block(&mut self, block: &MetaBlock) -> Result<(), MetaFileError> {
    let buf = to_raw(block);
    self.file.write_at(&buf, 0)?;
    Ok(())
  }
}
