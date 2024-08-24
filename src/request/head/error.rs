#[derive(thiserror::Error, Debug)]
pub enum RequestHeadParseError {
  #[error("Request Head Not found")]
  NoHead,
  #[error("Method not found")]
  NoMethod,
  #[error("Path not found")]
  NoPath,
  #[error("Http Version not found")]
  NoHTTPVersion,
  #[error("Invalid HTTP Version")]
  InvalidHTTPVersion,
}
