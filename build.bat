@echo off
setlocal
cls

echo ========================================================
echo MSVC x86 and x64 (Static CRT) - Default
echo  (Requires: rustup target add i686-pc-windows-msvc)
echo ========================================================

echo Building with MSVC x86 (Static CRT)...
REM Set RUSTFLAGS to statically link the CRT
set RUSTFLAGS=-C target-feature=+crt-static
cargo build --release --target i686-pc-windows-msvc
if %ERRORLEVEL% NEQ 0 (
  echo Build failed.
  goto end
)
echo Copying artifact...
copy /Y ".\target\i686-pc-windows-msvc\release\exchange_name_lib.lib" ".\name_exchanger_x86.lib"
copy /Y ".\target\i686-pc-windows-msvc\release\exchange_name_lib.dll" ".\name_exchanger_x86.dll"

echo Building with MSVC x64 (Static CRT)...
REM Set RUSTFLAGS to statically link the CRT
set RUSTFLAGS=-C target-feature=+crt-static
cargo build --release --target x86_64-pc-windows-msvc
if %ERRORLEVEL% NEQ 0 (
  echo Build failed.
  goto end
)
echo Copying artifact...
copy /Y ".\target\x86_64-pc-windows-msvc\release\exchange_name_lib.lib" ".\name_exchanger_x64.lib"
copy /Y ".\target\x86_64-pc-windows-msvc\release\exchange_name_lib.dll" ".\name_exchanger_x64.dll"
goto end

:end
endlocal
