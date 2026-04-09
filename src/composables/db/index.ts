/**
 * 数据库模块统一导出
 *
 * 所有数据库操作通过 Tauri command 调用后端
 */

// 从 API 层重新导出所有函数和类型
export * from '../../api/database';
