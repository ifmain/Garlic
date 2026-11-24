//! Binary data frames used by the deployed garlic-infinite-storage worker.

const MAGIC: &[u8; 4] = b"GRL1";
const HEADER: usize = 25;
pub const MAX_PAYLOAD: usize = 0x100028;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Direction {
    One = 1,
    Two = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Frame<'a> {
    pub direction: Direction,
    pub flow_id: [u8; 16],
    pub payload: &'a [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidFrame;

impl Frame<'_> {
    pub fn encode(&self) -> Result<Vec<u8>, InvalidFrame> {
        if !(1..=MAX_PAYLOAD).contains(&self.payload.len()) {
            return Err(InvalidFrame);
        }
        let mut data = Vec::with_capacity(HEADER + self.payload.len());
        data.extend_from_slice(MAGIC);
        data.push(self.direction as u8);
        data.extend_from_slice(&self.flow_id);
        data.extend_from_slice(&(self.payload.len() as u32).to_be_bytes());
        data.extend_from_slice(self.payload);
        Ok(data)
    }
}

pub fn decode(data: &[u8]) -> Result<Frame<'_>, InvalidFrame> {
    if !(HEADER + 1..=HEADER + MAX_PAYLOAD).contains(&data.len()) || &data[..4] != MAGIC {
        return Err(InvalidFrame);
    }
    let direction = match data[4] {
        1 => Direction::One,
        2 => Direction::Two,
        _ => return Err(InvalidFrame),
    };
    let payload = &data[HEADER..];
    let length = u32::from_be_bytes(data[21..25].try_into().unwrap());
    if length as usize != payload.len() {
        return Err(InvalidFrame);
    }
    Ok(Frame {
        direction,
        flow_id: data[5..21].try_into().unwrap(),
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deployed_wire_format() {
        let bytes = b"GRL1\x020123456789abcdef\x00\x00\x00\x03\x00\xffA";
        let frame = Frame {
            direction: Direction::Two,
            flow_id: *b"0123456789abcdef",
            payload: b"\x00\xffA",
        };
        assert_eq!(decode(bytes), Ok(frame));
        assert_eq!(frame.encode().unwrap(), bytes);
    }

    #[test]
    fn size_boundaries_and_direction_tags() {
        for n in [0, 1, 256, MAX_PAYLOAD, MAX_PAYLOAD + 1] {
            let payload = vec![0xa7; n];
            for direction in [Direction::One, Direction::Two] {
                let frame = Frame {
                    direction,
                    flow_id: [0; 16],
                    payload: &payload,
                };
                if (1..=MAX_PAYLOAD).contains(&n) {
                    assert_eq!(decode(&frame.encode().unwrap()), Ok(frame));
                } else {
                    assert_eq!(frame.encode(), Err(InvalidFrame));
                }
            }
        }
    }

    #[test]
    fn header_and_length_must_match() {
        let bytes = *b"GRL1\x010123456789abcdef\x00\x00\x00\x01x";
        for n in 0..bytes.len() {
            assert_eq!(decode(&bytes[..n]), Err(InvalidFrame));
        }
        for offset in [0, 4, 21, 24] {
            let mut bad = bytes;
            bad[offset] ^= 0xff;
            assert_eq!(decode(&bad), Err(InvalidFrame));
        }
    }
}
