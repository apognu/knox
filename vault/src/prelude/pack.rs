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
