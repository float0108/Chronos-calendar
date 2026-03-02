# Chronos - 桌面日历助手

一个基于 Vue 3 + TypeScript + Tauri 构建的精美桌面日历应用。

## 功能特性

- 📅 **日历显示** - 清晰的月历视图，支持日期导航和快速跳转
- 📝 **日程管理** - 单击即可添加日程，支持标记完成、删除、多行编辑
- 🎨 **主题定制** - 浅色/深色模式切换，支持自定义主题颜色和透明度
- 🎯 **格子着色** - 右键菜单为日历格子添加背景色标记
- 🪟 **窗口控制** - 支持拖动移动、调整大小、窗口锁定
- ✨ **毛玻璃效果** - Windows Mica / macOS Vibrancy 视觉效果
- 📌 **桌面置底** - 始终显示在桌面最底层，不干扰正常工作
- 💾 **数据持久化** - SQLite 本地数据库存储，窗口状态自动保存
- 🔔 **Toast 通知** - 友好的操作反馈提示

## 项目结构

```
chronos/
├── index.html                 # 应用入口 HTML
├── package.json               # 项目依赖和脚本配置
├── vite.config.ts             # Vite 构建配置
├── tsconfig.json              # TypeScript 配置
├── tailwind.config.js         # Tailwind CSS 配置
├── postcss.config.js          # PostCSS 配置
│
├── src/                       # 前端源代码
│   ├── main.ts                # 应用入口文件
│   ├── App.vue                # 根组件
│   ├── styles.css             # 全局样式和 CSS 变量
│   │
│   ├── components/            # Vue 组件
│   │   ├── CalendarCell.vue       # 日历格子（单击编辑日程）
│   │   ├── CalendarGrid.vue       # 日历网格（7x6 格子布局）
│   │   ├── CalendarHeader.vue     # 日历头部（标题、日期切换、菜单）
│   │   ├── ColorPicker.vue        # 颜色选择器组件
│   │   ├── DropdownMenu.vue       # 下拉菜单（设置/退出）
│   │   ├── MiniCalendar.vue       # 小日历弹窗（快速跳转日期）
│   │   ├── ResizeHandles.vue      # 窗口调整大小手柄
│   │   ├── ScheduleItem.vue       # 日程项（勾选/删除）
│   │   ├── SettingsPanel.vue      # 设置面板（皮肤设置）
│   │   ├── SliderControl.vue      # 滑块控制组件
│   │   └── ToastContainer.vue     # Toast 通知容器
│   │
│   ├── composables/           # Vue 组合式函数
│   │   ├── useDatabase.ts         # 数据库操作（SQLite）
│   │   ├── useSchedules.ts        # 日程管理逻辑（增删改查）
│   │   ├── useSettings.ts         # 设置管理（localStorage）
│   │   └── useToast.ts            # Toast 通知系统
│   │
│   ├── types/                 # TypeScript 类型定义
│   │   ├── index.ts               # 类型导出入口
│   │   └── schedule.ts            # 日程相关类型
│   │
│   └── utils/                 # 工具函数
│       ├── date.ts                # 日期处理函数
│       ├── window.ts              # Tauri 窗口操作
│       └── color.ts               # 颜色处理函数
│
├── src-tauri/                 # Tauri 后端代码
│   ├── src/
│   │   ├── main.rs                # Rust 程序入口
│   │   └── lib.rs                 # Tauri 应用配置（窗口效果、插件）
│   ├── Cargo.toml                 # Rust 依赖配置
│   ├── tauri.conf.json            # Tauri 应用配置
│   ├── capabilities/              # Tauri 权限配置
│   └── icons/                     # 应用图标资源
│
└── docs/                      # 项目文档
    └── structure.md               # 项目结构详细说明
```

## 核心实现

### 1. 日历显示
- **日期计算**: `utils/date.ts` - 生成日历天数数组，支持周一/周日作为起始日
- **网格布局**: `components/CalendarGrid.vue` - 7x6 响应式网格布局
- **单元格组件**: `components/CalendarCell.vue` - 单击进入编辑模式，右键菜单着色
- **快速跳转**: `components/MiniCalendar.vue` - 小日历弹窗快速选择日期

### 2. 日程管理
- **数据层**: `composables/useDatabase.ts` - SQLite 数据库初始化和查询
- **业务逻辑**: `composables/useSchedules.ts` - 日程增删改查、缓存优化
- **展示组件**: `components/ScheduleItem.vue` - 日程项展示、勾选完成、删除
- **数据类型**: `types/schedule.ts` - Schedule 接口定义

### 3. 主题系统
- **设置管理**: `composables/useSettings.ts` - 设置读写、CSS 变量动态应用
- **设置面板**: `components/SettingsPanel.vue` - 可视化配置界面
- **类型定义**: `types/index.ts` - AppSettings、ThemeMode 等类型
- **颜色工具**: `utils/color.ts` - 颜色转换、亮度计算

### 4. 窗口控制
- **窗口操作**: `utils/window.ts` - 拖动、调整大小、窗口锁定
- **调整手柄**: `components/ResizeHandles.vue` - 四边和四角拖动手柄
- **Tauri 配置**: `src-tauri/tauri.conf.json` - 透明窗口、置底、无边框
- **视觉效果**: `src-tauri/src/lib.rs` - Windows Mica / macOS Vibrancy 效果

### 5. 用户反馈
- **通知系统**: `composables/useToast.ts` - 全局 Toast 状态管理
- **容器组件**: `components/ToastContainer.vue` - 通知展示、自动消失
- **操作反馈**: 成功/失败提示，提升用户体验

## CSS 变量系统

应用使用 CSS 变量实现主题系统，支持动态切换：

```css
/* 主题色 */
--primary              /* 主题色 */
--primary-light        /* 主题色浅色背景 */

/* 文字色 */
--text-primary         /* 主文字颜色 */
--text-secondary       /* 次要文字颜色 */
--text-muted           /* 淡化文字颜色 */

/* 背景色（带透明度） */
--glass-bg             /* 窗口背景色 */
--cell-bg              /* 日历格子背景 */
--cell-bg-muted        /* 非当月格子背景 */

/* 字号系统 */
--font-size-base       /* 基础字号（默认 14px）*/
--font-size-xs         /* 小字号（基础 * 0.75）*/
--font-size-sm         /* 中字号（基础 * 0.875）*/
--font-size-lg         /* 大字号（基础 * 1.125）*/
```

## 数据存储

### SQLite 数据库 (`chronos.db`)

**schedules 表** - 日程数据
```sql
CREATE TABLE schedules (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  date TEXT NOT NULL,              -- 日期 (YYYY-MM-DD)
  content TEXT,                    -- 日程内容
  is_done INTEGER DEFAULT 0,       -- 是否完成 (0/1)
  priority INTEGER DEFAULT 0,      -- 优先级
  created_at TEXT DEFAULT CURRENT_TIMESTAMP
)
```

**cell_colors 表** - 格子颜色标记
```sql
CREATE TABLE cell_colors (
  date TEXT PRIMARY KEY,           -- 日期 (YYYY-MM-DD)
  color TEXT                       -- 背景颜色 (十六进制)
)
```

### localStorage

设置数据存储键：
- `chronos_settings` - 当前使用的设置
- `chronos_settings_light` - 浅色模式设置
- `chronos_settings_dark` - 深色模式设置

## 关键交互

| 操作 | 功能 | 说明 |
|------|------|------|
| **单击格子** | 编辑日程 | 进入编辑模式，支持多行文本 |
| **点击外部** | 自动保存 | 失焦自动保存日程内容 |
| **勾选日程** | 标记完成 | 实色勾选框，视觉反馈 |
| **右键格子** | 颜色标记 | 快捷菜单添加背景色 |
| **设置按钮** | 打开设置 | 主题、字号、透明度配置 |
| **锁定按钮** | 窗口锁定 | 禁止编辑和窗口操作 |
| **边缘拖动** | 调整大小 | 八个方向调整窗口尺寸 |
| **标题栏拖动** | 移动窗口 | 自由移动窗口位置 |

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl/Cmd + ,` | 打开设置面板 |
| `Esc` | 关闭弹窗/取消编辑 |

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

### 前端
- **框架**: Vue 3 (Composition API)
- **语言**: TypeScript
- **构建工具**: Vite 6
- **样式**: Tailwind CSS + CSS 变量
- **图标**: Lucide Vue Next
- **日期处理**: dayjs

### 后端
- **桌面框架**: Tauri 2.x
- **语言**: Rust
- **数据库**: SQLite (via @tauri-apps/plugin-sql)
- **窗口状态**: tauri-plugin-window-state

## 环境要求

- **Node.js**: >= 18.0.0
- **Rust**: >= 1.70.0
- **包管理器**: npm / pnpm / yarn
- **操作系统**: Windows 10+ / macOS 10.15+ / Linux

## 安装和运行

### 1. 克隆项目

```bash
git clone <repository-url>
cd chronos
```

### 2. 安装依赖

```bash
npm install
```

### 3. 开发模式

```bash
npm run tauri:dev
```

### 4. 构建发布

```bash
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

## 开发脚本

```bash
# 启动前端开发服务器
npm run dev

# 构建前端
npm run build

# 预览构建结果
npm run preview

# 启动 Tauri 开发模式
npm run tauri:dev

# 构建 Tauri 应用
npm run tauri:build
```

## 项目特点

### 🎯 性能优化
- **Rust 后端**: 使用 Tauri 替代 Electron，大幅减少内存占用
- **缓存机制**: 日程数据内存缓存，减少数据库查询
- **按需渲染**: 仅渲染可见区域的日历格子

### 🎨 用户体验
- **响应式设计**: 自适应窗口大小
- **流畅动画**: CSS 过渡动画
- **原生体验**: 跟随系统主题、原生窗口效果
- **桌面集成**: 置底显示、任务栏隐藏

### 💪 开发体验
- **类型安全**: TypeScript 全栈类型检查
- **组合式 API**: Vue 3 Composition API，逻辑复用性强
- **模块化**: 清晰的代码组织结构
- **热重载**: Vite HMR，开发效率高

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

## 作者

Chronos Team
