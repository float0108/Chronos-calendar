<!-- components/VirtualList.vue -->
<script setup lang="ts" generic="T extends { id?: number; content: string; create_date: string; is_done: boolean }">
import { RecycleScroller } from 'vue-virtual-scroller';
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css';
import ListItem from './ListItem.vue';

interface Props {
  items: T[];
  selectedId: number | null;
  themeStyle: Record<string, string>;
  cellStyle: Record<string, string>;
}

defineProps<Props>();

const emit = defineEmits<{
  select: [item: T];
  toggleDone: [item: T];
  delete: [itemId: number];
}>();
</script>

<template>
  <RecycleScroller
    :items="items"
    :item-size="56"
    key-field="id"
    class="h-full w-full"
    v-slot="{ item }"
  >
    <ListItem
      :key="item.id"
      :title="item.content"
      :date="item.create_date"
      :is-done="item.is_done"
      center-calendar
      :selected="selectedId === item.id"
      @toggle-done="emit('toggleDone', item)"
      @delete="emit('delete', item.id!)"
      @click="emit('select', item)"
    />
  </RecycleScroller>
</template>
