import { useEffect, useState, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Terminal, ChevronDown, ChevronUp, Loader2 } from 'lucide-react';
import clsx from 'clsx';
import { api } from '../../lib/tauri';

interface LogLine {
  id: string;
  text: string;
  level: 'info' | 'warn' | 'error' | 'debug';
}

const LEVEL_COLORS: Record<string, string> = {
  info: 'text-green-400',
  warn: 'text-yellow-400',
  error: 'text-red-400',
  debug: 'text-gray-400',
};

export function ServiceLogs() {
  const [logs, setLogs] = useState<LogLine[]>([]);
  const [isExpanded, setIsExpanded] = useState(false);
  const [loading, setLoading] = useState(false);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const pollingRef = useRef<number | null>(null);

  const fetchLogs = async () => {
    try {
      setLoading(true);
      const lines = await api.getLogs(100);
      console.log('[ServiceLogs] 获取到日志条数:', lines.length);
      
      if (lines.length > 0) {
        console.log('[ServiceLogs] 最新日志:', lines[lines.length - 1]);
      }
      
      const parsedLogs: LogLine[] = lines
        .filter(text => {
          // 过滤掉 Node.js 错误堆栈
          if (text.includes('Symbol(') || 
              text.includes('_eventsCount:') || 
              text.includes('_currentRequest:') ||
              text.includes('_currentUrl:') ||
              text.includes('_onNativeResponse:') ||
              text.includes('GetAddrInfoReqWrap') ||
              text.includes('node:dns') ||
              text.includes('cause:') ||
              text.includes('syscall:') ||
              text.includes('errno:') ||
              text.includes('hostname:') ||
              text.includes('code:')) {
            return false;
          }
          // 过滤掉飞书 API 错误日志
          if (text.includes('log_id:') || 
              text.includes('troubleshooter:') ||
              text.includes('no user authority')) {
            return false;
          }
          // 过滤掉过长的行（可能是错误堆栈）
          if (text.length > 300) {
            return false;
          }
          return true;
        })
        .map((text, index) => {
        let level: LogLine['level'] = 'info';
        const lowerText = text.toLowerCase();
        if (lowerText.includes('error') || lowerText.includes('failed') || lowerText.includes('失败')) {
          level = 'error';
        } else if (lowerText.includes('warn') || lowerText.includes('警告')) {
          level = 'warn';
        } else if (lowerText.includes('debug')) {
          level = 'debug';
        }
        return {
          id: `${Date.now()}-${index}`,
          text,
          level,
        };
      });
      
      setLogs(parsedLogs);
    } catch (e) {
      console.error('[ServiceLogs] 获取日志失败:', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (!isExpanded) {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }
      setLogs([]);
      return;
    }
    
    fetchLogs();
    
    pollingRef.current = window.setInterval(() => {
      fetchLogs();
    }, 3000);
    
    return () => {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
      }
    };
  }, [isExpanded]);

  const formatLogText = (text: string) => {
    // 过滤掉 Node.js 错误堆栈关键字
    if (text.includes('Symbol(') || 
        text.includes('_eventsCount:') || 
        text.includes('_currentRequest:') || 
        text.includes('_currentUrl:') ||
        text.includes('_onNativeResponse:') ||
        text.includes('GetAddrInfoReqWrap') ||
        text.includes('node:dns') ||
        text.includes('cause:') ||
        text.includes('syscall:') ||
        text.includes('errno:') ||
        text.includes('hostname:') ||
        text.includes('code:')) {
      return '';
    }
    if (text.length > 200) {
      return text.substring(0, 200) + '...';
    }
    return text;
  };

  const latestLogs = logs.slice(-100);

  return (
    <div 
      className="bg-dark-800 rounded-lg border border-dark-600 overflow-hidden"
      onMouseUp={(e) => e.stopPropagation()}
      onMouseDown={(e) => e.stopPropagation()}
      onClick={(e) => e.stopPropagation()}
    >
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex items-center justify-between p-4 hover:bg-dark-700/50 transition-colors"
      >
        <div className="flex items-center gap-2">
          <Terminal size={16} className="text-cyan-400" />
          <h3 className="text-sm font-medium text-gray-300">服务日志</h3>
          {logs.length > 0 && (
            <span className="text-xs text-gray-500">显示 {logs.length} 条</span>
          )}
          {loading && (
            <Loader2 size={12} className="text-cyan-400 animate-spin" />
          )}
        </div>
        {isExpanded ? (
          <ChevronUp size={16} className="text-gray-500" />
        ) : (
          <ChevronDown size={16} className="text-gray-500" />
        )}
      </button>

      <AnimatePresence>
        {isExpanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="overflow-hidden"
          >
            <div className="px-4 pb-4">
              <div 
                className="bg-dark-900 rounded-lg p-3 h-56 overflow-y-auto scroll-container font-mono text-xs"
                style={{ userSelect: 'text', WebkitUserSelect: 'text' }}
                onMouseDown={(e) => e.stopPropagation()}
                onMouseUp={(e) => e.stopPropagation()}
              >
                {latestLogs.length === 0 ? (
                  <div className="text-gray-500 text-center py-8">
                    等待服务启动...
                  </div>
                ) : (
                  <div className="space-y-0.5">
                    {latestLogs.map((log) => (
                      <div
                        key={log.id}
                        className={clsx(
                          'py-0.5 leading-tight',
                          LEVEL_COLORS[log.level]
                        )}
                      >
                        {formatLogText(log.text)}
                      </div>
                    ))}
                    <div ref={logsEndRef} />
                  </div>
                )}
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
