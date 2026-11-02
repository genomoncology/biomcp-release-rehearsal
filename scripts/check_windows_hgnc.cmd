@echo off
call C:\tt0106\BuildTools\VC\Auxiliary\Build\vcvars64.bat >nul
if errorlevel 1 exit /b %errorlevel%
set "PATH=C:\Users\Docker\.cargo\bin;C:\Users\Docker\AppData\Roaming\uv\python\cpython-3.13.13-windows-x86_64-none;%PATH%"
set "RUSTUP_TOOLCHAIN=1.95.0"
set "BIOMCP_BUILD_REVISION=%1"
set "CARGO_TARGET_DIR=C:\h\biomcp-0566-target-lean"
set "CARGO_BUILD_JOBS=2"
set "CARGO_PROFILE_DEV_DEBUG=0"
set "CARGO_PROFILE_TEST_DEBUG=0"
set "CARGO_INCREMENTAL=0"
cd /d C:\h\biomcp-0566
python scripts\check_hgnc_recovery.py
