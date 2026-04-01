#!/usr/bin/env bash
# uninstall-all-macos.sh — macOS 完整环境卸载脚本
# Steps: openclaw / Node.js / Git
# Author: OpenValley - Haoming Chen
#
# 用法：
#   bash uninstall-all-macos.sh
#   SKIP_PAUSE=1 bash uninstall-all-macos.sh   # 跳过所有暂停（CI/自动化场景）
#
# 说明：
#   - 需要管理员权限（脚本会自动通过 sudo 申请）
#   - 若 Git 非通过 .pkg 安装（如 Xcode CLT），卸载步骤会跳过并给出提示

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
echo "      Full Environment Uninstall"
echo "  Author: OpenValley - Haoming Chen"
echo "  Steps: openclaw / Node.js / Git"
echo "========================================"
echo

# ═══════════════════════════════════════════════════════════════════════════════
#  STEP 1/3 - openclaw
# ═══════════════════════════════════════════════════════════════════════════════
echo "----------------------------------------"
echo " STEP 1/3  Uninstall openclaw"
echo "----------------------------------------"
echo

if ! command -v npm &>/dev/null; then
    echo "[INFO] npm not found. Skipping openclaw uninstall."
    echo
else
    if HOME="$REAL_HOME" npm list -g --depth=0 2>/dev/null | grep -qi "openclaw"; then
        echo "Uninstalling openclaw globally..."
        HOME="$REAL_HOME" npm uninstall -g openclaw
        npm_exit=$?
        if [[ $npm_exit -ne 0 ]]; then
            echo "[ERROR] Failed to uninstall openclaw (npm exit code: $npm_exit)."
            pause_if_needed
        else
            echo "[OK] openclaw uninstalled successfully"
        fi
        echo
    else
        echo "[INFO] openclaw is not installed globally. Skipping."
        echo
    fi
fi

# ═══════════════════════════════════════════════════════════════════════════════
#  STEP 2/3 - Node.js
# ═══════════════════════════════════════════════════════════════════════════════
echo "----------------------------------------"
echo " STEP 2/3  Uninstall Node.js"
echo "----------------------------------------"
echo

if ! command -v node &>/dev/null; then
    echo "[INFO] Node.js not detected. Skipping."
    echo
else
    node_pkg_id=$(pkgutil --pkgs 2>/dev/null | grep -i "nodejs\|node\.js" | head -1 || true)
    if [[ -n "$node_pkg_id" ]]; then
        echo "Found package: $node_pkg_id"
        pkgutil --forget "$node_pkg_id" 2>/dev/null \
            && echo "[OK] pkgutil record removed: $node_pkg_id" \
            || echo "[WARNING] pkgutil --forget failed for: $node_pkg_id"
    else
        echo "[INFO] Node.js package record not found in pkgutil. Proceeding to remove binaries."
    fi

    echo "Removing Node.js binaries and modules..."
    for f in /usr/local/bin/node /usr/local/bin/npm /usr/local/bin/npx /usr/local/bin/corepack; do
        if [[ -f "$f" ]] || [[ -L "$f" ]]; then
            rm -f "$f" && echo "[OK] Removed: $f" || echo "[WARNING] Failed to remove: $f"
        fi
    done
    for d in /usr/local/lib/node_modules/npm /usr/local/lib/node_modules/corepack; do
        if [[ -d "$d" ]]; then
            rm -rf "$d" && echo "[OK] Removed: $d" || echo "[WARNING] Failed to remove: $d"
        fi
    done
    echo "[OK] Node.js uninstalled"
    echo
fi

# ═══════════════════════════════════════════════════════════════════════════════
#  STEP 3/3 - Git
# ═══════════════════════════════════════════════════════════════════════════════
echo "----------------------------------------"
echo " STEP 3/3  Uninstall Git"
echo "----------------------------------------"
echo

if ! command -v git &>/dev/null; then
    echo "[INFO] Git not detected. Skipping."
    echo
else
    git_pkg_id=$(pkgutil --pkgs 2>/dev/null | grep -i "git" | grep -iv "apple\|com\.apple\|xcode" | head -1 || true)
    if [[ -n "$git_pkg_id" ]]; then
        echo "Found package: $git_pkg_id"
        pkgutil --forget "$git_pkg_id" 2>/dev/null \
            && echo "[OK] pkgutil record removed: $git_pkg_id" \
            || echo "[WARNING] pkgutil --forget failed for: $git_pkg_id"

        if [[ -f "/usr/local/bin/git" ]] || [[ -L "/usr/local/bin/git" ]]; then
            rm -f "/usr/local/bin/git" \
                && echo "[OK] Removed: /usr/local/bin/git" \
                || echo "[WARNING] Failed to remove: /usr/local/bin/git"
        fi
        echo "[OK] Git uninstalled"
    else
        echo "[INFO] No pkg-managed Git installation found."
        echo "       Git may have been installed via Xcode Command Line Tools or other means."
        echo "       Skipping automatic removal — please uninstall manually if needed."
    fi
    echo
fi

# ═══════════════════════════════════════════════════════════════════════════════
#  ALL DONE
# ═══════════════════════════════════════════════════════════════════════════════
echo "========================================"
echo "Uninstall Complete"
echo "========================================"
echo

if [[ "${SKIP_PAUSE:-}" != "1" ]]; then
    read -rp "Press Enter to exit..." < /dev/tty
fi
exit 0
