//! One WebSocket binary message carries an extensible header and one exchange packet.
//! The big-endian extension length counts TLVs (type:u8, length:u8, value).
//! Types below 128 are optional. Unknown mandatory types abort the exchange.
use crate::exchange::{Error, MAX_PACKET};

pub const CAPABILITY: &str = "gex4-tlv";
const PREFIX: &[u8] = b"GEX4";
const HEADER: usize = 6;
const MAX_EXTENSIONS: usize = 1024;
pub const MAX_MESSAGE: usize = HEADER + MAX_EXTENSIONS + MAX_PACKET;

pub fn wrap(packet: &[u8]) -> Vec<u8> {
    let mut message = Vec::with_capacity(HEADER + packet.len());
    message.extend_from_slice(PREFIX);
    message.extend_from_slice(&0u16.to_be_bytes());
    message.extend_from_slice(packet);
    message
}

pub fn unwrap(message: &[u8]) -> Result<&[u8], Error> {
    if message.len() < HEADER || !message.starts_with(PREFIX) {
        return Err(Error::Header);
    }
    let len = u16::from_be_bytes([message[4], message[5]]) as usize;
    if len > MAX_EXTENSIONS {
        return Err(Error::Size);
    }
    let (mut extensions, packet) = message[HEADER..]
        .split_at_checked(len)
        .ok_or(Error::Truncated)?;
    while !extensions.is_empty() {
        let header = extensions.get(..2).ok_or(Error::Truncated)?;
        if header[0] >= 128 {
            return Err(Error::Header);
        }
        extensions = extensions
            .get(2 + usize::from(header[1])..)
            .ok_or(Error::Truncated)?;
    }
    if packet.len() > MAX_PACKET {
        return Err(Error::Size);
    }
    Ok(packet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extensions_preserve_packet_boundaries() {
        assert_eq!(unwrap(&wrap(b"payload")), Ok(&b"payload"[..]));
        assert_eq!(unwrap(b"GEX4\0\x05\x01\x01x\x02\x00abc"), Ok(&b"abc"[..]));
        assert!(unwrap(b"GEX4\0\x02\x80\x00").is_err());
        assert!(unwrap(b"GEX4\0\x02\x01\x01abc").is_err());
        assert!(unwrap(b"GEX4\0\x01").is_err());
        assert!(unwrap(b"GEX1\0\x00abc").is_err());
        assert!(unwrap(&wrap(&vec![0; MAX_PACKET + 1])).is_err());
    }
}
