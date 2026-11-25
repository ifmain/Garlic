"""Sign a Garlic Alpha record profile with the network publisher's Ed25519 PEM."""
import argparse
import base64
import json
import os
from pathlib import Path
import tempfile
import time
import uuid

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--network", type=uuid.UUID, required=True)
    parser.add_argument("--key", type=Path, required=True)
    parser.add_argument("--sequence", type=int, required=True)
    parser.add_argument("--expires", type=int, required=True, help="UTC Unix timestamp")
    parser.add_argument("--frame-bytes", type=int, required=True)
    parser.add_argument("--module", type=Path, help="JSON with version, body and frame")
    parser.add_argument("--wasm", type=Path, help="Sandboxed record_limit WebAssembly module")
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    if not (0 < args.sequence < 2**64 and time.time() < args.expires < 2**64
            and 1024 <= args.frame_bytes <= 16384):
        parser.error("expected positive sequence, future expiry and 1024..16384 bytes")
    if args.output.resolve() == args.key.resolve():
        parser.error("output would overwrite the signing key")
    key = serialization.load_pem_private_key(args.key.read_bytes(), password=None)
    if not isinstance(key, Ed25519PrivateKey):
        parser.error("expected an Ed25519 private key")
    payload = dict(network=str(args.network), abi=1, sequence=args.sequence,
                   expires=args.expires, frame_bytes=args.frame_bytes)
    if args.module:
        module = json.loads(args.module.read_bytes())
        if (not isinstance(module, dict) or set(module) != {"version", "body", "frame"}
                or type(module["version"]) is not int or not 128 <= module["version"] <= 255
                or module["body"] not in ("base", "selective-ack")
                or module["frame"] not in ("fragments", "tlv")):
            parser.error("invalid module version or codec instructions")
        payload.update(abi=2, module=module)
    if args.wasm:
        code = args.wasm.read_bytes()
        if not code.startswith(b"\0asm") or len(code) > 6144:
            parser.error("invalid guest binary or size")
        payload.update(abi=3, wasm=base64.b64encode(code).decode("ascii"))
    raw = json.dumps(payload).encode()
    signature = key.sign(b"Garlic Alpha record profile v1\0" + raw)
    envelope = dict(payload=base64.b64encode(raw).decode(), signature=base64.b64encode(signature).decode())
    with tempfile.NamedTemporaryFile(dir=args.output.parent, delete=False, mode="w", encoding="ascii") as f:
        tmp = Path(f.name)
    try:
        tmp.write_text(json.dumps(envelope), encoding="ascii")
        os.replace(tmp, args.output)
    finally:
        tmp.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
