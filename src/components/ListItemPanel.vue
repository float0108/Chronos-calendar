<!-- components/ListItemPanel.vue -->
<script setup lang="ts" generic="T extends { id?: number; content: string; description?: string; create_date: string; is_done: boolean; done_date?: string }">
import { ref, watch } from 'vue';
import { LayoutList } from 'lucide-vue-next';
import ListItem from './ListItem.vue';

interface Props {
  items: T[];
  selectedItem: T | null;
  themeStyle: Record<string, string>;
  cellStyle: Record<string, string>;
  parentId?: number;
  searchPlaceholder?: string;
  searchKeyword?: string;
  showInfoEntry?: boolean;
  infoEntryIcon?: any;
  showHeader?: boolean;
  isAdding?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  searchPlaceholder: 'Search...',
  searchKeyword: '',
  showInfoEntry: false,
  infoEntryIcon: undefined,
  showHeader: false,
  isAdding: false,
});

const emit = defineEmits<{
  select: [item: T];
  add: [content: string, parentId?: number];
  delete: [itemId: number];
  toggleDone: [item: T];
  search: [keyword: string];
  infoClick: [];
  'update:isAdding': [value: boolean];
  'start-add': [];
}>();

// 本地状态
const isAddingLocal = ref(false);
const addingKey = ref(0);
const isSearchFocused = ref(false);

// 同步外部 isAdding prop
watch(() => props.isAdding, (newVal, oldVal) => {
  if (newVal && !oldVal) {
    // false -> true: 开始添加模式
    isAddingLocal.value = true;
    addingKey.value++;
  } else if (!newVal && oldVal) {
    // true -> false: 取消添加模式
    isAddingLocal.value = false;
    addingKey.value = 0;
  }
});

// ============ 事件处理 ============

function handleStartAdd() {
  isAddingLocal.value = true;
  addingKey.value++;
  emit('start-add');
}

function handleAdd(content: string) {
  const trimmed = content.trim();
  if (!trimmed) {
    isAddingLocal.value = false;
    addingKey.value = 0;
    return;
  }
  emit('add', trimmed, props.parentId);
  isAddingLocal.value = false;
  addingKey.value = 0;
}

function handleCancelAdd() {
  isAddingLocal.value = false;
  addingKey.value = 0;
  emit('update:isAdding', false);
}

function handleSelect(item: T) {
  emit('select', item);
}

function handleDelete(itemId: number) {
  emit('delete', itemId);
}

function handleToggleDone(item: T) {
  emit('toggleDone', item);
}

function handleSearch(keyword: string) {
  emit('search', keyword);
}

function handleInfoClick() {
  emit('infoClick');
}
</script>

<template>
  <div class="list-item-panel flex flex-col flex-1 min-w-0 min-h-0">
    <!-- Search Header -->
    <div
      v-if="showHeader && !$slots.header"
      class="flex items-center gap-2 border-b px-3 py-2 shrink-0"
      :style="{ borderColor: cellStyle.borderColor }"
    >
      <input
        type="text"
        :placeholder="searchPlaceholder"
        :value="searchKeyword"
        @input="(e) => handleSearch((e.target as HTMLInputElement).value)"
        @focus="isSearchFocused = true"
        @blur="isSearchFocused = false"
        class="flex-1 px-2 py-1 rounded text-sm bg-transparent outline-none"
        :style="{
          color: themeStyle['--theme-text'],
          borderColor: isSearchFocused ? themeStyle['--theme-primary'] : 'transparent',
          borderWidth: '1px',
          transition: 'border-color 0.2s',
        }"
      />
      <button
        @click="handleStartAdd"
        class="p-1.5 rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors shrink-0"
        :style="{ color: themeStyle['--theme-text-muted'] }"
        title="Add"
      >
        <slot name="addIcon">
          <span class="text-lg leading-none">+</span>
        </slot>
      </button>
    </div>

    <!-- Custom header slot -->
    <slot name="header" />

    <!-- Content List -->
    <div class="flex-1 min-h-0 overflow-y-auto custom-scrollbar px-3 pt-2 pb-3">
      <div class="space-y-2">
        <!-- Info Entry (for subtask panel) -->
        <div v-if="showInfoEntry" class="mt-2">
          <div
            class="group flex items-center gap-1 px-3 py-2 rounded-lg transition-all cursor-pointer hover:bg-black/5 dark:hover:bg-white/5"
            :style="cellStyle"
            @click="handleInfoClick"
          >
            <component
              :is="infoEntryIcon || 'span'"
              class="w-4 h-4 opacity-60 group-hover:opacity-100 transition-opacity"
              :style="{ color: themeStyle['--theme-text-muted'] }"
            />
            <span
              class="text-sm opacity-60 group-hover:opacity-100 transition-opacity"
              :style="{ color: themeStyle['--theme-text-muted'] }"
            >
              Task Info
            </span>
          </div>
        </div>

        <!-- Add New Mode (outside virtual scroll) -->
        <ListItem
          v-if="isAddingLocal"
          :key="`add-item-${addingKey}`"
          is-add-mode
          @add="handleAdd"
          @cancel="handleCancelAdd"
          @click.stop
        />

        <!-- Item List with Virtual Scrolling -->
        <template v-if="items.length > 0">
          <ListItem
            v-for="item in items"
            :key="item.id"
            :title="item.content"
            :date="item.create_date"
            :is-done="item.is_done"
            center-calendar
            :selected="selectedItem?.id === item.id"
            @toggle-done="handleToggleDone(item)"
            @delete="handleDelete(item.id!)"
            @click="handleSelect(item)"
          />
        </template>

        <!-- Empty State -->
        <div
          v-if="items.length === 0 && !isAddingLocal"
          class="flex flex-col items-center justify-center py-20 pointer-events-none transition-opacity"
        >
          <div class="p-4 rounded-full" :style="cellStyle">
            <LayoutList
              class="w-8 h-8 opacity-20"
              :style="{ color: themeStyle['--theme-text'] }"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: transparent transparent;
  transition: scrollbar-color 0.3s ease;
}

.custom-scrollbar:hover {
  scrollbar-color: var(--theme-border) transparent;
}

.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: transparent;
  border-radius: 4px;
}

.custom-scrollbar:hover::-webkit-scrollbar-thumb {
  background-color: var(--theme-border);
}
</style>
