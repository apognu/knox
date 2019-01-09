use std::error::Error;

use protobuf::{CodedOutputStream, Message};

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
  use crate::prelude::*;

  #[test]
  fn pack() {
    let tmp = crate::spec::setup();
    let handle = crate::spec::get_test_vault(tmp.path()).expect("could not get vault");

    let mut entry = Entry::default();
    entry.add_attribute("lorem", "ipsum");
    entry.add_confidential_attribute("dolor", "sit");

    let pack = handle.vault.pack().expect("could not pack vault");

    assert_eq!(
      pack,
      &[
        10, 23, 118, 97, 117, 108, 116, 64, 97, 112, 111, 103, 110, 117, 46, 103, 105, 116, 104,
        117, 98, 46, 99, 111, 109
      ]
    );

    let repack = handle.vault.pack().expect("could not pack vault");

    assert_eq!(pack, repack);
  }
}
