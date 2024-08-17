use crate::{
  manager::Manager,
  track::entry::{TrackPoint, TrackPointCompact},
  web::error::APIError,
};
use rocket::{get, post, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};

#[derive(Debug, Deserialize)]
pub struct PointDef {
  pub track_id: String,
  pub point: TrackPoint,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTracksRequest {
  pub data: Vec<PointDef>,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
  status: String,
}

#[derive(Debug, Serialize)]
pub struct TrackResponse {
  pub track_id: String,
  pub points: Vec<TrackPoint>,
  pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct TrackCompactResponse {
  pub track_id: String,
  pub points: Vec<TrackPointCompact>,
  pub count: usize,
}

#[post("/", data = "<req>")]
pub async fn update_tracks(
  req: Json<UpdateTracksRequest>,
  manager: &State<Arc<Manager>>,
) -> Result<Json<StatusResponse>, APIError> {
  let mut ids = HashSet::new();
  let mut count = 0;
  let mut store = manager.store.write().await;

  for pdef in req.data.iter() {
    ids.insert(&pdef.track_id);
    count += 1;
    store.append(&pdef.track_id, &pdef.point, true)?;
  }

  let status = format!("{} points received, {} tracks updated", count, ids.len());
  Ok(Json(StatusResponse { status }))
}

#[get("/<track_id>/json?<interpolate>&<after>")]
pub async fn show_track(
  track_id: &str,
  interpolate: Option<bool>,
  after: Option<i64>,
  manager: &State<Arc<Manager>>,
) -> Result<Json<TrackResponse>, APIError> {
  let store = manager.store.read().await;
  let interpolate = interpolate.unwrap_or(false);
  let points = store.load_track(track_id, interpolate, after)?;
  let count = points.len();
  Ok(Json(TrackResponse {
    track_id: track_id.into(),
    points,
    count,
  }))
}

#[get("/<track_id>/compact?<interpolate>&<after>")]
pub async fn show_track_compact(
  track_id: &str,
  interpolate: Option<bool>,
  after: Option<i64>,
  manager: &State<Arc<Manager>>,
) -> Result<Json<TrackCompactResponse>, APIError> {
  let store = manager.store.read().await;
  let interpolate = interpolate.unwrap_or(false);
  let points = store.load_track_compact(track_id, interpolate, after)?;
  // TODO: stop reading the entire file when `after` is set
  let points = if let Some(after) = after {
    points.into_iter().filter(|p| p.ts > after).collect()
  } else {
    points
  };

  let count = points.len();

  Ok(Json(TrackCompactResponse {
    track_id: track_id.into(),
    points,
    count,
  }))
}
