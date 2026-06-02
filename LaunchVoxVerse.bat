@echo off
setlocal enabledelayedexpansion

:: Get the directory of this script
set "BASE_DIR=%~dp0"

set "REL_EXE=!BASE_DIR!src-tauri\target\release\voxverse.exe"
set "DEB_EXE=!BASE_DIR!src-tauri\target\debug\voxverse.exe"

set "TARGET="

if exist "!REL_EXE!" (
    set "TARGET=!REL_EXE!"
) else if exist "!DEB_EXE!" (
    set "TARGET=!DEB_EXE!"
)


if not defined TARGET (
    powershell -NoProfile -Command "[void][System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms'); [System.Windows.Forms.MessageBox]::Show('VoxVerse executables not found.\n\nPlease compile the project first using one of the following:\n  - pnpm tauri dev (for development)\n  - pnpm tauri build (for release)', 'VoxVerse Launcher', [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Warning)"
    exit /b 1
)

:: Start the program and exit
start "" "!TARGET!"
exit /b 0
