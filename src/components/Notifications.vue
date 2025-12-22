<script setup lang="ts">
import { useNotifications } from '../composables/useNotifications';

const { notifications, remove } = useNotifications();

const getIcon = (type: string) => {
  const icons = {
    success: '✓',
    error: '✗',
    warning: '⚠',
    info: 'ℹ',
  };
  return icons[type as keyof typeof icons] || 'ℹ';
};

const getColors = (type: string) => {
  const colors = {
    success: 'bg-green-50 border-green-300 text-green-800',
    error: 'bg-red-50 border-red-300 text-red-800',
    warning: 'bg-yellow-50 border-yellow-300 text-yellow-800',
    info: 'bg-blue-50 border-blue-300 text-blue-800',
  };
  return colors[type as keyof typeof colors] || colors.info;
};

const getIconColors = (type: string) => {
  const colors = {
    success: 'bg-green-500 text-white',
    error: 'bg-red-500 text-white',
    warning: 'bg-yellow-500 text-white',
    info: 'bg-blue-500 text-white',
  };
  return colors[type as keyof typeof colors] || colors.info;
};
</script>

<template>
  <div class="fixed top-4 right-4 z-[100] space-y-2 pointer-events-none">
    <transition-group name="notification">
      <div
        v-for="notification in notifications"
        :key="notification.id"
        :class="[
          'pointer-events-auto w-80 rounded-lg border-2 shadow-lg overflow-hidden',
          getColors(notification.type)
        ]"
      >
        <div class="flex items-start gap-3 p-3">
          <!-- Icon -->
          <div :class="['w-6 h-6 rounded-full flex items-center justify-center flex-shrink-0 text-sm font-bold', getIconColors(notification.type)]">
            {{ getIcon(notification.type) }}
          </div>

          <!-- Content -->
          <div class="flex-1 min-w-0">
            <div class="font-bold text-sm mb-0.5">{{ notification.title }}</div>
            <div v-if="notification.message" class="text-xs opacity-90">
              {{ notification.message }}
            </div>
          </div>

          <!-- Close button -->
          <button
            @click="remove(notification.id)"
            class="flex-shrink-0 w-5 h-5 rounded hover:bg-black/10 flex items-center justify-center text-xs font-bold"
          >
            ✕
          </button>
        </div>

        <!-- Progress bar -->
        <div
          v-if="notification.duration && notification.duration > 0"
          class="h-1 bg-black/10"
        >
          <div
            class="h-full bg-current opacity-50 animate-progress"
            :style="{ animationDuration: `${notification.duration}ms` }"
          ></div>
        </div>
      </div>
    </transition-group>
  </div>
</template>

<style scoped>
.notification-enter-active,
.notification-leave-active {
  transition: all 0.3s ease;
}

.notification-enter-from {
  opacity: 0;
  transform: translateX(100px);
}

.notification-leave-to {
  opacity: 0;
  transform: translateX(100px) scale(0.8);
}

.notification-move {
  transition: transform 0.3s ease;
}

@keyframes progress {
  from {
    width: 100%;
  }
  to {
    width: 0%;
  }
}

.animate-progress {
  animation: progress linear forwards;
}
</style>
