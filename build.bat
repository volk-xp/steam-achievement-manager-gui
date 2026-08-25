@echo off
rem ---------------------------------------------------------------------------
rem Builds sam.exe and copies it, with the Steam DLL it needs, into a folder you
rem can run it from.
rem
rem   build.bat                  puts it in the default folder below
rem   build.bat "D:\Games\SAM"   puts it wherever you say instead
rem ---------------------------------------------------------------------------

setlocal

set "DEST=%~1"
if "%DEST%"=="" set "DEST=C:\Users\MSI\Videos\Volk SAM"

rem Run from the folder this script lives in, so it works from any terminal.
cd /d "%~dp0"

where cargo >nul 2>nul
if errorlevel 1 (
    echo.
    echo Rust is not installed, or this terminal was opened before you installed it.
    echo Install it from https://rustup.rs, then close and reopen the terminal.
    echo See BUILD.md, step 1.
    exit /b 1
)

echo Building. The first run takes 5 to 12 minutes.
echo.
cargo build --release
if errorlevel 1 (
    echo.
    echo The build failed, so nothing was copied. BUILD.md has a section called
    echo "If the build fails" covering each error this can produce.
    exit /b 1
)

rem steamworks-sys unpacks the Steam SDK during the same build, so on a cold
rem first build the copy step can run before the DLL exists. Once more fixes it.
if not exist "target\release\steam_api64.dll" (
    echo.
    echo steam_api64.dll was not ready in time. Building once more...
    echo.
    cargo build --release
)

if not exist "target\release\sam.exe" (
    echo.
    echo Could not find target\release\sam.exe. Nothing was copied.
    exit /b 1
)

if not exist "target\release\steam_api64.dll" (
    echo.
    echo Built sam.exe but could not find steam_api64.dll, which it needs to run.
    echo See BUILD.md, "If the build fails", for how to copy it by hand.
    exit /b 1
)

if not exist "%DEST%" mkdir "%DEST%"

copy /y "target\release\sam.exe" "%DEST%\" >nul
if errorlevel 1 goto copyfailed
copy /y "target\release\steam_api64.dll" "%DEST%\" >nul
if errorlevel 1 goto copyfailed

echo.
echo Done. sam.exe and steam_api64.dll are in:
echo   %DEST%
echo.
echo Start Steam and sign in, then run sam.exe.
endlocal
exit /b 0

:copyfailed
echo.
echo Could not copy into "%DEST%".
echo If sam.exe is currently running, close it and run this again.
exit /b 1
