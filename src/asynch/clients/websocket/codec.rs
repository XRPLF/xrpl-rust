use alloc::{io, vec::Vec};
use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

pub struct Codec;

impl Decoder for Codec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Vec<u8>>, io::Error> {
        if !src.is_empty() {
            let len = src.len();
            let data = src.split_to(len).to_vec();
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
}

impl Encoder<&[u8]> for Codec {
    type Error = io::Error;

    fn encode(&mut self, data: &[u8], buf: &mut BytesMut) -> Result<(), io::Error> {
        buf.reserve(data.len());
        buf.put(data);
        Ok(())
    }
}
