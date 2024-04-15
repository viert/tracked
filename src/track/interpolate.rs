use super::entry::TrackPoint;
use splines::{Interpolation, Key, Spline};

fn create_spline(
  dataset: &[TrackPoint],
  extract: Box<dyn Fn(&TrackPoint) -> f64>,
) -> Spline<f64, f64> {
  let keys = dataset
    .iter()
    .map(|p| Key::new(p.ts as f64, extract(p), Interpolation::CatmullRom));
  Spline::from_iter(keys)
}

pub fn interpolate_track(points: &[TrackPoint]) -> Vec<TrackPoint> {
  if points.len() < 3 {
    points.to_vec()
  } else {
    let lat_spline = create_spline(points, Box::new(|p| p.lat));
    let lng_spline = create_spline(points, Box::new(|p| p.lng));
    let alt_spline = create_spline(points, Box::new(|p| p.alt as f64));
    let gs_spline = create_spline(points, Box::new(|p| p.gs as f64));
    let hdg_spline = create_spline(points, Box::new(|p| p.hdg as f64));

    let step_ms = 1000;
    let first_ts = points.first().unwrap().ts / step_ms;
    let last_ts = points.last().unwrap().ts / step_ms;

    let mut interpolated: Vec<TrackPoint> = (first_ts..last_ts)
      .map(|ts| {
        let ts = ts * step_ms;
        let lat = lat_spline.sample(ts as f64);
        let lng = lng_spline.sample(ts as f64);
        let hdg = hdg_spline.sample(ts as f64);
        let gs = gs_spline.sample(ts as f64);
        let alt = alt_spline.sample(ts as f64);

        if [lat, lng, hdg, gs, alt].iter().any(|item| item.is_none()) {
          None
        } else {
          Some(TrackPoint {
            ts,
            lat: lat.unwrap(),
            lng: lng.unwrap(),
            hdg: hdg.unwrap() as i32,
            gs: gs.unwrap() as i32,
            alt: alt.unwrap() as i32,
          })
        }
      })
      .flatten()
      .collect();
    interpolated.push(points.last().cloned().unwrap());
    interpolated
  }
}
