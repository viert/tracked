use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[repr(C)]
pub struct TrackPoint {
  pub ts: i64,
  pub lat: f64,
  pub lng: f64,
  pub hdg: i32,
  pub gs: i32,
  pub alt: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackPointCompact {
  pub ts: i64,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "la")]
  pub lat: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "lo")]
  pub lng: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "h")]
  pub hdg: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "g")]
  pub gs: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "a")]
  pub alt: Option<i32>,
}

impl PartialEq for TrackPoint {
  fn eq(&self, other: &Self) -> bool {
    self.lat == other.lat
      && self.lng == other.lng
      && self.hdg == other.hdg
      && self.gs == other.gs
      && self.alt == other.alt
  }
}
