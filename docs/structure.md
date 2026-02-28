# Chronos 项目结构文档

## 项目概述

Chronos 是一个基于 Tauri + Vue 3 构建的桌面日历应用程序。项目采用前后端分离架构，前端使用 Vue 3 + TypeScript + Tailwind CSS，后端使用 Rust (Tauri)。

---

## 目录结构

```
chronos/
├── index.html                 # 入口 HTML 文件
├── package.json               # npm 配置文件，定义项目依赖和脚本
├── package-lock.json          # npm 依赖锁定文件
├── tsconfig.json              # TypeScript 配置文件
├── tsconfig.node.json         # Node.js 环境的 TypeScript 配置
├── vite.config.ts             # Vite 构建工具配置
├── postcss.config.js          # PostCSS 配置
├── tailwind.config.js         # Tailwind CSS 配置
├── README.md                  # 项目说明文档
│
├── docs/                      # 项目文档目录
│   └── structure.md           # 项目结构说明文档
│
├── src/                       # 前端源代码目录
│   ├── main.ts                # 应用入口文件
│   ├── App.vue                # 根组件
│   ├── styles.css             # 全局样式文件
│   │
│   ├── components/            # Vue 组件目录
│   │   ├── CalendarCell.vue       # 日历单元格组件