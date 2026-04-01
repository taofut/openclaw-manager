@echo off
setlocal enabledelayedexpansion

:: ─── Require Administrator privileges ────────────────────────────────────────
net session >nul 2>&1
if not "!errorlevel!"=="0" (
    echo [INFO] Requesting administrator privileges...
    powershell -Command "Start-Process cmd.exe -ArgumentList '/c \"%~f0\"' -Verb RunAs -Wait"
    exit /b 0
)

echo ========================================
echo       Full Environment Uninstall
echo   Author: OpenValley - Haoming Chen
echo   Steps: openclaw / VC++ / Node.js / Git
echo ========================================
echo.



:: ═══════════════════════════════════════════════════════════════════════════════
::  STEP 1/4 - openclaw
:: ═══════════════════════════════════════════════════════════════════════════════
echo ----------------------------------------
echo  STEP 1/4  Uninstall openclaw
echo ----------------------------------------
echo.

where npm >nul 2>&1
if not "!errorlevel!"=="0" (
    echo [INFO] npm not found. Skipping openclaw uninstall.
    echo.
) else (
    cmd /c "npm list -g --depth=0 2>nul" | findstr /i "openclaw" >nul 2>&1
    if not "!errorlevel!"=="0" (
        echo [INFO] openclaw is not installed globally. Skipping.
        echo.
    ) else (
        echo Uninstalling openclaw globally...
        cmd /c "npm uninstall -g openclaw"
        set "npm_exit=!errorlevel!"
        if not "!npm_exit!"=="0" (
            echo [ERROR] Failed to uninstall openclaw ^(npm exit code: !npm_exit!^).
            if not "%SKIP_PAUSE%"=="1" pause
        ) else (
            echo [OK] openclaw uninstalled successfully
        )
        echo.
    )
)

:: ═══════════════════════════════════════════════════════════════════════════════
::  STEP 2/4 - Visual C++ Redistributable
:: ═══════════════════════════════════════════════════════════════════════════════
echo ----------------------------------------
echo  STEP 2/4  Uninstall Visual C++ Redistributable
echo ----------------------------------------
echo.

for /f "delims=" %%U in ('powershell -NoProfile -Command ^
    "Get-ChildItem 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall','HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall' -ErrorAction SilentlyContinue | ForEach-Object { $p = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue; if ($p.DisplayName -match 'Visual C\+\+.*Redistributable.*x64') { $p.UninstallString } } | Select-Object -First 1" 2^>nul') do set "vc_uninst=%%U"
if defined vc_uninst (
    echo Found: !vc_uninst!
    cmd /c "!vc_uninst! /quiet /norestart"
    set "inst_exit=!errorlevel!"
    if not "!inst_exit!"=="0" if not "!inst_exit!"=="3010" (
        echo [WARNING] Uninstaller returned exit code: !inst_exit!
    ) else (
        echo [OK] Visual C++ Redistributable uninstalled successfully
    )
) else (
    echo [INFO] Visual C++ Redistributable not found in registry. Skipping.
)
echo.

:: ═══════════════════════════════════════════════════════════════════════════════
::  STEP 3/4 - Node.js
:: ═══════════════════════════════════════════════════════════════════════════════
echo ----------------------------------------
echo  STEP 3/4  Uninstall Node.js
echo ----------------------------------------
echo.

where node >nul 2>&1
if not "!errorlevel!"=="0" (
    echo [INFO] Node.js not detected. Skipping.
    echo.
) else (
    for /f "delims=" %%U in ('powershell -NoProfile -Command ^
        "Get-ChildItem 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall','HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall' -ErrorAction SilentlyContinue | ForEach-Object { $p = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue; if ($p.DisplayName -match 'Node.js') { $_.PSChildName } } | Select-Object -First 1" 2^>nul') do set "node_guid=%%U"
    if defined node_guid (
        echo Found GUID: !node_guid!
        msiexec /x !node_guid! /quiet /norestart
        set "msi_exit=!errorlevel!"
        if not "!msi_exit!"=="0" if not "!msi_exit!"=="3010" (
            echo [WARNING] msiexec returned exit code: !msi_exit!
        ) else (
            echo [OK] Node.js uninstalled successfully
        )
    ) else (
        echo [INFO] Node.js not found in registry. Skipping.
    )

    echo.
)

:: ═══════════════════════════════════════════════════════════════════════════════
::  STEP 4/4 - Git
:: ═══════════════════════════════════════════════════════════════════════════════
echo ----------------------------------------
echo  STEP 4/4  Uninstall Git
echo ----------------------------------------
echo.

where git >nul 2>&1
if not "!errorlevel!"=="0" (
    echo [INFO] Git not detected. Skipping.
    echo.
) else (
    for /f "delims=" %%U in ('powershell -NoProfile -Command ^
        "Get-ChildItem 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall','HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall' -ErrorAction SilentlyContinue | ForEach-Object { $p = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue; if ($p.DisplayName -match 'Git') { $p.UninstallString } } | Select-Object -First 1" 2^>nul') do set "git_uninst=%%U"
    if defined git_uninst (
        echo Found: !git_uninst!
        cmd /c "!git_uninst! /VERYSILENT /NORESTART"
        set "inst_exit=!errorlevel!"
        if not "!inst_exit!"=="0" (
            echo [WARNING] Git uninstaller returned exit code: !inst_exit!
        ) else (
            echo [OK] Git uninstalled successfully
        )
    ) else (
        echo [INFO] Git not found in registry. Skipping.
    )
    echo.
)

:: ═══════════════════════════════════════════════════════════════════════════════
::  ALL DONE
:: ═══════════════════════════════════════════════════════════════════════════════
echo ========================================
echo Uninstall Complete
echo ========================================
echo.
if "%SKIP_PAUSE%"=="1" (
    exit /b 0
)
echo Press any key to exit...
pause >nul
exit /b 0
