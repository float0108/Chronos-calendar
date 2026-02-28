<script setup lang="ts">
import { useToast } from '../composables/useToast';

const { toasts, removeToast } = useToast();
</script>

<template>
  <div class="toast-container">
    <TransitionGroup name="toast">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        :class="['toast', `toast-${toast.type}`]"
        @click="removeToast(toast.id)"
      >
        <span class="toast-message">{{ toast.message }}</span>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: center;
}

.toast {
  padding: 6px 16px;
  border-radius: 20px;
  background: var(--glass-bg);
  backdrop-filter: blur(8px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  cursor: pointer;
  font-size: var(--font-size-xs);
}

.toast-error {
  background: rgba(239, 68, 68, 0.9);
  color: white;
}

.toast-success {
  background: rgba(34, 197, 94, 0.9);
  color: white;
}

.toast-info {
  background: rgba(59, 130, 246, 0.9);
  color: white;
}

.toast-message {
  color: inherit;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.2s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateY(-10px);
}

.toast-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
