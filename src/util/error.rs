use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone)]
pub(crate) struct VaultError {
  message: String,
}

impl<'a> VaultError {
  fn new(message: &'a str) -> Self {
    Self {
      message: message.to_string(),
    }
  }

  pub(crate) fn throw(message: &str) -> Box<dyn Error> {
    Box::new(VaultError::new(message))
  }
}

impl<'a> Display for VaultError {
  fn fmt(&self, f: &mut Formatter) -> Result {
    write!(f, "{}", self.message)
  }
}

impl<'a> Error for VaultError {
  fn description(&self) -> &str {
    &self.message
  }

  fn cause(&self) -> Option<&Error> {
    None
  }
}
