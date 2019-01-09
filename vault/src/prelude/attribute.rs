use crate::pb::*;

#[derive(Debug, PartialEq)]
pub enum AttributeValue {
  String(String),
  Binary(Vec<u8>),
}

impl Attribute {
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
  use crate::prelude::*;

  #[test]
  fn set_and_get_value() {
    let mut entry = Entry::default();
    entry.add_attribute("standard", "lorem");
    entry.add_confidential_attribute("confidential", "ipsum");
    entry.add_file_attribute("file_string", "dolor".as_bytes());
    entry.add_file_attribute("file_binary", &[0, 159, 146, 150]);

    assert_eq!(
      entry
        .attributes
        .get("standard")
        .expect("could not get attribute")
        .value(),
      AttributeValue::String("lorem".to_string())
    );

    assert_eq!(
      entry
        .attributes
        .get("confidential")
        .expect("could not get attribute")
        .value(),
      AttributeValue::String("ipsum".to_string())
    );

    assert_eq!(
      entry
        .attributes
        .get("file_string")
        .expect("could not get attribute")
        .value(),
      AttributeValue::String("dolor".to_string())
    );

    assert_eq!(
      entry
        .attributes
        .get("file_binary")
        .expect("could not get attribute")
        .value(),
      AttributeValue::Binary(vec![0, 159, 146, 150])
    );
  }
}
