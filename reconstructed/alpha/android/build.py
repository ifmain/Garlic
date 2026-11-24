"""Build an isolated Alpha APK from the recovered UI. Uses the workspace SDK and JDK."""
import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[3]
TOOLS = ROOT / ".toolchains"
JDK = next(TOOLS.glob("jdk-*/bin"))
SDK = TOOLS / "android-sdk"
BT = SDK / "build-tools/35.0.0"
JAVA = JDK / "java.exe"


def change(path, before, after):
    text = path.read_text(encoding="utf-8")
    if text.count(before) != 1:
        raise ValueError(f"unexpected input in {path.name}")
    path.write_text(text.replace(before,after),encoding="utf-8")


def build(output, keystore):
    env = dict(os.environ, JAVA_HOME=str(JDK.parent), PATH=str(JDK)+os.pathsep+os.environ["PATH"])
    def run(*cmd):
        subprocess.run(cmd,check=True,env=env)
    output.parent.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        d=Path(tmp)
        src=d/"src"
        shutil.copytree(ROOT/"reconstructed/android-apk",src,ignore=shutil.ignore_patterns("build","original"))
        manifest=src/"AndroidManifest.xml"
        change(manifest,'package="org.garlic.client"','package="ru.ifmain.garlic"')
        change(manifest,'android:label="Garlic"','android:label="Garlic Alpha"')
        change(manifest,' android:appComponentFactory="org.garlic.client.InterfaceFactory"','')
        change(manifest,'<service android:exported="false"', '<service android:exported="true" android:stopWithTask="false"')
        change(manifest,'</service>','<meta-data android:name="android.net.VpnService.SUPPORTS_ALWAYS_ON" android:value="true"/>\n        </service>')
        smali=src/"smali/org/garlic/client"
        change(smali/"GarlicApplication.smali",
               'invoke-static {p0}, Lorg/garlic/client/InterfaceFactory;->initialize(Landroid/app/Application;)V','nop')
        # Only package-owned native libraries are loadable in this preview.
        loader=smali/"CoreLoader.smali"
        text=loader.read_text(encoding="utf-8")
        start=text.index('    .line 19', text.index('.method static initialize('))
        end=text.index('    invoke-virtual {p0}, Landroid/content/Context;->getPackageManager()',start)
        text=text[:start]+"    invoke-static {p0}, Lorg/garlic/client/BackgroundSettings;->applyProtocols(Landroid/content/Context;)V\n\n"+text[end:]
        text,n=re.subn(r'(?ms)(^\.method public static check\(Landroid/content/Context;\)V).*?^\.end method',
                       r'\1\n    .locals 0\n    return-void\n.end method',text)
        assert n==1
        loader.write_text(text,encoding="utf-8")
        for abi, target in (("arm64-v8a","aarch64-linux-android"),("armeabi-v7a","armv7-linux-androideabi")):
            core = ROOT/"reconstructed/rust/target"/target/"release/libgarlic_internal_network.so"
            shutil.copyfile(core, src/"lib"/abi/core.name)
        for lib in (src/"lib").glob("*/libgarlic_updates.so"):
            lib.unlink()
        activity=smali/"MainActivity.smali"
        change(activity,':cond_0\n    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->chooseApps()V',
               ':cond_0\n    const/4 v0, 0x2\n    if-ne p2, v0, :check_protocols\n    invoke-static {p0}, Lorg/garlic/client/BackgroundSettings;->show(Landroid/app/Activity;)V\n    return-void\n    :check_protocols\n    const/4 v0, 0x3\n    if-ne p2, v0, :choose_apps\n    invoke-static {p0}, Lorg/garlic/client/BackgroundSettings;->chooseProtocols(Landroid/app/Activity;)V\n    return-void\n    :choose_apps\n    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->chooseApps()V')
        text=activity.read_text(encoding="utf-8")
        # The home screen exists only after account restoration. Start once per process,
        # and leave an already connecting or running VPN alone.
        home=text.index('.method private showHome()V')
        end_home=text.index('.end method',home)
        last_return=text.rindex('    return-void',home,end_home)
        text=text[:last_return]+'''    invoke-static {p0}, Lorg/garlic/client/BackgroundSettings;->shouldStart(Landroid/app/Activity;)Z
    move-result v0
    if-eqz v0, :alpha_home_done
    sget-boolean v0, Lorg/garlic/client/GarlicProxyService;->isRunning:Z
    if-nez v0, :alpha_home_done
    sget-boolean v0, Lorg/garlic/client/GarlicProxyService;->isConnecting:Z
    if-nez v0, :alpha_home_done
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->startVpn()V
    :alpha_home_done
'''+text[last_return:]
        idx=text.index('.method synthetic lambda$showHome$12')
        end=text.index('.end method',idx)
        block=text[idx:end]
        assert block.count('filled-new-array {v0, v1}')==1
        activity.write_text(text[:idx]+block.replace('.locals 3','.locals 4').replace('filled-new-array {v0, v1}',
            'const-string v2, "Работа в фоне"\n\n    const-string v3, "Протоколы"\n\n    filled-new-array {v0, v1, v2, v3}')+text[end:],encoding="utf-8")
        jar=SDK/"platforms/android-35/android.jar"
        classes=d/"classes"
        classes.mkdir()
        run(JDK/"javac.exe","--release","8","-encoding","UTF-8","-cp",jar,"-d",classes,
            Path(__file__).with_name("BackgroundSettings.java"))
        run(BT/"d8.bat","--min-api","29","--lib",jar,"--output",d,*classes.rglob("*.class"))
        unsigned=d/"unsigned.apk"
        run(JAVA,"-jar",TOOLS/"apktool_3.0.3.jar","b","-p",d/"framework",src,"-o",unsigned)
        with zipfile.ZipFile(unsigned,"a") as z:
            z.write(d/"classes.dex","classes2.dex")
        aligned=d/"aligned.apk"
        run(BT/"zipalign.exe","-P","16","-f","4",unsigned,aligned)
        # A local preview key. Distribution requires the owner's release signing key.
        if not keystore.exists():
            run(JDK/"keytool.exe","-genkeypair","-keystore",keystore,"-storepass","android",
                "-keypass","android","-alias","preview","-keyalg","RSA","-keysize","3072",
                "-validity","3650","-dname","CN=Garlic Local Preview")
        run(BT/"apksigner.bat","sign","--ks",keystore,"--ks-pass","pass:android",
            "--out",output,aligned)
        run(BT/"apksigner.bat","verify","--verbose",output)


if __name__=="__main__":
    p=argparse.ArgumentParser()
    p.add_argument("output",type=Path)
    a=p.parse_args()
    build(a.output.resolve(),TOOLS/"garlic-alpha-preview.p12")
