declare module 'vue-virtual-scroller' {
  import { DefineComponent } from 'vue';

  interface RecycleScrollerProps {
    items: any[];
    itemSize: number | null;
    keyField?: string;
    direction?: 'vertical' | 'horizontal';
    buffer?: number;
    pageMode?: boolean;
    prerender?: number;
    emitUpdate?: boolean;
  }

  export const RecycleScroller: DefineComponent<RecycleScrollerProps>;
  export const DynamicScroller: DefineComponent<RecycleScrollerProps>;
  export const DynamicScrollerItem: DefineComponent<any>;
}