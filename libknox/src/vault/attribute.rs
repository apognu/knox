//! Abstract over [Attribute](struct.Attribute.html) values

use crate::pb::*;

/// An [Attribute](struct.Attribute.html)'s different kinds of value.
///
/// An attribute's value can be stored in differnet form. This enum abstracts
/// over those different types to provide a unique interface to the values.
#[derive(Debug, PartialEq)]
pub enum AttributeValue {
  /// The typical representation of a standard attribute, a simple UTF-8 string
  String(String),
  /// A binary presentation of a value, used for file contents.
  Binary(Vec<u8>),
}

impl Attribute {
  /// Retrieve an [Attribute](struct.Attribute.html)'s value.
  ///
  /// Extracts a value from an attribute, and return an enum that allows to
  /// abstract over the actuel storage representation.
  pub fn value(&self) -> AttributeValue {
    if self.file {
      match String::from_utf8(self.bytes_value.clone()) {
        Ok(string) => AttributeValue::String(string),
        Err(_) => AttributeValue::Binary(self.bytes_value.clone()),
      }
    } else {
      AttributeValue::String(self.value.clone())
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::*;

  #[test]
  fn set_and_get_value() {
    let mut entry = Entry::default();
    entry.add_attribute("standard", "lorem");
    entry.add_confidential_attribute("confidential", "ipsum");
    entry.add_file_attribute("file_string", "dolor".as_bytes());
    entry.add_file_attribute("file_binary", &[0, 159, 146, 150]);

    assert_eq!(entry.attributes.get("standard").expect("could not get attribute").value(), AttributeValue::String("lorem".to_string()));

    assert_eq!(
      entry.attributes.get("confidential").expect("could not get attribute").value(),
      AttributeValue::String("ipsum".to_string())
    );

    assert_eq!(
      entry.attributes.get("file_string").expect("could not get attribute").value(),
      AttributeValue::String("dolor".to_string())
    );

    assert_eq!(
      entry.attributes.get("file_binary").expect("could not get attribute").value(),
      AttributeValue::Binary(vec![0, 159, 146, 150])
    );
  }
}
