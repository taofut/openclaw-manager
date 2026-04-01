use crate::models::ServiceStatus;
use crate::utils::shell;
use tauri::command;
use std::process::Command;
use std::sync::Mutex;
use log::{info, debug};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Windows CREATE_NO_WINDOW 标志，用于隐藏控制台窗口
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const SERVICE_PORT: u16 = 18789;

/// 服务操作互斥锁，防止并发启动/停止导致竞态条件
static SERVICE_OPERATION_LOCK: Mutex<()> = Mutex::new(());

/// 读取文件最后 N 行的辅助函数
fn read_last_n_lines(path: &str, n: usize) -> Vec<String> {
    if let Ok(content) = std::fs::read_to_string(path) {
        let lines: Vec<&str> = content.lines().collect();
        let start = if lines.len() > n { lines.len() - n } else { 0 };
        lines[start..].iter().map(|s| s.to_string()).collect()
    } else {
        vec![]
    }
}

/// 检测端口是否有服务在监听，返回 PID
/// 简单直接：端口被占用 = 服务运行中
fn check_port_listening(port: u16) -> Option<u32> {
    #[cfg(unix)]
    {
        let output = Command::new("lsof")
            .args(["-ti", &format!(":{}", port)])
            .output()
            .ok()?;
        
        if output.status.success() {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .and_then(|line| line.trim().parse::<u32>().ok())
        } else {
            None
        }
    }
    
    #[cfg(windows)]
    {
        let mut cmd = Command::new("netstat");
        cmd.args(["-ano"]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        
        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                debug!("[监听检查] netstat 执行失败: {}", e);
                return None;
            }
        };
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut found_pids: Vec<u32> = Vec::new();
            
            for line in stdout.lines() {
                if line.contains(&format!(":{}", port)) && line.contains("LISTENING") {
                    debug!("[监听检查] 发现监听: {}", line);
                    if let Some(pid_str) = line.split_whitespace().last() {
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            found_pids.push(pid);
                        }
                    }
                }
            }
            
            if !found_pids.is_empty() {
                debug!("[监听检查] 找到端口 {} 的监听进程: {:?}", port, found_pids);
                return Some(found_pids[0]);
            } else {
                debug!("[监听检查] 端口 {} 无监听进程", port);
            }
        }
        None
    }
}

/// 检查端口是否被占用（更宽松的检测，包括 LISTENING、ESTABLISHED 等状态）
fn is_port_in_use(port: u16) -> bool {
    #[cfg(unix)]
    {
        let output = Command::new("lsof")
            .args(["-ti", &format!(":{}", port)])
            .output();
        
        output.map(|o| o.status.success()).unwrap_or(false)
    }
    
    #[cfg(windows)]
    {
        let mut cmd = Command::new("netstat");
        cmd.args(["-ano"]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        
        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                debug!("[端口检查] netstat 执行失败: {}", e);
                return false;
            }
        };
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut found = false;
            
            for line in stdout.lines() {
                if line.contains(&format!(":{}", port)) {
                    debug!("[端口检查] 发现端口 {} 相关: {}", port, line);
                    // 只要有任何连接状态就认为端口被占用
                    if line.contains("LISTENING") || 
                       line.contains("ESTABLISHED") || 
                       line.contains("TIME_WAIT") ||
                       line.contains("CLOSE_WAIT") {
                        found = true;
                    }
                }
            }
            
            if found {
                debug!("[端口检查] 端口 {} 已被占用", port);
            }
            return found;
        }
        false
    }
}

/// 通过 PID 获取进程信息（内存和运行时间）
fn get_process_info(pid: u32) -> (Option<f64>, Option<u64>) {
    #[cfg(unix)]
    {
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "rss="])
            .output();
        
        let memory_mb = output
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse::<f64>()
                    .ok()
            })
            .map(|rss_kb| rss_kb / 1024.0);
        
        (memory_mb, None)
    }
    
    #[cfg(windows)]
    {
        let script = format!(
            "$p = Get-Process -Id {} -ErrorAction SilentlyContinue; \
            if ($p) {{ \
                $mem = [math]::Round($p.WorkingSet64 / 1MB, 1); \
                $start = $p.StartTime; \
                $now = Get-Date; \
                $uptime = [math]::Round(($now - $start).TotalSeconds); \
                Write-Output \"$mem,$uptime\" \
            }} else {{ \
                Write-Output \"\" \
            }}",
            pid
        );
        
        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        
        let output = match cmd.output() {
            Ok(o) => o,
            Err(_) => return (None, None),
        };
        
        if !output.status.success() {
            return (None, None);
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stdout = stdout.trim();
        
        if stdout.is_empty() {
            return (None, None);
        }
        
        let parts: Vec<&str> = stdout.split(',').collect();
        if parts.len() >= 2 {
            let memory_mb: Option<f64> = parts[0].trim().parse().ok();
            let uptime_seconds: Option<u64> = parts[1].trim().parse().ok();
            return (memory_mb, uptime_seconds);
        }
        
        (None, None)
    }
}

/// 获取服务状态（包含内存和运行时间）
#[command]
pub async fn get_service_status() -> Result<ServiceStatus, String> {
    let pid = check_port_listening(SERVICE_PORT);
    let running = pid.is_some();
    
    let (memory_mb, uptime_seconds) = if let Some(pid) = pid {
        get_process_info(pid)
    } else {
        (None, None)
    };
    
    Ok(ServiceStatus {
        running,
        pid,
        port: SERVICE_PORT,
        uptime_seconds,
        memory_mb,
        cpu_percent: None,
    })
}

/// 启动服务
#[command]
pub async fn start_service() -> Result<String, String> {
    // 获取互斥锁，防止并发启动/停止导致竞态条件
    let _lock = SERVICE_OPERATION_LOCK.lock().map_err(|e| format!("获取锁失败: {}", e))?;
    
    info!("========== [服务] 开始启动服务 ==========");
    let start_time = std::time::Instant::now();
    
    // 检查端口是否被占用（包括已崩溃但端口未释放的情况）
    info!("[服务] 步骤1: 检查端口 {} 是否被占用...", SERVICE_PORT);
    let port_in_use = is_port_in_use(SERVICE_PORT);
    info!("[服务] 端口 {} 被占用: {}", SERVICE_PORT, port_in_use);
    if port_in_use {
        info!("[服务] 端口 {} 已被占用，先清理旧进程...", SERVICE_PORT);
        kill_process_on_port(SERVICE_PORT);
    }
    
    // 检查 openclaw 命令是否存在
    info!("[服务] 步骤2: 查找 openclaw 命令...");
    let openclaw_path = shell::get_openclaw_path();
    if openclaw_path.is_none() {
        info!("[服务] 找不到 openclaw 命令");
        return Err("找不到 openclaw 命令，请先通过 npm install -g openclaw 安装".to_string());
    }
    info!("[服务] 找到 openclaw 路径: {:?}", openclaw_path);
    
    // 清理 openclaw 的锁文件/注册状态
    cleanup_openclaw_lock();
    
    // 额外等待并强制清理占用端口的进程
    info!("[服务] 等待并清理端口 {} ...", SERVICE_PORT);
    std::thread::sleep(std::time::Duration::from_secs(2));
    kill_process_on_port(SERVICE_PORT);
    
    // 直接后台启动 gateway（不等待 doctor，避免阻塞）
    info!("[服务] 步骤3: 后台启动 gateway...");
    let spawn_start = std::time::Instant::now();
    shell::spawn_openclaw_gateway()
        .map_err(|e| {
            info!("[服务] 启动 gateway 失败: {}", e);
            format!("启动服务失败: {}", e)
        })?;
    info!("[服务] spawn_openclaw_gateway 耗时: {:?}", spawn_start.elapsed());
    
    // 轮询等待端口开始监听（最多 30 秒）
    info!("[服务] 步骤4: 等待端口 {} 开始监听...", SERVICE_PORT);
    for i in 1..=30 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        // 每次循环都检查是否有启动失败的错误日志
        if let Some(failed_pid) = extract_pid_from_error_log() {
            info!("[服务] 检测到启动失败，尝试清理 PID: {}", failed_pid);
            let _ = kill_process_by_pid(failed_pid);
            std::thread::sleep(std::time::Duration::from_millis(500));
            // 同时清理计划任务
            let _ = Command::new("schtasks")
                .args(["/End", "/TN", "OpenClaw Gateway"])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
            let _ = Command::new("schtasks")
                .args(["/Delete", "/TN", "OpenClaw Gateway", "/F"])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        }
        
        if let Some(pid) = check_port_listening(SERVICE_PORT) {
            let _total_time = start_time.elapsed();
            info!("[服务] ✓ 启动成功! 耗时: {}秒, PID: {}", i, pid);
            info!("========== [服务] 启动完成，总耗时: {:?} ==========", _total_time);
            return Ok(format!("服务已启动，PID: {}", pid));
        }
        
        info!("[服务] 等待中... ({}/30秒)", i);
    }
    
    // 第一次启动超时，尝试重试一次
    info!("[服务] 第一次启动超时，准备重试...");
    cleanup_openclaw_lock();
    std::thread::sleep(std::time::Duration::from_secs(1));
    kill_process_on_port(SERVICE_PORT);
    
    info!("[服务] 重试启动 gateway...");
    if let Err(e) = shell::spawn_openclaw_gateway() {
        info!("[服务] 重试启动失败: {}", e);
    }
    
    // 重试等待端口（最多 30 秒）
    info!("[服务] 重试等待端口 {} ...", SERVICE_PORT);
    for i in 1..=30 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        if let Some(failed_pid) = extract_pid_from_error_log() {
            info!("[服务] 重试检测到冲突 PID: {}", failed_pid);
            let _ = kill_process_by_pid(failed_pid);
        }
        
        if let Some(pid) = check_port_listening(SERVICE_PORT) {
            let _total_time = start_time.elapsed();
            info!("[服务] ✓ 重试启动成功! 耗时: {}秒, PID: {}", i, pid);
            return Ok(format!("服务已启动（重试），PID: {}", pid));
        }
        
        info!("[服务] 重试等待中... ({}/30秒)", i);
    }
    
    info!("[服务] 重试也超时了");
    Err("服务启动超时，请检查 openclaw 日志".to_string())
}

/// 从错误日志中提取最近的 "gateway already running (pid XXXX)" 中的 PID
fn extract_pid_from_error_log() -> Option<u32> {
    let config_dir = crate::utils::platform::get_config_dir();
    let err_log_path = format!("{}/logs/gateway.err.log", config_dir);
    
    // 只读取最后 20 行
    let lines = read_last_n_lines(&err_log_path, 20);
    
    // 从后往前找最新的错误
    for line in lines.iter().rev() {
        if line.contains("gateway already running") && line.contains("pid ") {
            // 提取 PID: "gateway already running (pid 7604)"
            if let Some(pid_str) = line.split("pid ").nth(1) {
                if let Ok(pid) = pid_str.split_whitespace().next().unwrap_or("0").parse::<u32>() {
                    return Some(pid);
                }
            }
        }
    }
    None
}

/// 获取监听指定端口的所有 PID
fn get_pids_on_port(port: u16) -> Vec<u32> {
    #[cfg(unix)]
    {
        let output = Command::new("lsof")
            .args(["-ti", &format!(":{}", port)])
            .output();
        
        match output {
            Ok(out) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .filter_map(|line| line.trim().parse::<u32>().ok())
                    .collect()
            }
            _ => vec![],
        }
    }
    
    #[cfg(windows)]
    {
        let mut cmd = Command::new("netstat");
        cmd.args(["-ano"]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        
        match cmd.output() {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.lines()
                    .filter(|line| line.contains(&format!(":{}", port)) && line.contains("LISTENING"))
                    .filter_map(|line| line.split_whitespace().last())
                    .filter_map(|pid_str| pid_str.parse::<u32>().ok())
                    .collect()
            }
            _ => vec![],
        }
    }
}

/// 清理占用指定端口的进程
fn kill_process_on_port(port: u16) {
    let pids = get_pids_on_port(port);
    if pids.is_empty() {
        // 如果没有监听中的进程，检查所有占用端口的进程
        if is_port_in_use(port) {
            info!("[服务] 端口 {} 被占用，尝试找到并终止进程...", port);
            // 使用更宽松的方式查找
            #[cfg(windows)]
            {
                let mut cmd = Command::new("netstat");
                cmd.args(["-ano"]);
                cmd.creation_flags(CREATE_NO_WINDOW);
                
                if let Ok(out) = cmd.output() {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    for line in stdout.lines() {
                        if line.contains(&format!(":{}", port)) {
                            if let Some(pid_str) = line.split_whitespace().last() {
                                if let Ok(pid) = pid_str.parse::<u32>() {
                                    info!("[服务] 终止占用端口 {} 的进程 PID: {}", port, pid);
                                    let _ = kill_process_by_pid(pid);
                                }
                            }
                        }
                    }
                }
            }
        }
        return;
    }
    
    for pid in pids {
        info!("[服务] 终止占用端口 {} 的进程 PID: {}", port, pid);
        if kill_process_by_pid(pid) {
            info!("[服务] 进程 {} 已终止", pid);
        }
    }
    
    // 等待一下让端口释放
    std::thread::sleep(std::time::Duration::from_secs(1));
}

/// 根据 PID 终止进程
fn kill_process_by_pid(pid: u32) -> bool {
    info!("[服务] 尝试强制杀死进程 PID: {}", pid);
    
    #[cfg(windows)]
    {
        // 先检查进程是否存在
        let mut check_cmd = Command::new("tasklist");
        check_cmd.args(["/FI", &format!("PID eq {}", pid)]);
        check_cmd.creation_flags(CREATE_NO_WINDOW);
        
        if let Ok(output) = check_cmd.output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.contains(&pid.to_string()) {
                info!("[服务] 进程 {} 不存在，无需终止", pid);
                return true;
            }
        }
        
        let mut cmd = Command::new("taskkill");
        cmd.args(["/F", "/PID", &pid.to_string()]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        
        match cmd.output() {
            Ok(o) => {
                if o.status.success() {
                    info!("[服务] 进程 {} 已终止", pid);
                    
                    // 验证进程是否真的被终止
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if let Ok(output) = check_cmd.output() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if stdout.contains(&pid.to_string()) {
                            info!("[服务] 警告: 进程 {} 仍在运行，尝试再次终止", pid);
                            let _ = Command::new("taskkill")
                                .args(["/F", "/T", "/PID", &pid.to_string()])
                                .creation_flags(CREATE_NO_WINDOW)
                                .output();
                        }
                    }
                    true
                } else {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    info!("[服务] 终止进程 {} 失败: {}", pid, stderr);
                    false
                }
            }
            Err(e) => {
                info!("[服务] 终止进程 {} 出错: {}", pid, e);
                false
            }
        }
    }
    
    #[cfg(unix)]
    {
        Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// 通过 PID 杀死进程
fn kill_process(pid: u32, force: bool) -> bool {
    info!("[服务] 杀死进程 PID: {}, force: {}", pid, force);
    
    #[cfg(unix)]
    {
        let signal = if force { "-9" } else { "-TERM" };
        Command::new("kill")
            .args([signal, &pid.to_string()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    
    #[cfg(windows)]
    {
        let mut cmd = Command::new("taskkill");
        if force {
            cmd.args(["/F", "/PID", &pid.to_string()]);
        } else {
            cmd.args(["/PID", &pid.to_string()]);
        }
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// 停止服务（通过杀死监听端口的进程）
#[command]
pub async fn stop_service() -> Result<String, String> {
    // 获取互斥锁，防止并发启动/停止导致竞态条件
    let _lock = SERVICE_OPERATION_LOCK.lock().map_err(|e| format!("获取锁失败: {}", e))?;
    
    info!("[服务] 停止服务...");
    
    // 第一步：使用 openclaw gateway stop 命令停止服务
    if let Some(path) = shell::get_openclaw_path() {
        info!("[服务] 执行 openclaw gateway stop...");
        let _ = stop_gateway_service(&path);
    }
    
    // 第二步：杀死 openclaw.exe 进程
    #[cfg(windows)]
    {
        info!("[服务] 杀死 openclaw.exe 进程...");
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "openclaw.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        
        // 第三步：终止 Windows 计划任务
        info!("[服务] 终止计划任务...");
        let _ = Command::new("schtasks")
            .args(["/End", "/TN", "OpenClaw Gateway"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        
        let _ = Command::new("schtasks")
            .args(["/Delete", "/TN", "OpenClaw Gateway", "/F"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
    
    // 等待一下让服务完全停止
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    let pids = get_pids_on_port(SERVICE_PORT);
    if pids.is_empty() {
        info!("[服务] 端口 {} 无进程监听，服务未运行", SERVICE_PORT);
        return Ok("服务未在运行".to_string());
    }
    
    info!("[服务] 发现 {} 个进程监听端口 {}: {:?}", pids.len(), SERVICE_PORT, pids);
    
    // 第四步：优雅终止 (SIGTERM)
    for &pid in &pids {
        kill_process(pid, false);
    }
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    // 检查是否已停止
    let remaining = get_pids_on_port(SERVICE_PORT);
    if remaining.is_empty() {
        info!("[服务] ✓ 已停止");
        return Ok("服务已停止".to_string());
    }
    
    // 第五步：强制终止 (SIGKILL)
    info!("[服务] 仍有 {} 个进程存活，强制终止...", remaining.len());
    for &pid in &remaining {
        kill_process(pid, true);
    }
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    let still_running = get_pids_on_port(SERVICE_PORT);
    if still_running.is_empty() {
        info!("[服务] ✓ 已强制停止");
        Ok("服务已停止".to_string())
    } else {
        Err(format!("无法停止服务，仍有进程: {:?}", still_running))
    }
}

/// 重启服务
#[command]
pub async fn restart_service() -> Result<String, String> {
    // 获取互斥锁，防止并发启动/停止导致竞态条件
    let _lock = SERVICE_OPERATION_LOCK.lock().map_err(|e| format!("获取锁失败: {}", e))?;
    
    info!("[服务] 重启服务...");
    
    // 先停止（不需要再获取锁，因为 stop_service 内部也需要锁，这里会死锁）
    // 改为直接调用内部清理逻辑
    info!("[服务] 执行重启前的清理...");
    if let Some(path) = shell::get_openclaw_path() {
        let _ = stop_gateway_service(&path);
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "openclaw.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        let _ = Command::new("schtasks")
            .args(["/End", "/TN", "OpenClaw Gateway"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        let _ = Command::new("schtasks")
            .args(["/Delete", "/TN", "OpenClaw Gateway", "/F"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
    std::thread::sleep(std::time::Duration::from_secs(2));
    kill_process_on_port(SERVICE_PORT);
    cleanup_openclaw_lock();
    
    // 再启动
    drop(_lock); // 释放锁，让 start_service 可以获取锁
    start_service().await
}

/// 获取日志（直接读取日志文件，比 RPC 更可靠）
#[command]
pub async fn get_logs(lines: Option<u32>) -> Result<Vec<String>, String> {
    let n = lines.unwrap_or(100) as usize;
    
    let config_dir = crate::utils::platform::get_config_dir();
    
    // 尝试多个已知的日志文件位置
    let log_files = vec![
        format!("{}/logs/gateway.log", config_dir),
        format!("{}/logs/gateway.err.log", config_dir),
        format!("{}/logs/manager.log", config_dir),  // manager.log 在 logs 目录
    ];
    
    let mut all_lines: Vec<String> = Vec::new();
    
    for log_file in &log_files {
        if !std::path::Path::new(log_file).exists() {
            continue;
        }
        
        // 读取文件并获取最后 N 行（跨平台实现）
        match std::fs::read_to_string(log_file) {
            Ok(content) => {
                let file_lines: Vec<&str> = content.lines().collect();
                let start = if file_lines.len() > n {
                    file_lines.len() - n
                } else {
                    0
                };
                for line in &file_lines[start..] {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        all_lines.push(trimmed.to_string());
                    }
                }
            }
            Err(e) => {
                debug!("[日志] 读取文件失败 {}: {}", log_file, e);
            }
        }
    }
    
    // 尝试按时间戳排序（日志格式通常以 ISO 时间戳开头）
    all_lines.sort();
    
    // 去重并保留最后 N 行
    all_lines.dedup();
    let total = all_lines.len();
    if total > n as usize {
        all_lines = all_lines.split_off(total - n as usize);
    }
    
    Ok(all_lines)
}

/// 停止 gateway 服务（使用 openclaw gateway stop）
fn stop_gateway_service(openclaw_path: &str) -> Result<(), String> {
    info!("[服务] 执行 openclaw gateway stop...");
    
    #[cfg(windows)]
    {
        let mut cmd = Command::new("cmd");
        cmd.args(["/c", openclaw_path, "gateway", "stop"]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        
        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    info!("[服务] openclaw gateway stop 执行成功");
                    // 等待一下让服务完全停止
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    info!("[服务] openclaw gateway stop 执行失败: {} {}", stdout, stderr);
                    // 即使失败也继续尝试启动
                    Ok(())
                }
            }
            Err(e) => {
                info!("[服务] openclaw gateway stop 执行出错: {}", e);
                Ok(())
            }
        }
    }
    
    #[cfg(unix)]
    {
        let mut cmd = Command::new(openclaw_path);
        cmd.args(["gateway", "stop"]);
        
        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    info!("[服务] openclaw gateway stop 执行成功");
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    Ok(())
                } else {
                    info!("[服务] openclaw gateway stop 执行失败");
                    Ok(())
                }
            }
            Err(e) => {
                info!("[服务] openclaw gateway stop 执行出错: {}", e);
                Ok(())
            }
        }
    }
}

/// 清理 openclaw 的锁文件/注册状态
fn cleanup_openclaw_lock() {
    info!("[服务] 清理 openclaw 锁文件/注册状态...");
    
    #[cfg(windows)]
    {
        // 1. 首先终止 Windows 计划任务（关键：服务注册信息在这里）
        info!("[服务] 终止计划任务 OpenClaw Gateway...");
        let _ = Command::new("schtasks")
            .args(["/End", "/TN", "OpenClaw Gateway"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        
        // 2. 删除 Windows 任务计划程序中的 OpenClaw Gateway 任务
        info!("[服务] 删除计划任务 OpenClaw Gateway...");
        let _ = Command::new("schtasks")
            .args(["/Delete", "/TN", "OpenClaw Gateway", "/F"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        
        // 等待计划任务完全删除（Windows 计划任务是异步的）
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        // 3. 杀死所有 openclaw.exe 进程
        info!("[服务] 杀死 openclaw.exe 进程...");
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "openclaw.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        
        // 4. 杀死 node.exe 和 bun.exe 进程
        info!("[服务] 杀死 node.exe 和 bun.exe 进程...");
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "node.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "bun.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        
        // 5. 等待一下让进程完全退出
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        // 6. 最后清理占用端口的进程
        let current_pids = get_pids_on_port(SERVICE_PORT);
        if !current_pids.is_empty() {
            info!("[服务] 清理占用端口 {} 的残留进程: {:?}", SERVICE_PORT, current_pids);
            for pid in &current_pids {
                let _ = kill_process_by_pid(*pid);
            }
        }
        
        // 7. 清理可能的锁文件
        if let Some(home) = std::env::var_os("USERPROFILE") {
            let home_str = std::path::PathBuf::from(home);
            let lock_paths = vec![
                home_str.join(".openclaw").join("gateway.lock"),
                home_str.join(".openclaw").join(".gateway.lock"),
                home_str.join("AppData").join("Local").join("openclaw").join("gateway.lock"),
            ];
            
            for lock_path in lock_paths {
                if lock_path.exists() {
                    info!("[服务] 删除锁文件: {:?}", lock_path);
                    let _ = std::fs::remove_file(&lock_path);
                }
            }
        }
        
        info!("[服务] 清理完成");
    }
    
    #[cfg(unix)]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let home_str = std::path::PathBuf::from(home);
            let lock_paths = vec![
                home_str.join(".openclaw").join("gateway.lock"),
                home_str.join(".openclaw").join(".gateway.lock"),
            ];
            
            for lock_path in lock_paths {
                if lock_path.exists() {
                    info!("[服务] 删除锁文件: {:?}", lock_path);
                    let _ = std::fs::remove_file(&lock_path);
                }
            }
        }
        
        info!("[服务] 清理完成");
    }
}
