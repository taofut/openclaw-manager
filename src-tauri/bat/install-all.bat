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
echo        Full Environment Setup
echo   Steps: Git / Node.js / VC++ / openclaw / Init
echo   Author: OpenValley - Haoming Chen
echo ========================================
echo.

:: ═══════════════════════════════════════════════════════════════════════════════
::  STEP 1/5 - Git
:: ═══════════════════════════════════════════════════════════════════════════════
echo ----------------------------------------
echo  STEP 1/5  Git Environment Setup
echo ----------------------------------------
echo.

set "GIT_INSTALLER=Git-2.53.0-64-bit.exe"
set "GIT_URL=http://192.168.18.77:12080/%GIT_INSTALLER%"
set "GIT_INSTALL_DIR=D:\Git"

where git >nul 2>&1
if not "!errorlevel!"=="0" (
    echo [INFO] Git not detected. Starting automatic installation...
    echo.

    if exist "%~dp0%GIT_INSTALLER%" (
        echo [OK] Installer already exists, skipping download: %~dp0%GIT_INSTALLER%
    ) else (
        echo Downloading %GIT_INSTALLER% ...
        curl -L --progress-bar -o "%~dp0%GIT_INSTALLER%" "%GIT_URL%"
    )
    if not exist "%~dp0%GIT_INSTALLER%" (
        echo [ERROR] Download failed. Please install manually: %GIT_URL%
        if not "%SKIP_PAUSE%"=="1" pause
        exit /b 1
    )
    echo [OK] Download complete: %~dp0%GIT_INSTALLER%
    echo.

    echo Installing Git to %GIT_INSTALL_DIR% ...
    "%~dp0%GIT_INSTALLER%" /VERYSILENT /NORESTART /NOCANCEL /SP- /CLOSEAPPLICATIONS /RESTARTAPPLICATIONS /DIR="%GIT_INSTALL_DIR%"
    set "inst_exit=!errorlevel!"
    if not "!inst_exit!"=="0" (
        echo [ERROR] Installation failed ^(exit code: !inst_exit!^).
        if not "%SKIP_PAUSE%"=="1" pause
        exit /b 1
    )
    echo [OK] Git installed to %GIT_INSTALL_DIR%
    echo.

    set "PATH=%GIT_INSTALL_DIR%\bin;!PATH!"
) else (
    echo [OK] Git detected
    echo      Version:
    git --version
    echo.
)

:: Configure GitHub HTTPS redirect
echo Configuring GitHub HTTPS redirect...
git.exe config --global url."https://github.com/".insteadOf git@github.com:
for /f "delims=" %%V in ('git.exe config --global --get url."https://github.com/".insteadOf 2^>nul') do set "gitval=%%V"
if "!gitval!"=="git@github.com:" (
    echo [OK] GitHub HTTPS redirect configured successfully
    echo      Current value: !gitval!
) else (
    echo [ERROR] Failed to configure GitHub HTTPS redirect, current value: !gitval!
    if not "%SKIP_PAUSE%"=="1" pause
)
echo.

:: Configure git:// protocol HTTPS redirect
echo Configuring git:// protocol HTTPS redirect...
git.exe config --global url."https://".insteadOf git://
for /f "delims=" %%V in ('git.exe config --global --get url."https://".insteadOf 2^>nul') do set "gitval=%%V"
if "!gitval!"=="git://" (
    echo [OK] git:// protocol HTTPS redirect configured successfully
    echo      Current value: !gitval!
) else (
    echo [ERROR] Failed to configure git:// protocol HTTPS redirect, current value: !gitval!
    if not "%SKIP_PAUSE%"=="1" pause
)
echo.

:: ═══════════════════════════════════════════════════════════════════════════════
::  STEP 2/5 - Node.js
:: ═══════════════════════════════════════════════════════════════════════════════
echo ----------------------------------------
echo  STEP 2/5  Node.js Environment Setup
echo ----------------------------------------
echo.

set "NODE_INSTALLER=node-v24.14.0-x64.msi"
set "NODE_URL=http://192.168.18.77:12080/%NODE_INSTALLER%"
set "NODE_INSTALL_DIR=D:\nodejs"

where node >nul 2>&1
if not "!errorlevel!"=="0" (
    echo [INFO] Node.js not detected. Starting automatic installation...
    echo.

    if exist "%~dp0%NODE_INSTALLER%" (
        echo [OK] Installer already exists, skipping download: %~dp0%NODE_INSTALLER%
    ) else (
        echo Downloading %NODE_INSTALLER% ...
        curl -L --progress-bar -o "%~dp0%NODE_INSTALLER%" "%NODE_URL%"
    )
    if not exist "%~dp0%NODE_INSTALLER%" (
        echo [ERROR] Download failed. Please install manually: %NODE_URL%
        if not "%SKIP_PAUSE%"=="1" pause
        exit /b 1
    )
    echo [OK] Download complete: %~dp0%NODE_INSTALLER%
    echo.

    echo Installing Node.js to %NODE_INSTALL_DIR% ...
    msiexec /i "%~dp0%NODE_INSTALLER%" /quiet /norestart INSTALLDIR="%NODE_INSTALL_DIR%\"
    set "msi_exit=!errorlevel!"
    if not "!msi_exit!"=="0" if not "!msi_exit!"=="3010" (
        echo [ERROR] Installation failed ^(msiexec exit code: !msi_exit!^).
        if not "%SKIP_PAUSE%"=="1" pause
        exit /b 1
    )
    echo [OK] Node.js installed to %NODE_INSTALL_DIR%
    echo.

    set "PATH=%NODE_INSTALL_DIR%;!PATH!"
) else (
    for /f %%A in ('node -v') do set "nodever=%%A"
    set "nodever=!nodever:~1!"
    for /f "tokens=1 delims=." %%A in ("!nodever!") do set "major=%%A"

    echo [OK] Node.js detected
    echo      Version: v!nodever!
    echo.

    if !major! LSS 22 (
        echo [WARNING] Node.js v!nodever! is below the required minimum ^(v22+^).
        echo           Please uninstall it first, then re-run this script.
        echo.
        if not "%SKIP_PAUSE%"=="1" pause
        exit /b 1
    )

    echo [OK] Node.js version meets requirements ^(v22+^)
    echo.
)

:: Configure npm mirror
where npm >nul 2>&1
if not "!errorlevel!"=="0" (
    echo [WARNING] npm was not found. Skipping mirror configuration.
    echo.
) else (
    echo [OK] npm detected
    echo      Version:
    cmd /c "npm -v"
    echo.

    echo Clearing npm cache...
    cmd /c "npm cache clean --force"
    set "npm_cache_exit=!errorlevel!"
    if "!npm_cache_exit!"=="0" (
        echo [OK] npm cache cleared
    ) else (
        echo [WARNING] npm cache clean failed ^(exit code: !npm_cache_exit!^).
    )
    echo.

    echo Configuring npm mirror ^(npmmirror^)...
    cmd /c "npm config set registry https://registry.npmmirror.com"
    for /f "delims=" %%R in ('cmd /c "npm config get registry" 2^>nul') do set "registry=%%R"
    if /i not "!registry:npmmirror=!"=="!registry!" (
        echo [OK] npm mirror configured successfully
        echo      Current registry: !registry!
    ) else (
        echo [ERROR] Failed to configure npm mirror, current value: !registry!
        if not "%SKIP_PAUSE%"=="1" pause
    )
    echo.
)

:: ═══════════════════════════════════════════════════════════════════════════════
::  STEP 3/5 - Visual C++ Redistributable
:: ═══════════════════════════════════════════════════════════════════════════════
echo ----------------------------------------
echo  STEP 3/5  Visual C++ Redistributable
echo ----------------------------------------
echo.

set "VC_INSTALLER=VC_redist.x64.exe"
set "VC_URL=http://192.168.18.77:12080/%VC_INSTALLER%"

where cmake >nul 2>&1
if "!errorlevel!"=="0" (
    echo [OK] cmake detected
    echo      Version:
    cmake --version | findstr /i "cmake version"
    echo.
    echo [INFO] cmake is already available. Skipping VC++ installation.
    echo.
) else (
    echo [INFO] cmake not detected. Installing Visual C++ Redistributable...
    echo.

    if exist "%~dp0%VC_INSTALLER%" (
        echo [OK] Installer already exists, skipping download: %~dp0%VC_INSTALLER%
    ) else (
        echo Downloading %VC_INSTALLER% ...
        curl -L --progress-bar -o "%~dp0%VC_INSTALLER%" "%VC_URL%"
    )
    if not exist "%~dp0%VC_INSTALLER%" (
        echo [ERROR] Download failed. Please install manually: %VC_URL%
        if not "%SKIP_PAUSE%"=="1" pause
        exit /b 1
    )
    echo [OK] Download complete: %~dp0%VC_INSTALLER%
    echo.

    echo Installing %VC_INSTALLER% ...
    "%~dp0%VC_INSTALLER%" /install /quiet /norestart
    set "inst_exit=!errorlevel!"
    if "!inst_exit!"=="1638" (
        echo [OK] A newer version of Visual C++ Redistributable is already installed. Skipping.
    ) else if not "!inst_exit!"=="0" if not "!inst_exit!"=="3010" (
        echo [ERROR] Installation failed ^(exit code: !inst_exit!^).
        if not "%SKIP_PAUSE%"=="1" pause
        exit /b 1
    ) else (
        echo [OK] Visual C++ Redistributable installed successfully
    )
    echo.
)

:: ═══════════════════════════════════════════════════════════════════════════════
::  STEP 4/5 - openclaw
:: ═══════════════════════════════════════════════════════════════════════════════
echo ----------------------------------------
echo  STEP 4/5  openclaw Installer
echo ----------------------------------------
echo.

where node >nul 2>&1
if not "!errorlevel!"=="0" (
    echo [ERROR] Node.js not detected. Ensure Step 2 completed successfully.
    echo.
    if not "%SKIP_PAUSE%"=="1" pause
    exit /b 1
)

for /f %%A in ('node -v') do set "nodever=%%A"
set "nodever=!nodever:~1!"
for /f "tokens=1 delims=." %%A in ("!nodever!") do set "major=%%A"

echo [OK] Node.js detected
echo      Version: v!nodever!
echo.

if !major! LSS 22 (
    echo [ERROR] Node.js v!nodever! is below the required minimum ^(v22+^).
    echo         Ensure Step 2 completed successfully.
    echo.
    if not "%SKIP_PAUSE%"=="1" pause
    exit /b 1
)

echo [OK] Node.js version meets requirements ^(v22+^)
echo.

echo Installing openclaw globally...
cmd /c "npm install -g openclaw@latest"
set "npm_exit=!errorlevel!"
if not "!npm_exit!"=="0" (
    echo [ERROR] Failed to install openclaw ^(npm exit code: !npm_exit!^).
    if not "%SKIP_PAUSE%"=="1" pause
    exit /b 1
)
echo [OK] openclaw installed successfully
echo.

echo Refreshing PATH with npm global directory...
for /f "delims=" %%P in ('cmd /c "npm prefix -g" 2^>nul') do set "npm_global=%%P"
if defined npm_global (
    set "PATH=!npm_global!;!PATH!"
    echo [OK] Added to PATH: !npm_global!
) else (
    echo [WARNING] Could not determine npm global directory.
)
echo.

:: ═══════════════════════════════════════════════════════════════════════════════
::  STEP 5/5 - openclaw Init
:: ═══════════════════════════════════════════════════════════════════════════════
echo ----------------------------------------
echo  STEP 5/5  openclaw Init
echo ----------------------------------------
echo.

set "ZIP_FILE=.openclaw-template.zip"
set "SCRIPT_DIR=%~dp0res\"
set "TARGET_DIR=%USERPROFILE%"

echo Looking for %ZIP_FILE% in %SCRIPT_DIR%
if not exist "%SCRIPT_DIR%%ZIP_FILE%" (
    echo [ERROR] %ZIP_FILE% not found in script directory: %SCRIPT_DIR%
    if not "%SKIP_PAUSE%"=="1" pause
    exit /b 1
)

echo Extracting %ZIP_FILE% to %TARGET_DIR% ...
powershell -NoProfile -Command "Expand-Archive -LiteralPath '%SCRIPT_DIR%%ZIP_FILE%' -DestinationPath '%TARGET_DIR%' -Force"
set "ps_exit=!errorlevel!"
if "!ps_exit!"=="0" (
    echo [OK] Extraction complete
) else (
    echo [ERROR] Extraction failed ^(exit code: !ps_exit!^).
    if not "%SKIP_PAUSE%"=="1" pause
    exit /b 1
)
echo.

set "CONFIG=%USERPROFILE%\.openclaw\openclaw.json"
if not exist "%CONFIG%" (
    echo [WARNING] Config not found, skipping workspace update: %CONFIG%
    echo.
) else (
    echo Updating agents.defaults.workspace in %CONFIG%
    powershell -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; try { $q=[char]34; $f='%CONFIG%'; $c=[System.IO.File]::ReadAllText($f,[Text.Encoding]::UTF8); $pat=$q+'workspace'+$q+':\s*'+$q+'[^'+$q+']*'+$q; $path=($env:USERPROFILE+'\.openclaw\worksapce') -replace '\\','\\'; $rep=$q+'workspace'+$q+': '+$q+$path+$q; $c=$c -replace $pat,$rep; [System.IO.File]::WriteAllText($f,$c,[Text.Encoding]::UTF8); exit 0 } catch { Write-Host ('[ERROR] '+$_.Exception.Message); exit 1 }"
    set "ps_exit=!errorlevel!"
    if "!ps_exit!"=="0" (
        echo [OK] agents.defaults.workspace updated successfully
    ) else (
        echo [ERROR] Failed to update config ^(exit code: !ps_exit!^)
        if not "%SKIP_PAUSE%"=="1" pause
        exit /b 1
    )
    echo.
)

:: ═══════════════════════════════════════════════════════════════════════════════
::  ALL DONE
:: ═══════════════════════════════════════════════════════════════════════════════
echo ========================================
echo All Steps Complete
echo ========================================
echo.
if "%SKIP_PAUSE%"=="1" (
    exit /b 0
)
echo Press any key to exit...
pause >nul
exit /b 0
