use rocket::{
  catch,
  http::Status,
  response::{status::Custom, Responder},
  serde::json::json,
};

use crate::track::error::{MetaFileError, TrackFileError};

#[derive(Debug)]
pub struct APIError {
  pub code: u16,
  pub message: String,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for APIError {
  fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
    let resp = Custom(
      Status { code: self.code },
      json!({
        "error": self.message
      }),
    );
    resp.respond_to(request)
  }
}

impl From<TrackFileError> for APIError {
  fn from(value: TrackFileError) -> Self {
    match value {
      TrackFileError::IOError(err) => {
        APIError::internal_server_error(Some(format!("IOERROR: {err}")))
      }
      TrackFileError::InvalidMagicNumber => {
        APIError::internal_server_error(Some("invalid magic number".into()))
      }
      TrackFileError::InvalidFileLength(exp, real) => APIError::internal_server_error(Some(
        format!("Invalid file length, expected {exp}, got {real}"),
      )),
      TrackFileError::InsufficientDataLength(_, _) => {
        APIError::internal_server_error(Some("Error unmarshalling track data".into()))
      }
      TrackFileError::IndexError(idx) => {
        APIError::internal_server_error(Some(format!("error reading track file at index {idx}")))
      }
      TrackFileError::NotFound(msg) => APIError::not_found(&msg),
      TrackFileError::SequenceError(_) => APIError::bad_request(&format!("{value}")),
    }
  }
}

impl From<MetaFileError> for APIError {
  fn from(value: MetaFileError) -> Self {
    APIError::internal_server_error(Some(format!("metafile error: {value}")))
  }
}

impl APIError {
  pub fn new(code: u16, message: &str) -> Self {
    Self {
      code,
      message: message.into(),
    }
  }

  pub fn not_found(message: &str) -> Self {
    Self::new(404, message)
  }

  pub fn bad_request(message: &str) -> Self {
    Self::new(403, message)
  }

  pub fn internal_server_error(message: Option<String>) -> Self {
    let message = message.unwrap_or("internal server error".into());
    APIError::new(500, &message)
  }
}

#[catch(404)]
pub fn catch404() -> APIError {
  APIError::not_found("not found")
}

#[catch(500)]
pub fn catch500() -> APIError {
  APIError::internal_server_error(None)
}
