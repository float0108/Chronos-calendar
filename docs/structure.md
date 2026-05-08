# Chronos 项目结构文档

## 项目概述

Chronos 是一个基于 Tauri v2 + Vue 3 构建的跨平台桌面日历应用程序。

### 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | Vue 3.5 + TypeScript |
| UI 样式 | Tailwind CSS |
| 后端框架 | Tauri v2 (Rust) |
| 数据库 | SQLite |
| 构建工具 | Vite |

### 项目目录

```
chronos/
├── docs/                       # 项目文档
│   ├── structure.md           # 项目结构文档
│   ├── features.md            # 功能说明书
│   └── mcp-service.md         # MCP 服务文档
│
├── src/                       # 前端源代码
│   ├── main.ts               # 应用入口
│   ├── App.vue               # 主窗口根组件
│   │
│   ├── api/                  # API 层
│   │   └── database.ts       # 数据库操作 API (Tauri invoke 调用)
│   │
│   ├── components/           # 通用组件
│   │   ├── calendar/         # 日历相关组件
│   │   │   ├── CalendarGrid.vue    # 日历网格
│   │   │   ├── CalendarCell.vue    # 日历单元格
│   │   │   ├── CalendarHeader.vue  # 日历头部
│   │   │   └── MiniCalendar.vue    # 迷你日历
│   │   ├── dialogs/          # 对话框组件
│   │   │   ├── BatchTaskDialog.vue   # 批量添加任务
│   │   │   ├── DescriptionDialog.vue  # 描述编辑
│   │   │   └── ImportDialog.vue      # 导入对话框
│   │   ├── ui/               # UI 组件
│   │   │   ├── ColorPicker.vue      # 颜色选择器
│   │   │   ├── DropdownMenu.vue     # 下拉菜单
│   │   │   ├── ResizeHandles.vue    # 窗口调整手柄
│   │   │   ├── ScheduleTooltip.vue  # 日程提示
│   │   │   ├── SliderControl.vue    # 滑块控件
│   │   │   └── ToastContainer.vue   # 吐司通知
│   │   ├── ListItem.vue           # 列表项组件
│   │   ├── ScheduleEditor.vue      # 日程编辑器
│   │   └── WindowTitleBar.vue      # 窗口标题栏
│   │
│   ├── composables/           # 组合式函数
│   │   ├── useDatabase.ts     # 数据库操作封装
│   │   ├── useSchedules.ts    # 日程业务逻辑
│   │   ├── useSettings.ts     # 设置管理
│   │   ├── useTaskWindow.ts   # 任务窗口逻辑
│   │   ├── useScheduleUndo.ts # 日程撤销/重做
│   │   ├── useContextMenu.ts  # 右键菜单
│   │   ├── useToast.ts        # 吐司通知
│   │   ├── useImport.ts       # 导入功能
│   │   ├── useFonts.ts        # 字体管理
│   │   ├── useSystemTheme.ts  # 系统主题
│   │   ├── useTaskTheme.ts    # 任务主题
│   │   ├── useUndoHistory.ts  # 撤销历史
│   │   └── useEditHistory.ts  # 编辑历史
│   │
│   ├── windows/              # 子窗口组件
│   │   ├── BoardWindow.vue    # 看板窗口
│   │   ├── TodoWindow.vue     # 待办窗口
│   │   ├── TaskWindow.vue     # 任务详情窗口
│   │   ├── NoteWindow.vue     # 笔记窗口
│   │   ├── SearchWindow.vue   # 搜索窗口
│   │   ├── SettingsPanel.vue  # 设置面板
│   │   ├── BasicSettings.vue  # 基础设置
│   │   ├── PageSettings.vue   # 页面设置
│   │   ├── CommonSettings.vue  # 通用设置
│   │   ├── ModeSettings.vue   # 模式设置
│   │   └── TaskWindowTitleBar.vue  # 任务窗口标题栏
│   │
│   ├── types/               # 类型定义
│   │   └── index.ts          # 类型导出
│   │
│   ├── utils/               # 工具函数
│   │   ├── color.ts          # 颜色处理
│   │   ├── date.ts           # 日期处理
│   │   ├── window.ts         # 窗口操作
│   │   ├── export.ts         # 导出功能
│   │   └── csv/              # CSV 处理
│   │       ├── encoding.ts   # 编码处理
│   │       ├── parser.ts     # CSV 解析
│   │       └── generator.ts   # CSV 生成
│   │
│   ├── constants/            # 常量定义
│   │   └── index.ts
│   │
│   └── *-entry.ts          # 各窗口入口文件
│       ├── board-entry.ts    # 看板窗口入口
│       ├── todo-entry.ts     # 待办窗口入口
│       ├── task-entry.ts     # 任务窗口入口
│       ├── note-entry.ts     # 笔记窗口入口
│       ├── search-entry.ts   # 搜索窗口入口
│       └── settings-entry.ts  # 设置窗口入口
│
├── src-tauri/               # Rust 后端
│   ├── src/
│   │   ├── main.rs          # 程序入口
│   │   ├── lib.rs           # 库入口
│   │   ├── windows.rs       # 窗口管理
│   │   ├── tray.rs          # 系统托盘
│   │   │
│   │   ├── commands/        # Tauri 命令
│   │   │   ├── mod.rs       # 命令模块
│   │   │   ├── database.rs  # 数据库操作命令
│   │   │   ├── app.rs       # 应用命令
│   │   │   └── window.rs    # 窗口命令
│   │   │
│   │   ├── db/              # 数据库层
│   │   │   ├── mod.rs       # 模块导出
│   │   │   ├── manager.rs   # 数据库管理器
│   │   │   ├── schedule.rs  # 日程表操作
│   │   │   ├── main_task.rs # 主任务操作
│   │   │   ├── note.rs      # 笔记操作
│   │   │   ├── cell.rs      # 单元格操作
│   │   │   ├── backup.rs    # 备份导出
│   │   │   ├── stats.rs     # 统计
│   │   │   └── types.rs    # 数据库类型
│   │   │
│   │   └── mcp/             # MCP 服务
│   │       ├── mod.rs       # 模块导出
│   │       ├── server.rs    # MCP 服务器
│   │       ├── service.rs    # MCP 服务
│   │       ├── tools.rs     # MCP 工具
│   │       └── types.rs     # MCP 类型
│   │
│   └── Cargo.toml           # Rust 依赖配置
│
└── package.json             # npm 依赖配置
```

## 数据流架构

### 前端调用链

```
Vue 组件 → Composables → API (invoke) → Rust Commands → Database
```

### 跨窗口数据同步

```
┌─────────────────────────────────────────────────────────┐
│  前端窗口                                                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │ App      │  │ Todo     │  │ Board    │  │ Task     │ │
│  │ Window   │  │ Window   │  │ Window   │  │ Window   │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘ │
│       │             │             │             │        │
│       └─────────────┴──────┬──────┴─────────────┘        │
│                            │                             │
│                    listen('schedule-changed')            │
│                   ← 收到事件，重新加载数据                │
└────────────────────────────┼────────────────────────────┘
                             │
                      ┌──────▼──────┐
                      │  Rust 后端   │
                      │   (SQLite)   │
                      └──────┬──────┘
                             │
            数据库修改操作后 → emit('schedule-changed')
```

### 事件传递

| 事件名 | 触发者 | 监听者 | 作用 |
|--------|--------|--------|------|
| `schedule-changed` | Rust 后端 (数据库修改后) | App, Todo, Board, Task 窗口 | 通知数据已变更，各窗口刷新 |

## 窗口类型

### 主窗口 (App.vue)
- 日历主视图，显示月历网格
- 管理所有其他窗口的显示/隐藏

### 子窗口

| 窗口 | 入口文件 | 功能 |
|------|----------|------|
| TodoWindow | todo-entry.ts | 待办事项管理 |
| BoardWindow | board-entry.ts | 主任务看板视图 |
| TaskWindow | task-entry.ts | 任务详情和子任务 |
| NoteWindow | note-entry.ts | 笔记管理 |
| SearchWindow | search-entry.ts | 日程搜索 |
| SettingsPanel | settings-entry.ts | 设置面板 |

## 数据库表结构

### schedules (日程表)
| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER | 主键 |
| create_date | TEXT | 创建日期 |
| content | TEXT | 内容 |
| is_done | INTEGER | 是否完成 (0/1) |
| priority | INTEGER | 优先级 |
| done_date | TEXT | 完成日期 |
| description | TEXT | 描述/备注 |
| father_task | INTEGER | 关联主任务 ID |

### main_tasks (主任务表)
| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER | 主键 |
| content | TEXT | 内容 |
| description | TEXT | 描述 |
| is_done | INTEGER | 是否完成 |
| priority | INTEGER | 优先级 |
| create_date | TEXT | 创建日期 |
| done_date | TEXT | 完成日期 |

### notes (笔记表)
| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER | 主键 |
| title | TEXT | 标题 |
| content | TEXT | 内容 |
| create_date | TEXT | 创建日期 |

### cell_metadata (单元格元数据)
| 字段 | 类型 | 说明 |
|------|------|------|
| date | TEXT | 日期 (主键) |
| cell_color | TEXT | 单元格颜色 |
