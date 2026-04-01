import { Play, Square } from 'lucide-react';
import clsx from 'clsx';

interface ServiceStatus {
  running: boolean;
  pid: number | null;
  port: number;
}

interface QuickActionsProps {
  status: ServiceStatus | null;
  loading: boolean;
  actionType: 'start' | 'stop' | 'restart' | null;
  progress: number;
  onStart: () => void;
  onStop: () => void;
}

export function QuickActions({
  status,
  loading,
  actionType,
  progress,
  onStart,
  onStop,
}: QuickActionsProps) {
  const isRunning = status?.running || false;

  const isStarting = loading && actionType === 'start';
  const isStopping = loading && actionType === 'stop';

  return (
    <div className="bg-dark-800 rounded-lg p-4 border border-dark-600">
      <h3 className="text-sm font-medium text-gray-300 mb-3">快捷操作</h3>

      <div className="grid grid-cols-2 gap-4">
        {/* 启动按钮 */}
        <button
          onClick={onStart}
          disabled={loading || isRunning}
          className={clsx(
            'flex flex-col items-center gap-3 p-4 rounded-xl transition-all',
            'border border-dark-500',
            (loading && !isStarting) && 'opacity-30',
            isRunning && !loading
              ? 'bg-dark-600 opacity-50 cursor-not-allowed'
              : 'bg-dark-700 hover:bg-green-500/10 hover:border-green-500/30'
          )}
        >
          <div
            className={clsx(
              'relative w-12 h-12 rounded-full flex items-center justify-center',
              isRunning && !loading ? 'bg-dark-500' : 'bg-green-500/20'
            )}
          >
            {isStarting ? (
              <svg className="absolute w-12 h-12 -rotate-90" viewBox="0 0 48 48">
                <circle
                  cx="24"
                  cy="24"
                  r="20"
                  stroke="currentColor"
                  strokeWidth="3"
                  fill="none"
                  className="text-green-500/30"
                />
                <circle
                  cx="24"
                  cy="24"
                  r="20"
                  stroke="currentColor"
                  strokeWidth="3"
                  fill="none"
                  strokeDasharray="126"
                  strokeDashoffset={126 - (126 * progress) / 100}
                  strokeLinecap="round"
                  className="text-green-400 transition-all duration-300"
                />
              </svg>
            ) : (
              <Play
                size={20}
                className={isRunning ? 'text-gray-500' : 'text-green-400'}
              />
            )}
          </div>
          <span
            className={clsx(
              'text-sm font-medium',
              isRunning && !loading ? 'text-gray-500' : 'text-gray-300'
            )}
          >
            {isStarting ? '启动中...' : '启动'}
          </span>
        </button>

        {/* 停止按钮 */}
        <button
          onClick={onStop}
          disabled={loading || !isRunning}
          className={clsx(
            'flex flex-col items-center gap-3 p-4 rounded-xl transition-all',
            'border border-dark-500',
            (loading && !isStopping) && 'opacity-30',
            !isRunning && !loading
              ? 'bg-dark-600 opacity-50 cursor-not-allowed'
              : 'bg-dark-700 hover:bg-red-500/10 hover:border-red-500/30'
          )}
        >
          <div
            className={clsx(
              'relative w-12 h-12 rounded-full flex items-center justify-center',
              !isRunning && !loading ? 'bg-dark-500' : 'bg-red-500/20'
            )}
          >
            {isStopping ? (
              <svg className="absolute w-12 h-12 -rotate-90" viewBox="0 0 48 48">
                <circle
                  cx="24"
                  cy="24"
                  r="20"
                  stroke="currentColor"
                  strokeWidth="3"
                  fill="none"
                  className="text-red-500/30"
                />
                <circle
                  cx="24"
                  cy="24"
                  r="20"
                  stroke="currentColor"
                  strokeWidth="3"
                  fill="none"
                  strokeDasharray="126"
                  strokeDashoffset={126 - (126 * progress) / 100}
                  strokeLinecap="round"
                  className="text-red-400 transition-all duration-300"
                />
              </svg>
            ) : (
              <Square
                size={20}
                className={!isRunning ? 'text-gray-500' : 'text-red-400'}
              />
            )}
          </div>
          <span
            className={clsx(
              'text-sm font-medium',
              !isRunning && !loading ? 'text-gray-500' : 'text-gray-300'
            )}
          >
            {isStopping ? '停止中...' : '停止'}
          </span>
        </button>
      </div>
    </div>
  );
}
