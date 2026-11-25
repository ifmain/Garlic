param([string]$Target = 'x86_64-pc-windows-msvc', [string]$CorePath)
$ErrorActionPreference='Stop'
$root=(Resolve-Path "$PSScriptRoot/../..").Path
Push-Location $root
try {
    if (!$CorePath) {
        cargo build --locked --release --target $Target -p garlic-client --features alpha --bin garlic-internal-network --lib
        if ($LASTEXITCODE) {throw 'Core build failed'}
        $targetDir=(cargo metadata --locked --no-deps --format-version 1 | ConvertFrom-Json).target_directory
        $CorePath="$targetDir/$Target/release/garlic-internal-network.exe"
    }
    $out=New-Item -ItemType Directory -Path "$root/dist" -Force
    $work=New-Item -ItemType Directory -Path "$root/target/bundle-$([guid]::NewGuid())"
    $runtime=New-Item -ItemType Directory -Path "$work/runtime"
    Copy-Item "$PSScriptRoot/runtime/*" $runtime.FullName -Recurse
    Copy-Item -LiteralPath $CorePath -Destination "$runtime/garlic-internal-network.exe"
    Copy-Item -LiteralPath (Join-Path (Split-Path $CorePath) "garlic_internal_network.dll") -Destination $runtime.FullName
    Compress-Archive "$runtime/*" "$work/payload.zip"
    $fx="$env:WINDIR/Microsoft.NET/Framework64/v4.0.30319"
    Set-Location -LiteralPath $PSScriptRoot
    & "$fx/csc.exe" /nologo /define:GARLIC_ALPHA,GARLIC_BUNDLE /platform:x64 /optimize+ /target:winexe "/out:$out/Garlic-Alpha-Windows-x64.exe" "/win32icon:$runtime/assets/branding/garlic.ico" "/resource:$work/payload.zip,Garlic.Bundle" /resource:release-public.xml,Garlic.ReleasePublicKey /resource:Garlic.Interface.xaml,Garlic.Interface.xaml "/r:$fx/System.IO.Compression.dll" "/r:$fx/WPF/WindowsBase.dll" "/r:$fx/WPF/PresentationCore.dll" "/r:$fx/WPF/PresentationFramework.dll" "/r:$fx/System.Windows.Forms.dll" "/r:$fx/System.Drawing.dll" "/r:$fx/System.Xaml.dll" "/r:$fx/System.Web.Extensions.dll" Garlic.cs ReleaseClient.cs PluginFiles.cs ModuleCanvas.cs TrayWindow.cs
    if ($LASTEXITCODE) {throw 'Windows application build failed'}
} finally {Pop-Location}
