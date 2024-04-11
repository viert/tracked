use crate::{
  manager::Manager,
  track::entry::{TrackPoint, TrackPointCompact},
  web::error::APIError,
};
use rocket::{
  get,
  http::{ContentType, Status},
  post,
  response::Responder,
  serde::json::Json,
  State,
};
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

#[derive(Responder)]
#[response(status = 200, content_type = "application/avro")]
pub struct TrackAvroResponse {
  inner: (Status, Vec<u8>),
  header: ContentType,
}

#[post("/", data = "<req>")]
pub async fn update_tracks(
  req: Json<UpdateTracksRequest>,
  manager: &State<Arc<Manager>>,
) -> Result<Json<StatusResponse>, APIError> {
  let mut ids = HashSet::new();
  let mut count = 0;

  for pdef in req.data.iter() {
    ids.insert(&pdef.track_id);
    count += 1;

    let mut tf = manager.store.open_or_create(&pdef.track_id)?;
    tf.append(&pdef.point)?;
  }

  let status = format!("{} points received, {} tracks updated", count, ids.len());
  Ok(Json(StatusResponse { status }))
}

#[get("/<track_id>/json?<interpolate>")]
pub async fn show_track(
  track_id: &str,
  interpolate: Option<bool>,
  manager: &State<Arc<Manager>>,
) -> Result<Json<TrackResponse>, APIError> {
  let interpolate = interpolate.unwrap_or(false);
  let points = manager.store.load_track(track_id, interpolate)?;
  let count = points.len();
  Ok(Json(TrackResponse {
    track_id: track_id.into(),
    points,
    count,
  }))
}

#[get("/<track_id>/compact?<interpolate>")]
pub async fn show_track_compact(
  track_id: &str,
  interpolate: Option<bool>,
  manager: &State<Arc<Manager>>,
) -> Result<Json<TrackCompactResponse>, APIError> {
  let interpolate = interpolate.unwrap_or(false);
  let points = manager.store.load_track_compact(track_id, interpolate)?;
  let count = points.len();

  Ok(Json(TrackCompactResponse {
    track_id: track_id.into(),
    points,
    count,
  }))
}

// #[get("/<track_id>/avro?<interpolate>")]
// pub async fn show_track_avro(
//   track_id: &str,
//   interpolate: Option<bool>,
//   manager: &State<Arc<Manager>>,
// ) -> Result<TrackAvroResponse, APIError> {
//   let interpolate = interpolate.unwrap_or(false);
//   let points = manager.store.load_track_compact(track_id, interpolate)?;
//   let count = points.len();
// }
