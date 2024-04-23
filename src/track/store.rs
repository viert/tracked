use log::{debug, error};
use walkdir::WalkDir;

use super::{
  entry::{TrackPoint, TrackPointCompact},
  error::TrackFileError,
  interpolate::interpolate_track,
  trackfile::TrackFile,
};
use crate::config::TrackConfig;
use std::{
  collections::{hash_map::Entry, HashMap, HashSet},
  io::{self, Write},
  path::PathBuf,
};

const SUBKEY_LENGTH: usize = 3;
const NESTING_LEVEL: usize = 2;

#[derive(Debug)]
pub struct TrackStore {
  folder: String,
  track_ids: HashSet<String>,
  point_counters: HashMap<String, u64>,
}

impl TrackStore {
  pub fn new(cfg: &TrackConfig) -> Self {
    let mut tc = Self {
      folder: cfg.folder.clone(),
      track_ids: HashSet::new(),
      point_counters: HashMap::new(),
    };

    let res = tc.load_stats();
    if let Err(err) = res {
      error!("error reading track store stats: {err}")
    }
    tc
  }

  fn load_stats(&mut self) -> Result<(), TrackFileError> {
    let mut count: usize = 0;
    let mut total: u64 = 0;

    print!("Collecting tracks metadata");
    io::stdout().flush()?;

    for entry in WalkDir::new(&self.folder) {
      if let Ok(entry) = entry {
        let md = entry.metadata();
        if let Ok(md) = md {
          if md.is_file() {
            let res = TrackFile::open(entry.path());
            if let Ok(tf) = res {
              let file_name = entry
                .path()
                .file_name()
                .and_then(|file_name| file_name.to_str());

              if let Some(file_name) = file_name {
                let track_id = file_name[..4].to_owned();
                let c = tf.count().unwrap_or(0);
                self.track_ids.insert(track_id.clone());
                self.point_counters.insert(track_id, c);
                total += c;
              }
            }

            count += 1;
            if count % 5000 == 0 {
              print!(".");
              io::stdout().flush()?;
            }
          }
        }
      }
    }
    println!();
    debug!(
      "found {} tracks with total {} points",
      self.track_ids.len(),
      total
    );
    Ok(())
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

  fn open_or_create(&mut self, track_id: &str) -> Result<TrackFile, TrackFileError> {
    let path = self.target_directory(track_id);
    let path = path.join(format!("{track_id}.bin"));
    let res = TrackFile::open(&path);
    match res {
      Ok(tf) => Ok(tf),
      Err(err) => match err {
        TrackFileError::NotFound(_) => {
          let tf = TrackFile::create(&path)?;
          self.point_counters.insert(track_id.into(), 0);
          self.track_ids.insert(track_id.into());
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

    tf.append(entry)?;

    match self.point_counters.entry(track_id.into()) {
      Entry::Occupied(mut e) => e.insert(e.get() + 1),
      Entry::Vacant(_) => unreachable!(),
    };
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

  pub fn get_track_ids(&self) -> Vec<String> {
    self.track_ids.iter().cloned().collect()
  }

  pub fn get_point_counters(&self) -> HashMap<String, u64> {
    self.point_counters.clone()
  }
}
