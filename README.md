# Chronos - 桌面日历助手

一个基于 Vue 3 + TypeScript + Tauri 构建的精美桌面日历应用。

## 功能特性

- 📅 **日历显示** - 清晰的月历视图，支持日期导航
- 📝 **日程管理** - 轻点即可添加日程，支持标记完成、删除
- 🎨 **主题定制** - 浅色/深色模式切换，支持自定义主题颜色
- 🪟 **窗口控制** - 支持拖动移动、调整大小
- 💾 **数据持久化** - SQLite 本地数据库存储

## 项目结构

```
src/
├── components/          # Vue 组件
│   ├── CalendarCell.vue      # 日历格子（单击编辑日程）
│   ├── CalendarGrid.vue      # 日历网格（7x6格子布局）
│   ├── CalendarHeader.vue    # 日历头部（标题、日期切换、菜单）
│   ├── ColorPicker.vue       # 颜色选择器组件
│   ├── DropdownMenu.vue      # 下拉菜单（设置/退出）
│   ├── MiniCalendar.vue      # 小日历弹窗
│   ├── ResizeHandles.vue     # 窗口调整大小手柄
│   ├── ScheduleItem.vue      # 日程项（勾选/删除）
│   ├── SettingsPanel.vue     # 设置面板（皮肤设置）
│   ├── SliderControl.vue     # 滑块控制组件
│   └── ToastContainer.vue   #  Toast 通知组件
│
├── composables/         # Vue 组合式函数
│   ├── useDatabase.ts        # 数据库操作（SQLite）
│   ├── useSchedules.ts       # 日程管理逻辑
│   ├── useSettings.ts        # 设置管理（localStorage）
│   └── useToast.ts           #  Toast 通知系统
│
├── types/               # TypeScript 类型定义
│   └── index.ts              # 所有类型导出
│
├── utils/               # 工具函数
│   ├── date.ts               # 日期处理函数
│   ├── window.ts             # Tauri 窗口操作
│   └── color.ts              # 颜色处理函数
│
├── App.vue              # 主应用组件
├── main.ts              # 应用入口
└── styles.css           # 全局样式 + CSS变量
```

## 核心实现

### 1. 日历显示
- `utils/date.ts` - 生成日历天数数组，正确处理星期排列
- `components/CalendarGrid.vue` - 7x6 网格布局
- `components/CalendarCell.vue` - 单个格子，单击进入编辑模式

### 2. 日程管理
- `composables/useSchedules.ts` - 日程增删改查逻辑（含缓存机制）
- `composables/useDatabase.ts` - SQLite 数据库操作
- `components/ScheduleItem.vue` - 日程项展示（实色文字）

### 3. 皮肤设置
- `composables/useSettings.ts` - 设置读写、CSS变量应用
- `components/SettingsPanel.vue` - 设置界面（固定白底黑字）
- `types/index.ts` - 设置类型定义
- `utils/color.ts` - 颜色处理工具函数

### 4. 窗口控制
- `utils/window.ts` - 拖动、调整大小
- `components/ResizeHandles.vue` - 边缘拖动手柄
- `src-tauri/src/lib.rs` - 透明窗口、置底、无边框配置

### 5. 错误处理
- `composables/useToast.ts` - 全局 Toast 通知系统
- `components/ToastContainer.vue` - 通知容器组件
- 用户操作反馈（成功/失败提示）

## CSS 变量系统

```css
/* 主题色 */
--primary              # 主题色
--primary-light        # 主题色浅色背景

/* 文字色（实色） */
--text-primary         # 主文字
--text-secondary       # 次要文字
--text-muted           # 淡化文字

/* 背景透明度 */
--glass-bg             # 窗口背景（带透明度）
--cell-bg              # 格子背景（带透明度）
--cell-bg-muted        # 非当月格子背景

/* 字号 */
--font-size-base       # 基础字号（默认14px）
--font-size-xs         # 小字号（基础 * 0.75）
```

## 数据存储

- **日程数据**: SQLite (`chronos.db`)
  - `schedules` 表: id, date, content, is_done, priority, created_at

- **设置数据**: localStorage
  - `chronos_settings`: 当前使用的设置
  - `chronos_settings_light`: 浅色模式设置
  - `chronos_settings_dark`: 深色模式设置

## 关键交互

1. **单击格子** → 进入编辑模式（多行文本框）
2. **点击外部** → 自动保存
3. **勾选日程** → 标记完成（实色勾选框）
4. **设置按钮** → 打开设置面板（固定配色）
5. **边缘拖动** → 调整窗口大小

## 开发

### 环境要求

- Node.js 18+
- Rust 1.70+
- Tauri CLI

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 构建发布

```bash
npm run tauri build
```

## 技术栈

- **前端框架**: Vue 3 + TypeScript
- **桌面框架**: Tauri 2.x
- **数据库**: SQLite (via @tauri-apps/plugin-sql)
- **日期处理**: dayjs
- **样式**: Tailwind CSS + 自定义 CSS 变量
