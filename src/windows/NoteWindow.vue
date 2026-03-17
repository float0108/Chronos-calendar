<script setup lang="ts">
import { ref, onMounted, nextTick, computed, onUnmounted, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { X, Trash2, FileText, StickyNote, ChevronLeft, Plus, PenLine } from 'lucide-vue-next';
import ListItem from '../components/ListItem.vue';
import {
  loadNotes,
  searchNotes,
  createNote,
  updateNote,
  updateNoteCreateDate,
  deleteNote,
  type Note
} from '../composables/db';
import { initDatabase } from '../composables/db/connection';
import { hexToRgba, adjustBrightness } from '../utils/color';
import type { AppSettings } from '../types';
import { defaultLightSettings, defaultDarkSettings } from '../types';

const settings = ref<AppSettings>({ ...defaultLightSettings });
const notes = ref<Note[]>([]);
const currentNote = ref<Note | null>(null);
const title = ref('');
const content = ref('');
const searchKeyword = ref('');

// DOM Refs 用于自动聚焦
const titleInputRef = ref<HTMLInputElement | null>(null);
const searchInputRef = ref<HTMLInputElement | null>(null);

// 视图控制：false 显示列表，true 显示编辑器
const isEditing = ref(false);

// 搜索框焦点状态
const isSearchFocused = ref(false);

// 防抖保存
let saveTimeout: ReturnType<typeof setTimeout> | null = null;

// 动态主题样式
const themeStyle = computed(() => {
  const s = settings.value;
  const bgOpacity = s.bg_opacity / 100;
  const cellOpacity = s.cell_opacity / 100;
  return {
    '--theme-bg': hexToRgba(s.bg_color, bgOpacity),
    '--theme-cell': hexToRgba(s.cell_color, cellOpacity),
    '--theme-text': s.text_color,
    '--theme-text-secondary': adjustBrightness(s.text_color, 30),
    '--theme-text-muted': adjustBrightness(s.text_color, 50),
    '--theme-primary': s.primary_color,
    '--theme-primary-alpha': hexToRgba(s.primary_color, 0.15),
    '--theme-border': s.cell_border_color || (s.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.05)'),
    '--theme-font-family': s.font_family,
    '--theme-font-size': `${s.font_size}px`,
    'font-family': s.font_family,
    'font-size': `${s.font_size}px`,
  };
});

function loadSettings() {
  const saved = localStorage.getItem('chronos_settings');
  if (saved) {
    const parsed = JSON.parse(saved);
    const defaults = parsed.theme_mode === 'dark' ? defaultDarkSettings : defaultLightSettings;
    settings.value = { ...defaults, ...parsed };
  }
  applyTheme();
}

function applyTheme() {
  const s = settings.value;
  const root = document.documentElement;
  root.style.setProperty('--primary', s.primary_color);
  root.style.setProperty('--text-primary', s.text_color);
  root.style.setProperty('--text-muted', adjustBrightness(s.text_color, 60));
  root.setAttribute('data-theme', s.theme_mode);
}

function handleSettingsUpdate() {
  loadSettings();
}

async function loadNotesList() {
  if (searchKeyword.value.trim()) {
    notes.value = await searchNotes(searchKeyword.value);
  } else {
    notes.value = await loadNotes();
  }
}

// 进入编辑模式
async function handleSelectNote(note: Note) {
  currentNote.value = note;
  title.value = note.title;
  content.value = note.content;
  isEditing.value = true;
}

// 返回列表模式
function handleBackToList() {
  isEditing.value = false;
  setTimeout(() => {
    currentNote.value = null;
  }, 300);
}

// 新建备忘录并直接进入编辑
async function handleNewNote() {
  const id = await createNote('', ''); 
  if (id) {
    await loadNotesList();
    const newNote = notes.value.find(n => n.id === id);
    if (newNote) {
      handleSelectNote(newNote);
      await nextTick();
      titleInputRef.value?.focus();
    }
  }
}

async function handleDeleteNote() {
  if (!currentNote.value?.id) return;

  if (confirm('Delete?')) { // 使用极简英文，或后续替换为弹窗组件
    await deleteNote(currentNote.value.id);
    handleBackToList();
    await loadNotesList();
  }
}

async function handleDeleteNoteFromList(note: Note) {
  if (!note.id) return;

  if (confirm('Delete?')) {
    await deleteNote(note.id);
    await loadNotesList();
  }
}

// 更新备忘录标题
async function handleUpdateNoteTitle(note: Note, newTitle: string) {
  if (!note.id) return;
  const trimmed = newTitle.trim();
  if (!trimmed) return;
  if (trimmed === note.title) return;
  try {
    await updateNote(note.id, trimmed, note.content);
    await loadNotesList();
  } catch (error) {
    console.error('Failed to update note title:', error);
  }
}

// 更新备忘录日期
async function handleUpdateNoteDate(note: Note, newDate: string) {
  if (!note.id) return;
  try {
    await updateNoteCreateDate(note.id, newDate);
    await loadNotesList();
  } catch (error) {
    console.error('Failed to update note date:', error);
  }
}

// 自动保存（防抖）
function scheduleSave() {
  if (saveTimeout) {
    clearTimeout(saveTimeout);
  }
  saveTimeout = setTimeout(async () => {
    if (currentNote.value?.id) {
      await updateNote(currentNote.value.id, title.value, content.value);
      const noteInList = notes.value.find(n => n.id === currentNote.value!.id);
      if (noteInList) {
        noteInList.title = title.value;
        noteInList.content = content.value;
      }
    }
  }, 500);
}

watch([title, content], () => {
  if (currentNote.value?.id && isEditing.value) {
    scheduleSave();
  }
});

watch(isSearchFocused, (focused) => {
  if (focused) {
    nextTick(() => {
      searchInputRef.value?.focus();
    });
  }
});

async function handleClose() {
  const win = getCurrentWindow();
  await win.hide();
}

async function handleIconDrag() {
  try {
    const win = getCurrentWindow();
    await win.startDragging();
  } catch (error) {
    console.error('Drag failed:', error);
  }
}

onMounted(async () => {
  loadSettings();
  await initDatabase();
  await loadNotesList();
  window.addEventListener('storage', handleSettingsUpdate);

  await nextTick();
  requestAnimationFrame(async () => {
    const win = getCurrentWindow();
    await win.show();
    await win.setFocus();
  });
});

onUnmounted(() => {
  window.removeEventListener('storage', handleSettingsUpdate);
  if (saveTimeout) {
    clearTimeout(saveTimeout);
  }
});
</script>

<template>
  <div class="note-overlay fixed inset-0 z-[9999] flex w-full h-full" :style="themeStyle">
    <div class="note-panel relative flex flex-col overflow-hidden w-full h-full rounded-lg transition-colors shadow-lg"
         :style="{
           backgroundColor: 'var(--theme-bg)',
           border: '1px solid var(--theme-border)',
           backdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
           WebkitBackdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
         }">

      <div class="title-bar flex items-center gap-2 px-3 py-2.5 shrink-0 select-none group"
           data-tauri-drag-region>
        <template v-if="!isEditing">
          <button @mousedown="handleIconDrag"
                  class="shrink-0 w-6 h-6 flex items-center justify-center cursor-grab active:cursor-grabbing hover:opacity-80 transition-opacity"
                  :style="{ color: 'var(--theme-text)' }"
                  title="Drag">
            <StickyNote class="w-4 h-4" />
          </button>

          <div class="flex-1 min-w-0 flex justify-center items-center relative h-6"
               @mousedown="(e) => e.target === e.currentTarget && handleIconDrag()">
            <span v-show="!isSearchFocused"
                  class="text-[14px] font-medium leading-relaxed transition-opacity cursor-text"
                  :style="{ color: 'var(--theme-text)' }"
                  @click="isSearchFocused = true">
              Notes
            </span>
            <input
              ref="searchInputRef"
              v-show="isSearchFocused"
              v-model="searchKeyword"
              type="text"
              placeholder="..."
              class="absolute inset-0 w-full h-full bg-black/5 dark:bg-white/5 rounded-md px-2 outline-none text-[13px] leading-relaxed text-center selection:bg-[var(--theme-primary-alpha)] caret-[var(--theme-text)]"
              :style="{ color: 'var(--theme-text)' }"
              @input="loadNotesList"
              @focus="isSearchFocused = true"
              @blur="isSearchFocused = false"
            />
          </div>

          <button @click="handleNewNote"
                  class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
                  :style="{ color: 'var(--theme-text)' }">
            <Plus class="w-4 h-4" />
          </button>
        </template>

        <template v-else>
          <button @click="handleBackToList"
                  class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
                  :style="{ color: 'var(--theme-text)' }">
            <ChevronLeft class="w-4 h-4" />
          </button>

          <div class="flex-1 min-w-0 flex justify-center items-center h-6"
               @mousedown="handleIconDrag">
            <input
              ref="titleInputRef"
              v-model="title"
              type="text"
              placeholder="Aa"
              class="bg-transparent outline-none text-[14px] font-medium leading-relaxed text-center selection:bg-[var(--theme-primary-alpha)] max-w-[200px] caret-[var(--theme-text)]"
              :style="{ color: 'var(--theme-text)' }"
              @mousedown.stop
            />
          </div>

          <button @click="handleDeleteNote"
                  class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-red-50 dark:hover:bg-red-900/30"
                  :style="{ color: 'var(--theme-text)' }">
            <Trash2 class="w-4 h-4 hover:text-red-500 dark:hover:text-red-400 transition-colors" />
          </button>
        </template>

        <button @click="handleClose"
          class="close-btn shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
          :style="{ color: 'var(--theme-text)' }">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="flex-1 relative overflow-hidden">
        <Transition name="view-slide">

          <div v-if="!isEditing" class="absolute inset-0 flex flex-col w-full h-full">
            <div class="flex-1 overflow-y-auto custom-scrollbar px-3 pt-2 pb-3">
              <TransitionGroup name="list" tag="div" class="space-y-2">
                <ListItem
                  v-for="note in notes"
                  :key="note.id"
                  :title="note.title"
                  :preview="note.content"
                  :date="note.create_date"
                  @update:title="(val) => handleUpdateNoteTitle(note, val)"
                  @update:date="(val) => handleUpdateNoteDate(note, val)"
                  @delete="handleDeleteNoteFromList(note)"
                  @click="handleSelectNote(note)"
                />
              </TransitionGroup>

              <div v-if="notes.length === 0" class="flex flex-col items-center justify-center py-20 pointer-events-none transition-opacity">
                <div class="p-4 rounded-full" :style="{ backgroundColor: 'var(--theme-cell)' }">
                  <FileText class="w-8 h-8 opacity-20" :style="{ color: 'var(--theme-text)' }" />
                </div>
              </div>
            </div>
          </div>

          <div v-else class="absolute inset-0 flex flex-col w-full h-full z-10">
            <div class="flex-1 overflow-y-auto custom-scrollbar px-3 pt-2 pb-3">
              <div class="rounded-lg p-4 flex flex-col min-h-full shadow-sm"
                   :style="{
                     backgroundColor: 'var(--theme-cell)',
                     border: '1px solid var(--theme-border)',
                   }">
                <textarea
                  v-model="content"
                  placeholder="..."
                  class="w-full flex-1 bg-transparent resize-none outline-none text-[14px] leading-[1.8] custom-scrollbar placeholder:transition-opacity focus:placeholder:opacity-40 selection:bg-[var(--theme-primary-alpha)]"
                  :style="{ color: 'var(--theme-text-secondary)' }"
                ></textarea>

                <div class="mt-3 flex justify-end shrink-0">
                  <div class="flex items-center gap-1.5 px-2 py-1 rounded text-[12px] opacity-40 transition-opacity" 
                       :style="{ color: 'var(--theme-text-muted)', backgroundColor: 'var(--theme-bg)' }">
                    <PenLine class="w-3 h-3" />
                    <span>{{ content.length }}</span>
                  </div>
                </div>

              </div>
            </div>
          </div>

        </Transition>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 优雅的悬浮滚动条，默认隐藏，hover时显示，比 no-scrollbar 更好用 */
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

::selection {
  background-color: var(--theme-primary-alpha);
  color: inherit;
}

input, textarea {
  -webkit-appearance: none;
  appearance: none;
}

/* 列表动画 */
.list-move,
.list-enter-active,
.list-leave-active {
  transition: all 0.35s cubic-bezier(0.4, 0, 0.2, 1);
}

.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateY(4px) scale(0.99);
}

.list-leave-active {
  position: absolute;
  width: calc(100% - 24px);
}

/* 视图切换动画 */
.view-slide-enter-active,
.view-slide-leave-active {
  transition: all 0.35s cubic-bezier(0.4, 0, 0.2, 1);
}

.view-slide-enter-from {
  opacity: 0;
  transform: translateX(16px) scale(0.99);
}

.view-slide-leave-to {
  opacity: 0;
  transform: translateX(-16px) scale(0.99);
}
</style>