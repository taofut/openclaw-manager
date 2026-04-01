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
echo        Node.js Environment Setup
echo ========================================
echo.

set "NODE_INSTALLER=node-v24.14.0-x64.msi"
set "NODE_URL=http://192.168.18.77:12080/%NODE_INSTALLER%"
set "NODE_INSTALL_DIR=D:\nodejs"

:: ─── Check Node.js ───────────────────────────────────────────────────────────
where node >nul 2>&1
if not "!errorlevel!"=="0" (
    echo [INFO] Node.js not detected. Starting automatic installation...
    echo.

    :: Download installer to current script directory
    echo Downloading %NODE_INSTALLER% ...
    curl -L --progress-bar -o "%~dp0%NODE_INSTALLER%" "%NODE_URL%"
    if not exist "%~dp0%NODE_INSTALLER%" (
        echo [ERROR] Download failed. Please install manually: %NODE_URL%
        if not "%SKIP_PAUSE%"=="1" pause
        exit /b 1
    )
    echo [OK] Download complete: %~dp0%NODE_INSTALLER%
    echo.

    :: Silent install to D:\nodejs
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

    :: Make npm available in the current session
    set "PATH=%NODE_INSTALL_DIR%;!PATH!"
    goto :configure_npm
) else (
    :: ─── Node.js is installed - check version ───────────────────────────────
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
    goto :configure_npm
)

:configure_npm
where npm >nul 2>&1
if not "!errorlevel!"=="0" (
    echo [WARNING] npm was not found. Skipping mirror configuration.
    echo.
    goto :done
)

echo [OK] npm detected
echo      Version:
cmd /c "npm -v"
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

:done
echo ========================================
echo Setup Complete
echo ========================================
echo.
if "%SKIP_PAUSE%"=="1" (
    exit /b 0
)
echo Press any key to exit...
pause >nul
exit /b 0
