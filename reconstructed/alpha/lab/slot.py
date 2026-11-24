"""One declarative GUC profile per pinned network. The host owns the connection."""
import hashlib
import json
import sqlite3
import time

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

DOMAIN = b"Garlic Alpha GUC profile v1\0"
BUILTINS = ("GEX3", "GEX2")


def validate(raw, network, now):
    if not 1 <= len(raw) <= 1024:
        raise ValueError("profile exceeds size limit")
    def unique(pairs):
        obj = dict(pairs)
        if len(obj) != len(pairs):
            raise ValueError("duplicate field")
        return obj
    p = json.loads(raw.decode("ascii"), object_pairs_hook=unique)
    if type(p) is not dict or p.keys() != {"network","abi","sequence","expires","frame_bytes"}:
        raise ValueError("unsupported profile fields")
    if p["network"] != network or any(type(p[k]) != int for k in ("abi","sequence","expires","frame_bytes")):
        raise ValueError("invalid profile identity or type")
    if p["abi"] != 1 or not 0 < p["sequence"] < 2**63 or not now < p["expires"] < 2**63:
        raise ValueError("incompatible or expired profile")
    if not 1024 <= p["frame_bytes"] <= 16384:
        raise ValueError("invalid frame size")
    return p


class Slots:
    def __init__(self, path, pinned_key):
        self.key = Ed25519PublicKey.from_public_bytes(pinned_key)
        self.network = hashlib.sha256(pinned_key).hexdigest()
        self.db = sqlite3.connect(path)
        self.db.executescript("""
            CREATE TABLE IF NOT EXISTS profiles(network TEXT PRIMARY KEY, seq INTEGER NOT NULL,
                payload BLOB NOT NULL, signature BLOB NOT NULL);
            CREATE TABLE IF NOT EXISTS settings(network TEXT, name TEXT, enabled INTEGER NOT NULL,
                PRIMARY KEY(network,name));
        """)

    def install(self, raw, signature):
        if len(raw) > 1024:
            raise ValueError("profile exceeds size limit")
        self.key.verify(signature, DOMAIN + raw)
        p = validate(raw, self.network, time.time())
        with self.db:
            cursor = self.db.execute("""INSERT INTO profiles VALUES(?,?,?,?)
                ON CONFLICT(network) DO UPDATE SET seq=excluded.seq, payload=excluded.payload,
                signature=excluded.signature WHERE excluded.seq>profiles.seq""",
                (self.network, p["sequence"], raw, signature))
            if cursor.rowcount != 1:
                raise ValueError("old or duplicate profile")

    def enable(self, name, enabled):
        if name not in (*BUILTINS, "slot") or type(enabled) is not bool:
            raise ValueError("invalid local setting")
        with self.db:
            self.db.execute("INSERT OR REPLACE INTO settings VALUES(?,?,?)",
                            (self.network, name, enabled))

    def candidates(self):
        disabled = {row[0] for row in self.db.execute(
            "SELECT name FROM settings WHERE network=? AND enabled=0",(self.network,))}
        row = self.db.execute("SELECT payload,signature FROM profiles WHERE network=?",
                              (self.network,)).fetchone()
        modes = []
        if "slot" not in disabled and row:
            from cryptography.exceptions import InvalidSignature
            try:
                self.key.verify(row[1], DOMAIN + row[0])
                p = validate(row[0], self.network, time.time())
                modes.append(("GUC1", p["frame_bytes"]))
            except (InvalidSignature, ValueError):
                pass
        modes.extend((mode, None) for mode in BUILTINS if mode not in disabled)
        return modes
