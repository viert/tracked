use std::sync::Arc;

use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use tracked::{
  config::read_in_config,
  manager::Manager,
  web::{
    error::{catch404, catch500},
    routes::tracks::{show_track, show_track_compact, update_tracks},
  },
};

#[macro_use]
extern crate rocket;

#[launch]
async fn rocket() -> _ {
  let config = read_in_config(None);
  let figment = rocket::Config::figment()
    .merge(("port", config.web.port))
    .merge(("limits.json", "10MiB"));

  TermLogger::init(
    config.log.level,
    Config::default(),
    TerminalMode::Stdout,
    ColorChoice::Always,
  )
  .unwrap();

  let m = Manager::new(config);
  let m = Arc::new(m);

  rocket::custom(figment)
    .manage(m)
    .mount(
      "/api/v1/tracks",
      routes![update_tracks, show_track, show_track_compact],
    )
    .register("/", catchers![catch404, catch500])
}
