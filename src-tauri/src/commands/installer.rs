use crate::utils::{platform, shell};
use serde::{Deserialize, Serialize};
use tauri::command;
use log::{info, warn, error, debug};
use std::path::PathBuf;
use std::io::Write;
use zip::ZipArchive;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentStatus {
    /// Node.js 是否安装
    pub node_installed: bool,
    /// Node.js 版本
    pub node_version: Option<String>,
    /// Node.js 版本是否满足要求 (>=22)
    pub node_version_ok: bool,
    /// OpenClaw 是否安装
    pub openclaw_installed: bool,
    /// OpenClaw 版本
    pub openclaw_version: Option<String>,
    /// 配置目录是否存在
    pub config_dir_exists: bool,
    /// 是否全部就绪
    pub ready: bool,
    /// 操作系统
    pub os: String,
}

/// 安装进度
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub step: String,
    pub progress: u8,
    pub message: String,
    pub error: Option<String>,
}

/// 安装结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
}

/// 检查环境状态
#[command]
pub async fn check_environment() -> Result<EnvironmentStatus, String> {
    info!("[环境检查] 开始检查系统环境...");
    
    let os = platform::get_os();
    info!("[环境检查] 操作系统: {}", os);
    
    // 检查 Node.js
    info!("[环境检查] 检查 Node.js...");
    let node_version = get_node_version();
    let node_installed = node_version.is_some();
    let node_version_ok = check_node_version_requirement(&node_version);
    info!("[环境检查] Node.js: installed={}, version={:?}, version_ok={}", 
        node_installed, node_version, node_version_ok);
    
    // 检查 OpenClaw
    info!("[环境检查] 检查 OpenClaw...");
    let openclaw_version = get_openclaw_version();
    let openclaw_installed = openclaw_version.is_some();
    info!("[环境检查] OpenClaw: installed={}, version={:?}", 
        openclaw_installed, openclaw_version);
    
    // 检查配置目录
    let config_dir = platform::get_config_dir();
    let config_dir_exists = std::path::Path::new(&config_dir).exists();
    info!("[环境检查] 配置目录: {}, exists={}", config_dir, config_dir_exists);
    
    let ready = node_installed && node_version_ok && openclaw_installed;
    info!("[环境检查] 环境就绪状态: ready={}", ready);
    
    Ok(EnvironmentStatus {
        node_installed,
        node_version,
        node_version_ok,
        openclaw_installed,
        openclaw_version,
        config_dir_exists,
        ready,
        os,
    })
}

/// 获取 Node.js 版本
/// 检测多个可能的安装路径，因为 GUI 应用不继承用户 shell 的 PATH
fn get_node_version() -> Option<String> {
    if platform::is_windows() {
        // Windows: 先尝试直接调用（如果 PATH 已更新）
        if let Ok(v) = shell::run_cmd_output("node --version") {
            let version = v.trim().to_string();
            if !version.is_empty() && version.starts_with('v') {
                info!("[环境检查] 通过 PATH 找到 Node.js: {}", version);
                return Some(version);
            }
        }
        
        // Windows: 检查常见的安装路径
        let possible_paths = get_windows_node_paths();
        for path in possible_paths {
            if std::path::Path::new(&path).exists() {
                // 使用完整路径执行
                let cmd = format!("\"{}\" --version", path);
                if let Ok(output) = shell::run_cmd_output(&cmd) {
                    let version = output.trim().to_string();
                    if !version.is_empty() && version.starts_with('v') {
                        info!("[环境检查] 在 {} 找到 Node.js: {}", path, version);
                        return Some(version);
                    }
                }
            }
        }
        
        None
    } else {
        // 先尝试直接调用
        if let Ok(v) = shell::run_command_output("node", &["--version"]) {
            return Some(v.trim().to_string());
        }
        
        // 检测常见的 Node.js 安装路径（macOS/Linux）
        let possible_paths = get_unix_node_paths();
        for path in possible_paths {
            if std::path::Path::new(&path).exists() {
                if let Ok(output) = shell::run_command_output(&path, &["--version"]) {
                    info!("[环境检查] 在 {} 找到 Node.js: {}", path, output.trim());
                    return Some(output.trim().to_string());
                }
            }
        }
        
        // 尝试通过 shell 加载用户环境来检测
        if let Ok(output) = shell::run_bash_output("source ~/.zshrc 2>/dev/null || source ~/.bashrc 2>/dev/null; node --version 2>/dev/null") {
            if !output.is_empty() && output.starts_with('v') {
                info!("[环境检查] 通过用户 shell 找到 Node.js: {}", output.trim());
                return Some(output.trim().to_string());
            }
        }
        
        None
    }
}

/// 获取 Unix 系统上可能的 Node.js 路径
fn get_unix_node_paths() -> Vec<String> {
    let mut paths = Vec::new();
    
    // Homebrew (macOS)
    paths.push("/opt/homebrew/bin/node".to_string()); // Apple Silicon
    paths.push("/usr/local/bin/node".to_string());     // Intel Mac
    
    // 系统安装
    paths.push("/usr/bin/node".to_string());
    
    // nvm (检查常见版本)
    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();
        
        // nvm 默认版本
        paths.push(format!("{}/.nvm/versions/node/v22.0.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.1.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.2.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.11.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.12.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v23.0.0/bin/node", home_str));
        
        // 尝试 nvm alias default（读取 nvm 的 default alias）
        let nvm_default = format!("{}/.nvm/alias/default", home_str);
        if let Ok(version) = std::fs::read_to_string(&nvm_default) {
            let version = version.trim();
            if !version.is_empty() {
                paths.insert(0, format!("{}/.nvm/versions/node/v{}/bin/node", home_str, version));
            }
        }
        
        // fnm
        paths.push(format!("{}/.fnm/aliases/default/bin/node", home_str));
        
        // volta
        paths.push(format!("{}/.volta/bin/node", home_str));
        
        // asdf
        paths.push(format!("{}/.asdf/shims/node", home_str));
        
        // mise (formerly rtx)
        paths.push(format!("{}/.local/share/mise/shims/node", home_str));
    }
    
    paths
}

/// 获取 Windows 系统上可能的 Node.js 路径
fn get_windows_node_paths() -> Vec<String> {
    let mut paths = Vec::new();
    
    // 1. 标准安装路径 (Program Files)
    paths.push("C:\\Program Files\\nodejs\\node.exe".to_string());
    paths.push("C:\\Program Files (x86)\\nodejs\\node.exe".to_string());
    
    // 2. nvm for Windows (nvm4w) - 常见安装位置
    paths.push("C:\\nvm4w\\nodejs\\node.exe".to_string());
    
    // 3. 用户目录下的各种安装
    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();
        
        // nvm for Windows 用户安装
        paths.push(format!("{}\\AppData\\Roaming\\nvm\\current\\node.exe", home_str));
        
        // fnm (Fast Node Manager) for Windows
        paths.push(format!("{}\\AppData\\Roaming\\fnm\\aliases\\default\\node.exe", home_str));
        paths.push(format!("{}\\AppData\\Local\\fnm\\aliases\\default\\node.exe", home_str));
        paths.push(format!("{}\\.fnm\\aliases\\default\\node.exe", home_str));
        
        // volta
        paths.push(format!("{}\\AppData\\Local\\Volta\\bin\\node.exe", home_str));
        // volta 通过 shim 调用，检查 bin 目录即可
        
        // scoop 安装
        paths.push(format!("{}\\scoop\\apps\\nodejs\\current\\node.exe", home_str));
        paths.push(format!("{}\\scoop\\apps\\nodejs-lts\\current\\node.exe", home_str));
        
        // chocolatey 安装
        paths.push("C:\\ProgramData\\chocolatey\\lib\\nodejs\\tools\\node.exe".to_string());
    }
    
    // 4. 从注册表读取的安装路径（通过环境变量间接获取）
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        paths.push(format!("{}\\nodejs\\node.exe", program_files));
    }
    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        paths.push(format!("{}\\nodejs\\node.exe", program_files_x86));
    }
    
    // 5. nvm-windows 的符号链接路径（NVM_SYMLINK 环境变量）
    if let Ok(nvm_symlink) = std::env::var("NVM_SYMLINK") {
        paths.insert(0, format!("{}\\node.exe", nvm_symlink));
    }
    
    // 6. nvm-windows 的 NVM_HOME 路径下的当前版本
    if let Ok(nvm_home) = std::env::var("NVM_HOME") {
        // 尝试读取当前激活的版本
        let settings_path = format!("{}\\settings.txt", nvm_home);
        if let Ok(content) = std::fs::read_to_string(&settings_path) {
            for line in content.lines() {
                if line.starts_with("current:") {
                    if let Some(version) = line.strip_prefix("current:") {
                        let version = version.trim();
                        if !version.is_empty() {
                            paths.insert(0, format!("{}\\v{}\\node.exe", nvm_home, version));
                        }
                    }
                }
            }
        }
    }
    
    paths
}

/// 获取 OpenClaw 版本
fn get_openclaw_version() -> Option<String> {
    // 使用 run_openclaw 统一处理各平台
    shell::run_openclaw(&["--version"])
        .ok()
        .map(|v| v.trim().to_string())
}

/// 检查 Node.js 版本是否 >= 22
fn check_node_version_requirement(version: &Option<String>) -> bool {
    if let Some(v) = version {
        // 解析版本号 "v22.1.0" -> 22
        let major = v.trim_start_matches('v')
            .split('.')
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        major >= 22
    } else {
        false
    }
}

/// 安装 Node.js
#[command]
pub async fn install_nodejs() -> Result<InstallResult, String> {
    info!("[安装Node.js] 开始安装 Node.js...");
    let os = platform::get_os();
    info!("[安装Node.js] 检测到操作系统: {}", os);
    
    let result = match os.as_str() {
        "windows" => {
            info!("[安装Node.js] 使用 Windows 安装方式...");
            install_nodejs_windows().await
        },
        "macos" => {
            info!("[安装Node.js] 使用 macOS 安装方式 (Homebrew)...");
            install_nodejs_macos().await
        },
        "linux" => {
            info!("[安装Node.js] 使用 Linux 安装方式...");
            install_nodejs_linux().await
        },
        _ => {
            error!("[安装Node.js] 不支持的操作系统: {}", os);
            Ok(InstallResult {
                success: false,
                message: "不支持的操作系统".to_string(),
                error: Some(format!("不支持的操作系统: {}", os)),
            })
        },
    };
    
    match &result {
        Ok(r) if r.success => info!("[安装Node.js] ✓ 安装成功"),
        Ok(r) => warn!("[安装Node.js] ✗ 安装失败: {}", r.message),
        Err(e) => error!("[安装Node.js] ✗ 安装错误: {}", e),
    }
    
    result
}

/// Windows 安装 Node.js
async fn install_nodejs_windows() -> Result<InstallResult, String> {
    // 使用 winget 安装 Node.js（Windows 10/11 自带）
    let script = r#"
$ErrorActionPreference = 'Stop'

# 检查是否已安装
$nodeVersion = node --version 2>$null
if ($nodeVersion) {
    Write-Host "Node.js 已安装: $nodeVersion"
    exit 0
}

# 优先使用 winget
$hasWinget = Get-Command winget -ErrorAction SilentlyContinue
if ($hasWinget) {
    Write-Host "使用 winget 安装 Node.js..."
    winget install --id OpenJS.NodeJS.LTS --accept-source-agreements --accept-package-agreements
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Node.js 安装成功！"
        exit 0
    }
}

# 备用方案：使用 fnm (Fast Node Manager)
Write-Host "尝试使用 fnm 安装 Node.js..."
$fnmInstallScript = "irm https://fnm.vercel.app/install.ps1 | iex"
Invoke-Expression $fnmInstallScript

# 配置 fnm 环境
$env:FNM_DIR = "$env:USERPROFILE\.fnm"
$env:Path = "$env:FNM_DIR;$env:Path"

# 安装 Node.js 22
fnm install 22
fnm default 22
fnm use 22

# 验证安装
$nodeVersion = node --version 2>$null
if ($nodeVersion) {
    Write-Host "Node.js 安装成功: $nodeVersion"
    exit 0
} else {
    Write-Host "Node.js 安装失败"
    exit 1
}
"#;
    
    match shell::run_powershell_output(script) {
        Ok(output) => {
            // 验证安装
            if get_node_version().is_some() {
                Ok(InstallResult {
                    success: true,
                    message: "Node.js 安装成功！请重启应用以使环境变量生效。".to_string(),
                    error: None,
                })
            } else {
                Ok(InstallResult {
                    success: false,
                    message: "安装后需要重启应用".to_string(),
                    error: Some(output),
                })
            }
        }
        Err(e) => Ok(InstallResult {
            success: false,
            message: "Node.js 安装失败".to_string(),
            error: Some(e),
        }),
    }
}

/// macOS 安装 Node.js
async fn install_nodejs_macos() -> Result<InstallResult, String> {
    // 使用 Homebrew 安装
    let script = r#"
# 检查 Homebrew
if ! command -v brew &> /dev/null; then
    echo "安装 Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # 配置 PATH
    if [[ -f /opt/homebrew/bin/brew ]]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
    elif [[ -f /usr/local/bin/brew ]]; then
        eval "$(/usr/local/bin/brew shellenv)"
    fi
fi

echo "安装 Node.js 22..."
brew install node@22
brew link --overwrite node@22

# 验证安装
node --version
"#;
    
    match shell::run_bash_output(script) {
        Ok(output) => Ok(InstallResult {
            success: true,
            message: format!("Node.js 安装成功！{}", output),
            error: None,
        }),
        Err(e) => Ok(InstallResult {
            success: false,
            message: "Node.js 安装失败".to_string(),
            error: Some(e),
        }),
    }
}

/// Linux 安装 Node.js
async fn install_nodejs_linux() -> Result<InstallResult, String> {
    // 使用 NodeSource 仓库安装
    let script = r#"
# 检测包管理器
if command -v apt-get &> /dev/null; then
    echo "检测到 apt，使用 NodeSource 仓库..."
    curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
    sudo apt-get install -y nodejs
elif command -v dnf &> /dev/null; then
    echo "检测到 dnf，使用 NodeSource 仓库..."
    curl -fsSL https://rpm.nodesource.com/setup_22.x | sudo bash -
    sudo dnf install -y nodejs
elif command -v yum &> /dev/null; then
    echo "检测到 yum，使用 NodeSource 仓库..."
    curl -fsSL https://rpm.nodesource.com/setup_22.x | sudo bash -
    sudo yum install -y nodejs
elif command -v pacman &> /dev/null; then
    echo "检测到 pacman..."
    sudo pacman -S nodejs npm --noconfirm
else
    echo "无法检测到支持的包管理器"
    exit 1
fi

# 验证安装
node --version
"#;
    
    match shell::run_bash_output(script) {
        Ok(output) => Ok(InstallResult {
            success: true,
            message: format!("Node.js 安装成功！{}", output),
            error: None,
        }),
        Err(e) => Ok(InstallResult {
            success: false,
            message: "Node.js 安装失败".to_string(),
            error: Some(e),
        }),
    }
}

/// 安装 OpenClaw
#[command]
pub async fn install_openclaw() -> Result<InstallResult, String> {
    info!("[安装OpenClaw] 开始安装 OpenClaw...");
    let os = platform::get_os();
    info!("[安装OpenClaw] 检测到操作系统: {}", os);
    
    let result = match os.as_str() {
        "windows" => {
            info!("[安装OpenClaw] 使用 Windows 安装方式...");
            install_openclaw_windows().await
        },
        _ => {
            info!("[安装OpenClaw] 使用 Unix 安装方式 (npm)...");
            install_openclaw_unix().await
        },
    };
    
    match &result {
        Ok(r) if r.success => info!("[安装OpenClaw] ✓ 安装成功"),
        Ok(r) => warn!("[安装OpenClaw] ✗ 安装失败: {}", r.message),
        Err(e) => error!("[安装OpenClaw] ✗ 安装错误: {}", e),
    }
    
    result
}

/// Windows 安装 OpenClaw
async fn install_openclaw_windows() -> Result<InstallResult, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'

# 检查 Node.js
$nodeVersion = node --version 2>$null
if (-not $nodeVersion) {
    Write-Host "错误：请先安装 Node.js"
    exit 1
}

Write-Host "使用 npm 安装 OpenClaw..."
npm install -g openclaw@latest --unsafe-perm

# 验证安装
$openclawVersion = openclaw --version 2>$null
if ($openclawVersion) {
    Write-Host "OpenClaw 安装成功: $openclawVersion"
    exit 0
} else {
    Write-Host "OpenClaw 安装失败"
    exit 1
}
"#;
    
    match shell::run_powershell_output(script) {
        Ok(output) => {
            if get_openclaw_version().is_some() {
                Ok(InstallResult {
                    success: true,
                    message: "OpenClaw 安装成功！".to_string(),
                    error: None,
                })
            } else {
                Ok(InstallResult {
                    success: false,
                    message: "安装后需要重启应用".to_string(),
                    error: Some(output),
                })
            }
        }
        Err(e) => Ok(InstallResult {
            success: false,
            message: "OpenClaw 安装失败".to_string(),
            error: Some(e),
        }),
    }
}

/// Unix 系统安装 OpenClaw
async fn install_openclaw_unix() -> Result<InstallResult, String> {
    let script = r#"
# 检查 Node.js
if ! command -v node &> /dev/null; then
    echo "错误：请先安装 Node.js"
    exit 1
fi

echo "使用 npm 安装 OpenClaw..."
npm install -g openclaw@latest --unsafe-perm

# 验证安装
openclaw --version
"#;
    
    match shell::run_bash_output(script) {
        Ok(output) => Ok(InstallResult {
            success: true,
            message: format!("OpenClaw 安装成功！{}", output),
            error: None,
        }),
        Err(e) => Ok(InstallResult {
            success: false,
            message: "OpenClaw 安装失败".to_string(),
            error: Some(e),
        }),
    }
}

/// 初始化 OpenClaw 配置
#[command]
pub async fn init_openclaw_config() -> Result<InstallResult, String> {
    info!("[初始化配置] 开始初始化 OpenClaw 配置...");
    
    let config_dir = platform::get_config_dir();
    info!("[初始化配置] 配置目录: {}", config_dir);
    
    // 创建配置目录
    info!("[初始化配置] 创建配置目录...");
    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        error!("[初始化配置] ✗ 创建配置目录失败: {}", e);
        return Ok(InstallResult {
            success: false,
            message: "创建配置目录失败".to_string(),
            error: Some(e.to_string()),
        });
    }
    
    // 创建子目录
    let subdirs = ["agents/main/sessions", "agents/main/agent", "credentials"];
    for subdir in subdirs {
        let path = format!("{}/{}", config_dir, subdir);
        info!("[初始化配置] 创建子目录: {}", subdir);
        if let Err(e) = std::fs::create_dir_all(&path) {
            error!("[初始化配置] ✗ 创建目录失败: {} - {}", subdir, e);
            return Ok(InstallResult {
                success: false,
                message: format!("创建目录失败: {}", subdir),
                error: Some(e.to_string()),
            });
        }
    }
    
    // 设置配置目录权限为 700（与 shell 脚本 chmod 700 一致）
    // 仅在 Unix 系统上执行
    #[cfg(unix)]
    {
        info!("[初始化配置] 设置目录权限为 700...");
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&config_dir) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o700);
            if let Err(e) = std::fs::set_permissions(&config_dir, perms) {
                warn!("[初始化配置] 设置权限失败: {}", e);
            } else {
                info!("[初始化配置] ✓ 权限设置成功");
            }
        }
    }
    
    // 设置 gateway mode 为 local
    info!("[初始化配置] 执行: openclaw config set gateway.mode local");
    let result = shell::run_openclaw(&["config", "set", "gateway.mode", "local"]);
    
    match result {
        Ok(output) => {
            info!("[初始化配置] ✓ 配置初始化成功");
            debug!("[初始化配置] 命令输出: {}", output);
            Ok(InstallResult {
                success: true,
                message: "配置初始化成功！".to_string(),
                error: None,
            })
        },
        Err(e) => {
            error!("[初始化配置] ✗ 配置初始化失败: {}", e);
            Ok(InstallResult {
                success: false,
                message: "配置初始化失败".to_string(),
                error: Some(e),
            })
        },
    }
}

/// 打开终端执行安装脚本（用于需要管理员权限的场景）
#[command]
pub async fn open_install_terminal(install_type: String) -> Result<String, String> {
    match install_type.as_str() {
        "nodejs" => open_nodejs_install_terminal().await,
        "openclaw" => open_openclaw_install_terminal().await,
        _ => Err(format!("未知的安装类型: {}", install_type)),
    }
}

/// 打开终端安装 Node.js
async fn open_nodejs_install_terminal() -> Result<String, String> {
    if platform::is_windows() {
        // Windows: 打开 PowerShell 执行安装
        let script = r#"
Start-Process powershell -ArgumentList '-NoExit', '-Command', '
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "    Node.js 安装向导" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 检查 winget
$hasWinget = Get-Command winget -ErrorAction SilentlyContinue
if ($hasWinget) {
    Write-Host "正在使用 winget 安装 Node.js 22..." -ForegroundColor Yellow
    winget install --id OpenJS.NodeJS.LTS --accept-source-agreements --accept-package-agreements
} else {
    Write-Host "请从以下地址下载安装 Node.js:" -ForegroundColor Yellow
    Write-Host "https://nodejs.org/en/download" -ForegroundColor Green
    Write-Host ""
    Start-Process "https://nodejs.org/en/download"
}

Write-Host ""
Write-Host "安装完成后请重启 OpenClaw Manager" -ForegroundColor Green
Write-Host ""
Read-Host "按回车键关闭此窗口"
' -Verb RunAs
"#;
        shell::run_powershell_output(script)?;
        Ok("已打开安装终端".to_string())
    } else if platform::is_macos() {
        // macOS: 打开 Terminal.app
        let script_content = r#"#!/bin/bash
clear
echo "========================================"
echo "    Node.js 安装向导"
echo "========================================"
echo ""

# 检查 Homebrew
if ! command -v brew &> /dev/null; then
    echo "正在安装 Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    if [[ -f /opt/homebrew/bin/brew ]]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
    elif [[ -f /usr/local/bin/brew ]]; then
        eval "$(/usr/local/bin/brew shellenv)"
    fi
fi

echo "正在安装 Node.js 22..."
brew install node@22
brew link --overwrite node@22

echo ""
echo "安装完成！"
node --version
echo ""
read -p "按回车键关闭此窗口..."
"#;
        
        let script_path = "/tmp/openclaw_install_nodejs.command";
        std::fs::write(script_path, script_content)
            .map_err(|e| format!("创建脚本失败: {}", e))?;
        
        std::process::Command::new("chmod")
            .args(["+x", script_path])
            .output()
            .map_err(|e| format!("设置权限失败: {}", e))?;
        
        std::process::Command::new("open")
            .arg(script_path)
            .spawn()
            .map_err(|e| format!("启动终端失败: {}", e))?;
        
        Ok("已打开安装终端".to_string())
    } else {
        Err("请手动安装 Node.js: https://nodejs.org/".to_string())
    }
}

/// 打开终端安装 OpenClaw
async fn open_openclaw_install_terminal() -> Result<String, String> {
    if platform::is_windows() {
        let script = r#"
Start-Process powershell -ArgumentList '-NoExit', '-Command', '
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "    OpenClaw 安装向导" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "正在安装 OpenClaw..." -ForegroundColor Yellow
npm install -g openclaw@latest

Write-Host ""
Write-Host "初始化配置..."
openclaw config set gateway.mode local

Write-Host ""
Write-Host "安装完成！" -ForegroundColor Green
openclaw --version
Write-Host ""
Read-Host "按回车键关闭此窗口"
'
"#;
        shell::run_powershell_output(script)?;
        Ok("已打开安装终端".to_string())
    } else if platform::is_macos() {
        let script_content = r#"#!/bin/bash
clear
echo "========================================"
echo "    OpenClaw 安装向导"
echo "========================================"
echo ""

echo "正在安装 OpenClaw..."
npm install -g openclaw@latest

echo ""
echo "初始化配置..."
openclaw config set gateway.mode local 2>/dev/null || true

mkdir -p ~/.openclaw/agents/main/sessions
mkdir -p ~/.openclaw/agents/main/agent
mkdir -p ~/.openclaw/credentials

echo ""
echo "安装完成！"
openclaw --version
echo ""
read -p "按回车键关闭此窗口..."
"#;
        
        let script_path = "/tmp/openclaw_install_openclaw.command";
        std::fs::write(script_path, script_content)
            .map_err(|e| format!("创建脚本失败: {}", e))?;
        
        std::process::Command::new("chmod")
            .args(["+x", script_path])
            .output()
            .map_err(|e| format!("设置权限失败: {}", e))?;
        
        std::process::Command::new("open")
            .arg(script_path)
            .spawn()
            .map_err(|e| format!("启动终端失败: {}", e))?;
        
        Ok("已打开安装终端".to_string())
    } else {
        // Linux
        let script_content = r#"#!/bin/bash
clear
echo "========================================"
echo "    OpenClaw 安装向导"
echo "========================================"
echo ""

echo "正在安装 OpenClaw..."
npm install -g openclaw@latest

echo ""
echo "初始化配置..."
openclaw config set gateway.mode local 2>/dev/null || true

mkdir -p ~/.openclaw/agents/main/sessions
mkdir -p ~/.openclaw/agents/main/agent
mkdir -p ~/.openclaw/credentials

echo ""
echo "安装完成！"
openclaw --version
echo ""
read -p "按回车键关闭..."
"#;
        
        let script_path = "/tmp/openclaw_install_openclaw.sh";
        std::fs::write(script_path, script_content)
            .map_err(|e| format!("创建脚本失败: {}", e))?;
        
        std::process::Command::new("chmod")
            .args(["+x", script_path])
            .output()
            .map_err(|e| format!("设置权限失败: {}", e))?;
        
        // 尝试不同的终端
        let terminals = ["gnome-terminal", "xfce4-terminal", "konsole", "xterm"];
        for term in terminals {
            if std::process::Command::new(term)
                .args(["--", script_path])
                .spawn()
                .is_ok()
            {
                return Ok("已打开安装终端".to_string());
            }
        }
        
        Err("无法启动终端，请手动运行: npm install -g openclaw".to_string())
    }
}

/// 卸载 OpenClaw
#[command]
pub async fn uninstall_openclaw() -> Result<InstallResult, String> {
    info!("[卸载OpenClaw] 开始卸载 OpenClaw...");
    let os = platform::get_os();
    info!("[卸载OpenClaw] 检测到操作系统: {}", os);
    
    // 先停止服务
    info!("[卸载OpenClaw] 尝试停止服务...");
    let _ = shell::run_openclaw(&["gateway", "stop"]);
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    let result = match os.as_str() {
        "windows" => {
            info!("[卸载OpenClaw] 使用 Windows 卸载方式...");
            uninstall_openclaw_windows().await
        },
        _ => {
            info!("[卸载OpenClaw] 使用 Unix 卸载方式 (npm)...");
            uninstall_openclaw_unix().await
        },
    };
    
    match &result {
        Ok(r) if r.success => info!("[卸载OpenClaw] ✓ 卸载成功"),
        Ok(r) => warn!("[卸载OpenClaw] ✗ 卸载失败: {}", r.message),
        Err(e) => error!("[卸载OpenClaw] ✗ 卸载错误: {}", e),
    }
    
    result
}

/// 使用 bat 脚本一键安装 Git
#[command]
pub async fn install_git_bat() -> Result<InstallResult, String> {
    info!("[一键安装Git] 开始执行 bat 脚本安装...");
    
    if !platform::is_windows() {
        return Ok(InstallResult {
            success: false,
            message: "此功能仅支持 Windows 系统".to_string(),
            error: Some("仅支持 Windows 系统".to_string()),
        });
    }
    
    // 获取 bat 文件路径
    let bat_path = get_bat_file_path("setup-git-1.0.0.bat");
    let bat_path_str = bat_path.display().to_string();
    info!("[一键安装Git] bat 文件路径: {}", bat_path_str);
    
    if !bat_path.exists() {
        return Ok(InstallResult {
            success: false,
            message: "找不到安装脚本".to_string(),
            error: Some(format!("文件不存在: {}", bat_path_str)),
        });
    }
    
    // 执行 bat 脚本，使用 PowerShell 请求管理员权限
    // 直接使用 -FilePath 参数运行 bat 文件
    let script = format!(
        "Start-Process -FilePath '{}' -Verb RunAs -Wait",
        bat_path_str
    );
    
    match shell::run_powershell_output(&script) {
        Ok(_output) => {
            info!("[一键安装Git] ✓ 安装完成");
            Ok(InstallResult {
                success: true,
                message: "Git 安装完成！请重启应用使环境变量生效。".to_string(),
                error: None,
            })
        }
        Err(e) => {
            error!("[一键安装Git] ✗ 安装失败: {}", e);
            Ok(InstallResult {
                success: false,
                message: "Git 安装失败".to_string(),
                error: Some(e),
            })
        }
    }
}

/// 使用 bat 脚本一键安装 Node.js
#[command]
pub async fn install_nodejs_bat() -> Result<InstallResult, String> {
    info!("[一键安装Node.js] 开始执行 bat 脚本安装...");
    
    if !platform::is_windows() {
        return Ok(InstallResult {
            success: false,
            message: "此功能仅支持 Windows 系统".to_string(),
            error: Some("仅支持 Windows 系统".to_string()),
        });
    }
    
    // 获取 bat 文件路径
    let bat_path = get_bat_file_path("setup-nodejs-1.0.0.bat");
    let bat_path_str = bat_path.display().to_string();
    info!("[一键安装Node.js] bat 文件路径: {}", bat_path_str);
    
    if !bat_path.exists() {
        return Ok(InstallResult {
            success: false,
            message: "找不到安装脚本".to_string(),
            error: Some(format!("文件不存在: {}", bat_path_str)),
        });
    }
    
    // 执行 bat 脚本，使用 PowerShell 请求管理员权限
    // 直接使用 -FilePath 参数运行 bat 文件
    let script = format!(
        "Start-Process -FilePath '{}' -Verb RunAs -Wait",
        bat_path_str
    );
    
    match shell::run_powershell_output(&script) {
        Ok(_output) => {
            info!("[一键安装Node.js] ✓ 安装完成");
            Ok(InstallResult {
                success: true,
                message: "Node.js 安装完成！请重启应用使环境变量生效。".to_string(),
                error: None,
            })
        }
        Err(e) => {
            error!("[一键安装Node.js] ✗ 安装失败: {}", e);
            Ok(InstallResult {
                success: false,
                message: "Node.js 安装失败".to_string(),
                error: Some(e),
            })
        }
    }
}

/// 使用 bat 脚本一键部署 OpenClaw（安装 Git + Node.js + VC++ + OpenClaw）
#[command]
pub async fn install_openclaw_bat() -> Result<InstallResult, String> {
    info!("[一键部署OpenClaw] 开始执行脚本安装...");
    
    let os = platform::get_os();
    
    if os == "windows" {
        let bat_path = get_bat_file_path("install-all.bat");
        let bat_path_str = bat_path.display().to_string();
        info!("[一键部署OpenClaw] bat 文件路径: {}", bat_path_str);
        
        if !bat_path.exists() {
            return Ok(InstallResult {
                success: false,
                message: "找不到安装脚本".to_string(),
                error: Some(format!("文件不存在: {}", bat_path_str)),
            });
        }
        
        let script = format!(
            "Start-Process -FilePath '{}' -Verb RunAs -Wait",
            bat_path_str
        );
        
        match shell::run_powershell_output(&script) {
            Ok(_output) => {
                info!("[一键部署OpenClaw] ✓ 安装完成");
                Ok(InstallResult {
                    success: true,
                    message: "OpenClaw 部署完成！请重启应用使环境变量生效。".to_string(),
                    error: None,
                })
            }
            Err(e) => {
                error!("[一键部署OpenClaw] ✗ 安装失败: {}", e);
                Ok(InstallResult {
                    success: false,
                    message: "OpenClaw 部署失败".to_string(),
                    error: Some(e),
                })
            }
        }
    } else if os == "macos" {
        let sh_path = get_sh_file_path("install-all-macos.sh");
        let sh_path_str = sh_path.display().to_string();
        info!("[一键部署OpenClaw] sh 文件路径: {}", sh_path_str);
        
        if !sh_path.exists() {
            return Ok(InstallResult {
                success: false,
                message: "找不到安装脚本".to_string(),
                error: Some(format!("文件不存在: {}", sh_path_str)),
            });
        }
        
        match shell::run_bash_output(&format!("chmod +x '{}' && bash '{}'", sh_path_str, sh_path_str)) {
            Ok(_output) => {
                info!("[一键部署OpenClaw] ✓ 安装完成");
                Ok(InstallResult {
                    success: true,
                    message: "OpenClaw 部署完成！请重启应用使环境变量生效。".to_string(),
                    error: None,
                })
            }
            Err(e) => {
                error!("[一键部署OpenClaw] ✗ 安装失败: {}", e);
                Ok(InstallResult {
                    success: false,
                    message: "OpenClaw 部署失败".to_string(),
                    error: Some(e),
                })
            }
        }
    } else {
        Ok(InstallResult {
            success: false,
            message: "此功能仅支持 Windows 和 macOS 系统".to_string(),
            error: Some("仅支持 Windows 和 macOS 系统".to_string()),
        })
    }
}

/// 使用 bat 脚本一键清理 OpenClaw（卸载所有组件）
#[command]
pub async fn uninstall_all_bat() -> Result<InstallResult, String> {
    info!("[一键清理] 开始执行脚本卸载...");
    
    let os = platform::get_os();
    
    if os == "windows" {
        let bat_path = get_bat_file_path("uninstall-all.bat");
        let bat_path_str = bat_path.display().to_string();
        info!("[一键清理] bat 文件路径: {}", bat_path_str);
        
        if !bat_path.exists() {
            return Ok(InstallResult {
                success: false,
                message: "找不到卸载脚本".to_string(),
                error: Some(format!("文件不存在: {}", bat_path_str)),
            });
        }
        
        let script = format!(
            "Start-Process -FilePath '{}' -Verb RunAs -Wait",
            bat_path_str
        );
        
        match shell::run_powershell_output(&script) {
            Ok(_output) => {
                info!("[一键清理] ✓ 卸载完成");
                Ok(InstallResult {
                    success: true,
                    message: "OpenClaw 清理完成！请重启应用使环境变量生效。".to_string(),
                    error: None,
                })
            }
            Err(e) => {
                error!("[一键清理] ✗ 卸载失败: {}", e);
                Ok(InstallResult {
                    success: false,
                    message: "OpenClaw 清理失败".to_string(),
                    error: Some(e),
                })
            }
        }
    } else if os == "macos" {
        let sh_path = get_sh_file_path("uninstall-all-macos.sh");
        let sh_path_str = sh_path.display().to_string();
        info!("[一键清理] sh 文件路径: {}", sh_path_str);
        
        if !sh_path.exists() {
            return Ok(InstallResult {
                success: false,
                message: "找不到卸载脚本".to_string(),
                error: Some(format!("文件不存在: {}", sh_path_str)),
            });
        }
        
        match shell::run_bash_output(&format!("chmod +x '{}' && bash '{}'", sh_path_str, sh_path_str)) {
            Ok(_output) => {
                info!("[一键清理] ✓ 卸载完成");
                Ok(InstallResult {
                    success: true,
                    message: "OpenClaw 清理完成！请重启应用使环境变量生效。".to_string(),
                    error: None,
                })
            }
            Err(e) => {
                error!("[一键清理] ✗ 卸载失败: {}", e);
                Ok(InstallResult {
                    success: false,
                    message: "OpenClaw 清理失败".to_string(),
                    error: Some(e),
                })
            }
        }
    } else {
        Ok(InstallResult {
            success: false,
            message: "此功能仅支持 Windows 和 macOS 系统".to_string(),
            error: Some("仅支持 Windows 和 macOS 系统".to_string()),
        })
    }
}

/// 获取 bat 文件路径（支持开发和发布模式）
fn get_bat_file_path(bat_filename: &str) -> std::path::PathBuf {
    let mut possible_paths = vec![];
    
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // 1. exe 同级 bat 目录
            possible_paths.push(exe_dir.join("bat").join(bat_filename));
            
            // 2. exe 同级 resources\bat 目录
            possible_paths.push(exe_dir.join("resources").join("bat").join(bat_filename));
            
            // 3. 开发模式：项目源码目录
            if let Some(parent) = exe_dir.parent() {
                if let Some(grandparent) = parent.parent() {
                    possible_paths.push(grandparent.join("bat").join(bat_filename));
                }
            }
            
            // 4. NSIS 便携版：App 目录
            possible_paths.push(exe_dir.join("app").join("bat").join(bat_filename));
            
            // 5. 尝试在 exe 目录的上一级查找
            if let Some(parent) = exe_dir.parent() {
                possible_paths.push(parent.join("bat").join(bat_filename));
            }
        }
    }
    
    // 尝试所有可能的路径
    for path in &possible_paths {
        if path.exists() {
            info!("[bat路径] 找到文件: {}", path.display());
            return path.clone();
        }
    }
    
    // 如果都没找到，返回空路径
    for path in &possible_paths {
        info!("[bat路径] 尝试路径: {}", path.display());
    }
    
    std::path::PathBuf::new()
}

/// 获取 sh 文件路径（支持开发和发布模式，macOS/Linux）
fn get_sh_file_path(sh_filename: &str) -> std::path::PathBuf {
    let mut possible_paths = vec![];
    
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // 1. exe 同级 bat 目录
            possible_paths.push(exe_dir.join("bat").join(sh_filename));
            
            // 2. exe 同级 resources/bat 目录
            possible_paths.push(exe_dir.join("resources").join("bat").join(sh_filename));
            
            // 3. 开发模式：项目源码目录
            if let Some(parent) = exe_dir.parent() {
                if let Some(grandparent) = parent.parent() {
                    possible_paths.push(grandparent.join("bat").join(sh_filename));
                }
            }
            
            // 4. 尝试在 exe 目录的上一级查找
            if let Some(parent) = exe_dir.parent() {
                possible_paths.push(parent.join("bat").join(sh_filename));
            }
        }
    }
    
    // 尝试所有可能的路径
    for path in &possible_paths {
        if path.exists() {
            info!("[sh路径] 找到文件: {}", path.display());
            return path.clone();
        }
    }
    
    // 如果都没找到，返回空路径
    for path in &possible_paths {
        info!("[sh路径] 尝试路径: {}", path.display());
    }
    
    std::path::PathBuf::new()
}

/// Windows 卸载 OpenClaw
async fn uninstall_openclaw_windows() -> Result<InstallResult, String> {
    // 使用 cmd.exe 执行 npm uninstall，避免 PowerShell 执行策略问题
    info!("[卸载OpenClaw] 执行 npm uninstall -g openclaw...");
    
    match shell::run_cmd_output("npm uninstall -g openclaw") {
        Ok(output) => {
            info!("[卸载OpenClaw] npm 输出: {}", output);
            
            // 验证卸载是否成功
            std::thread::sleep(std::time::Duration::from_millis(500));
            if get_openclaw_version().is_none() {
                Ok(InstallResult {
                    success: true,
                    message: "OpenClaw 已成功卸载！".to_string(),
                    error: None,
                })
            } else {
                Ok(InstallResult {
                    success: false,
                    message: "卸载命令已执行，但 OpenClaw 仍然存在，请尝试手动卸载".to_string(),
                    error: Some(output),
                })
            }
        }
        Err(e) => {
            warn!("[卸载OpenClaw] npm uninstall 失败: {}", e);
            Ok(InstallResult {
                success: false,
                message: "OpenClaw 卸载失败".to_string(),
                error: Some(e),
            })
        }
    }
}

/// Unix 系统卸载 OpenClaw
async fn uninstall_openclaw_unix() -> Result<InstallResult, String> {
    let script = r#"
echo "卸载 OpenClaw..."
npm uninstall -g openclaw

# 验证卸载
if command -v openclaw &> /dev/null; then
    echo "警告：openclaw 命令仍然存在"
    exit 1
else
    echo "OpenClaw 已成功卸载"
    exit 0
fi
"#;
    
    match shell::run_bash_output(script) {
        Ok(output) => Ok(InstallResult {
            success: true,
            message: format!("OpenClaw 已成功卸载！{}", output),
            error: None,
        }),
        Err(e) => Ok(InstallResult {
            success: false,
            message: "OpenClaw 卸载失败".to_string(),
            error: Some(e),
        }),
    }
}

/// 版本更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// 是否有更新可用
    pub update_available: bool,
    /// 当前版本
    pub current_version: Option<String>,
    /// 最新版本
    pub latest_version: Option<String>,
    /// 错误信息
    pub error: Option<String>,
}

/// 检查 OpenClaw 更新
#[command]
pub async fn check_openclaw_update() -> Result<UpdateInfo, String> {
    info!("[版本检查] 开始检查 OpenClaw 更新...");
    
    // 获取当前版本
    let current_version = get_openclaw_version();
    info!("[版本检查] 当前版本: {:?}", current_version);
    
    if current_version.is_none() {
        info!("[版本检查] OpenClaw 未安装");
        return Ok(UpdateInfo {
            update_available: false,
            current_version: None,
            latest_version: None,
            error: Some("OpenClaw 未安装".to_string()),
        });
    }
    
    // 获取最新版本
    let latest_version = get_latest_openclaw_version();
    info!("[版本检查] 最新版本: {:?}", latest_version);
    
    if latest_version.is_none() {
        return Ok(UpdateInfo {
            update_available: false,
            current_version,
            latest_version: None,
            error: Some("无法获取最新版本信息".to_string()),
        });
    }
    
    // 比较版本
    let current = current_version.clone().unwrap();
    let latest = latest_version.clone().unwrap();
    let update_available = compare_versions(&current, &latest);
    
    info!("[版本检查] 是否有更新: {}", update_available);
    
    Ok(UpdateInfo {
        update_available,
        current_version,
        latest_version,
        error: None,
    })
}

/// 获取 npm registry 上的最新版本
fn get_latest_openclaw_version() -> Option<String> {
    // 使用 npm view 获取最新版本
    let result = if platform::is_windows() {
        shell::run_cmd_output("npm view openclaw version")
    } else {
        shell::run_bash_output("npm view openclaw version 2>/dev/null")
    };
    
    match result {
        Ok(version) => {
            let v = version.trim().to_string();
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        }
        Err(e) => {
            warn!("[版本检查] 获取最新版本失败: {}", e);
            None
        }
    }
}

/// 比较版本号，返回是否有更新可用
/// current: 当前版本 (如 "1.0.0" 或 "OpenClaw 2026.3.13 (61d171a)")
/// latest: 最新版本 (如 "1.0.1")
fn compare_versions(current: &str, latest: &str) -> bool {
    // 从完整版本字符串中提取出版本号
    // 例如: "OpenClaw 2026.3.13 (61d171a)" -> "2026.3.13"
    let current = extract_version(current);
    let latest = latest.trim().trim_start_matches('v');
    
    // 分割版本号
    let current_parts: Vec<u32> = current
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let latest_parts: Vec<u32> = latest
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    
    // 比较每个部分
    for i in 0..3 {
        let c = current_parts.get(i).unwrap_or(&0);
        let l = latest_parts.get(i).unwrap_or(&0);
        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }
    
    false
}

/// 从版本字符串中提取出版本号
/// 例如: "OpenClaw 2026.3.13 (61d171a)" -> "2026.3.13"
fn extract_version(version: &str) -> String {
    let version = version.trim();
    
    // 尝试匹配 "2026.3.13" 这样的日期版本号
    // 或 "1.2.3" 这样的语义版本号
    let parts: Vec<&str> = version.split_whitespace().collect();
    for part in parts {
        // 检查是否包含数字和点（如 "2026.3.13" 或 "1.0.0"）
        if part.contains('.') && part.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            // 移除可能的括号内容（如 (61d171a)）
            if let Some(idx) = part.find('(') {
                return part[..idx].trim().to_string();
            }
            return part.to_string();
        }
    }
    
    // 如果没找到，返回原始字符串
    version.to_string()
}

/// 更新 OpenClaw
#[command]
pub async fn update_openclaw() -> Result<InstallResult, String> {
    info!("[更新OpenClaw] 开始更新 OpenClaw...");
    let os = platform::get_os();
    
    // 先停止服务
    info!("[更新OpenClaw] 尝试停止服务...");
    let _ = shell::run_openclaw(&["gateway", "stop"]);
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    let result = match os.as_str() {
        "windows" => {
            info!("[更新OpenClaw] 使用 Windows 更新方式...");
            update_openclaw_windows().await
        },
        _ => {
            info!("[更新OpenClaw] 使用 Unix 更新方式 (npm)...");
            update_openclaw_unix().await
        },
    };
    
    match &result {
        Ok(r) if r.success => info!("[更新OpenClaw] ✓ 更新成功"),
        Ok(r) => warn!("[更新OpenClaw] ✗ 更新失败: {}", r.message),
        Err(e) => error!("[更新OpenClaw] ✗ 更新错误: {}", e),
    }
    
    result
}

/// Windows 更新 OpenClaw
async fn update_openclaw_windows() -> Result<InstallResult, String> {
    info!("[更新OpenClaw] 执行 openclaw update...");
    
    match shell::run_cmd_output("openclaw update") {
        Ok(output) => {
            info!("[更新OpenClaw] 输出: {}", output);
            
            // 获取新版本
            let new_version = get_openclaw_version();
            
            Ok(InstallResult {
                success: true,
                message: format!("OpenClaw 已更新到 {}", new_version.unwrap_or("最新版本".to_string())),
                error: None,
            })
        }
        Err(e) => {
            warn!("[更新OpenClaw] 更新失败: {}", e);
            Ok(InstallResult {
                success: false,
                message: "OpenClaw 更新失败".to_string(),
                error: Some(e),
            })
        }
    }
}

/// Unix 系统更新 OpenClaw
async fn update_openclaw_unix() -> Result<InstallResult, String> {
    let script = r#"
echo "更新 OpenClaw..."
openclaw update

# 验证更新
openclaw --version
"#;
    
    match shell::run_bash_output(script) {
        Ok(output) => Ok(InstallResult {
            success: true,
            message: format!("OpenClaw 已更新！{}", output),
            error: None,
        }),
        Err(e) => Ok(InstallResult {
            success: false,
            message: "OpenClaw 更新失败".to_string(),
            error: Some(e),
        }),
    }
}

/// 技能安装参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInstallParams {
    /// 技能代码
    pub skill_code: String,
    /// 下载 URL
    pub download_url: String,
}

/// 安装技能
#[command]
pub async fn install_skill(params: SkillInstallParams) -> Result<InstallResult, String> {
    info!("[技能安装] 开始安装技能: {}", params.skill_code);
    
    let config_dir = platform::get_config_dir();
    let skills_dir = if platform::is_windows() {
        format!("{}\\skills", config_dir)
    } else {
        format!("{}/skills", config_dir)
    };
    
    info!("[技能安装] 技能目录: {}", skills_dir);
    
    // 创建 skills 目录
    let skills_path = PathBuf::from(&skills_dir);
    if !skills_path.exists() {
        if let Err(e) = std::fs::create_dir_all(&skills_path) {
            error!("[技能安装] 创建目录失败: {}", e);
            return Ok(InstallResult {
                success: false,
                message: "创建技能目录失败".to_string(),
                error: Some(e.to_string()),
            });
        }
    }
    
    // 下载 zip 文件
    let zip_path = if platform::is_windows() {
        format!("{}\\{}.zip", skills_dir, params.skill_code)
    } else {
        format!("{}/{}.zip", skills_dir, params.skill_code)
    };
    
    info!("[技能安装] 下载文件: {}", params.download_url);
    
    // 过滤 URL 中的所有非 ASCII 字符和特殊字符
    let clean_url: String = params.download_url
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ':' || *c == '/' || *c == '.' || *c == '-' || *c == '_' || *c == '~' || *c == '%' || *c == '?' || *c == '=' || *c == '&')
        .collect();
    info!("[技能安装] 清理后的URL: {}", clean_url);
    
    info!("[技能安装] 保存路径: {}", zip_path);
    
    // 使用 Rust 代码直接下载
    info!("[技能安装] 开始下载...");
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    
    let response = client.get(&clean_url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;
    
    if !response.status().is_success() {
        return Ok(InstallResult {
            success: false,
            message: format!("下载失败，状态码: {}", response.status()),
            error: None,
        });
    }
    
    let bytes = response.bytes().await.map_err(|e| format!("读取下载内容失败: {}", e))?;
    
    let mut file = std::fs::File::create(&zip_path)
        .map_err(|e| format!("创建文件失败: {}", e))?;
    file.write_all(&bytes)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
    info!("[技能安装] 下载完成，文件大小: {} bytes", bytes.len());
    
    // 检查 zip 文件是否有效
    let zip_size = bytes.len();
    if zip_size == 0 {
        return Ok(InstallResult {
            success: false,
            message: "下载的文件为空".to_string(),
            error: Some("zip 文件大小为 0".to_string()),
        });
    }
    info!("[技能安装] zip 文件大小: {} bytes", zip_size);
    
    // 解压 zip 文件
    let extract_dir = if platform::is_windows() {
        format!("{}\\{}", skills_dir, params.skill_code)
    } else {
        format!("{}/{}", skills_dir, params.skill_code)
    };
    
    info!("[技能安装] 解压到: {}", extract_dir);
    
    // 如果技能目录已存在，先删除再重新创建（完全覆盖）
    let extract_path = PathBuf::from(&extract_dir);
    if extract_path.exists() {
        if let Err(e) = std::fs::remove_dir_all(&extract_path) {
            return Ok(InstallResult {
                success: false,
                message: "删除旧技能目录失败".to_string(),
                error: Some(e.to_string()),
            });
        }
        info!("[技能安装] 已删除旧技能目录");
    }
    
    // 创建技能目录
    if let Err(e) = std::fs::create_dir_all(&extract_path) {
        return Ok(InstallResult {
            success: false,
            message: "创建技能目录失败".to_string(),
            error: Some(e.to_string()),
        });
    }
    
    // 使用 Rust zip 库解压
    info!("[技能安装] 开始解压...");
    
    let zip_file = std::fs::File::open(&zip_path)
        .map_err(|e| format!("打开 zip 文件失败: {}", e))?;
    
    let mut archive = ZipArchive::new(zip_file)
        .map_err(|e| format!("读取 zip 归档失败: {}", e))?;
    
    let total_files = archive.len();
    info!("[技能安装] zip 内文件数量: {}", total_files);
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("读取 zip 内文件失败: {}", e))?;
        
        let outpath = PathBuf::from(&extract_dir).join(file.name());
        
        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)
                        .map_err(|e| format!("创建父目录失败: {}", e))?;
                }
            }
            
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }
    }
    
    info!("[技能安装] 解压完成");
    
    // 检查解压后的文件
    if let Ok(entries) = std::fs::read_dir(&extract_dir) {
        let count = entries.count();
        info!("[技能安装] 解压后文件数量: {}", count);
    }
    
    // 删除 zip 文件
    let _ = std::fs::remove_file(&zip_path);
    
    Ok(InstallResult {
        success: true,
        message: format!("技能 {} 安装成功！", params.skill_code),
        error: None,
    })
}

/// 卸载技能
#[command]
pub async fn uninstall_skill(skill_code: String) -> Result<InstallResult, String> {
    info!("[技能卸载] 开始卸载技能: {}", skill_code);
    
    let config_dir = platform::get_config_dir();
    let skills_dir = if platform::is_windows() {
        format!("{}\\skills", config_dir)
    } else {
        format!("{}/skills", config_dir)
    };
    
    let skill_dir = if platform::is_windows() {
        format!("{}\\{}", skills_dir, skill_code)
    } else {
        format!("{}/{}", skills_dir, skill_code)
    };
    
    info!("[技能卸载] 技能目录: {}", skill_dir);
    
    let skill_path = PathBuf::from(&skill_dir);
    
    if !skill_path.exists() {
        return Ok(InstallResult {
            success: true,
            message: format!("技能 {} 未安装", skill_code),
            error: None,
        });
    }
    
    // 删除技能目录
    match std::fs::remove_dir_all(&skill_path) {
        Ok(_) => {
            info!("[技能卸载] 技能 {} 卸载成功", skill_code);
            Ok(InstallResult {
                success: true,
                message: format!("技能 {} 卸载成功！", skill_code),
                error: None,
            })
        }
        Err(e) => {
            error!("[技能卸载] 技能 {} 卸载失败: {}", skill_code, e);
            Ok(InstallResult {
                success: false,
                message: format!("技能 {} 卸载失败", skill_code),
                error: Some(e.to_string()),
            })
        }
    }
}

/// 检查技能是否已安装
#[command]
pub async fn check_skill_installed(skill_code: String) -> Result<bool, String> {
    let config_dir = platform::get_config_dir();
    let skill_dir = if platform::is_windows() {
        format!("{}\\skills\\{}", config_dir, skill_code)
    } else {
        format!("{}/skills/{}", config_dir, skill_code)
    };
    
    let skill_path = PathBuf::from(&skill_dir);
    Ok(skill_path.exists() && skill_path.is_dir())
}

/// 获取已安装技能列表
#[command]
pub async fn get_installed_skills() -> Result<Vec<String>, String> {
    let config_dir = platform::get_config_dir();
    let skills_dir = if platform::is_windows() {
        format!("{}\\skills", config_dir)
    } else {
        format!("{}/skills", config_dir)
    };
    
    let skills_path = PathBuf::from(&skills_dir);
    
    if !skills_path.exists() {
        return Ok(vec![]);
    }
    
    let mut installed = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&skills_path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    installed.push(name.to_string());
                }
            }
        }
    }
    
    Ok(installed)
}
