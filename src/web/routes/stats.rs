use std::sync::Arc;

use crate::{manager::Manager, web::error::APIError};
use rocket::{get, response::content, State};

#[get("/metrics")]
pub async fn get_metrics(
  manager: &State<Arc<Manager>>,
) -> Result<content::RawText<String>, APIError> {
  let mut store = manager.store.write().await;
  let block = store.get_metablock()?;

  let response = format!(
    r#"# HELP tracked_track_count number of tracks currently stored
# TYPE tracked_track_count gauge
tracked_track_count {}

# HELP tracked_point_count number of points currently stored in all track files
# TYPE tracked_point_count gauge
tracked_point_count {}
"#,
    block.track_count, block.point_count
  );
  let response = content::RawText(response);
  Ok(response)
}
