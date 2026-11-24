//! Batches exchanged by a client and a coordinator. Sequence and
//! acknowledgement numbers belong to the exchange, not to an individual flow.
pub const MAX_PACKET: usize = 0x102028;
const MAX_RECORDS: usize = 256;
const MAX_PAYLOAD: usize = 0x100028;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record<'a> {
    pub owner: [u8; 16],
    pub kind: u8,
    pub payload: &'a [u8],
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet<'a> {
    pub sequence: u64,
    pub acknowledgement: u64,
    pub flag: bool,
    pub records: Vec<Record<'a>>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Header,
    Count,
    Truncated,
    Record,
    Trailing,
    Size,
}

impl<'a> Packet<'a> {
    pub fn decode(bytes: &'a [u8]) -> Result<Self, Error> {
        Self::decode_with_magic(bytes, b"GEX1")
    }

    pub fn decode_gex3(bytes: &'a [u8]) -> Result<(Self, u32), Error> {
        if !(28..=MAX_PACKET).contains(&bytes.len()) {
            return Err(Error::Header);
        }
        let (header_and_records, feedback) = bytes.split_at(bytes.len() - 4);
        let packet = Self::decode_with_magic(header_and_records, b"GEX3")?;
        Ok((packet, u32::from_be_bytes(feedback.try_into().unwrap())))
    }

    fn decode_with_magic(bytes: &'a [u8], magic: &[u8; 4]) -> Result<Self, Error> {
        if !(24..=MAX_PACKET).contains(&bytes.len()) || &bytes[..4] != magic {
            return Err(Error::Header);
        }
        let sequence = u64::from_be_bytes(bytes[4..12].try_into().unwrap());
        let acknowledgement = u64::from_be_bytes(bytes[12..20].try_into().unwrap());
        let raw = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        let count = (raw & 0x7fff_ffff) as usize;
        if count > MAX_RECORDS || (count != 0 && sequence == 0) {
            return Err(Error::Count);
        }
        let mut rest = &bytes[24..];
        let mut records = Vec::with_capacity(count);
        for _ in 0..count {
            let header = rest.get(..21).ok_or(Error::Truncated)?;
            let owner = header[..16].try_into().unwrap();
            let kind = header[16];
            let len = u32::from_be_bytes(header[17..].try_into().unwrap()) as usize;
            if owner == [0; 16] || kind > 3 || len > MAX_PAYLOAD {
                return Err(Error::Record);
            }
            let payload = rest.get(21..21 + len).ok_or(Error::Record)?;
            records.push(Record {
                owner,
                kind,
                payload,
            });
            rest = &rest[21 + len..];
        }
        if !rest.is_empty() {
            return Err(Error::Trailing);
        }
        Ok(Self {
            sequence,
            acknowledgement,
            flag: raw >> 31 != 0,
            records,
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        if self.records.len() > MAX_RECORDS || (!self.records.is_empty() && self.sequence == 0) {
            return Err(Error::Count);
        }
        let mut size = 24;
        for record in &self.records {
            if record.owner == [0; 16] || record.kind > 3 || record.payload.len() > MAX_PAYLOAD {
                return Err(Error::Record);
            }
            size += 21 + record.payload.len();
            if size > MAX_PACKET {
                return Err(Error::Size);
            }
        }
        let mut bytes = Vec::with_capacity(size);
        bytes.extend_from_slice(b"GEX1");
        bytes.extend_from_slice(&self.sequence.to_be_bytes());
        bytes.extend_from_slice(&self.acknowledgement.to_be_bytes());
        let count = (self.records.len() as u32) | ((self.flag as u32) << 31);
        bytes.extend_from_slice(&count.to_be_bytes());
        for r in &self.records {
            bytes.extend_from_slice(&r.owner);
            bytes.push(r.kind);
            bytes.extend_from_slice(&(r.payload.len() as u32).to_be_bytes());
            bytes.extend_from_slice(r.payload);
        }
        Ok(bytes)
    }
}

/// Reuse the records of a packet already encoded by `Packet::encode` and
/// attach the current GEX3 feedback without reparsing the records.
pub fn gex3_wire(
    mut packet: Vec<u8>,
    acknowledgement: u64,
    selective: u32,
) -> Result<Vec<u8>, Error> {
    if !(24..=MAX_PACKET - 4).contains(&packet.len()) || &packet[..4] != b"GEX1" {
        return Err(Error::Header);
    }
    packet[..4].copy_from_slice(b"GEX3");
    packet[12..20].copy_from_slice(&acknowledgement.to_be_bytes());
    packet.extend_from_slice(&selective.to_be_bytes());
    Ok(packet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gex3_reuses_records_and_appends_big_endian_feedback() {
        let packet = Packet {
            sequence: 19,
            acknowledgement: 3,
            flag: true,
            records: vec![Record {
                owner: [1; 16],
                kind: 0,
                payload: b"fixture",
            }],
        };
        let wire = gex3_wire(packet.encode().unwrap(), 17, 0x8000_0004).unwrap();
        assert_eq!(&wire[wire.len() - 4..], &[0x80, 0, 0, 4]);
        assert!(Packet::decode(&wire).is_err());
        let (decoded, selective) = Packet::decode_gex3(&wire).unwrap();
        assert_eq!(
            decoded,
            Packet {
                acknowledgement: 17,
                ..packet
            }
        );
        assert_eq!(selective, 0x8000_0004);
        assert!(Packet::decode_gex3(&wire[..wire.len() - 1]).is_err());
    }
}
