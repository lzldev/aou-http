use crate::request::{HeaderOptions, RequestHead, RequestHeaders, VecOffset};

use super::ParserResult;

#[derive(Debug, thiserror::Error)]
pub enum ParserStateError {
  #[error("Parser state can only be converted into a parserResult if it's a body")]
  NotBody,
}

#[derive(Debug)]
pub enum ParserState {
  Start {
    read_until: Option<usize>,
  },
  Head {
    cursor: usize,
    read_until: usize,
    head: RequestHead,
  },
  Headers {
    cursor: usize,
    read_until: usize,
    head: RequestHead,
    headers: RequestHeaders,
  },
  Body {
    cursor: usize,
    read_until: usize,
    head: RequestHead,
    headers: RequestHeaders,
    header_options: HeaderOptions,
    body: VecOffset,
  },
}

impl ParserState {
  pub fn read_until(&self) -> usize {
    match self {
      ParserState::Start { read_until } => read_until.unwrap_or(0),
      ParserState::Head { read_until, .. } => *read_until,
      ParserState::Headers { read_until, .. } => *read_until,
      ParserState::Body { read_until, .. } => *read_until,
    }
  }

  pub fn is_body(&self) -> bool {
    match self {
      ParserState::Body { .. } => true,
      _ => false,
    }
  }

  pub fn into_parser_result(self, buf: Vec<u8>) -> Result<ParserResult, ParserStateError> {
    match self {
      ParserState::Body {
        head,
        headers,
        header_options,
        body,
        ..
      } => Ok(ParserResult {
        buf,
        head,
        headers,
        header_options,
        body,
      }),
      _ => Err(ParserStateError::NotBody),
    }
  }
}
