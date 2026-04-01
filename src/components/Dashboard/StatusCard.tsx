import { Activity, Cpu, HardDrive, Clock } from 'lucide-react';
import clsx from 'clsx';

interface ServiceStatus {
  running: boolean;
  pid: number | null;
  port: number;
  uptime_seconds: number | null;
  memory_mb: number | null;
  cpu_percent: number | null;
}

interface StatusCardProps {
  status: ServiceStatus | null;
  loading: boolean;
}

export function StatusCard({ status, loading }: StatusCardProps) {
  const formatUptime = (seconds: number | null) => {
    if (!seconds) return '--';
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };

  return (
    <div className="bg-dark-800 rounded-lg p-4 border border-dark-600">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-sm font-medium text-gray-300">服务状态</h3>
        <div className="flex items-center gap-2">
          <div
            className={clsx(
              'status-dot',
              loading ? 'warning' : status?.running ? 'running' : 'stopped'
            )}
          />
          <span
            className={clsx(
              'text-sm font-medium',
              loading
                ? 'text-yellow-400'
                : status?.running
                ? 'text-green-400'
                : 'text-red-400'
            )}
          >
            {loading ? '检测中...' : status?.running ? '运行中' : '已停止'}
          </span>
        </div>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div className="bg-dark-700 rounded-md p-3">
          <div className="flex items-center gap-2 mb-1.5">
            <Activity size={14} className="text-brand-400" />
            <span className="text-xs text-gray-500">端口</span>
          </div>
          <p className="text-base font-medium text-gray-200">
            {status?.port || 18789}
          </p>
        </div>

        <div className="bg-dark-700 rounded-md p-3">
          <div className="flex items-center gap-2 mb-1.5">
            <Cpu size={14} className="text-brand-400" />
            <span className="text-xs text-gray-500">进程 ID</span>
          </div>
          <p className="text-base font-medium text-gray-200">
            {status?.pid || '--'}
          </p>
        </div>

        <div className="bg-dark-700 rounded-md p-3">
          <div className="flex items-center gap-2 mb-1.5">
            <HardDrive size={14} className="text-brand-400" />
            <span className="text-xs text-gray-500">内存</span>
          </div>
          <p className="text-base font-medium text-gray-200">
            {status?.memory_mb ? `${status.memory_mb.toFixed(1)} MB` : '--'}
          </p>
        </div>

        <div className="bg-dark-700 rounded-md p-3">
          <div className="flex items-center gap-2 mb-1.5">
            <Clock size={14} className="text-brand-400" />
            <span className="text-xs text-gray-500">运行时间</span>
          </div>
          <p className="text-base font-medium text-gray-200">
            {formatUptime(status?.uptime_seconds || null)}
          </p>
        </div>
      </div>
    </div>
  );
}
