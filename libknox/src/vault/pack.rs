use std::error::Error;

use protobuf::{CodedOutputStream, Message};

/// Trait for message serialization
pub trait Packing {
  fn pack(&self) -> Result<Vec<u8>, Box<dyn Error>>;
}

impl<M> Packing for M
where
  M: Message,
{
  fn pack(&self) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut pack = Vec::new();
    let mut cos = CodedOutputStream::new(&mut pack);

    self.write_to(&mut cos)?;
    cos.flush()?;

    Ok(pack)
  }
}

#[cfg(test)]
mod tests {
  use knox_testing::spec;

  use crate::*;

  #[test]
  fn pack() {
    let tmp = spec::setup();
    let context = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");

    let mut entry = Entry::default();
    entry.add_attribute("lorem", "ipsum");
    entry.add_confidential_attribute("dolor", "sit");

    let pack = context.vault.pack().expect("could not pack vault");

    assert_eq!(
      pack,
      vec![10, 40, 54, 65, 50, 53, 70, 67, 70, 50, 49, 51, 67, 55, 55, 55, 57, 65, 68, 50, 54, 68, 67, 53, 48, 55, 48, 54, 67, 66, 54, 52, 51, 66, 52, 50, 69, 55, 67, 68, 51, 69]
    );

    let repack = context.vault.pack().expect("could not pack vault");

    assert_eq!(pack, repack);
  }
}
