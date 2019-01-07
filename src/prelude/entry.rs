use std::error::Error;
use std::fs::File;
use std::path::Path;

use protobuf::parse_from_bytes;

use crate::gpg;
use crate::pb::*;
use crate::util;

impl Entry {
  pub fn read<P>(path: P) -> Result<Entry, Box<dyn Error>>
  where
    P: AsRef<Path>,
  {
    println!("{}", path.as_ref().display());

    let pack = gpg::decrypt(&mut File::open(util::normalize_path(&path))?)?;
    let message = parse_from_bytes::<Entry>(&pack)?;

    Ok(message)
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use crate::prelude::*;

  #[test]
  fn read() {
    let _tmp = crate::tests::setup();
    let mut vault = crate::tests::get_test_vault();

    let entry = Entry {
      attributes: {
        let mut map = HashMap::new();

        map.insert(
          "lorem".to_string(),
          Attribute {
            value: "ipsum".to_string(),
            ..Attribute::default()
          },
        );

        map.insert(
          "foo".to_string(),
          Attribute {
            value: "bar".to_string(),
            ..Attribute::default()
          },
        );

        map
      },
      ..Entry::default()
    };

    vault
      .write_entry("pack.bin", &entry)
      .expect("could not write pack");

    let retrieved = vault.read_entry("pack.bin").expect("could not read pack");

    assert_eq!(retrieved, entry);
  }
}
