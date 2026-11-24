//! GSP1 stream frames carried inside the payload of a GRL1 data frame.

const HEADER: usize = 40;
pub const MAX_BYTES: usize = 1 << 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Content<'a> {
    Open,
    Opened,
    Data { offset: u64, bytes: &'a [u8] },
    Window { offset: u64, credit: u32 },
    HalfClose(u8),
    Reset(u8),
    Close,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StreamFrame<'a> {
    pub stream_id: [u8; 16],
    pub content: Content<'a>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    ShortHeader,
    ShortPayload,
    Magic,
    Version(u8),
    Kind(u8),
    Flags(u8),
    Reserved,
    PayloadLimit,
    UnexpectedBody,
    WindowLimit,
}

impl StreamFrame<'_> {
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        let (kind, flag, offset, credit, payload): (u8, u8, u64, u32, &[u8]) = match self.content {
            Content::Open => (1, 0, 0, 0, &[]),
            Content::Opened => (2, 0, 0, 0, &[]),
            Content::Data { offset, bytes } if bytes.len() <= MAX_BYTES => (3, 0, offset, 0, bytes),
            Content::Data { .. } => return Err(Error::PayloadLimit),
            Content::Window { offset, credit } if credit <= MAX_BYTES as u32 => {
                (4, 0, offset, credit, &[])
            }
            Content::Window { .. } => return Err(Error::WindowLimit),
            Content::HalfClose(flag @ 1..=2) => (5, flag, 0, 0, &[]),
            Content::Reset(flag @ 1..=7) => (6, flag, 0, 0, &[]),
            Content::HalfClose(flag) | Content::Reset(flag) => return Err(Error::Flags(flag)),
            Content::Close => (7, 0, 0, 0, &[]),
        };
        let mut data = Vec::with_capacity(HEADER + payload.len());
        data.extend_from_slice(b"GSP1");
        data.extend_from_slice(&[1, kind, flag, 0]);
        data.extend_from_slice(&self.stream_id);
        data.extend_from_slice(&offset.to_be_bytes());
        data.extend_from_slice(&credit.to_be_bytes());
        data.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        data.extend_from_slice(payload);
        Ok(data)
    }
}

/// Decode the first frame and return the consumed length, as the native decoder does.
pub fn decode(data: &[u8]) -> Result<(StreamFrame<'_>, usize), Error> {
    if data.len() < HEADER {
        return Err(Error::ShortHeader);
    }
    if &data[..4] != b"GSP1" {
        return Err(Error::Magic);
    }
    if data[4] != 1 {
        return Err(Error::Version(data[4]));
    }
    let kind = data[5];
    if !(1..=7).contains(&kind) {
        return Err(Error::Kind(kind));
    }
    if data[7] != 0 {
        return Err(Error::Reserved);
    }
    let count = u32::from_be_bytes(data[36..40].try_into().unwrap()) as usize;
    if count > MAX_BYTES {
        return Err(Error::PayloadLimit);
    }
    let used = HEADER + count;
    let payload = data.get(HEADER..used).ok_or(Error::ShortPayload)?;
    let offset = u64::from_be_bytes(data[24..32].try_into().unwrap());
    let credit = u32::from_be_bytes(data[32..36].try_into().unwrap());
    let flag = data[6];
    let content = match (kind, flag) {
        (3, 0) => Content::Data {
            offset,
            bytes: payload,
        },
        (3, _) => return Err(Error::Flags(flag)),
        _ if count != 0 => return Err(Error::UnexpectedBody),
        (1, 0) => Content::Open,
        (2, 0) => Content::Opened,
        (4, 0) if credit > MAX_BYTES as u32 => return Err(Error::WindowLimit),
        (4, 0) => Content::Window { offset, credit },
        (5, 1..=2) => Content::HalfClose(flag),
        (6, 1..=7) => Content::Reset(flag),
        (5 | 6, _) => return Err(Error::Flags(flag)),
        (7, 0) => Content::Close,
        _ => return Err(Error::UnexpectedBody),
    };
    Ok((
        StreamFrame {
            stream_id: data[8..24].try_into().unwrap(),
            content,
        },
        used,
    ))
}
