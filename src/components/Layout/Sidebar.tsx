import { motion } from 'framer-motion';
import {
  LayoutDashboard,
  MessageCircle,
  Bot,
  MessageSquare,
  Sparkles,
} from 'lucide-react';
import { PageType } from '../../App';
import clsx from 'clsx';

interface ServiceStatus {
  running: boolean;
  pid: number | null;
  port: number;
}

interface SidebarProps {
  currentPage: PageType;
  onNavigate: (page: PageType) => void;
  serviceStatus: ServiceStatus | null;
}

const menuItems: { id: PageType; label: string; icon: React.ElementType }[] = [
  { id: 'dashboard', label: '概览', icon: LayoutDashboard },
  { id: 'chat', label: '聊天', icon: MessageCircle },
  { id: 'ai', label: 'AI 配置', icon: Bot },
  { id: 'channels', label: '消息渠道', icon: MessageSquare },
  { id: 'skills', label: '企业技能', icon: Sparkles },
];

const hiddenItems: PageType[] = ['testing', 'logs', 'settings'];

export function Sidebar({ currentPage, onNavigate, serviceStatus }: SidebarProps) {
  const isRunning = serviceStatus?.running ?? false;
  return (
    <aside className="w-60 bg-dark-900 border-r border-dark-700 flex flex-col">
      {/* Logo 区域（macOS 标题栏拖拽） */}
      <div className="h-12 flex items-center px-4 titlebar-drag border-b border-dark-700">
        <div className="flex items-center gap-2.5 titlebar-no-drag">
          <div className="w-7 h-7 rounded-md bg-brand-600 flex items-center justify-center">
            <span className="text-sm font-medium">OC</span>
          </div>
          <div>
            <h1 className="text-sm font-medium text-gray-200">开鸿小龙虾</h1>
          </div>
        </div>
      </div>

      {/* 导航菜单 */}
      <nav className="flex-1 py-3 px-2">
        <ul className="space-y-0.5">
          {menuItems.filter(item => !hiddenItems.includes(item.id)).map((item) => {
            const isActive = currentPage === item.id;
            const Icon = item.icon;
            
            return (
              <li key={item.id}>
                <button
                  onClick={() => onNavigate(item.id)}
                  className={clsx(
                    'w-full flex items-center gap-2.5 px-3 py-2 rounded-md text-sm font-normal transition-all relative',
                    isActive
                      ? 'text-brand-400 bg-brand-500/10'
                      : 'text-gray-400 hover:text-gray-200 hover:bg-dark-800'
                  )}
                >
                  {isActive && (
                    <motion.div
                      layoutId="activeIndicator"
                      className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-5 bg-brand-500 rounded-r-full"
                      transition={{ type: 'spring', stiffness: 300, damping: 30 }}
                    />
                  )}
                  <Icon size={16} className={isActive ? 'text-brand-400' : ''} />
                  <span>{item.label}</span>
                </button>
              </li>
            );
          })}
        </ul>
      </nav>

      {/* 底部信息 */}
      <div className="p-3 border-t border-dark-700">
        <div className="px-3 py-2 bg-dark-800 rounded-md">
          <div className="flex items-center gap-2 mb-1.5">
            <div className={clsx('status-dot', isRunning ? 'running' : 'stopped')} />
            <span className="text-xs text-gray-500">
              {isRunning ? '服务运行中' : '服务未启动'}
            </span>
          </div>
          <p className="text-xs text-gray-600">端口: {serviceStatus?.port ?? 18789}</p>
        </div>
      </div>
    </aside>
  );
}
