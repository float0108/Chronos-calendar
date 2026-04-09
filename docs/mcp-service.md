# Chronos MCP 服务

Chronos 提供 Model Context Protocol (MCP) 服务，允许 AI 助手直接操作日历数据。

## 服务配置

服务默认在端口 3000 启动，使用 SSE (Server-Sent Events) 传输。

### Claude Code 配置

在项目根目录的 `.mcp.json` 中配置：

```json
{
  "mcpServers": {
    "chronos": {
      "type": "sse",
      "url": "http://127.0.0.1:3000/sse"
    }
  }
}
```

### 服务管理

- 应用启动时自动启动 MCP 服务
- 默认端口：3000

## 工具

### 日程工具

| 工具名 | 描述 | 返回值 |
|--------|------|--------|
| `add_schedule` | 添加单个日程 | 新创建的日程 ID |
| `add_schedules` | 批量添加日程 | 新创建的日程 ID 列表 |
| `update_schedule` | 完整更新日程（需提供所有字段） | 是否成功 |
| `patch_schedule` | 部分更新日程（只修改指定字段） | 是否成功 |
| `delete_schedule` | 删除单个日程 | 是否成功 |
| `delete_schedules` | 批量删除日程 | 删除数量 |
| `get_schedule` | 根据 ID 获取日程详情 | 日程对象 |
| `search_schedules` | 搜索日程（按内容或描述） | 日程列表 |
| `list_schedules` | 获取所有日程列表 | 日程列表 |
| `get_schedules_by_date` | 获取指定日期的日程 | 日程列表 |

### 主任务工具

| 工具名 | 描述 | 返回值 |
|--------|------|--------|
| `add_main_task` | 添加单个主任务 | 新创建的任务 ID |
| `add_main_tasks` | 批量添加主任务 | 新创建的任务 ID 列表 |
| `update_main_task` | 完整更新主任务（需提供所有字段） | 是否成功 |
| `patch_main_task` | 部分更新主任务（只修改指定字段） | 是否成功 |
| `delete_main_task` | 删除单个主任务 | 是否成功 |
| `delete_main_tasks` | 批量删除主任务 | 删除数量 |
| `get_main_task` | 根据 ID 获取主任务详情 | 任务对象 |
| `search_main_tasks` | 搜索主任务（按内容或描述） | 任务列表 |
| `list_main_tasks` | 获取所有主任务列表 | 任务列表 |

## 资源

MCP 服务提供以下资源，可直接读取：

| URI | 描述 |
|-----|------|
| `chronos://summary/today` | 今日概览（总日程数、已完成数、待办数） |
| `chronos://summary/week` | 本周概览 |
| `chronos://schedules/pending` | 待办日程列表 |
| `chronos://main_tasks/active` | 进行中的主任务 |

## 数据结构

### 日程项 (ScheduleItem)

| 字段 | 类型 | 必填 | 描述 |
|------|------|------|------|
| `content` | string | ✓ | 日程内容 |
| `create_date` | string | | 计划日期 (YYYY-MM-DD)，默认今天 |
| `is_done` | boolean | | 是否已完成，默认 false |
| `priority` | integer | | 优先级 (0-9)，默认 0 |
| `description` | string | | 详细描述 |
| `done_date` | string | | 完成日期 |
| `father_task` | integer | | 关联的主任务 ID |

### 主任务项 (MainTaskItem)

| 字段 | 类型 | 必填 | 描述 |
|------|------|------|------|
| `content` | string | ✓ | 任务内容 |
| `description` | string | | 详细描述 |
| `is_done` | boolean | | 是否已完成，默认 false |
| `priority` | integer | | 优先级 (0-9)，默认 0 |
| `create_date` | string | | 创建日期，默认今天 |
| `done_date` | string | | 完成日期 |

## 使用示例

### 添加日程

```
明天下午3点开会
```

AI 会调用 `add_schedule` 工具：
```json
{
  "content": "下午3点开会",
  "create_date": "2026-04-10"
}
```

### 标记完成

```
把"下午3点开会"标记为已完成
```

AI 会先搜索该日程，然后调用 `patch_schedule` 更新状态。

### 查看待办

```
今天有什么待办事项？
```

AI 会读取 `chronos://summary/today` 资源或调用 `get_schedules_by_date` 工具。

### 添加主任务

```
创建一个主任务：完成项目报告
```

AI 会调用 `add_main_task` 工具：
```json
{
  "content": "完成项目报告",
  "priority": 5
}
```

## 技术实现

- 基于 [rmcp](https://github.com/anthropics/rmcp) 库实现
- 数据存储：SQLite
- 传输方式：SSE (Server-Sent Events)
- 数据变更会自动通知应用窗口刷新
