import { Toaster } from 'sonner';

export function ToastProvider() {
  return (
    <Toaster
      position="top-center"
      toastOptions={{
        duration: 3000,
        className: 'text-sm',
      }}
    />
  );
}
