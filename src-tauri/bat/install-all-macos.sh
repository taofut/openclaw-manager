#!/usr/bin/env bash
# install-all-macos.sh — macOS 完整环境安装脚本
# Steps: Git / Node.js / openclaw / Init
# Author: OpenValley - Haoming Chen
#
# 用法：
#   bash install-all-macos.sh
#   SKIP_PAUSE=1 bash install-all-macos.sh   # 跳过所有暂停（CI/自动化场景）
#
# 前置条件：
#   内网服务器 http://192.168.18.77:12080/ 需提供：
#     git-2.46.0.pkg
#     node-v24.14.0.pkg
#
# 说明：
#   - 需要管理员权限（脚本会自动通过 sudo 申请）
#   - macOS 无 VC++ 对应物，git-2.46.0.pkg 自带编译工具链，共 4 步

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_URL="http://192.168.18.77:12080"

# ─── 验证系统 ──────────────────────────────────────────────────────────────────
if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "[ERROR] This script is for macOS only. Detected OS: $(uname -s)"
    exit 1
fi

# ─── 申请管理员权限 ───────────────────────────────────────────────────────────
if [[ $EUID -ne 0 ]]; then
    echo "[INFO] Requesting administrator privileges..."
    exec sudo env SKIP_PAUSE="${SKIP_PAUSE:-}" bash "$0" "$@"
fi

# ─── 获取真实用户信息（兼容 sudo 场景）──────────────────────────────────────
if [[ -n "${SUDO_USER:-}" ]]; then
    REAL_USER="${SUDO_USER}"
    REAL_HOME=$(eval echo "~${SUDO_USER}")
else
    REAL_USER="$(whoami)"
    REAL_HOME="${HOME}"
fi

# ─── 确保 Homebrew 路径在 sudo 环境中可用 ──────────────────────────────────
for _brew_path in "/opt/homebrew/bin" "/opt/homebrew/sbin" "/usr/local/bin" "/usr/local/sbin"; do
    [[ ":$PATH:" != *":${_brew_path}:"* ]] && [[ -d "$_brew_path" ]] && export PATH="${_brew_path}:$PATH"
done
unset _brew_path

pause_if_needed() {
    if [[ "${SKIP_PAUSE:-}" != "1" ]]; then
        read -rp "Press Enter to continue..." < /dev/tty
    fi
}

echo "========================================"
echo "       Full Environment Setup"
echo "  Steps: Git / Node.js / openclaw / Init"
echo "  Author: OpenValley - Haoming Chen"
echo "========================================"
echo

# ═══════════════════════════════════════════════════════════════════════════════
#  STEP 1/4 - Git
# ═══════════════════════════════════════════════════════════════════════════════
echo "----------------------------------------"
echo " STEP 1/4  Git Environment Setup"
echo "----------------------------------------"
echo

GIT_INSTALLER="git-2.46.0.pkg"
GIT_URL="${BASE_URL}/${GIT_INSTALLER}"

if command -v git &>/dev/null; then
    echo "[OK] Git detected"
    echo "     Version: $(git --version)"
    echo
else
    echo "[INFO] Git not detected. Starting automatic installation..."
    echo

    if [[ -f "${SCRIPT_DIR}/${GIT_INSTALLER}" ]]; then
        echo "[OK] Installer already exists, skipping download: ${SCRIPT_DIR}/${GIT_INSTALLER}"
    else
        echo "Downloading ${GIT_INSTALLER} ..."
        curl -L --progress-bar -o "${SCRIPT_DIR}/${GIT_INSTALLER}" "${GIT_URL}"
    fi

    if [[ ! -f "${SCRIPT_DIR}/${GIT_INSTALLER}" ]]; then
        echo "[ERROR] Download failed. Please install manually: ${GIT_URL}"
        pause_if_needed
        exit 1
    fi
    echo "[OK] Download complete: ${SCRIPT_DIR}/${GIT_INSTALLER}"
    echo

    echo "Installing ${GIT_INSTALLER} ..."
    installer -pkg "${SCRIPT_DIR}/${GIT_INSTALLER}" -target /
    inst_exit=$?
    if [[ $inst_exit -ne 0 ]]; then
        echo "[ERROR] Installation failed (exit code: ${inst_exit})."
        pause_if_needed
        exit 1
    fi
    echo "[OK] Git installed"
    echo
fi

# Configure GitHub HTTPS redirect
echo "Configuring GitHub HTTPS redirect..."
HOME="$REAL_HOME" git config --global url."https://github.com/".insteadOf git@github.com:
gitval=$(HOME="$REAL_HOME" git config --global --get url."https://github.com/".insteadOf 2>/dev/null || true)
if [[ "$gitval" == "git@github.com:" ]]; then
    echo "[OK] GitHub HTTPS redirect configured successfully"
    echo "     Current value: $gitval"
else
    echo "[ERROR] Failed to configure GitHub HTTPS redirect, current value: $gitval"
    pause_if_needed
fi
echo

# Configure git:// protocol HTTPS redirect
echo "Configuring git:// protocol HTTPS redirect..."
HOME="$REAL_HOME" git config --global url."https://".insteadOf git://
gitval=$(HOME="$REAL_HOME" git config --global --get url."https://".insteadOf 2>/dev/null || true)
if [[ "$gitval" == "git://" ]]; then
    echo "[OK] git:// protocol HTTPS redirect configured successfully"
    echo "     Current value: $gitval"
else
    echo "[ERROR] Failed to configure git:// protocol HTTPS redirect, current value: $gitval"
    pause_if_needed
fi
echo

# ═══════════════════════════════════════════════════════════════════════════════
#  STEP 2/4 - Node.js
# ═══════════════════════════════════════════════════════════════════════════════
echo "----------------------------------------"
echo " STEP 2/4  Node.js Environment Setup"
echo "----------------------------------------"
echo

NODE_INSTALLER="node-v24.14.0.pkg"
NODE_URL="${BASE_URL}/${NODE_INSTALLER}"

if command -v node &>/dev/null; then
    nodever=$(node -v | sed 's/^v//')
    major=$(echo "$nodever" | cut -d. -f1)
    echo "[OK] Node.js detected"
    echo "     Version: v${nodever}"
    echo

    if [[ "$major" -lt 22 ]]; then
        echo "[WARNING] Node.js v${nodever} is below the required minimum (v22+)."
        echo "          Please uninstall it first, then re-run this script."
        echo
        pause_if_needed
        exit 1
    fi
    echo "[OK] Node.js version meets requirements (v22+)"
    echo
else
    echo "[INFO] Node.js not detected. Starting automatic installation..."
    echo

    if [[ -f "${SCRIPT_DIR}/${NODE_INSTALLER}" ]]; then
        echo "[OK] Installer already exists, skipping download: ${SCRIPT_DIR}/${NODE_INSTALLER}"
    else
        echo "Downloading ${NODE_INSTALLER} ..."
        curl -L --progress-bar -o "${SCRIPT_DIR}/${NODE_INSTALLER}" "${NODE_URL}"
    fi

    if [[ ! -f "${SCRIPT_DIR}/${NODE_INSTALLER}" ]]; then
        echo "[ERROR] Download failed. Please install manually: ${NODE_URL}"
        pause_if_needed
        exit 1
    fi
    echo "[OK] Download complete: ${SCRIPT_DIR}/${NODE_INSTALLER}"
    echo

    echo "Installing ${NODE_INSTALLER} ..."
    installer -pkg "${SCRIPT_DIR}/${NODE_INSTALLER}" -target /
    inst_exit=$?
    if [[ $inst_exit -ne 0 ]]; then
        echo "[ERROR] Installation failed (exit code: ${inst_exit})."
        pause_if_needed
        exit 1
    fi
    echo "[OK] Node.js installed"
    echo

    export PATH="/usr/local/bin:$PATH"
fi

# Configure npm
if ! command -v npm &>/dev/null; then
    echo "[WARNING] npm was not found. Skipping mirror configuration."
    echo
else
    echo "[OK] npm detected"
    echo "     Version: $(npm -v)"
    echo

    echo "Clearing npm cache..."
    HOME="$REAL_HOME" npm cache clean --force
    npm_cache_exit=$?
    if [[ $npm_cache_exit -eq 0 ]]; then
        echo "[OK] npm cache cleared"
    else
        echo "[WARNING] npm cache clean failed (exit code: ${npm_cache_exit})."
    fi
    echo

    echo "Configuring npm mirror (npmmirror)..."
    HOME="$REAL_HOME" npm config set registry https://registry.npmmirror.com
    registry=$(HOME="$REAL_HOME" npm config get registry 2>/dev/null || true)
    if [[ "$registry" == *"npmmirror"* ]]; then
        echo "[OK] npm mirror configured successfully"
        echo "     Current registry: $registry"
    else
        echo "[ERROR] Failed to configure npm mirror, current value: $registry"
        pause_if_needed
    fi
    echo
fi

# ═══════════════════════════════════════════════════════════════════════════════
#  STEP 3/4 - openclaw
# ═══════════════════════════════════════════════════════════════════════════════
echo "----------------------------------------"
echo " STEP 3/4  openclaw Installer"
echo "----------------------------------------"
echo

if ! command -v node &>/dev/null; then
    echo "[ERROR] Node.js not detected. Ensure Step 2 completed successfully."
    echo
    pause_if_needed
    exit 1
fi

nodever=$(node -v | sed 's/^v//')
major=$(echo "$nodever" | cut -d. -f1)
echo "[OK] Node.js detected"
echo "     Version: v${nodever}"
echo

if [[ "$major" -lt 22 ]]; then
    echo "[ERROR] Node.js v${nodever} is below the required minimum (v22+)."
    echo "        Ensure Step 2 completed successfully."
    echo
    pause_if_needed
    exit 1
fi
echo "[OK] Node.js version meets requirements (v22+)"
echo

echo "Installing openclaw globally..."
HOME="$REAL_HOME" npm install -g openclaw@latest
npm_exit=$?
if [[ $npm_exit -ne 0 ]]; then
    echo "[ERROR] Failed to install openclaw (npm exit code: ${npm_exit})."
    pause_if_needed
    exit 1
fi
echo "[OK] openclaw installed successfully"
echo

echo "Refreshing PATH with npm global directory..."
npm_global=$(HOME="$REAL_HOME" npm prefix -g 2>/dev/null || true)
if [[ -n "$npm_global" ]]; then
    export PATH="${npm_global}/bin:$PATH"
    echo "[OK] Added to PATH: ${npm_global}/bin"
else
    echo "[WARNING] Could not determine npm global directory."
fi
echo

# ═══════════════════════════════════════════════════════════════════════════════
#  STEP 4/4 - openclaw Init
# ═══════════════════════════════════════════════════════════════════════════════
echo "----------------------------------------"
echo " STEP 4/4  openclaw Init"
echo "----------------------------------------"
echo

ZIP_FILE=".openclaw-template.zip"
ZIP_PATH="${SCRIPT_DIR}/res/${ZIP_FILE}"
TARGET_DIR="${REAL_HOME}"

echo "Looking for ${ZIP_FILE} in ${SCRIPT_DIR}/res/"
if [[ ! -f "${ZIP_PATH}" ]]; then
    echo "[ERROR] ${ZIP_FILE} not found: ${ZIP_PATH}"
    pause_if_needed
    exit 1
fi

echo "Extracting ${ZIP_FILE} to ${TARGET_DIR} ..."
unzip -o "${ZIP_PATH}" -d "${TARGET_DIR}"
unzip_exit=$?
if [[ $unzip_exit -eq 0 ]]; then
    echo "[OK] Extraction complete"
else
    echo "[ERROR] Extraction failed (exit code: ${unzip_exit})."
    pause_if_needed
    exit 1
fi
echo

# Fix file ownership in sudo scenario
if [[ -n "${SUDO_USER:-}" ]] && [[ -d "${REAL_HOME}/.openclaw" ]]; then
    chown -R "${REAL_USER}" "${REAL_HOME}/.openclaw" 2>/dev/null || true
fi

CONFIG="${REAL_HOME}/.openclaw/openclaw.json"
if [[ ! -f "$CONFIG" ]]; then
    echo "[WARNING] Config not found, skipping workspace update: ${CONFIG}"
    echo
else
    echo "Updating agents.defaults.workspace in ${CONFIG}..."
    WORKSPACE_PATH="${REAL_HOME}/.openclaw/workspace"

    python3 - "$CONFIG" "$WORKSPACE_PATH" <<'PYEOF'
import json, sys
config_path    = sys.argv[1]
workspace_path = sys.argv[2]
try:
    with open(config_path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    data.setdefault('agents', {}).setdefault('defaults', {})['workspace'] = workspace_path
    with open(config_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
    sys.exit(0)
except Exception as e:
    print(f"[ERROR] {e}", file=sys.stderr)
    sys.exit(1)
PYEOF

    py_exit=$?
    if [[ $py_exit -eq 0 ]]; then
        echo "[OK] agents.defaults.workspace updated successfully"
    else
        echo "[ERROR] Failed to update config (exit code: ${py_exit})"
        pause_if_needed
        exit 1
    fi
    echo
fi

# ═══════════════════════════════════════════════════════════════════════════════
#  ALL DONE
# ═══════════════════════════════════════════════════════════════════════════════
echo "========================================"
echo "All Steps Complete"
echo "========================================"
echo

if [[ "${SKIP_PAUSE:-}" != "1" ]]; then
    read -rp "Press Enter to exit..." < /dev/tty
fi
exit 0
