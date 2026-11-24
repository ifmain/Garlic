"""Create and verify the Garlic ingress and exit package for Linux x86-64."""

import argparse
import gzip
import hashlib
import io
import os
from pathlib import Path
import struct
import tarfile
import tempfile


PROGRAMS = (
    "garlic-auth",
    "garlic-coordinator",
    "garlic-dns",
    "garlic-infinite-storage",
    "garlic-internal-network",
    "garlic-legacy-header",
    "garlic-relay-worker",
    "garlic-xui-legacy",
    "probe",
    "w-legacy-probe",
    "w-public-probe",
)
CONFIG_FILES = (
    "README.md",
    "entry.env.example",
    "exit.env.example",
    "garlic-entry.service",
    "garlic-exit.service",
)


def executable(path):
    contents = path.read_bytes()
    if len(contents) < 64 or contents[:6] != b"\x7fELF\x02\x01":
        raise ValueError(f"{path.name} is not a little-endian ELF64 binary")
    if struct.unpack_from("<H", contents, 18)[0] != 62:
        raise ValueError(f"{path.name} is not built for x86-64")
    programs = struct.unpack_from("<Q", contents, 32)[0]
    entry_size, count = struct.unpack_from("<HH", contents, 54)
    if programs < 64 or entry_size != 56 or not count or programs + entry_size * count > len(contents):
        raise ValueError(f"{path.name} has invalid program headers")
    if any(struct.unpack_from("<I", contents, programs + index * entry_size)[0] == 3
           for index in range(count)):
        raise ValueError(f"{path.name} requires a dynamic loader")
    return contents


def create(binary_dir, output):
    templates = Path(__file__).resolve().parents[2] / "deploy/linux"
    contents = {f"bin/{name}": executable(binary_dir / name) for name in PROGRAMS}
    for name in CONFIG_FILES:
        contents[name] = (templates / name).read_bytes()
    contents["SHA256SUMS"] = "".join(
        f"{hashlib.sha256(data).hexdigest()}  {name}\n" for name, data in sorted(contents.items())
    ).encode("ascii")
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=output.parent) as staging:
        partial = Path(staging) / output.name
        with partial.open("wb") as destination:
            with gzip.GzipFile(filename="", mode="wb", fileobj=destination, mtime=0) as compressed:
                with tarfile.open(fileobj=compressed, mode="w|", format=tarfile.USTAR_FORMAT) as archive:
                    for name, data in sorted(contents.items()):
                        member = tarfile.TarInfo(name)
                        member.size = len(data)
                        member.mode = 0o755 if name.startswith("bin/") else 0o644
                        archive.addfile(member, io.BytesIO(data))
        with tarfile.open(partial, "r:gz") as archive:
            if archive.getnames() != sorted(contents):
                raise ValueError("archive members do not match the requested package")
            for member in archive.getmembers():
                with archive.extractfile(member) as source:
                    if source.read() != contents[member.name]:
                        raise ValueError(f"archive contents changed for {member.name}")
                expected = 0o755 if member.name.startswith("bin/") else 0o644
                if member.mode != expected:
                    raise ValueError(f"invalid permissions on {member.name}")
        os.replace(partial, output)
    with output.open("rb") as archive:
        print(f"{hashlib.file_digest(archive, 'sha256').hexdigest()}  {output.name}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary_dir", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    create(args.binary_dir, args.output)
