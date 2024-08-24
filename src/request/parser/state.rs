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
    header_options: HeaderOptions,
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

pub struct FullParserState {
  pub read_until: Option<usize>,
  pub cursor: Option<usize>,
  pub head: Option<RequestHead>,
  pub headers: Option<RequestHeaders>,
  pub header_options: Option<HeaderOptions>,
  pub body: Option<VecOffset>,
}
impl Default for FullParserState {
  fn default() -> Self {
    Self {
      cursor: Default::default(),
      read_until: Default::default(),
      head: Default::default(),
      headers: Default::default(),
      header_options: Default::default(),
      body: Default::default(),
    }
  }
}

impl FullParserState {
  pub fn from_state(state: ParserState) -> FullParserState {
    match state {
      ParserState::Start { read_until } => FullParserState {
        read_until,
        ..Default::default()
      },
      ParserState::Head {
        cursor,
        read_until,
        head,
      } => FullParserState {
        cursor: Some(cursor),
        read_until: Some(read_until),
        head: Some(head),
        ..Default::default()
      },
      ParserState::Headers {
        cursor,
        read_until,
        head,
        headers,
        header_options,
      } => FullParserState {
        cursor: Some(cursor),
        read_until: Some(read_until),
        head: Some(head),
        headers: Some(headers),
        header_options: Some(header_options),
        ..Default::default()
      },
      ParserState::Body {
        cursor,
        read_until,
        head,
        headers,
        header_options,
        body,
      } => FullParserState {
        cursor: Some(cursor),
        read_until: Some(read_until),
        head: Some(head),
        headers: Some(headers),
        header_options: Some(header_options),
        body: Some(body),
      },
    }
  }
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

  pub fn head(&self) -> Option<&RequestHead> {
    match self {
      ParserState::Head { head, .. }
      | ParserState::Headers { head, .. }
      | ParserState::Body { head, .. } => Some(head),
      _ => None,
    }
  }

  pub fn headers(&self) -> Option<&RequestHeaders> {
    match self {
      ParserState::Headers { headers, .. } | ParserState::Body { headers, .. } => Some(headers),
      _ => None,
    }
  }

  pub fn header_options(&self) -> Option<&HeaderOptions> {
    match self {
      ParserState::Headers { header_options, .. } | ParserState::Body { header_options, .. } => {
        Some(header_options)
      }
      _ => None,
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
