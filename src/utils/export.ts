import { save, open } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import type { Schedule } from '../types';

interface CellColorData {
  date: string;
  color: string;
}

interface ParsedCSVData {
  schedules: Schedule[];
  cellColors: CellColorData[];
}

/**
 * Escape a CSV field value
 * - If the field contains comma, quote, or newline, wrap it in quotes
 * - Escape any quotes by doubling them
 */
function escapeCSVField(value: string): string {
  if (value.includes(',') || value.includes('"') || value.includes('\n') || value.includes('\r')) {
    return '"' + value.replace(/"/g, '""') + '"';
  }
  return value;
}

/**
 * Unescape a CSV field value
 * - Remove surrounding quotes if present
 * - Unescape doubled quotes
 */
function unescapeCSVField(value: string): string {
  if (value.startsWith('"') && value.endsWith('"')) {
    return value.slice(1, -1).replace(/""/g, '"');
  }
  return value;
}

/**
 * Parse a CSV line handling quoted fields
 */
function parseCSVLine(line: string): string[] {
  const fields: string[] = [];
  let currentField = '';
  let inQuotes = false;

  for (let i = 0; i < line.length; i++) {
    const char = line[i];

    if (inQuotes) {
      if (char === '"') {
        // Check if this is an escaped quote
        if (i + 1 < line.length && line[i + 1] === '"') {
          currentField += '"';
          i++; // Skip next quote
        } else {
          // End of quoted field
          inQuotes = false;
        }
      } else {
        currentField += char;
      }
    } else {
      if (char === '"') {
        inQuotes = true;
      } else if (char === ',') {
        fields.push(unescapeCSVField(currentField));
        currentField = '';
      } else {
        currentField += char;
      }
    }
  }

  // Add the last field
  fields.push(unescapeCSVField(currentField));

  return fields;
}

/**
 * Clean a field value by removing carriage returns and trimming whitespace
 */
function cleanFieldValue(value: string): string {
  return value.replace(/\r/g, '').trim();
}

/**
 * Convert schedules and cell colors to CSV string
 */
export function schedulesToCSV(schedules: Schedule[], cellColors: Map<string, string>): string {
  const header = 'create_date,content,is_done,priority,cell_color,done_date,description';
  const lines: string[] = [header];

  // Create a map of date to cell color for quick lookup
  const colorMap = new Map(cellColors);

  // Group schedules by date
  const grouped = new Map<string, Schedule[]>();
  schedules.forEach(s => {
    if (!grouped.has(s.create_date)) grouped.set(s.create_date, []);
    grouped.get(s.create_date)!.push(s);
  });

  // Export schedules
  schedules.forEach(schedule => {
    const color = colorMap.get(schedule.create_date) || '';
    const line = [
      schedule.create_date,
      escapeCSVField(schedule.content),
      schedule.is_done ? 'true' : 'false',
      String(schedule.priority),
      color,
      schedule.done_date || '',
      escapeCSVField(schedule.description || '')
    ].join(',');
    lines.push(line);
    // Remove from colorMap after processing
    colorMap.delete(schedule.create_date);
  });

  // Export dates with only colors (no schedules)
  colorMap.forEach((color, date) => {
    const line = [
      date,
      '', // empty content
      'false',
      '0',
      color,
      '', // empty done_date
      '' // empty description
    ].join(',');
    lines.push(line);
  });

  return lines.join('\n');
}

/**
 * Parse CSV string to schedules and cell colors
 */
export function csvToSchedules(csvString: string): ParsedCSVData {
  const lines = csvString.split('\n').filter(line => line.trim());

  if (lines.length === 0) {
    throw new Error('CSV 文件为空');
  }

  // Validate header
  const header = lines[0].split(',');

  // New format (create_date, done_date, description)
  const expectedHeaderNew = ['create_date', 'content', 'is_done', 'priority', 'cell_color', 'done_date', 'description'];
  // Old format V3 (date, created_at, done_time, description)
  const expectedHeaderV3 = ['date', 'content', 'is_done', 'priority', 'cell_color', 'created_at', 'done_time', 'description'];
  // Old format V2 (date, created_at, done_time)
  const expectedHeaderV2 = ['date', 'content', 'is_done', 'priority', 'cell_color', 'created_at', 'done_time'];
  // Old format V1 (date, created_at)
  const expectedHeaderV1 = ['date', 'content', 'is_done', 'priority', 'cell_color', 'created_at'];

  const isNewFormat = header.length === 7 && header.every((h, i) => h.trim() === expectedHeaderNew[i]);
  const isV3Format = header.length === 8 && header.every((h, i) => h.trim() === expectedHeaderV3[i]);
  const isV2Format = header.length === 7 && header.every((h, i) => h.trim() === expectedHeaderV2[i]);
  const isV1Format = header.length === 6 && header.every((h, i) => h.trim() === expectedHeaderV1[i]);

  if (!isNewFormat && !isV3Format && !isV2Format && !isV1Format) {
    throw new Error('CSV 文件格式不正确，表头应为: ' + expectedHeaderNew.join(','));
  }

  const schedules: Schedule[] = [];
  const cellColors: CellColorData[] = [];
  const seenColors = new Map<string, string>();

  // Parse data rows
  for (let i = 1; i < lines.length; i++) {
    const fields = parseCSVLine(lines[i]);

    const expectedFieldCount = isNewFormat ? 7 : (isV3Format ? 8 : (isV2Format ? 7 : 6));
    if (fields.length < expectedFieldCount) {
      console.warn(`跳过格式错误的第 ${i + 1} 行`);
      continue;
    }

    // Clean all field values
    let createDate: string, content: string, isDoneStr: string, priorityStr: string, cellColor: string, doneDate: string, description: string;

    if (isNewFormat) {
      // New format: create_date, content, is_done, priority, cell_color, done_date, description
      [createDate, content, isDoneStr, priorityStr, cellColor, doneDate, description] = fields.map(cleanFieldValue);
    } else if (isV3Format) {
      // V3 format: date, content, is_done, priority, cell_color, created_at, done_time, description
      // Map to new: create_date=date, done_date=done_time, ignore created_at
      [createDate, content, isDoneStr, priorityStr, cellColor, , doneDate, description] = fields.map(cleanFieldValue);
    } else if (isV2Format) {
      // V2 format: date, content, is_done, priority, cell_color, created_at, done_time
      [createDate, content, isDoneStr, priorityStr, cellColor, , doneDate] = fields.map(cleanFieldValue);
      description = '';
    } else {
      // V1 format: date, content, is_done, priority, cell_color, created_at
      [createDate, content, isDoneStr, priorityStr, cellColor] = fields.map(cleanFieldValue);
      doneDate = '';
      description = '';
    }

    // Normalize date format: convert YYYY/M/D to YYYY-MM-DD
    if (createDate.includes('/')) {
      const parts = createDate.split('/');
      if (parts.length === 3) {
        const year = parts[0];
        const month = parts[1].padStart(2, '0');
        const day = parts[2].padStart(2, '0');
        createDate = `${year}-${month}-${day}`;
      }
    }

    // Store cell color
    if (cellColor) {
      seenColors.set(createDate, cellColor);
    }

    // Only create schedule if there's content
    if (content.trim()) {
      // Normalize done_date format (just date part YYYY-MM-DD)
      if (doneDate && doneDate.includes('/')) {
        const parts = doneDate.split(' ')[0].split('/');
        if (parts.length === 3) {
          const year = parts[0];
          const month = parts[1].padStart(2, '0');
          const day = parts[2].padStart(2, '0');
          doneDate = `${year}-${month}-${day}`;
        }
      } else if (doneDate && doneDate.includes('T')) {
        // Extract date from ISO format
        doneDate = doneDate.substring(0, 10);
      }

      schedules.push({
        create_date: createDate,
        content,
        is_done: isDoneStr.toLowerCase() === 'true',
        priority: parseInt(priorityStr, 10) || 0,
        done_date: doneDate || undefined,
        description: description || undefined
      });
    }
  }

  // Convert seenColors to array
  seenColors.forEach((color, date) => {
    cellColors.push({ date, color });
  });

  return { schedules, cellColors };
}

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
      // User cancelled
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
 * Detect if a buffer contains UTF-8 BOM
 */
function hasBOM(buffer: Uint8Array): boolean {
  return buffer.length >= 3 && buffer[0] === 0xEF && buffer[1] === 0xBB && buffer[2] === 0xBF;
}

/**
 * Try to decode buffer as UTF-8, fallback to ANSI (Windows-1252)
 */
function decodeCSVBuffer(buffer: Uint8Array): string {
  // Try UTF-8 first
  const utf8Decoder = new TextDecoder('utf-8', { fatal: false });

  // If has BOM, skip it
  if (hasBOM(buffer)) {
    return utf8Decoder.decode(buffer.slice(3));
  }

  // Try UTF-8
  const utf8Text = utf8Decoder.decode(buffer);

  // Check for replacement characters that indicate decoding issues
  // If there are many replacement characters, it's likely not UTF-8
  const replacementCount = (utf8Text.match(/\uFFFD/g) || []).length;
  const totalChars = utf8Text.length;

  // If less than 5% are replacement characters, it's probably UTF-8
  if (replacementCount / totalChars < 0.05) {
    return utf8Text;
  }

  // Fallback to GBK for Chinese Windows systems
  try {
    const gbkDecoder = new TextDecoder('gbk');
    const gbkText = gbkDecoder.decode(buffer);

    // Check if GBK decoding looks reasonable (contains Chinese characters)
    const chineseCharCount = (gbkText.match(/[\u4e00-\u9fa5]/g) || []).length;
    if (chineseCharCount > 0) {
      return gbkText;
    }
  } catch (error) {
    console.warn('GBK decoding failed:', error);
  }

  // Fallback to Windows-1252 (ANSI)
  try {
    const ansiDecoder = new TextDecoder('windows-1252');
    return ansiDecoder.decode(buffer);
  } catch (error) {
    console.error('Failed to decode file:', error);
    throw new Error('无法识别文件编码，请使用 UTF-8 或 GBK 编码');
  }
}

/**
 * Import data from CSV file using Tauri dialog
 */
export async function importFromFile(): Promise<string | null> {
  try {
    const { readFile } = await import('@tauri-apps/plugin-fs');

    const filePath = await open({
      multiple: false,
      filters: [{
        name: 'CSV',
        extensions: ['csv']
      }]
    });

    if (!filePath) {
      // User cancelled
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
