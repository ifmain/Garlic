param([string]$Target = 'x86_64-pc-windows-msvc')
$ErrorActionPreference = 'Stop'
$project = (Resolve-Path "$PSScriptRoot/../..").Path
Push-Location $project
try {
    cargo build --locked --release --target $Target -p garlic-client --features alpha --bin garlic-internal-network
    if ($LASTEXITCODE) { throw 'Rust client build failed' }
    $targetDir = (cargo metadata --locked --no-deps --format-version 1 | ConvertFrom-Json).target_directory
    $output = New-Item -Force -ItemType Directory "$project/dist/windows"
    Copy-Item "$PSScriptRoot/runtime/*" $output.FullName -Recurse -Force
    Copy-Item "$targetDir/$Target/release/garlic-internal-network.exe" $output.FullName -Force
    $fx = "$env:WINDIR/Microsoft.NET/Framework64/v4.0.30319"
    Set-Location $PSScriptRoot
    & "$fx/csc.exe" /nologo /define:GARLIC_ALPHA /target:winexe /platform:x64 /optimize+ "/out:$($output.FullName)\Garlic.exe" "/win32icon:$($output.FullName)\assets\branding\garlic.ico" /resource:Garlic.Interface.xaml,Garlic.Interface.xaml "/r:$fx/WPF/WindowsBase.dll" "/r:$fx/WPF/PresentationCore.dll" "/r:$fx/WPF/PresentationFramework.dll" "/r:$fx/System.Xaml.dll" /r:System.Web.Extensions.dll Garlic.cs
    if ($LASTEXITCODE) { throw 'Windows UI build failed' }
    Compress-Archive "$output/*" "$project/dist/Garlic-Alpha-Windows-x64.zip" -Force
} finally { Pop-Location }
