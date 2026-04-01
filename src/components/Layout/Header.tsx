import { useState } from 'react';
import { PageType } from '../../App';
import { RefreshCw, ExternalLink, Loader2 } from 'lucide-react';
import { open } from '@tauri-apps/plugin-shell';
import { invoke } from '@tauri-apps/api/core';

interface HeaderProps {
  currentPage: PageType;
}

const pageTitles: Record<PageType, { title: string; description: string }> = {
  dashboard: { title: '概览', description: '服务状态、日志与快捷操作' },
  chat: { title: '聊天', description: 'OpenClaw Web 聊天界面' },
  ai: { title: 'AI 模型配置', description: '配置 AI 提供商和模型' },
  channels: { title: '消息渠道', description: '配置 Telegram、Discord、飞书等' },
  skills: { title: '企业技能', description: '安装和管理 OpenClaw 技能插件' },
  testing: { title: '测试诊断', description: '系统诊断与问题排查' },
  logs: { title: '应用日志', description: '查看 Manager 应用的控制台日志' },
  settings: { title: '设置', description: '身份配置与高级选项' },
};

export function Header({ currentPage }: HeaderProps) {
  const { title, description } = pageTitles[currentPage];
  const [opening, setOpening] = useState(false);

  const handleOpenDashboard = async () => {
    setOpening(true);
    try {
      // 获取带 token 的 Dashboard URL（如果没有 token 会自动生成）
      const url = await invoke<string>('get_dashboard_url');
      await open(url);
    } catch (e) {
      console.error('打开 Dashboard 失败:', e);
      // 降级方案：使用 window.open（不带 token）
      window.open('http://localhost:18789', '_blank');
    } finally {
      setOpening(false);
    }
  };

  return (
    <header className="h-12 bg-dark-900/80 border-b border-dark-700 flex items-center justify-between px-5 titlebar-drag backdrop-blur-sm">
      {/* 左侧：页面标题 */}
      <div className="titlebar-no-drag">
        <h2 className="text-base font-medium text-gray-200">{title}</h2>
        <p className="text-xs text-gray-500">{description}</p>
      </div>

      {/* 右侧：操作按钮 */}
      <div className="flex items-center gap-2 titlebar-no-drag">
        <button
          onClick={() => window.location.reload()}
          className="icon-button text-gray-500 hover:text-gray-300"
          title="刷新"
        >
          <RefreshCw size={15} />
        </button>
        <button
          onClick={handleOpenDashboard}
          disabled={opening}
          className="flex items-center gap-2 px-3 py-1.5 rounded-md bg-dark-700 hover:bg-dark-600 text-sm text-gray-400 hover:text-gray-200 transition-colors disabled:opacity-50 border border-dark-600"
          title="打开 Web Dashboard"
        >
          {opening ? <Loader2 size={13} className="animate-spin" /> : <ExternalLink size={13} />}
          <span>Dashboard</span>
        </button>
      </div>
    </header>
  );
}
