import json
from pathlib import Path
import tempfile
import time
import unittest

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from slot import DOMAIN, Slots


class SlotTest(unittest.TestCase):
    def test_network_ownership_replacement_and_switches(self):
        with tempfile.TemporaryDirectory() as tmp:
            signer = Ed25519PrivateKey.generate()
            db = Slots(Path(tmp)/"state", signer.public_key().public_bytes_raw())
            self.addCleanup(db.db.close)
            try:
                def update(seq, **extra):
                    p = dict(network=db.network, abi=1, sequence=seq,
                             expires=int(time.time())+3600, frame_bytes=8192)
                    p.update(extra)
                    raw = json.dumps(p).encode()
                    db.install(raw, signer.sign(DOMAIN+raw))
                update(1)
                self.assertEqual(db.candidates()[0], ("GUC1",8192))
                db.enable("slot",False)
                update(2,frame_bytes=4096)
                self.assertEqual(db.candidates(),[("GEX3",None),("GEX2",None)])
                for extra in [dict(command="whoami"),dict(network="other"),dict(frame_bytes=-1),
                              dict(abi=True),dict(expires=0)]:
                    with self.assertRaises(ValueError):
                        update(3,**extra)
                with self.assertRaises(ValueError):
                    update(1)
                with self.assertRaises(InvalidSignature):
                    db.install(b"{}",b"x"*64)
                db.enable("slot",True)
                self.assertEqual(db.candidates()[0],("GUC1",4096))
                for mode in ["slot","GEX2","GEX3"]:
                    db.enable(mode,False)
                self.assertEqual(db.candidates(),[])
            finally:
                db.db.close()


if __name__ == "__main__":
    unittest.main()
