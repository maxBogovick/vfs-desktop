import { ref } from 'vue';

export type NotificationType = 'success' | 'error' | 'warning' | 'info';

export interface Notification {
  id: number;
  type: NotificationType;
  title: string;
  message?: string;
  duration?: number;
}

const notifications = ref<Notification[]>([]);
let nextId = 1;

export function useNotifications() {
  const show = (
    type: NotificationType,
    title: string,
    message?: string,
    duration: number = 3000
  ) => {
    const notification: Notification = {
      id: nextId++,
      type,
      title,
      message,
      duration,
    };

    notifications.value.push(notification);

    if (duration > 0) {
      setTimeout(() => {
        remove(notification.id);
      }, duration);
    }

    return notification.id;
  };

  const remove = (id: number) => {
    const index = notifications.value.findIndex(n => n.id === id);
    if (index !== -1) {
      notifications.value.splice(index, 1);
    }
  };

  const success = (title: string, message?: string, duration?: number) => {
    return show('success', title, message, duration);
  };

  const error = (title: string, message?: string, duration?: number) => {
    return show('error', title, message, duration);
  };

  const warning = (title: string, message?: string, duration?: number) => {
    return show('warning', title, message, duration);
  };

  const info = (title: string, message?: string, duration?: number) => {
    return show('info', title, message, duration);
  };

  const clear = () => {
    notifications.value = [];
  };

  return {
    notifications,
    show,
    remove,
    success,
    error,
    warning,
    info,
    clear,
  };
}
