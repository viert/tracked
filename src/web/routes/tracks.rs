use crate::{
  manager::Manager,
  track::{
    entry::{TrackPoint, TrackPointCompact},
    interpolate::interpolate_track,
  },
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

  for pdef in req.data.iter() {
    ids.insert(&pdef.track_id);
    count += 1;

    let mut tf = manager.store.open_or_create(&pdef.track_id)?;
    tf.append(&pdef.point)?;
  }

  let status = format!("{} points received, {} tracks updated", count, ids.len());
  Ok(Json(StatusResponse { status }))
}

#[get("/<track_id>?<interpolate>")]
pub async fn show_track(
  track_id: &str,
  interpolate: Option<bool>,
  manager: &State<Arc<Manager>>,
) -> Result<Json<TrackResponse>, APIError> {
  let interpolate = interpolate.unwrap_or(false);

  let tf = manager.store.open(track_id)?;
  let points = tf.read_all()?;
  let points = if interpolate {
    interpolate_track(&points)
  } else {
    points
  };

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

  let tf = manager.store.open(track_id)?;

  let points = tf.read_all()?;
  let points = if interpolate {
    interpolate_track(&points)
  } else {
    points
  };

  let count = points.len();

  let mut compact = vec![];
  if count > 0 {
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

  Ok(Json(TrackCompactResponse {
    track_id: track_id.into(),
    points: compact,
    count,
  }))
}
