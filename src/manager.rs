use crate::{config::Config, track::store::TrackStore};
use tokio::sync::RwLock;

pub struct Manager {
  pub store: RwLock<TrackStore>,
}

impl Manager {
  pub fn new(cfg: Config) -> Self {
    let res = TrackStore::new(&cfg.tracks);
    if let Err(err) = res {
      panic!("can't create track store: {err}")
    }

    Self {
      store: RwLock::new(res.unwrap()),
    }
  }
}
