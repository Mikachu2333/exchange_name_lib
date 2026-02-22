@echo off
setlocal
cls

echo ========================================================
echo Select Build Toolchain:
echo 1. MSVC x86 and x64 (Static CRT) - Default
echo  (Requires: rustup target add i686-pc-windows-msvc)
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
echo Copying artifact...
copy /Y ".\target\i686-pc-windows-msvc\release\name_exchanger_rs.lib" ".\name_exchanger_x86.lib"

echo Building with MSVC x64 (Static CRT)...
REM Set RUSTFLAGS to statically link the CRT
set RUSTFLAGS=-C target-feature=+crt-static
cargo build --release --target x86_64-pc-windows-msvc
if %ERRORLEVEL% NEQ 0 (
  echo Build failed.
  goto end
)
echo Copying artifact...
copy /Y ".\target\x86_64-pc-windows-msvc\release\name_exchanger_rs.lib" ".\name_exchanger_x64.lib"
goto end

:end
endlocal
