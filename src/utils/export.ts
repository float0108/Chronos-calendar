/**
 * 文件导入导出工具
 * 统一入口，向后兼容
 */
import { save, open } from '@tauri-apps/plugin-dialog';
import { writeTextFile, readFile } from '@tauri-apps/plugin-fs';
import { decodeCSVBuffer } from './csv';

// 重新导出 CSV 相关函数
export { schedulesToCSV, csvToSchedules } from './csv';

/**
 * Export data to CSV file using Tauri dialog
 */
export async function exportToFile(data: string, filename: string): Promise<boolean> {
  try {
    const filePath = await save({
      defaultPath: filename,
      filters: [{
        name: 'CSV',
        extensions: ['csv']
      }]
    });

    if (!filePath) {
      return false;
    }

    await writeTextFile(filePath, data);
    return true;
  } catch (error) {
    console.error('Failed to export file:', error);
    throw new Error('导出文件失败，请重试');
  }
}

/**
 * Import data from CSV file using Tauri dialog
 */
export async function importFromFile(): Promise<string | null> {
  try {
    const filePath = await open({
      multiple: false,
      filters: [{
        name: 'CSV',
        extensions: ['csv']
      }]
    });

    if (!filePath) {
      return null;
    }

    // Read as binary buffer
    const buffer = await readFile(filePath as string);

    // Decode with encoding detection
    const content = decodeCSVBuffer(buffer);
    return content;
  } catch (error) {
    console.error('Failed to import file:', error);
    const message = error instanceof Error ? error.message : '导入文件失败，请重试';
    throw new Error(message);
  }
}
