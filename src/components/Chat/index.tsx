import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Loader2, ExternalLink, AlertCircle } from 'lucide-react';
import { isTauri } from '../../lib/tauri';

export function Chat() {
  const [url, setUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchUrl = async () => {
    if (!isTauri()) {
      setUrl('http://localhost:18789');
      setLoading(false);
      return;
    }

    try {
      const dashboardUrl = await invoke<string>('get_dashboard_url');
      console.log('Dashboard URL:', dashboardUrl);
      setUrl(dashboardUrl);
      setError(null);
    } catch (e) {
      console.error('获取 Dashboard URL 失败:', e);
      setUrl('http://localhost:18789');
      setError('无法连接到 OpenClaw 服务，请确保服务已启动');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchUrl();
  }, []);

  const handleOpenInNewWindow = async () => {
    if (!url) return;
    
    try {
      const webview = new WebviewWindow('chat-webview', {
        url: url,
        title: 'OpenClaw 聊天',
        width: 1200,
        height: 800,
        center: true,
      });
      
      webview.once('tauri://error', (e) => {
        console.error('Webview 创建失败:', e);
      });
    } catch (e) {
      console.error('打开新窗口失败:', e);
      window.open(url, '_blank');
    }
  };

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center bg-dark-900">
        <div className="flex flex-col items-center gap-4">
          <Loader2 size={32} className="animate-spin text-claw-400" />
          <p className="text-gray-400">正在加载聊天界面...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col items-center justify-center bg-dark-900">
      <div className="text-center max-w-md">
        <div className="mb-6">
          <div className="w-20 h-20 mx-auto rounded-2xl bg-gradient-to-br from-claw-400 to-claw-600 flex items-center justify-center">
            <span className="text-4xl">🦞</span>
          </div>
        </div>
        
        <h2 className="text-xl font-semibold text-white mb-2">OpenClaw 聊天界面</h2>
        <p className="text-gray-400 mb-6">
          点击下方按钮打开聊天窗口
        </p>

        {error && (
          <div className="flex items-center justify-center gap-2 text-red-400 text-sm mb-4">
            <AlertCircle size={16} />
            <span>{error}</span>
          </div>
        )}

        <div className="flex flex-col gap-3">
          <button
            onClick={handleOpenInNewWindow}
            className="flex items-center justify-center gap-2 px-6 py-3 bg-claw-500 hover:bg-claw-600 text-white rounded-lg font-medium transition-colors"
          >
            <ExternalLink size={20} />
            <span>打开聊天窗口</span>
          </button>
          
          <a
            href={url || 'http://localhost:18789'}
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center justify-center gap-2 px-6 py-3 bg-dark-700 hover:bg-dark-600 text-gray-300 rounded-lg font-medium transition-colors"
          >
            <ExternalLink size={20} />
            <span>在新浏览器标签页打开</span>
          </a>
        </div>

        {url && (
          <p className="text-xs text-gray-500 mt-4">
            地址: {url}
          </p>
        )}
      </div>
    </div>
  );
}
