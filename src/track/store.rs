use super::{
  entry::{TrackPoint, TrackPointCompact},
  error::TrackFileError,
  interpolate::interpolate_track,
  trackfile::TrackFile,
};
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
