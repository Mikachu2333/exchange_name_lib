@echo off
setlocal
cls

echo ========================================================
echo Select Build Toolchain:
echo 1. MSVC x86 and x64 (Static CRT) - Default
echo  (Requires: rustup target add i686-pc-windows-msvc)
echo 2. GNU (x86 and x64)
echo  (Requires: rustup target add i686-pc-windows-gnu x86_64-pc-windows-gnu)
echo ========================================================

set "choice=1"
set /p "choice=Enter choice (1 or 2, default is 1): "

if "%choice%"=="1" goto build_msvc
if "%choice%"=="2" goto build_gnu

echo Invalid choice.
goto end

:build_msvc
echo.
echo Building with MSVC x86 (Static CRT)...
REM Set RUSTFLAGS to statically link the CRT
set RUSTFLAGS=-C target-feature=+crt-static
cargo build --release --target i686-pc-windows-msvc
if %ERRORLEVEL% NEQ 0 (
  echo Build failed.
  goto end
)
echo Copying artifact to exchange_x86.dll...
copy /Y ".\target\i686-pc-windows-msvc\release\name_exchanger_rs.dll" ".\name_exchanger_rs.dll"
copy /Y ".\target\i686-pc-windows-msvc\release\name_exchanger_rs.lib" ".\name_exchanger.lib"

echo Building with MSVC x64 (Static CRT)...
REM Set RUSTFLAGS to statically link the CRT
set RUSTFLAGS=-C target-feature=+crt-static
cargo build --release --target x86_64-pc-windows-msvc
if %ERRORLEVEL% NEQ 0 (
  echo Build failed.
  goto end
)
echo Copying artifact to exchange_x86.dll...
copy /Y ".\target\x86_64-pc-windows-msvc\release\name_exchanger_rs.dll" ".\name_exchanger_rs_x64.dll"
copy /Y ".\target\x86_64-pc-windows-msvc\release\name_exchanger_rs.lib" ".\name_exchanger_x64.lib"
goto end

:build_gnu
echo.
echo Building with GNU toolchain...
REM GNU could be downloaded from https://winlibs.com/ (Use UCRT runtime please)
echo.
cargo build --release --target i686-pc-windows-gnu
if %ERRORLEVEL% NEQ 0 (
  echo Build x86 GNU failed.
  goto end
)
cargo build --release --target x86_64-pc-windows-gnu
if %ERRORLEVEL% NEQ 0 (
  echo Build x64 GNU failed.
  goto end
)

echo Copying artifacts...
copy /Y ".\target\i686-pc-windows-gnu\release\name_exchanger_rs.dll" ".\name_exchanger_rs_x86.dll"
copy /Y ".\target\x86_64-pc-windows-gnu\release\name_exchanger_rs.dll" ".\name_exchanger_rs.dll"
goto end

:end
endlocal
