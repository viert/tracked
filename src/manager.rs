use crate::{config::Config, track::store::TrackStore};

pub struct Manager {
  pub store: TrackStore,
}

impl Manager {
  pub fn new(cfg: Config) -> Self {
    let store = TrackStore::new(&cfg.tracks);
    Self { store }
  }
}
