import { ref } from 'vue';
import type { Schedule } from '../types';
import { importFromFile, csvToSchedules } from '../utils/export';
import { useToast } from './useToast';

export function useImport() {
  const { showSuccess, showError } = useToast();

  const showImportDialog = ref(false);
  const pendingImportRecordCount = ref(0);
  const pendingImportData = ref<{ schedules: Schedule[], cellColors: { date: string, color: string }[] } | null>(null);

  async function handleImport(
    _importSchedulesFromData: (schedules: Schedule[], cellColors: { date: string, color: string }[], mode: 'merge' | 'overwrite') => Promise<{ success: boolean, scheduleStats?: { inserted: number, updated: number }, colorStats?: { inserted: number, updated: number } }>
  ): Promise<void> {
    try {
      const csvContent = await importFromFile();

      if (!csvContent) {
        // User cancelled
        return;
      }

      console.log('CSV content length:', csvContent.length);
      console.log('CSV first 500 chars:', csvContent.substring(0, 500));

      const { schedules, cellColors } = csvToSchedules(csvContent);
      console.log('Parsed data:', { schedules, cellColors });

      pendingImportRecordCount.value = schedules.length + cellColors.length;

      if (pendingImportRecordCount.value === 0) {
        showError('CSV 文件中没有数据');
        return;
      }

      // Store the parsed data for later use
      pendingImportData.value = { schedules, cellColors };

      // Show import dialog to ask user for strategy
      showImportDialog.value = true;
    } catch (error) {
      console.error('Import error:', error);
      const message = error instanceof Error ? error.message : '导入失败，请重试';
      showError(message);
    }
  }

  async function performImport(
    mode: 'merge' | 'overwrite',
    importSchedulesFromData: (schedules: Schedule[], cellColors: { date: string, color: string }[], mode: 'merge' | 'overwrite') => Promise<{ success: boolean, scheduleStats?: { inserted: number, updated: number }, colorStats?: { inserted: number, updated: number } }>
  ): Promise<void> {
    showImportDialog.value = false;

    if (!pendingImportData.value) {
      showError('导入数据丢失，请重试');
      return;
    }

    const { schedules, cellColors } = pendingImportData.value;
    pendingImportData.value = null;

    const result = await importSchedulesFromData(schedules, cellColors, mode);
    if (result.success && result.scheduleStats && result.colorStats) {
      const messages = [];

      if (mode === 'merge') {
        // 合并模式：显示新增和更新
        if (result.scheduleStats.inserted > 0) {
          messages.push(`新增 ${result.scheduleStats.inserted} 条日程`);
        }
        if (result.scheduleStats.updated > 0) {
          messages.push(`更新 ${result.scheduleStats.updated} 条日程`);
        }
        if (result.colorStats.inserted > 0) {
          messages.push(`新增 ${result.colorStats.inserted} 个颜色标记`);
        }
        if (result.colorStats.updated > 0) {
          messages.push(`更新 ${result.colorStats.updated} 个颜色标记`);
        }
        showSuccess(messages.length > 0 ? `导入成功：${messages.join('，')}` : '导入成功：无数据变化');
      } else {
        // 覆盖模式：只显示插入数量
        if (result.scheduleStats.inserted > 0) {
          messages.push(`${result.scheduleStats.inserted} 条日程`);
        }
        if (result.colorStats.inserted > 0) {
          messages.push(`${result.colorStats.inserted} 个颜色标记`);
        }
        showSuccess(messages.length > 0 ? `导入成功：${messages.join('，')}` : '导入成功');
      }
    }
  }

  function cancelImport(): void {
    showImportDialog.value = false;
  }

  return {
    showImportDialog,
    pendingImportRecordCount,
    handleImport,
    performImport,
    cancelImport,
  };
}
