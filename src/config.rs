use log::LevelFilter;
use serde::Deserialize;
use std::{fs::File, io::Read, path::Path};

#[derive(Debug, Deserialize, Clone)]
pub struct TrackConfig {
  pub folder: String,
}

impl Default for TrackConfig {
  fn default() -> Self {
    Self {
      folder: "/var/lib/tracks".into(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct WebConfig {
  pub port: u16,
  pub host: String,
}

impl Default for WebConfig {
  fn default() -> Self {
    Self {
      port: 9441,
      host: "127.0.0.1".into(),
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct LogConfig {
  pub level: LevelFilter,
}

impl Default for LogConfig {
  fn default() -> Self {
    Self {
      level: LevelFilter::Debug,
    }
  }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
  pub tracks: TrackConfig,
  pub log: LogConfig,
  pub web: WebConfig,
}

pub fn read_in_config(filename: Option<&str>) -> Config {
  let mut filenames = vec!["./tracked.toml", "/etc/tracked/tracked.toml"];
  if let Some(filename) = filename {
    filenames.insert(0, filename);
  }

  for fname in filenames {
    let path = Path::new(fname);
    println!("trying config file {}...", fname);
    if path.is_file() {
      let res = File::open(path);
      if let Err(err) = res {
        println!("error opening config file {}: {}", fname, err);
        continue;
      }
      let mut f = res.unwrap();
      let mut config_raw = String::new();
      let res = f.read_to_string(&mut config_raw);
      if let Err(err) = res {
        println!("error reading config file {}: {}", fname, err);
        continue;
      }
      let res: Result<Config, toml::de::Error> = toml::from_str(&config_raw);
      if let Err(err) = res {
        println!("error parsing config file {}: {}", fname, err);
        continue;
      }
      return res.unwrap();
    }
    println!("config file {} does not exist", fname);
  }
  println!("no config files can be read, using default settings");
  Default::default()
}
