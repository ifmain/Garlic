"""Experimental GUC v1 framing over an already authenticated TLS 1.3 stream."""
import asyncio
import ssl
import struct

ALPN = "garlic-guc/1"
MAX_DATA = 16384
HEADER = struct.Struct("!BH")
DATA, FIN = 1, 2


def contexts(directory):
    server = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    client = ssl.create_default_context(cafile=str(directory/"ca.pem"))
    server.load_verify_locations(cafile=str(directory/"ca.pem"))
    server.verify_mode = ssl.CERT_REQUIRED
    for ctx, name in ((server, "server"), (client, "client")):
        ctx.minimum_version = ssl.TLSVersion.TLSv1_3
        ctx.set_alpn_protocols([ALPN])
        ctx.load_cert_chain(directory/(name+".pem"), directory/(name+".key"))
    return server, client


def negotiated(writer):
    tls = writer.get_extra_info("ssl_object")
    if tls is None or tls.selected_alpn_protocol() != ALPN or not tls.getpeercert():
        raise ValueError("missing authenticated GUC negotiation")


async def read(reader):
    kind, length = HEADER.unpack(await reader.readexactly(HEADER.size))
    if kind == FIN and length == 0:
        return None
    if kind != DATA or not 1 <= length <= MAX_DATA:
        raise ValueError("invalid GUC frame")
    return await reader.readexactly(length)


async def write(writer, data):
    if data is None:
        writer.write(HEADER.pack(FIN, 0))
    else:
        if not 1 <= len(data) <= MAX_DATA:
            raise ValueError("invalid GUC payload size")
        writer.writelines((HEADER.pack(DATA, len(data)), data))
    await writer.drain()
