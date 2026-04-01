import { create } from 'zustand';
import type { ServiceStatus, SystemInfo } from '../lib/tauri';

interface AppState {
  // 服务状态
  serviceStatus: ServiceStatus | null;
  setServiceStatus: (status: ServiceStatus | null) => void;

  // 系统信息
  systemInfo: SystemInfo | null;
  setSystemInfo: (info: SystemInfo | null) => void;

  // UI 状态
  loading: boolean;
  setLoading: (loading: boolean) => void;

  // 服务操作状态（启动/停止）
  actionLoading: boolean;
  actionType: 'start' | 'stop' | null;
  actionProgress: number;
  setActionLoading: (loading: boolean, actionType?: 'start' | 'stop' | null) => void;
  setActionProgress: (progress: number) => void;

  // 安装/卸载操作状态
  installLoading: boolean;
  setInstallLoading: (loading: boolean) => void;

  // 通知
  notifications: Notification[];
  addNotification: (notification: Omit<Notification, 'id'>) => void;
  removeNotification: (id: string) => void;
}

interface Notification {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message?: string;
}

export const useAppStore = create<AppState>((set) => ({
  // 服务状态
  serviceStatus: null,
  setServiceStatus: (status) => set({ serviceStatus: status }),

  // 系统信息
  systemInfo: null,
  setSystemInfo: (info) => set({ systemInfo: info }),

  // UI 状态
  loading: false,
  setLoading: (loading) => set({ loading }),

  // 服务操作状态（启动/停止）
  actionLoading: false,
  actionType: null,
  actionProgress: 0,
  setActionLoading: (loading, actionType = null) => set({ actionLoading: loading, actionType }),
  setActionProgress: (progress) => set({ actionProgress: progress }),

  // 安装/卸载操作状态
  installLoading: false,
  setInstallLoading: (loading) => set({ installLoading: loading }),

  // 通知
  notifications: [],
  addNotification: (notification) =>
    set((state) => ({
      notifications: [
        ...state.notifications,
        { ...notification, id: Date.now().toString() },
      ],
    })),
  removeNotification: (id) =>
    set((state) => ({
      notifications: state.notifications.filter((n) => n.id !== id),
    })),
}));
