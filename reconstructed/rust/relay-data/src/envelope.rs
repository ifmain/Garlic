//! Fragment assembly for the GEX2 records negotiated by the deployed worker.
use std::mem;
use std::num::NonZeroU32;

const HEADER: usize = 20;
pub const MAX_RECORD: usize = 16_000;
pub const MAX_MESSAGE: usize = 0x102029;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordError {
    Header,
    Bounds,
    Padding,
    EmptyFragment,
    MissingFirst,
    OutOfOrder,
}

#[derive(Default)]
pub struct Decoder {
    id: u32,
    total: usize,
    payload: Vec<u8>,
}

impl Decoder {
    pub fn receive(&mut self, record: &[u8]) -> Result<Option<Vec<u8>>, RecordError> {
        if !(HEADER..=MAX_RECORD).contains(&record.len()) || &record[..4] != b"GEX2" {
            return Err(RecordError::Header);
        }
        let field = |offset| u32::from_be_bytes(record[offset..offset + 4].try_into().unwrap());
        let id = field(4);
        let offset = field(8) as usize;
        let total = field(12) as usize;
        let count = field(16) as usize;
        if total > MAX_MESSAGE
            || offset > total
            || count > total - offset
            || count > record.len() - HEADER
        {
            return Err(RecordError::Bounds);
        }
        if id == 0 {
            return if total == 0 {
                Ok(None)
            } else {
                Err(RecordError::Padding)
            };
        }
        if count == 0 {
            return Err(RecordError::EmptyFragment);
        }
        if self.payload.is_empty() {
            if offset != 0 {
                return Err(RecordError::MissingFirst);
            }
            self.id = id;
            self.total = total;
        } else if self.id != id || self.total != total || self.payload.len() != offset {
            return Err(RecordError::OutOfOrder);
        }
        self.payload
            .extend_from_slice(&record[HEADER..HEADER + count]);
        if self.payload.len() == total {
            self.total = 0;
            Ok(Some(mem::take(&mut self.payload)))
        } else {
            Ok(None)
        }
    }
}

/// Emit complete data records in order. The receiving worker allows the final
/// fragment to end directly after its declared payload.
pub fn fragments(
    id: NonZeroU32,
    message: &[u8],
) -> Result<impl Iterator<Item = Vec<u8>> + '_, RecordError> {
    fragments_sized(id, message, MAX_RECORD)
}

pub fn fragments_sized(
    id: NonZeroU32,
    message: &[u8],
    record_size: usize,
) -> Result<impl Iterator<Item = Vec<u8>> + '_, RecordError> {
    if !(HEADER + 1..=MAX_RECORD).contains(&record_size) {
        return Err(RecordError::Bounds);
    }
    if message.is_empty() || message.len() > MAX_MESSAGE {
        return Err(RecordError::Bounds);
    }
    let chunk_size = record_size - HEADER;
    Ok(message
        .chunks(chunk_size)
        .enumerate()
        .map(move |(index, chunk)| {
            let mut record = Vec::with_capacity(HEADER + chunk.len());
            record.extend_from_slice(b"GEX2");
            for field in [
                id.get(),
                (index * chunk_size) as u32,
                message.len() as u32,
                chunk.len() as u32,
            ] {
                record.extend_from_slice(&field.to_be_bytes());
            }
            record.extend_from_slice(chunk);
            record
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reassembles_after_padding_and_rejected_fragment() {
        let data = vec![0xab; MAX_RECORD];
        let id = NonZeroU32::new(47).unwrap();
        let mut records = fragments(id, &data).unwrap();
        let first = records.next().unwrap();
        let last = records.next().unwrap();
        drop(records);
        let mut decoder = Decoder::default();
        assert_eq!(decoder.receive(&last), Err(RecordError::MissingFirst));
        assert_eq!(decoder.receive(&first), Ok(None));
        assert_eq!(decoder.receive(&first), Err(RecordError::OutOfOrder));
        let mut padding = vec![0; 40];
        padding[..4].copy_from_slice(b"GEX2");
        assert_eq!(decoder.receive(&padding), Ok(None));
        assert_eq!(decoder.receive(&last), Ok(Some(data)));
    }

    #[test]
    fn accepts_padded_fragments_and_maximum_message() {
        for length in [1, MAX_RECORD - HEADER, MAX_MESSAGE] {
            let data = vec![0x1f; length];
            let mut decoder = Decoder::default();
            let mut complete = None;
            for mut record in fragments(NonZeroU32::MIN, &data).unwrap() {
                record.resize(MAX_RECORD, 0x55);
                complete = decoder.receive(&record).unwrap();
            }
            assert_eq!(complete, Some(data));
        }
        assert!(fragments(NonZeroU32::MIN, &vec![0; MAX_MESSAGE + 1]).is_err());
    }
}
