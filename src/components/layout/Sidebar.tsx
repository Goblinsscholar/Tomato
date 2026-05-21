import { useEffect, useState } from 'react';
import { NavLink } from 'react-router-dom';
import { Clock, BarChart3, Settings, Minus, Square, Minimize2, X } from 'lucide-react';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

const navItems = [
  { to: '/', label: '首页', icon: Clock },
  { to: '/statistics', label: '统计', icon: BarChart3 },
  { to: '/settings', label: '设置', icon: Settings },
];

const appWindow = getCurrentWebviewWindow();

export function Sidebar() {
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    appWindow.isMaximized().then(setIsMaximized).catch(() => {});
    const unlisten = appWindow.onResized(() => {
      appWindow.isMaximized().then(setIsMaximized).catch(() => {});
    }).catch(() => () => {});
    return () => { unlisten.then((fn) => fn()).catch(() => {}); };
  }, []);

  const handleMinimize = () => { appWindow.minimize().catch(() => {}); };
  const handleMaximize = () => { appWindow.toggleMaximize().catch(() => {}); };
  const handleClose = () => { appWindow.hide().catch(() => {}); };

  return (
    <nav className="drag-region flex items-center rounded-3xl bg-muted py-2 pl-3 overflow-hidden select-none">
      <div className="text-xs font-semibold uppercase tracking-[0.2em] text-muted-foreground">
        番茄钟
      </div>
      <div className="flex flex-1 items-center justify-center gap-1">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.to === '/'}
            className={({ isActive }) =>
              `no-drag inline-flex h-10 w-10 items-center justify-center rounded-2xl transition-colors ${
                isActive
                  ? 'bg-primary text-primary-foreground shadow-sm'
                  : 'text-muted-foreground hover:bg-muted hover:text-foreground'
              }`
            }
            title={item.label}
          >
            <item.icon className="h-5 w-5" />
          </NavLink>
        ))}
      </div>
      <div className="flex items-center">
        <button
          onClick={handleMinimize}
          className="window-control text-muted-foreground"
          title="最小化"
        >
          <Minus className="h-4 w-4" />
        </button>
        <button
          onClick={handleMaximize}
          className="window-control text-muted-foreground"
          title={isMaximized ? '还原' : '最大化'}
        >
          {isMaximized ? <Minimize2 className="h-4 w-4" /> : <Square className="h-4 w-4" />}
        </button>
        <button
          onClick={handleClose}
          className="window-control close-btn text-muted-foreground"
          title="关闭"
        >
          <X className="h-4 w-4" />
        </button>
      </div>
    </nav>
  );
}
