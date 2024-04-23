use std::sync::Arc;

use crate::{manager::Manager, web::error::APIError};
use rocket::{get, response::content, State};

#[get("/metrics")]
pub async fn get_metrics(
  manager: &State<Arc<Manager>>,
) -> Result<content::RawText<String>, APIError> {
  let store = manager.store.read().await;
  let track_count = store.get_track_ids().len();
  let point_count = store
    .get_point_counters()
    .values()
    .copied()
    .reduce(|acc, count| acc + count)
    .unwrap();

  let response = format!(
    r#"track_count {track_count}
point_count {point_count}"#
  );
  let response = content::RawText(response);
  Ok(response)
}
