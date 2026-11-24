"""Package the rebuilt client cores with the recovered user interfaces."""
import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import zipfile


def replace_members(source, output, replacements, exclude=()):
    with zipfile.ZipFile(source) as old, zipfile.ZipFile(output, "w") as new:
        if not replacements.keys() <= set(old.namelist()):
            raise ValueError("a native component is missing from the template")
        for item in old.infolist():
            if item.filename in exclude:
                continue
            value = replacements.get(item.filename)
            new.writestr(item, value.read_bytes() if value else old.read(item))


def verify_members(source, result, replacements, exclude=()):
    with zipfile.ZipFile(source) as old, zipfile.ZipFile(result) as new:
        assert new.testzip() is None
        for item in old.infolist():
            if item.filename in exclude:
                continue
            path = replacements.get(item.filename)
            expected = path.read_bytes() if path else old.read(item)
            if new.read(item.filename) != expected:
                raise ValueError("incorrect package member " + item.filename)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, help="Keep a candidate release separate from existing packages")
    for opt in ("android-target", "windows-core", "windows-updater", "build-tools", "keystore", "java"):
        parser.add_argument("--" + opt, type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    output = args.output_dir or root
    output.mkdir(parents=True, exist_ok=True)
    android = root.parent / "Garlic.apk"
    android_signature = {"META-INF/GARLIC.SF", "META-INF/GARLIC.RSA", "META-INF/MANIFEST.MF"}
    windows = root / "Garlic-Windows-rebuilt.zip"
    abi = {"arm64-v8a": "aarch64-linux-android", "armeabi-v7a": "armv7-linux-androideabi"}
    cores = {
        f"lib/{name}/{library}": args.android_target / target / "release" / library
        for name, target in abi.items()
        for library in ("libgarlic_bind.so", "libgarlic_internal_network.so", "libgarlic_updates.so")
    }
    windows_core = {
        "garlic-internal-network.exe": args.windows_core,
        "garlic-ui-update.exe": args.windows_updater,
    }
    aligner = args.build_tools / ("zipalign.exe" if os.name == "nt" else "zipalign")
    signer = [args.java, "-jar", args.build_tools / "lib/apksigner.jar"]
    with tempfile.TemporaryDirectory(dir=output) as tmp:
        stage = Path(tmp)
        apk = stage / "Garlic-rust-core-debug.apk"
        win = stage / "Garlic-Windows-rust-core.zip"
        replace_members(android, stage / "raw.apk", cores, android_signature)
        subprocess.run([aligner, "-P", "16", "-f", "4", stage / "raw.apk", stage / "aligned.apk"], check=True)
        subprocess.run(signer + ["sign", "--ks", args.keystore, "--ks-pass", "env:ANDROID_KEYSTORE_PASSWORD",
                                 "--v4-signing-enabled", "false", "--out", apk, stage / "aligned.apk"], check=True)
        subprocess.run(signer + ["verify", "--verbose", "--print-certs", apk], check=True)
        subprocess.run([aligner, "-c", "-P", "16", "4", apk], check=True)
        verify_members(android, apk, cores, android_signature)
        replace_members(windows, win, windows_core)
        verify_members(windows, win, windows_core)
        for build in [apk, win]:
            digest = hashlib.sha256(build.read_bytes()).hexdigest()
            build.replace(output / build.name)
            print(digest, build.name)


if __name__ == "__main__":
    main()
