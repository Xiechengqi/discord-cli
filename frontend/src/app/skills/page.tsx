'use client';

import { useEffect, useState } from 'react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { Spinner } from '@/components/spinner';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { SkillSpec } from '@/lib/types';

export default function SkillsPage() {
  const { lang } = useLang();
  const tr = t(lang).skills;
  const [skills, setSkills] = useState<SkillSpec[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const res = await api.getSkills();
        setSkills(res.data);
      } catch { /* 401 */ }
      finally { setLoading(false); }
    })();
  }, []);

  if (loading) {
    return (
      <>
        <Nav authenticated />
        <main className="max-w-5xl mx-auto px-4 sm:px-6 py-16 flex justify-center"><Spinner /></main>
      </>
    );
  }

  const skillDocs: Record<string, { description: string; steps: { step: string; command: string; params: string }[]; example: string }> = {
    list_workspace: {
      description: lang === 'zh'
        ? '列出 Discord 工作区结构：服务器、频道和成员。适用于快速了解当前所在的服务器环境。'
        : 'List the Discord workspace structure: servers, channels, and members. Useful for quickly understanding the current server environment.',
      steps: [
        { step: '1', command: 'servers', params: '-' },
        { step: '2', command: 'channels', params: 'server (optional)' },
        { step: '3', command: 'members', params: 'channel (optional)' },
      ],
      example: lang === 'zh'
        ? '/list_workspace\n→ 获取服务器列表、频道列表、成员列表'
        : '/list_workspace\n→ Get server list, channel list, member list',
    },
    search_conversation: {
      description: lang === 'zh'
        ? '在 Discord 频道中搜索对话内容并读取上下文消息。适用于查找历史讨论或特定信息。'
        : 'Search for conversation content in Discord channels and read contextual messages. Useful for finding historical discussions or specific information.',
      steps: [
        { step: '1', command: 'search', params: 'query' },
        { step: '2', command: 'read', params: 'count' },
      ],
      example: lang === 'zh'
        ? '/search_conversation "部署"\n→ 搜索相关消息并读取上下文'
        : '/search_conversation "deployment"\n→ Search related messages and read context',
    },
    send_notification: {
      description: lang === 'zh'
        ? '向指定 Discord 频道发送通知消息。适用于自动化提醒、状态更新或团队沟通。'
        : 'Send a notification message to a specified Discord channel. Useful for automated alerts, status updates, or team communication.',
      steps: [
        { step: '1', command: 'channels', params: 'server (optional)' },
        { step: '2', command: 'send', params: 'message' },
      ],
      example: lang === 'zh'
        ? '/send_notification "部署完成 ✅"\n→ 列出频道并发送通知消息'
        : '/send_notification "Deployment complete ✅"\n→ List channels and send notification message',
    },
  };

  return (
    <>
      <Nav authenticated />
      <main className="max-w-5xl mx-auto px-4 sm:px-6 py-16 space-y-8">
        <div>
          <h1 className="text-2xl font-bold text-slate-900 mb-2">{tr.skills_title}</h1>
          <p className="text-sm text-slate-500">{tr.skills_description}</p>
        </div>

        {skills.map((skill) => {
          const doc = skillDocs[skill.name];
          return (
            <Card key={skill.name} hover={false}>
              <div className="mb-4">
                <h2 className="text-lg font-bold text-slate-900 flex items-center gap-2">
                  <span className="text-brand-600">{skill.name}</span>
                  {skill.requires_auth && (
                    <span className="text-xs px-1.5 py-0.5 rounded-full bg-amber-50 text-amber-600">auth</span>
                  )}
                </h2>
                <p className="text-sm text-slate-500 mt-1">{doc?.description || skill.summary}</p>
              </div>

              {doc && (
                <>
                  <div className="mb-4">
                    <h3 className="text-sm font-semibold text-slate-700 mb-2">{lang === 'zh' ? '执行步骤' : 'Execution Steps'}</h3>
                    <div className="space-y-2">
                      {doc.steps.map((s, i) => (
                        <div key={i} className="flex items-start gap-3 text-sm">
                          <span className="w-6 h-6 rounded-full bg-brand-50 text-brand-600 flex items-center justify-center text-xs font-bold flex-shrink-0 mt-0.5">
                            {s.step}
                          </span>
                          <div>
                            <code className="text-brand-600">{s.command}</code>
                            {s.params !== '-' && <span className="text-slate-400 ml-1">({s.params})</span>}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className="bg-slate-50 rounded-lg p-3 text-sm">
                    <span className="text-slate-400 font-mono">{doc.example}</span>
                  </div>
                </>
              )}
            </Card>
          );
        })}
      </main>
    </>
  );
}
