@echo off
setlocal
cd /d "%~dp0"
if exist "target\release\heartwire.exe" (
  start "" "target\release\heartwire.exe"
  exit /b 0
)
if not defined CMAKE if exist "%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe" set "CMAKE=%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe"
where cargo >nul 2>nul
if errorlevel 1 (
  echo Instale o Rust para compilar ou use o instalador da HeartWire.
  pause
  exit /b 1
)
cargo run -p heartwire
