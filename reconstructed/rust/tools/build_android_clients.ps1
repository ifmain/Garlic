param(
    [Parameter(Mandatory)][string]$NdkRoot,
    [string]$TargetDirectory = "$PSScriptRoot/../target",
    [switch]$IncludeX86
)
$ErrorActionPreference = 'Stop'
$llvm = Join-Path (Resolve-Path -LiteralPath $NdkRoot) 'toolchains/llvm/prebuilt/windows-x86_64/bin'
$architectures = @(
    @{ Target='aarch64-linux-android'; Compiler='aarch64-linux-android29-clang.cmd' },
    @{ Target='armv7-linux-androideabi'; Compiler='armv7a-linux-androideabi29-clang.cmd' }
)
if ($IncludeX86) {
    $architectures += @{ Target='x86_64-linux-android'; Compiler='x86_64-linux-android29-clang.cmd' }
}
foreach ($arch in $architectures) {
    $target = $arch.Target.Replace('-','_')
    $upper = $target.ToUpperInvariant()
    $compiler = (Resolve-Path -LiteralPath (Join-Path $llvm $arch.Compiler)).Path
    $vars = @{
        "CC_$target" = $compiler
        "AR_$target" = (Join-Path $llvm 'llvm-ar.exe')
        "CARGO_TARGET_${upper}_LINKER" = $compiler
        "CARGO_TARGET_${upper}_RUSTFLAGS" = '-C link-arg=-Wl,-z,max-page-size=16384'
    }
    $previous = @{}
    try {
        foreach ($name in $vars.Keys) {
            $previous[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
            [Environment]::SetEnvironmentVariable($name, $vars[$name], 'Process')
        }
        & cargo build --release --locked --target $arch.Target --target-dir $TargetDirectory --manifest-path "$PSScriptRoot/../Cargo.toml" --lib -p garlic-bind -p garlic-client -p garlic-updates-android
        if ($LASTEXITCODE) { throw "Android build failed for $($arch.Target)" }
    } finally {
        foreach ($name in $previous.Keys) {
            [Environment]::SetEnvironmentVariable($name, $previous[$name], 'Process')
        }
    }
}
