use crate::{config::Config, track::store::TrackStore};
use tokio::sync::RwLock;

pub struct Manager {
  pub store: RwLock<TrackStore>,
}

impl Manager {
  pub fn new(cfg: Config) -> Self {
    let store = TrackStore::new(&cfg.tracks);
    Self {
      store: RwLock::new(store),
    }
  }
}
