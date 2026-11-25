"""Build the Alpha APK, including both native libraries for each supported ABI."""
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import zipfile

APP = Path(__file__).resolve().parent
ROOT = APP.parents[1]


def main():
    sdk = Path(os.environ["ANDROID_HOME"]).resolve()
    ndk = Path(os.environ["ANDROID_NDK_HOME"]).resolve()
    java = Path(os.environ["JAVA_HOME"]).resolve() / "bin"
    apktool = Path(os.environ["APKTOOL_JAR"]).resolve()
    windows = os.name == "nt"
    host = {"Windows": "windows-x86_64", "Linux": "linux-x86_64", "Darwin": "darwin-x86_64"}[platform.system()]
    llvm = ndk / "toolchains/llvm/prebuilt" / host / "bin"
    bt = sdk / "build-tools/35.0.0"
    jar = sdk / "platforms/android-35/android.jar"
    exe = ".exe" if windows else ""
    script = ".bat" if windows else ""
    env = dict(os.environ, PATH=str(java) + os.pathsep + os.environ["PATH"])
    def run(*args):
        subprocess.run(args, cwd=ROOT, env=env, check=True)
    target = Path(json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"], cwd=ROOT
    ))["target_directory"])
    keystore = ROOT / "target/android/debug.p12"
    output = ROOT / "dist/garlic-alpha.apk"
    output.parent.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        app = work / "app"
        shutil.copytree(APP / "app", app)
        for abi, triple, compiler in [
            ("arm64-v8a", "aarch64-linux-android", "aarch64-linux-android"),
            ("x86_64", "x86_64-linux-android", "x86_64-linux-android"),
            ("armeabi-v7a", "armv7-linux-androideabi", "armv7a-linux-androideabi"),
        ]:
            key = triple.replace("-", "_")
            linker = str(llvm / (compiler + "29-clang" + (".cmd" if windows else "")))
            env["CC_" + key] = linker
            env["AR_" + key] = str(llvm / ("llvm-ar" + exe))
            env["CARGO_TARGET_" + key.upper() + "_LINKER"] = linker
            env["CARGO_TARGET_" + key.upper() + "_RUSTFLAGS"] = "-C link-arg=-Wl,-z,max-page-size=16384"
            run("cargo", "build", "--locked", "--release", "--target", triple, "-p", "garlic-client",
                "-p", "garlic-bind", "--features", "garlic-client/alpha", "--lib")
            dest = app / "lib" / abi
            dest.mkdir(parents=True)
            for lib in ["libgarlic_internal_network.so", "libgarlic_bind.so"]:
                shutil.copyfile(target / triple / "release" / lib, dest / lib)
        classes = work / "classes"
        classes.mkdir()
        run(java / ("javac" + exe), "--release", "8", "-encoding", "UTF-8", "-cp", jar, "-d", classes, *sorted(APP.glob("*.java")))
        run(bt / ("d8" + script), "--min-api", "29", "--lib", jar, "--output", work, *classes.rglob("*.class"))
        unsigned = work / "unsigned.apk"
        run(java / ("java" + exe), "-jar", apktool, "b", "-p", work / "framework", app, "-o", unsigned)
        with zipfile.ZipFile(unsigned, "a") as archive:
            archive.write(work / "classes.dex", "classes2.dex")
        aligned = work / "aligned.apk"
        run(bt / ("zipalign" + exe), "-P", "16", "-f", "4", unsigned, aligned)
        if not keystore.exists():
            keystore.parent.mkdir(parents=True,exist_ok=True)
            run(java / ("keytool" + exe), "-genkeypair", "-keystore", keystore, "-storepass", "android",
                "-keypass", "android", "-alias", "debug", "-keyalg", "RSA", "-keysize", "3072",
                "-validity", "3650", "-dname", "CN=Garlic local build")
        run(bt / ("apksigner" + script), "sign", "--ks", keystore, "--ks-pass", "pass:android", "--out", output, aligned)
        run(bt / ("apksigner" + script), "verify", "--verbose", output)


if __name__ == "__main__":
    main()
