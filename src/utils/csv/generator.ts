import type { Schedule } from '../../types';
import { parseCSVLine, cleanFieldValue, escapeCSVField } from './parser';

interface CellColorData {
  date: string;
  color: string;
}

/**
 * Convert schedules and cell colors to CSV string
 */
export function schedulesToCSV(schedules: Schedule[], cellColors: Map<string, string>): string {
  const header = 'create_date,content,is_done,priority,cell_color,done_date,description';
  const lines: string[] = [header];

  // Create a map of date to cell color for quick lookup
  const colorMap = new Map(cellColors);

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
    colorMap.delete(schedule.create_date);
  });

  // Export dates with only colors (no schedules)
  colorMap.forEach((color, date) => {
    const line = [date, '', 'false', '0', color, '', ''].join(',');
    lines.push(line);
  });

  return lines.join('\n');
}

/**
 * Convert CSV data to schedules and cell colors
 */
export function csvToSchedules(csvString: string): { schedules: Schedule[], cellColors: CellColorData[] } {
  const lines = csvString.split('\n').filter(line => line.trim());

  if (lines.length === 0) {
    throw new Error('CSV 文件为空');
  }

  const header = lines[0].split(',');

  // New format (create_date, done_date, description)
  const expectedHeaderNew = ['create_date', 'content', 'is_done', 'priority', 'cell_color', 'done_date', 'description'];
  // Old formats
  const expectedHeaderV3 = ['date', 'content', 'is_done', 'priority', 'cell_color', 'created_at', 'done_time', 'description'];
  const expectedHeaderV2 = ['date', 'content', 'is_done', 'priority', 'cell_color', 'created_at', 'done_time'];
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

  for (let i = 1; i < lines.length; i++) {
    const fields = parseCSVLine(lines[i]);
    const expectedFieldCount = isNewFormat ? 7 : (isV3Format ? 8 : (isV2Format ? 7 : 6));

    if (fields.length < expectedFieldCount) {
      console.warn(`跳过格式错误的第 ${i + 1} 行`);
      continue;
    }

    let createDate: string, content: string, isDoneStr: string, priorityStr: string, cellColor: string, doneDate: string, description: string;

    if (isNewFormat) {
      [createDate, content, isDoneStr, priorityStr, cellColor, doneDate, description] = fields.map(cleanFieldValue);
    } else if (isV3Format) {
      [createDate, content, isDoneStr, priorityStr, cellColor, , doneDate, description] = fields.map(cleanFieldValue);
    } else if (isV2Format) {
      [createDate, content, isDoneStr, priorityStr, cellColor, , doneDate] = fields.map(cleanFieldValue);
      description = '';
    } else {
      [createDate, content, isDoneStr, priorityStr, cellColor] = fields.map(cleanFieldValue);
      doneDate = '';
      description = '';
    }

    // Normalize date format
    if (createDate.includes('/')) {
      const parts = createDate.split('/');
      if (parts.length === 3) {
        createDate = `${parts[0]}-${parts[1].padStart(2, '0')}-${parts[2].padStart(2, '0')}`;
      }
    }

    if (cellColor) {
      seenColors.set(createDate, cellColor);
    }

    if (content.trim()) {
      if (doneDate && doneDate.includes('/')) {
        const parts = doneDate.split(' ')[0].split('/');
        if (parts.length === 3) {
          doneDate = `${parts[0]}-${parts[1].padStart(2, '0')}-${parts[2].padStart(2, '0')}`;
        }
      } else if (doneDate && doneDate.includes('T')) {
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

  seenColors.forEach((color, date) => {
    cellColors.push({ date, color });
  });

  return { schedules, cellColors };
}
