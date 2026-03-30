// Schedule types
export interface Schedule {
  id?: number;
  create_date: string; // 计划日期
  content: string;
  is_done: boolean;
  priority: number;
  cell_color?: string;
  done_date?: string; // 完成日期
  description?: string; // 描述/备注
  father_task?: number; // 关联的主任务 ID
}

export interface EditingCell {
  date: string;
  index: number;
}

export type ResizeDirection = 'East' | 'West' | 'South' | 'SouthEast';

// View mode types
export type ViewMode = 'todo' | 'done';

// Settings types
export type ThemeMode = 'light' | 'dark';
export type WeekStartsOn = 0 | 1;

export type BorderStyle = 'solid' | 'dashed' | 'dotted' | 'dash-dot' | 'dash-dot-dot';

// 将配置拆分为公用和模式专有，并在 AppSettings 中合并
export interface CommonSettings {
  font_family: string;
  font_size: number;
  font_weight: number;
  cell_gap: number;
  cell_border_width: number;
  cell_border_style: BorderStyle;
  cell_border_dash_interval: number; // 虚线/点线的间隔（0-20px）
  week_starts_on: WeekStartsOn;
  display_mode: 'month' | 'floating_weeks'; // 显示模式：整月或浮动周
  floating_weeks_count: number; // 浮动周数量（默认3）
  autostart: boolean; // 开机自启动
  header_cell_style: boolean; // 标题栏应用单元格风格
  hide_weekends: boolean; // 隐藏周末
  desc_dialog_width: number; // 描述窗口宽度占比 (0-100)
  desc_dialog_height: number; // 描述窗口高度占比 (0-100)
}

export interface ModeSettings {
  theme_mode: ThemeMode;
  primary_color: string;
  text_color: string;
  bg_color: string;
  bg_opacity: number;
  cell_color: string; // 单元格背景颜色
  cell_opacity: number;
  cell_border_color: string;
  enable_blur: boolean; // 新增：毛玻璃效果开关
}

export interface AppSettings extends CommonSettings, ModeSettings {
  id?: number;
}

// 默认公用配置
export const defaultCommonSettings: CommonSettings = {
  font_family: 'system-ui, -apple-system, sans-serif',
  font_size: 14,
  font_weight: 400,
  cell_gap: 4,
  cell_border_width: 1,
  cell_border_style: 'solid',
  cell_border_dash_interval: 4,
  week_starts_on: 1,
  display_mode: 'month',
  floating_weeks_count: 3,
  autostart: false,
  header_cell_style: false,
  hide_weekends: false,
  desc_dialog_width: 40, // 默认宽度占比 40%
  desc_dialog_height: 70, // 默认高度占比 70%
};

// 默认浅色/深色模式配置
export const defaultLightSettings: Omit<AppSettings, 'id'> = {
  ...defaultCommonSettings,
  theme_mode: 'light',
  primary_color: '#3b82f6',
  text_color: '#1f2937',
  bg_color: '#ffffff',
  bg_opacity: 15,    // 降低窗口背景透明度，让原生毛玻璃更清透
  cell_color: '#ffffff', // 默认单元格背景颜色
  cell_opacity: 25,  // 降低单元格透明度
  cell_border_color: '#d1d5db',
  enable_blur: true, // 原生毛玻璃开关
};

export const defaultDarkSettings: Omit<AppSettings, 'id'> = {
  ...defaultCommonSettings,
  theme_mode: 'dark',
  primary_color: '#60a5fa',
  text_color: '#f9fafb',
  bg_color: '#000000',
  bg_opacity: 20,    // 降低窗口背景透明度，让原生毛玻璃更清透
  cell_color: '#1f2937', // 默认单元格背景颜色（深色模式）
  cell_opacity: 20,  // 调整单元格透明度
  cell_border_color: '#4b5563',
  enable_blur: true, // 原生毛玻璃开关
};

// 从 AppSettings 提取公用配置部分
export function extractCommonParts(settings: AppSettings): CommonSettings {
  return {
    font_family: settings.font_family,
    font_size: settings.font_size,
    font_weight: settings.font_weight,
    cell_gap: settings.cell_gap,
    cell_border_width: settings.cell_border_width,
    cell_border_style: settings.cell_border_style,
    cell_border_dash_interval: settings.cell_border_dash_interval,
    week_starts_on: settings.week_starts_on,
    display_mode: settings.display_mode,
    floating_weeks_count: settings.floating_weeks_count,
    autostart: settings.autostart,
    header_cell_style: settings.header_cell_style,
    hide_weekends: settings.hide_weekends,
    desc_dialog_width: settings.desc_dialog_width,
    desc_dialog_height: settings.desc_dialog_height,
  };
}

// 批量添加任务配置
export interface BatchTaskConfig {
  startDate: string;    // YYYY-MM-DD
  endDate: string;      // YYYY-MM-DD
  cycleType: 'day' | 'week' | 'month';
  cycleCount: number;   // 每隔几天/周/月
  title: string;
  description?: string;
}
