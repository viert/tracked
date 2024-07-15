use log::{debug, error, info};
use walkdir::WalkDir;

use super::{
  entry::{TrackPoint, TrackPointCompact},
  error::{MetaFileError, TrackFileError},
  interpolate::interpolate_track,
  metafile::{MetaBlock, MetaFile},
  trackfile::TrackFile,
};
use crate::config::TrackConfig;
use std::{
  fs::create_dir_all,
  path::{Path, PathBuf},
};

const SUBKEY_LENGTH: usize = 3;
const NESTING_LEVEL: usize = 2;

#[derive(Debug)]
pub struct TrackStore {
  folder: String,
  metafile: MetaFile,
}

fn inspect_trackfiles_meta(folder: &str) -> (u64, u64) {
  let mut tracks_count: u64 = 0;
  let mut points_count: u64 = 0;

  info!("Loading tracks metadata, this might take a while");

  for entry in WalkDir::new(folder) {
    if let Ok(entry) = entry {
      let md = entry.metadata();
      if let Ok(md) = md {
        if md.is_file() {
          let res = TrackFile::open(entry.path());
          if let Ok(tf) = res {
            let res = tf.count();
            if let Err(err) = res {
              error!(
                "TrackFile {} is corrupt: {err}",
                entry.file_name().to_str().unwrap()
              );
            } else {
              tracks_count += 1;
              points_count += res.unwrap();
            }
          }
          if tracks_count % 5000 == 0 {
            debug!("{tracks_count} tracks inspected")
          }
        }
      }
    }
  }
  debug!("found {tracks_count} tracks with total {points_count} points");
  (tracks_count, points_count)
}

fn setup_meta(folder: &str) -> Result<MetaFile, MetaFileError> {
  let path = Path::new(folder).join(".meta");
  let res = MetaFile::open(&path);
  match res {
    Ok(mf) => Ok(mf),
    Err(err) => match err {
      MetaFileError::NotFound(_) => {
        let mut mf = MetaFile::create(&path)?;
        let (track_count, point_count) = inspect_trackfiles_meta(folder);
        let mut block = mf.read_block()?;
        block.track_count = track_count;
        block.point_count = point_count;
        mf.write_block(&block)?;
        Ok(mf)
      }
      _ => Err(err),
    },
  }
}

impl TrackStore {
  pub fn new(cfg: &TrackConfig) -> Result<Self, MetaFileError> {
    let metafile = setup_meta(&cfg.folder)?;
    let ts = Self {
      folder: cfg.folder.clone(),
      metafile,
    };
    Ok(ts)
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

  pub fn get_metablock(&mut self) -> Result<MetaBlock, MetaFileError> {
    self.metafile.read_block()
  }

  fn inc_track_count(&mut self) {
    let res = self.metafile.read_block();
    if let Err(err) = res {
      error!("error reading metablock: {err}");
    } else {
      let mut block = res.unwrap();
      block.track_count += 1;
      let res = self.metafile.write_block(&block);
      if let Err(err) = res {
        error!("error writing metablock: {err}");
      }
    }
  }

  fn inc_point_count(&mut self) {
    let res = self.metafile.read_block();
    if let Err(err) = res {
      error!("error reading metablock: {err}");
    } else {
      let mut block = res.unwrap();
      block.point_count += 1;
      let res = self.metafile.write_block(&block);
      if let Err(err) = res {
        error!("error writing metablock: {err}");
      }
    }
  }

  fn open_or_create(&mut self, track_id: &str) -> Result<TrackFile, TrackFileError> {
    let target_dir = self.target_directory(track_id);
    let path = target_dir.join(format!("{track_id}.bin"));
    let res = TrackFile::open(&path);

    match res {
      Ok(tf) => Ok(tf),
      Err(err) => match err {
        TrackFileError::NotFound(_) => {
          create_dir_all(target_dir)?;
          let tf = TrackFile::create(&path)?;
          self.inc_track_count();
          Ok(tf)
        }
        _ => Err(err),
      },
    }
  }

  fn open(&self, track_id: &str) -> Result<TrackFile, TrackFileError> {
    let path = self.target_directory(track_id);
    let path = path.join(format!("{track_id}.bin"));
    TrackFile::open(path)
  }

  pub fn append(
    &mut self,
    track_id: &str,
    entry: &TrackPoint,
    create_if_not_exist: bool,
  ) -> Result<(), TrackFileError> {
    let mut tf = if create_if_not_exist {
      self.open_or_create(track_id)?
    } else {
      self.open(track_id)?
    };

    let appended = tf.append(entry)?;
    if appended {
      self.inc_point_count();
    }

    Ok(())
  }

  pub fn load_track(
    &self,
    track_id: &str,
    interpolate: bool,
    after: Option<i64>,
  ) -> Result<Vec<TrackPoint>, TrackFileError> {
    let tf = self.open(track_id)?;
    let points = tf.read_all()?;
    let points = if interpolate {
      interpolate_track(&points)
    } else {
      points
    };

    // TODO: stop reading the entire file when `after` is set
    let points = if let Some(after) = after {
      points.into_iter().filter(|p| p.ts > after).collect()
    } else {
      points
    };

    Ok(points)
  }

  pub fn load_track_compact(
    &self,
    track_id: &str,
    interpolate: bool,
    after: Option<i64>,
  ) -> Result<Vec<TrackPointCompact>, TrackFileError> {
    let points = self.load_track(track_id, interpolate, after)?;
    let mut compact = vec![];
    if points.len() > 0 {
      let mut curr = points.get(0).unwrap();
      compact.push(TrackPointCompact {
        ts: curr.ts,
        lat: Some(curr.lat),
        lng: Some(curr.lng),
        hdg: Some(curr.hdg),
        alt: Some(curr.alt),
        gs: Some(curr.gs),
      });

      for point in points[1..].iter() {
        let ts = point.ts - curr.ts;
        let lat = if point.lat != curr.lat {
          Some(point.lat)
        } else {
          None
        };
        let lng = if point.lng != curr.lng {
          Some(point.lng)
        } else {
          None
        };
        let hdg = if point.hdg != curr.hdg {
          Some(point.hdg)
        } else {
          None
        };
        let alt = if point.alt != curr.alt {
          Some(point.alt)
        } else {
          None
        };
        let gs = if point.gs != curr.gs {
          Some(point.gs)
        } else {
          None
        };
        compact.push(TrackPointCompact {
          ts,
          lat,
          lng,
          hdg,
          alt,
          gs,
        });
        curr = point;
      }
    }
    Ok(compact)
  }
}
