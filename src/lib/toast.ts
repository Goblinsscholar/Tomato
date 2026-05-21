import { toast } from 'sonner';

const friendlyErrors: Record<string, string> = {
  invalid_tag: '标签不能为空',
  invalid_duration: '请输入有效时长',
  timer_not_idle: '当前计时正在运行',
  timer_not_running: '计时尚未开始',
  timer_not_paused: '计时未处于暂停状态',
  internal_error: '内部错误，请重启应用',
  tag_name_empty: '标签名称不能为空',
};

function getFriendlyMessage(message: string) {
  if (message.startsWith('db_error:')) {
    return '数据库错误，请重试';
  }
  return friendlyErrors[message] ?? message;
}

export function showError(message: string) {
  toast.error(getFriendlyMessage(message), {
    duration: 4000,
  });
}

export function showSuccess(message: string) {
  toast.success(message, {
    duration: 3000,
  });
}
