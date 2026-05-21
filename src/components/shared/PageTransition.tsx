import { useRef, type ReactNode } from 'react';

interface PageTransitionProps {
  children: ReactNode;
}

export function PageTransition({ children }: PageTransitionProps) {
  const isFirst = useRef(true);
  const skip = isFirst.current;
  isFirst.current = false;

  if (skip) return <>{children}</>;

  return (
    <div
      className="animate-in fade-in duration-200"
      style={{
        animationName: 'fadeIn',
        animationDuration: '200ms',
        animationFillMode: 'both',
        willChange: 'transform, opacity',
      }}
    >
      {children}
    </div>
  );
}
