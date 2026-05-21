import { openUrl } from '@tauri-apps/plugin-opener';
import { Card } from '@/components/ui/card';

function Link({ href, children }: { href: string; children: React.ReactNode }) {
  return (
    <button
      onClick={() => openUrl(href)}
      className="block w-full text-center text-primary hover:underline cursor-pointer"
    >
      {children}
    </button>
  );
}

export function AboutSection() {
  return (
    <Card size="sm" className="p-3 text-center text-sm text-muted-foreground">
      <p className="font-medium text-foreground">Tomato</p>
      <p className="mt-1">Version 0.1.0</p>
      <p className="mt-1">本地优先的番茄钟专注计时器。</p>
      <p className="mt-1">使用 Tauri + React + Rust 构建</p>
      <div className="mt-3 space-y-1">
        <Link href="https://github.com/Goblinsscholar">
          GitHub: github.com/Goblinsscholar
        </Link>
        <Link href="https://www.cnblogs.com/Goblinscholar">
          博客: www.cnblogs.com/Goblinscholar
        </Link>
      </div>
    </Card>
  );
}
