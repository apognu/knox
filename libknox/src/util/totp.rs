use std::error::Error;

use oath::{totp_raw_custom_time, totp_raw_now, HashType};

use crate::{Entry, TotpConfig_Hash, VaultError};

pub fn get_totp(entry: &Entry, time: Option<u64>) -> Result<String, Box<dyn Error>> {
  if !entry.has_totp() {
    return Err(VaultError::throw(
      "TOTP generation was not configured for this entry",
    ));
  }

  let hash = match entry.get_totp().get_hash() {
    TotpConfig_Hash::SHA1 => HashType::SHA1,
    TotpConfig_Hash::SHA256 => HashType::SHA256,
    TotpConfig_Hash::SHA512 => HashType::SHA512,
  };

  let secret = entry.get_totp().get_secret();
  let totp = match time {
    Some(time) => totp_raw_custom_time(
      secret,
      entry.get_totp().get_length(),
      0,
      entry.get_totp().get_interval(),
      time,
      &hash,
    ),

    None => totp_raw_now(
      secret,
      entry.get_totp().get_length(),
      0,
      entry.get_totp().get_interval(),
      &hash,
    ),
  };

  Ok(format!("{:0>6}", totp))
}

#[cfg(test)]
mod test {
  use chrono::prelude::*;
  use protobuf::SingularPtrField;

  use crate::{Entry, TotpConfig, TotpConfig_Hash};

  #[test]
  fn get_totp() {
    let entry = Entry {
      totp: SingularPtrField::some(TotpConfig {
        secret: vec![],
        interval: 30,
        length: 6,
        hash: TotpConfig_Hash::SHA1,
        ..TotpConfig::default()
      }),
      ..Entry::default()
    };

    let totp = super::get_totp(
      &entry,
      Some(Utc.ymd(2014, 11, 28).and_hms(0, 0, 0).timestamp() as u64),
    );

    assert_eq!(totp.is_ok(), true);
    assert_eq!(totp.unwrap(), String::from("948000"));
  }
}
