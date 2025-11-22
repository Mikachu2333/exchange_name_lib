cls

@echo off

REM Please install GNU x86 and GNU x64 targets and relative Rust toolchains.
REM GNU could be downloaded from https://winlibs.com/ (Use UCRT runtime please)
REM Rust toolchains could be downloaded using `rustup toolchain install stable-x86_64-pc-windows-gnu`

@echo on
cargo build --release --target i686-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

copy /Y ".\target\i686-pc-windows-gnu\release\name_exchanger_rs.dll" ".\exchange_x86.dll"
copy /Y ".\target\x86_64-pc-windows-gnu\release\name_exchanger_rs.dll" ".\exchange_x64.dll"