use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone)]
pub(crate) struct GenericError {
  message: String,
}

impl<'a> GenericError {
  fn new(message: &'a str) -> Self {
    Self {
      message: message.to_string(),
    }
  }

  pub(crate) fn throw(message: &str) -> Box<dyn Error> {
    Box::new(GenericError::new(message))
  }
}

impl<'a> Display for GenericError {
  fn fmt(&self, f: &mut Formatter) -> Result {
    write!(f, "{}", self.message)
  }
}

impl<'a> Error for GenericError {
  fn description(&self) -> &str {
    &self.message
  }

  fn cause(&self) -> Option<&Error> {
    None
  }
}
