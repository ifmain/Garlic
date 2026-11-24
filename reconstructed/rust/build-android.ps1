param([Parameter(Mandatory)][string]$NdkSysroot)
$ErrorActionPreference = 'Stop'
$sysroot = (Resolve-Path -LiteralPath $NdkSysroot).Path
$rustHome = & rustc --print sysroot
if ($LASTEXITCODE) { throw 'Unable to locate the Rust toolchain' }
$linker = (Get-ChildItem (Join-Path $rustHome 'lib/rustlib') -Filter rust-lld.exe -Recurse | Select-Object -First 1).FullName
if (!$linker) { throw 'rust-lld.exe is required' }
foreach ($arch in @(
    @{Target='aarch64-linux-android'; Lib='aarch64-linux-android'; Abi='arm64-v8a'},
    @{Target='armv7-linux-androideabi'; Lib='arm-linux-androideabi'; Abi='armeabi-v7a'}
)) {
    $libs = Join-Path $sysroot ("usr/lib/" + $arch.Lib + "/21")
    if (!(Test-Path -LiteralPath "$libs/libdl.so")) { throw "Missing NDK API 21 library in $libs" }
    $cargoArgs = @('rustc','--release','--locked','--target',$arch.Target,'--target-dir',"$PSScriptRoot/target",
        '--manifest-path',"$PSScriptRoot/Cargo.toml",'--','-C',"linker=$linker",'-C','linker-flavor=ld.lld',
        '-L',"native=$libs",'-C','link-arg=-lc','-C','link-arg=--no-undefined','-C','link-arg=-soname=libgarlic_bind.so')
    & cargo @cargoArgs
    if ($LASTEXITCODE) { throw "Build failed for $($arch.Target)" }
    $out = New-Item -ItemType Directory -Force "$PSScriptRoot/build/$($arch.Abi)"
    Copy-Item "$PSScriptRoot/target/$($arch.Target)/release/libgarlic_bind.so" $out.FullName -Force
}
