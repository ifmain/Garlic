param([switch]$Alpha, [string]$Core)
$ErrorActionPreference = 'Stop'
$runtime = Join-Path $env:WINDIR 'Microsoft.NET/Framework64/v4.0.30319'
Push-Location $PSScriptRoot
try {
    $original = (Resolve-Path '../../recovery/windows-package').Path
    $dir = if ($Alpha) { 'build-alpha' } else { 'build' }
    $output = New-Item -ItemType Directory -Force $dir
    $flags = @()
    if ($Alpha) {
        if (!$Core) { throw 'Provide the compiled Alpha core with -Core' }
        $corePath = (Resolve-Path $Core).Path
        if ((& $corePath version | ConvertFrom-Json).alpha_profiles -ne $true) {
            throw 'Alpha requires a core built with --features alpha'
        }
        $flags += '/define:GARLIC_ALPHA'
    }
    foreach ($item in Get-ChildItem -LiteralPath $original) {
        if ($item.Name -ne 'Garlic.exe' -and !($Alpha -and $item.Name -in @('garlic-ui-update.exe','garlic-internal-network.exe'))) {
            Copy-Item -LiteralPath $item.FullName -Destination $output.FullName -Recurse -Force
        }
    }
    if ($Alpha) { Copy-Item -LiteralPath $corePath -Destination "$dir/garlic-internal-network.exe" -Force }
    & "$runtime/csc.exe" @flags /nologo /target:winexe /platform:anycpu /optimize+ "/out:$dir/Garlic.exe" "/win32icon:$dir/assets/branding/garlic.ico" /resource:Garlic.Interface.xaml,Garlic.Interface.xaml "/r:$runtime/WPF/WindowsBase.dll" "/r:$runtime/WPF/PresentationCore.dll" "/r:$runtime/WPF/PresentationFramework.dll" "/r:$runtime/System.Xaml.dll" /r:System.Web.Extensions.dll Garlic.cs
    if ($LASTEXITCODE) { throw "C# compiler failed with exit code $LASTEXITCODE" }
} finally {
    Pop-Location
}
