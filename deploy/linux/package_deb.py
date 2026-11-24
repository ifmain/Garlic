"""Pack statically linked x86-64 Linux binaries into a Debian binary package."""
import argparse
import gzip
import io
import os
from pathlib import Path
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[2]
VERSION = "0.9.2-1"


def archive(entries):
    data = io.BytesIO()
    with tarfile.open(fileobj=data, mode="w", format=tarfile.GNU_FORMAT) as tar:
        for name, content, mode in entries:
            info = tarfile.TarInfo("./" + name)
            info.size = len(content)
            info.mode = mode
            info.uname = info.gname = "root"
            tar.addfile(info, io.BytesIO(content))
    return gzip.compress(data.getvalue(), mtime=0)


def main():
    args = argparse.ArgumentParser(description=__doc__)
    args.add_argument("bin_dir", type=Path, help="release directory for x86_64-unknown-linux-musl")
    binaries = args.parse_args().bin_dir
    entries = []
    for name in ("garlic-server", "garlic-auth"):
        content = (binaries / name).read_bytes()
        if content[:6] != b"\x7fELF\x02\x01" or content[18:20] != b"\x3e\x00":
            raise ValueError(f"{name} is not an x86-64 Linux ELF")
        entries.append(("usr/bin/" + name, content, 0o755))
    for role in ("entry", "exit"):
        unit = f"garlic-{role}.service"
        entries.append(("usr/lib/systemd/user/" + unit, (ROOT/"deploy/linux"/unit).read_bytes(), 0o644))
        example = role + ".env.example"
        entries.append(("usr/share/doc/garlic-server/examples/" + example,
                        (ROOT/"deploy/linux"/example).read_bytes(), 0o644))
    control = (f"Package: garlic-server\nVersion: {VERSION}\nArchitecture: amd64\n"
               "Maintainer: Garlic <222815311+ifmain@users.noreply.github.com>\n"
               "Section: net\nPriority: optional\nDescription: Garlic server and account administration tools\n")
    dest = ROOT/"dist"/f"garlic-server_{VERSION}_amd64.deb"
    dest.parent.mkdir(exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=dest.parent, delete=False) as out:
        tmp = Path(out.name)
        try:
            out.write(b"!<arch>\n")
            for name, payload in [("debian-binary", b"2.0\n"),
                                  ("control.tar.gz", archive([("control", control.encode(), 0o644)])),
                                  ("data.tar.gz", archive(entries))]:
                out.write(f"{name+'/':<16}{0:<12}{0:<6}{0:<6}{'100644':<8}{len(payload):<10}`\n".encode())
                out.write(payload)
                if len(payload) % 2:
                    out.write(b"\n")
        except BaseException:
            out.close()
            tmp.unlink(missing_ok=True)
            raise
    os.replace(tmp, dest)
    print(dest)


if __name__ == "__main__":
    main()
