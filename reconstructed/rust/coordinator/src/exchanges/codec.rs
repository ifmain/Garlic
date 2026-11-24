use super::modules::{Body, Frame, Module};
use super::Outbound;
use axum::extract::ws::{Message, WebSocket};
use bytes::Bytes;
use garlic_relay_data::{
    envelope,
    exchange::{gex3_wire, Packet},
    gex4,
};
use garlic_relay_session::SessionError;
use std::num::NonZeroU32;
impl Module {
    pub(super) fn packet<'a>(self, bytes: &'a [u8]) -> Result<(Packet<'a>, u32), SessionError> {
        let decoded = match self.body {
            Body::Base => Packet::decode(bytes).map(|p| (p, 0)),
            Body::SelectiveAck => Packet::decode_gex3(bytes),
        };
        decoded.map_err(|_| "invalid client exchange packet".into())
    }
}

pub(super) struct LaneWire {
    module: Module,
    incoming: envelope::Decoder,
    next_message: NonZeroU32,
}
impl LaneWire {
    pub(super) fn new(module: Module) -> Self {
        Self {
            module,
            incoming: envelope::Decoder::default(),
            next_message: NonZeroU32::MIN,
        }
    }
    pub(super) fn receive(&mut self, bytes: Bytes) -> Result<Option<Bytes>, SessionError> {
        if self.module.frame == Frame::Tlv {
            let packet = gex4::unwrap(&bytes).map_err(|_| "invalid GEX4 header")?;
            return Ok(Some(bytes.slice(bytes.len() - packet.len()..)));
        }
        self.incoming
            .receive(&bytes)
            .map(|packet| packet.map(Bytes::from))
            .map_err(|_| "invalid framed client exchange".into())
    }
    pub(super) async fn send(
        &mut self,
        ws: &mut WebSocket,
        out: Outbound,
    ) -> Result<(), SessionError> {
        let encoded;
        let payload = match self.module.body {
            Body::Base => out.data.as_ref(),
            Body::SelectiveAck => {
                encoded = gex3_wire(out.data.to_vec(), out.feedback.0, out.feedback.1)
                    .map_err(|_| "invalid outgoing GEX3 exchange packet")?;
                &encoded[..]
            }
        };
        if self.module.frame == Frame::Tlv {
            ws.send(Message::Binary(gex4::wrap(payload).into())).await?;
            return Ok(());
        }
        let id = self.next_message;
        self.next_message = NonZeroU32::new(id.get().wrapping_add(1)).unwrap_or(NonZeroU32::MIN);
        for record in envelope::fragments_sized(
            id,
            payload,
            self.module.record_size.unwrap_or(envelope::MAX_RECORD),
        )
        .map_err(|_| "invalid outgoing exchange record")?
        {
            ws.send(Message::Binary(record.into())).await?;
        }
        Ok(())
    }
}
