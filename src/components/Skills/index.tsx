import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import {
  Sparkles,
  Download,
  Loader2,
  Check,
  Info,
  ExternalLink,
  AlertCircle,
  RefreshCw,
} from 'lucide-react';
import clsx from 'clsx';

interface Skill {
  skillCode: string;
  title: string;
  description: string;
  downloadUrl: string;
  installed?: boolean;
  installing?: boolean;
}

interface ApiResponse {
  code: number;
  data: Skill[];
  message: string;
}

interface InstallResult {
  success: boolean;
  message: string;
  error?: string;
}

const API_BASE = 'http://192.168.18.77:5000/api/public/skills';

export function Skills() {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedSkill, setSelectedSkill] = useState<string | null>(null);
  const [filter, setFilter] = useState<'all' | 'installed' | 'not_installed'>('all');

  const fetchSkills = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(API_BASE);
      if (!response.ok) {
        throw new Error(`请求失败: ${response.status}`);
      }
      const data: ApiResponse = await response.json();
      if (data.code === 0 && data.data) {
        // 获取已安装技能列表
        const installedSkills = await invoke<string[]>('get_installed_skills');
        
        setSkills(data.data.map((s) => ({
          ...s,
          installed: installedSkills.includes(s.skillCode),
        })));
      } else {
        throw new Error(data.message || '获取技能列表失败');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '网络错误');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSkills();
  }, []);

  const handleInstall = async (skill: Skill) => {
    setSkills((prev) =>
      prev.map((s) =>
        s.skillCode === skill.skillCode ? { ...s, installing: true } : s
      )
    );

    try {
      const result = await invoke<InstallResult>('install_skill', {
        params: {
          skill_code: skill.skillCode,
          download_url: skill.downloadUrl,
        },
      });

      if (result.success) {
        setSkills((prev) =>
          prev.map((s) =>
            s.skillCode === skill.skillCode
              ? { ...s, installed: true, installing: false }
              : s
          )
        );
      } else {
        setSkills((prev) =>
          prev.map((s) =>
            s.skillCode === skill.skillCode ? { ...s, installing: false } : s
          )
        );
        alert(result.message || '安装失败');
      }
    } catch (e) {
      setSkills((prev) =>
        prev.map((s) =>
          s.skillCode === skill.skillCode ? { ...s, installing: false } : s
        )
      );
      alert(`安装失败: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const handleUninstall = async (skillCode: string) => {
    setSkills((prev) =>
      prev.map((s) =>
        s.skillCode === skillCode ? { ...s, installing: true } : s
      )
    );

    try {
      const result = await invoke<{ success: boolean; message: string }>('uninstall_skill', {
        skillCode,
      });

      if (result.success) {
        setSkills((prev) =>
          prev.map((s) =>
            s.skillCode === skillCode
              ? { ...s, installed: false, installing: false }
              : s
          )
        );
      } else {
        setSkills((prev) =>
          prev.map((s) =>
            s.skillCode === skillCode ? { ...s, installing: false } : s
          )
        );
        alert(result.message || '卸载失败');
      }
    } catch (e) {
      setSkills((prev) =>
        prev.map((s) =>
          s.skillCode === skillCode ? { ...s, installing: false } : s
        )
      );
      alert(`卸载失败: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const filteredSkills = skills.filter((skill) => {
    if (filter === 'installed') return skill.installed;
    if (filter === 'not_installed') return !skill.installed;
    return true;
  });

  const filterCounts = {
    all: skills.length,
    installed: skills.filter((s) => s.installed).length,
    not_installed: skills.filter((s) => !s.installed).length,
  };

  if (loading) {
    return (
      <div className="h-full flex flex-col items-center justify-center">
        <Loader2 size={40} className="text-claw-500 animate-spin mb-4" />
        <p className="text-gray-400">正在加载技能列表...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="h-full flex flex-col items-center justify-center">
        <AlertCircle size={40} className="text-red-400 mb-4" />
        <p className="text-red-400 mb-4">{error}</p>
        <button
          onClick={fetchSkills}
          className="flex items-center gap-2 px-4 py-2 bg-claw-500 text-white rounded-lg hover:bg-claw-600 transition-colors"
        >
          <RefreshCw size={16} />
          重试
        </button>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="mb-4 flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-white">技能市场</h2>
        </div>
        <button
          onClick={fetchSkills}
          className="p-2 text-gray-400 hover:text-white hover:bg-dark-700 rounded-lg transition-colors"
          title="刷新"
        >
          <RefreshCw size={18} />
        </button>
      </div>

      <div className="mb-4 flex gap-2">
        {([
          { key: 'all', label: '全部技能', count: filterCounts.all },
          { key: 'installed', label: '已安装', count: filterCounts.installed },
          { key: 'not_installed', label: '未安装', count: filterCounts.not_installed },
        ] as const).map((item) => (
          <button
            key={item.key}
            onClick={() => setFilter(item.key)}
            className={clsx(
              'px-4 py-2 rounded-lg text-sm font-medium transition-all',
              filter === item.key
                ? 'bg-claw-500 text-white'
                : 'bg-dark-700 text-gray-400 hover:bg-dark-600 hover:text-white'
            )}
          >
            {item.label} ({item.count})
          </button>
        ))}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 overflow-y-auto pb-4">
        {filteredSkills.map((skill) => (
          <motion.div
            key={skill.skillCode}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className={clsx(
              'bg-dark-800 rounded-xl border p-4 transition-all',
              selectedSkill === skill.skillCode
                ? 'border-claw-500 bg-dark-700'
                : 'border-dark-600 hover:border-dark-500'
            )}
            onClick={() => setSelectedSkill(skill.skillCode)}
          >
            <div className="flex items-start justify-between mb-3">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-claw-400/20 to-purple-500/20 flex items-center justify-center">
                  <Sparkles size={20} className="text-claw-400" />
                </div>
                <div>
                  <h3 className="font-medium text-white">{skill.title}</h3>
                  <p className="text-xs text-gray-500">{skill.skillCode}</p>
                </div>
              </div>
              {skill.installed ? (
                <span className="px-2 py-1 bg-green-500/20 text-green-400 text-xs rounded-full flex items-center gap-1">
                  <Check size={12} />
                  已安装
                </span>
              ) : (
                <span className="px-2 py-1 bg-dark-600 text-gray-400 text-xs rounded-full">
                  未安装
                </span>
              )}
            </div>

            <p className="text-sm text-gray-400 mb-4 line-clamp-2">
              {skill.description}
            </p>

            <div className="flex items-center gap-2">
              {skill.installed ? (
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleUninstall(skill.skillCode);
                  }}
                  disabled={skill.installing}
                  className="flex-1 py-2 px-3 bg-red-500/20 text-red-400 rounded-lg text-sm font-medium hover:bg-red-500/30 transition-colors disabled:opacity-50"
                >
                  {skill.installing ? (
                    <span className="flex items-center justify-center gap-2">
                      <Loader2 size={14} className="animate-spin" />
                      卸载中...
                    </span>
                  ) : (
                    '卸载'
                  )}
                </button>
              ) : (
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleInstall(skill);
                  }}
                  disabled={skill.installing}
                  className="flex-1 py-2 px-3 bg-claw-500 text-white rounded-lg text-sm font-medium hover:bg-claw-600 transition-colors disabled:opacity-50"
                >
                  {skill.installing ? (
                    <span className="flex items-center justify-center gap-2">
                      <Loader2 size={14} className="animate-spin" />
                      安装中...
                    </span>
                  ) : (
                    <span className="flex items-center justify-center gap-2">
                      <Download size={14} />
                      安装
                    </span>
                  )}
                </button>
              )}
              <button
                onClick={(e) => {
                  e.stopPropagation();
                }}
                className="p-2 text-gray-400 hover:text-white hover:bg-dark-600 rounded-lg transition-colors"
                title="查看详情"
              >
                <Info size={16} />
              </button>
            </div>
          </motion.div>
        ))}
      </div>

      {selectedSkill && (
        <div className="mt-4 p-4 bg-dark-800 rounded-xl border border-dark-600">
          <div className="flex items-center gap-2 text-gray-400 text-sm">
            <ExternalLink size={14} />
            <span>从技能市场获取更多技能</span>
          </div>
        </div>
      )}
    </div>
  );
}
