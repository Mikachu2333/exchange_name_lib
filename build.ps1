$ErrorActionPreference = 'Stop'
[System.Console]::OutputEncoding = [System.Console]::InputEncoding = [System.Text.Encoding]::UTF8

$targets = @(
    @{ Triple = 'i686-pc-windows-msvc'; Suffix = 'x86' },
    @{ Triple = 'x86_64-pc-windows-msvc'; Suffix = 'x64' }
)

$previousRustFlags = $env:RUSTFLAGS
try {
    $env:RUSTFLAGS = '-C target-feature=+crt-static'
    foreach ($target in $targets) {
        rustup target add $target.Triple
        cargo build --release --target $target.Triple

        $source = Join-Path $PSScriptRoot "target/$($target.Triple)/release"
        Copy-Item -LiteralPath (Join-Path $source 'exchange_name_lib.lib') `
            -Destination (Join-Path $PSScriptRoot "name_exchanger_$($target.Suffix).lib") -Force
        Copy-Item -LiteralPath (Join-Path $source 'exchange_name_lib.dll') `
            -Destination (Join-Path $PSScriptRoot "name_exchanger_$($target.Suffix).dll") -Force
    }
}
finally {
    $env:RUSTFLAGS = $previousRustFlags
}
