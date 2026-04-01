import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { StatusCard } from './StatusCard';
import { QuickActions } from './QuickActions';
import { SystemInfo } from './SystemInfo';
import { ServiceLogs } from './ServiceLogs';
import { api, ServiceStatus, UpdateInfo, isTauri } from '../../lib/tauri';
import { useAppStore } from '../../stores/appStore';
import { Download, Trash2, AlertCircle, CheckCircle } from 'lucide-react';

export function Dashboard() {
  const [status, setStatus] = useState<ServiceStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [updateLoading, setUpdateLoading] = useState(true);
  const [actionProgress, setActionProgress] = useState(0);
  const { actionLoading, actionType, setActionLoading, installLoading, setInstallLoading } = useAppStore();

  const isUACCancelled = (error: unknown): boolean => {
    const errorStr = String(error).toLowerCase();
    return errorStr.includes('runas') || 
           errorStr.includes('cancel') || 
           errorStr.includes('denied') ||
           errorStr.includes('拒绝') ||
           errorStr.includes('取消') ||
           errorStr.includes('invalidoperation') ||
           errorStr.includes('start-process');
  };

  const handleInstallOpenclaw = async () => {
    if (!isTauri() || installLoading) return;
    const confirmed = confirm('确定要部署 OpenClaw 吗？');
    if (!confirmed) return;
    setInstallLoading(true);
    try {
      const result = await api.installOpenclawBat();
      if (result.success) {
        alert('OpenClaw 部署完成！请重启应用使环境变量生效。');
      } else {
        if (!isUACCancelled(result.error || result.message)) {
          alert(`部署失败: ${result.error || result.message}`);
        }
      }
    } catch (e) {
      if (!isUACCancelled(e)) {
        console.error('部署失败:', e);
        alert('部署失败，请查看日志');
      }
    } finally {
      setInstallLoading(false);
    }
  };

  const handleUninstallAll = async () => {
    if (!isTauri() || installLoading) return;
    const confirmed = confirm('确定要清理所有 OpenClaw 组件吗？此操作不可撤销。');
    if (!confirmed) return;
    setInstallLoading(true);
    try {
      const result = await api.uninstallAllBat();
      if (result.success) {
        alert('OpenClaw 清理完成！请重启应用使环境变量生效。');
      } else {
        if (!isUACCancelled(result.error || result.message)) {
          alert(`清理失败: ${result.error || result.message}`);
        }
      }
    } catch (e) {
      if (!isUACCancelled(e)) {
        console.error('清理失败:', e);
        alert('清理失败，请查看日志');
      }
    } finally {
      setInstallLoading(false);
    }
  };

  const handleUpgrade = async () => {
    if (!isTauri() || installLoading) return;
    const confirmed = confirm('确定要升级 OpenClaw 吗？');
    if (!confirmed) return;
    setInstallLoading(true);
    try {
      const result = await api.updateOpenclaw();
      if (result.success) {
        alert('OpenClaw 升级完成！请重启应用使环境变量生效。');
      } else {
        if (!isUACCancelled(result.error || result.message)) {
          alert(`升级失败: ${result.error || result.message}`);
        }
      }
    } catch (e) {
      if (!isUACCancelled(e)) {
        console.error('升级失败:', e);
        alert('升级失败，请查看日志');
      }
    } finally {
      setInstallLoading(false);
    }
  };

  const fetchStatus = async () => {
    if (!isTauri()) {
      setLoading(false);
      return;
    }
    try {
      const result = await api.getServiceStatus();
      setStatus(result);
    } catch {
      // 静默处理
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStatus();
    if (!isTauri()) return;
    
    const statusInterval = setInterval(fetchStatus, 3000);
    
    return () => {
      clearInterval(statusInterval);
    };
  }, []);

  useEffect(() => {
    const checkUpdate = async () => {
      if (!isTauri()) {
        setUpdateLoading(false);
        return;
      }
      try {
        const info = await api.checkOpenclawUpdate();
        setUpdateInfo(info);
      } catch (e) {
        console.error('检查更新失败:', e);
      } finally {
        setUpdateLoading(false);
      }
    };
    checkUpdate();
  }, []);

  const handleStart = async () => {
    if (!isTauri()) return;
    setActionLoading(true, 'start');
    setActionProgress(0);

    let currentProgress = 0;
    let progressInterval: ReturnType<typeof setInterval> | null = null;
    
    progressInterval = setInterval(() => {
      if (currentProgress >= 98) return;
      currentProgress += 2;
      setActionProgress(Math.min(currentProgress, 98));
    }, 1100);

    try {
      await api.startService();
    } catch (e) {
      console.warn('启动服务返回错误:', e);
    }
    
    const checkStatus = async () => {
      const currentStatus = await api.getServiceStatus();
      console.log(`[启动] 检查 - running:`, currentStatus.running);
      setStatus(currentStatus);
      if (currentStatus.running) {
        setActionProgress(100);
        console.log('[启动] 服务已启动成功!');
        return true;
      }
      return false;
    };

    console.log('[启动] 开始轮询检查服务状态...');
    try {
      if (await checkStatus()) {
        if (progressInterval) clearInterval(progressInterval);
        setTimeout(() => setActionProgress(0), 500);
        setActionLoading(false, null);
        return;
      }

      for (let i = 1; i <= 60; i++) {
        const delay = i <= 5 ? 500 : 1000;
        await new Promise(resolve => setTimeout(resolve, delay));
        if (await checkStatus()) {
          break;
        }
      }
    } catch (e) {
      console.error('轮询状态失败:', e);
    } finally {
      if (progressInterval) clearInterval(progressInterval);
      setTimeout(() => setActionProgress(0), 500);
      setActionLoading(false, null);
    }
  };

  const handleStop = async () => {
    if (!isTauri()) return;
    setActionLoading(true, 'stop');
    setActionProgress(0);

    let currentProgress = 0;
    let progressInterval: ReturnType<typeof setInterval> | null = null;
    
    progressInterval = setInterval(() => {
      if (currentProgress >= 95) return;
      currentProgress += 4;
      setActionProgress(Math.min(currentProgress, 95));
    }, 1000);

    try {
      try {
        await api.stopService();
      } catch (e) {
        console.warn('停止命令返回错误，继续轮询等待服务停止:', e);
      }
      
      const checkStopped = async () => {
        const currentStatus = await api.getServiceStatus();
        setStatus(currentStatus);
        if (!currentStatus.running) {
          setActionProgress(100);
          return true;
        }
        return false;
      };

      if (await checkStopped()) {
        if (progressInterval) clearInterval(progressInterval);
        setTimeout(() => setActionProgress(0), 500);
        setActionLoading(false, null);
        return;
      }

      for (let i = 1; i <= 25; i++) {
        const delay = i <= 3 ? 500 : 1000;
        await new Promise(resolve => setTimeout(resolve, delay));
        if (await checkStopped()) {
          break;
        }
      }
    } catch (e) {
      console.error('停止失败:', e);
    } finally {
      if (progressInterval) clearInterval(progressInterval);
      setTimeout(() => setActionProgress(0), 500);
      setActionLoading(false, null);
    }
  };

  const containerVariants = {
    hidden: { opacity: 0 },
    show: {
      opacity: 1,
      transition: {
        staggerChildren: 0.1,
      },
    },
  };

  const itemVariants = {
    hidden: { opacity: 0, y: 20 },
    show: { opacity: 1, y: 0 },
  };

  return (
    <div className="h-full overflow-y-auto scroll-container pr-2">
      <motion.div
        variants={containerVariants}
        initial="hidden"
        animate="show"
        className="space-y-6"
      >
        {/* 版本更新提示 - 仅当 OpenClaw 已安装时显示 */}
        {!updateLoading && updateInfo && updateInfo.current_version && (
          <motion.div variants={itemVariants}>
            {updateInfo.update_available ? (
              <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-4 flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <AlertCircle size={20} className="text-yellow-400" />
                  <div>
                    <p className="text-yellow-400 font-medium">
                      发现新版本：{updateInfo.latest_version}
                    </p>
                    <p className="text-yellow-400/70 text-sm">
                      当前版本：{updateInfo.current_version}，是否升级？
                    </p>
                  </div>
                </div>
                <button
                  onClick={handleUpgrade}
                  disabled={installLoading}
                  className="px-4 py-2 bg-yellow-500 hover:bg-yellow-600 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {installLoading ? '升级中...' : '立即升级'}
                </button>
              </div>
            ) : (
              <div className="bg-green-500/10 border border-green-500/30 rounded-lg p-4 flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <CheckCircle size={20} className="text-green-400" />
                  <div>
                    <p className="text-green-400 font-medium">
                      当前已是最新版本
                    </p>
                    <p className="text-green-400/70 text-sm">
                      版本：{updateInfo.current_version || '未知'}
                    </p>
                  </div>
                </div>
              </div>
            )}
          </motion.div>
        )}

        {/* 环境标题 */}
        <motion.div variants={itemVariants}>
          <div className="bg-dark-800 rounded-lg border border-dark-600 p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-lg bg-brand-500/20 flex items-center justify-center">
                  <Download size={20} className="text-brand-400" />
                </div>
                <div>
                  <h3 className="text-gray-200 font-medium">环境配置</h3>
                  <p className="text-xs text-gray-500">
                    快速安装开发环境
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-3">
                <button
                  onClick={handleInstallOpenclaw}
                  disabled={installLoading}
                  className="flex items-center gap-2 px-5 py-2.5 bg-brand-500 hover:bg-brand-600 text-white rounded-md font-medium transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <Download size={18} className={installLoading ? 'animate-pulse' : ''} />
                  {installLoading ? '部署中...' : '一键部署 OpenClaw'}
                </button>
                <button
                  onClick={handleUninstallAll}
                  disabled={installLoading}
                  className="flex items-center gap-2 px-5 py-2.5 bg-red-500/20 hover:bg-red-500/30 text-red-400 border border-red-500/30 rounded-md font-medium transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <Trash2 size={18} className={installLoading ? 'animate-pulse' : ''} />
                  {installLoading ? '清理中...' : '一键清理'}
                </button>
              </div>
            </div>
          </div>
        </motion.div>

        {/* 服务状态卡片 */}
        <motion.div variants={itemVariants}>
          <StatusCard status={status} loading={loading} />
        </motion.div>

        {/* 快捷操作 */}
        <motion.div variants={itemVariants}>
          <QuickActions
            status={status}
            loading={actionLoading}
            actionType={actionType}
            progress={actionProgress}
            onStart={handleStart}
            onStop={handleStop}
          />
        </motion.div>

        {/* 操作日志 */}
        <motion.div variants={itemVariants}>
          <ServiceLogs />
        </motion.div>

        {/* 系统信息 */}
        <motion.div variants={itemVariants}>
          <SystemInfo />
        </motion.div>
      </motion.div>
    </div>
  );
}
