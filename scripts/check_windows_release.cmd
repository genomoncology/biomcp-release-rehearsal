@echo off
setlocal
call C:\tt0106\BuildTools\VC\Auxiliary\Build\vcvars64.bat >nul
if errorlevel 1 exit /b %errorlevel%
set "RUSTUP_TOOLCHAIN=1.95.0"
set "CARGO_BUILD_JOBS=2"
set "PATH=C:\Users\Docker\.cargo\bin;C:\Users\Docker\AppData\Roaming\uv\python\cpython-3.13.13-windows-x86_64-none;%PATH%"
cd /d "%~1"
if errorlevel 1 exit /b %errorlevel%
set "BIOMCP_BUILD_REVISION=%~3"
python release\source_revision.py
if errorlevel 1 exit /b %errorlevel%
python -c "import sys; from pathlib import Path; from release.artifacts import candidate_record; candidate_record(Path('candidate'), sys.argv[1])" %2
if errorlevel 1 exit /b %errorlevel%
python release\gate.py --artifacts candidate --scratch proof
exit /b %errorlevel%
