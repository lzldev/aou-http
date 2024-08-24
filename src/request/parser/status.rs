use super::{ParserResult, ParserState};

#[derive(Debug)]
pub enum ParserStatus {
  Success(ParserResult),
  Incomplete((Vec<u8>, ParserState)),
  Invalid(String),
}

impl ParserStatus {
  pub fn is_incomplete(&self) -> bool {
    match self {
      ParserStatus::Incomplete(_) => true,
      _ => false,
    }
  }

  pub fn is_success(&self) -> bool {
    match self {
      ParserStatus::Success(_) => true,
      _ => false,
    }
  }

  pub fn is_invalid(&self) -> bool {
    match self {
      ParserStatus::Invalid(_) => true,
      _ => false,
    }
  }
}
