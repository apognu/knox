//! Standard error used by Knox

use std::error::Error;
use std::fmt::{Display, Formatter, Result};

/// All errors returned by Knox.
#[derive(Debug, Clone)]
pub struct VaultError {
  message: String,
}

impl<'a> VaultError {
  /// Simple constructor for a [VaultError](struct.VaultError.html)
  pub fn new(message: &'a str) -> Self {
    Self { message: message.to_string() }
  }

  /// Constructor for a boxed [VaultError](struct.VaultError.html)
  pub fn throw(message: &str) -> Box<dyn Error> {
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

  fn cause(&self) -> Option<&dyn Error> {
    None
  }
}

impl From<gpgme::Error> for VaultError {
  fn from(err: gpgme::Error) -> Self {
    Self::new(&err.description())
  }
}
