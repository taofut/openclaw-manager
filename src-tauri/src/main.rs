// 防止 Windows 系统显示控制台窗口
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod models;
mod utils;

use commands::{config, diagnostics, installer, process, service};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

static LOG_FILE: OnceLock<PathBuf> = OnceLock::new();

fn get_log_dir() -> PathBuf {
    // 使用 ~/.openclaw/logs/ 目录，与 gateway.log 在同一位置
    if let Some(home) = dirs::home_dir() {
        let log_dir = if cfg!(windows) {
            home.join(".openclaw").join("logs")
        } else {
            home.join(".openclaw").join("logs")
        };
        fs::create_dir_all(&log_dir).ok();
        log_dir
    } else {
        PathBuf::from("logs")
    }
}

fn get_log_file() -> PathBuf {
    LOG_FILE.get_or_init(|| get_log_dir().join("manager.log")).clone()
}

fn init_file_logging() {
    let log_file = get_log_file();
    
    // 设置 env_logger 同时输出到文件
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    )
    .format(move |buf, record| {
        let timestamp = chrono::Local::now().format("%Y-%m-%dT%H:%M:%SZ");
        let msg = format!(
            "[{} {} {}] {}\n",
            timestamp,
            record.level(),
            record.target(),
            record.args()
        );
        
        // 写入文件
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
        {
            let _ = file.write_all(msg.as_bytes());
        }
        
        writeln!(buf, "{}", msg.trim())
    })
    .init();
}

fn main() {
    init_file_logging();
    
    log::info!("🦞 OpenClaw Manager 启动");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            // 服务管理
            service::get_service_status,
            service::start_service,
            service::stop_service,
            service::restart_service,
            service::get_logs,
            // 进程管理
            process::check_openclaw_installed,
            process::get_openclaw_version,
            process::check_port_in_use,
            // 配置管理
            config::get_config,
            config::save_config,
            config::get_env_value,
            config::save_env_value,
            config::get_ai_providers,
            config::get_channels_config,
            config::save_channel_config,
            config::clear_channel_config,
            // Gateway Token
            config::get_or_create_gateway_token,
            config::get_dashboard_url,
            // AI 配置管理
            config::get_official_providers,
            config::get_ai_config,
            config::save_provider,
            config::delete_provider,
            config::set_primary_model,
            config::add_available_model,
            config::remove_available_model,
            // 飞书插件管理
            config::check_feishu_plugin,
            config::install_feishu_plugin,
            // 诊断测试
            diagnostics::run_doctor,
            diagnostics::test_ai_connection,
            diagnostics::test_channel,
            diagnostics::get_system_info,
            diagnostics::start_channel_login,
            // 安装器
            installer::check_environment,
            installer::install_nodejs,
            installer::install_nodejs_bat,
            installer::install_git_bat,
            installer::install_openclaw_bat,
            installer::uninstall_all_bat,
            installer::install_openclaw,
            installer::init_openclaw_config,
            installer::open_install_terminal,
            installer::uninstall_openclaw,
            installer::install_skill,
            installer::uninstall_skill,
            installer::check_skill_installed,
            installer::get_installed_skills,
            // 版本更新
            installer::check_openclaw_update,
            installer::update_openclaw,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时发生错误");
}
