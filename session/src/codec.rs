use codee::{Decoder, Encoder};
use serde::{Deserialize, Serialize};

pub struct BitcodeCodec;

impl<T: Serialize> Encoder<T> for BitcodeCodec {
    type Error = bitcode::Error;
    type Encoded = Vec<u8>;
    fn encode(val: &T) -> Result<Self::Encoded, Self::Error> {
        bitcode::serialize(val)
    }
}

impl<T: for<'de> Deserialize<'de>> Decoder<T> for BitcodeCodec {
    type Error = bitcode::Error;
    type Encoded = [u8];
    fn decode(val: &Self::Encoded) -> Result<T, Self::Error> {
        bitcode::deserialize(val)
    }
}
