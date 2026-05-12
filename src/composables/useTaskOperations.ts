// composables/useTaskOperations.ts
import { ref } from 'vue';
import type { MainTask, Schedule } from '../api/database';
import {
  loadMainTasks,
  searchMainTasks,
  saveMainTask,
  updateMainTaskContent,
  updateMainTaskCreateDate,
  toggleMainTaskStatus,
  deleteMainTask,
  loadSchedulesByFatherTask,
  saveSubTask,
  deleteSchedule,
  toggleScheduleStatus,
  updateScheduleContent,
  updateScheduleDescription,
  updateScheduleDate,
  updateMainTaskDescription,
  updateMainTaskDoneDate,
} from '../api/database';

/**
 * 任务操作组合式函数
 *
 * 职责：
 * - 管理主任务和子任务的数据状态
 * - 提供任务的 CRUD 操作
 * - 处理搜索功能
 *
 * 优势：
 * - 代码集中，易于维护
 * - 可被多个组件复用
 * - 与 UI 逻辑分离
 */
export function useTaskOperations() {
  const tasks = ref<MainTask[]>([]);
  const currentTask = ref<MainTask | null>(null);
  const subTasks = ref<Schedule[]>([]);
  const searchKeyword = ref('');

  // ============ 任务加载 ============

  /**
   * 加载主任务列表或搜索结果
   */
  async function loadTasks() {
    try {
      if (searchKeyword.value.trim()) {
        tasks.value = await searchMainTasks(searchKeyword.value);
      } else {
        tasks.value = await loadMainTasks();
      }

      // 保持当前选中的任务同步
      if (currentTask.value?.id) {
        const updated = tasks.value.find(t => t.id === currentTask.value!.id);
        if (updated) {
          currentTask.value = updated;
        } else {
          // 选中的任务已被删除
          currentTask.value = null;
          subTasks.value = [];
        }
      }
    } catch (error) {
      console.error('Failed to load tasks:', error);
      tasks.value = [];
    }
  }

  /**
   * 加载子任务列表
   */
  async function loadSubTasks() {
    if (!currentTask.value?.id) {
      subTasks.value = [];
      return;
    }

    try {
      subTasks.value = await loadSchedulesByFatherTask(currentTask.value.id);
    } catch (error) {
      console.error('Failed to load sub-tasks:', error);
      subTasks.value = [];
    }
  }

  /**
   * 选择任务并加载其子任务
   */
  async function selectTask(task: MainTask) {
    currentTask.value = task;
    await loadSubTasks();
  }

  // ============ 主任务操作 ============

  /**
   * 添加新主任务
   */
  async function handleAddTask(content: string) {
    const trimmed = content.trim();
    if (!trimmed) return;

    try {
      const newTaskId = await saveMainTask(trimmed);
      await loadTasks();

      if (newTaskId) {
        const newTask = tasks.value.find(t => t.id === newTaskId);
        if (newTask) {
          await selectTask(newTask);
        }
      }
    } catch (error) {
      console.error('Failed to add task:', error);
      throw error;
    }
  }

  /**
   * 更新主任务内容
   */
  async function handleUpdateTask(task: MainTask, newContent: string) {
    const trimmed = newContent.trim();
    if (!trimmed) return;

    try {
      await updateMainTaskContent(task.id!, trimmed);
      task.content = trimmed;
    } catch (error) {
      console.error('Failed to update task:', error);
      throw error;
    }
  }

  /**
   * 更新主任务创建日期
   */
  async function handleUpdateTaskDate(task: MainTask, newDate: string) {
    try {
      await updateMainTaskCreateDate(task.id!, newDate);
      task.create_date = newDate;
    } catch (error) {
      console.error('Failed to update task date:', error);
      throw error;
    }
  }

  /**
   * 切换主任务完成状态
   */
  async function handleToggleDone(task: MainTask) {
    if (!task.id) return;

    try {
      await toggleMainTaskStatus(task.id, !task.is_done);
      task.is_done = !task.is_done;
    } catch (error) {
      console.error('Failed to toggle task:', error);
      throw error;
    }
  }

  /**
   * 删除主任务
   */
  async function handleDeleteTask(taskId: number) {
    try {
      await deleteMainTask(taskId);
      await loadTasks();
    } catch (error) {
      console.error('Failed to delete task:', error);
      throw error;
    }
  }

  /**
   * 更新主任务描述
   */
  async function handleUpdateTaskDescription(taskId: number, description: string) {
    try {
      await updateMainTaskDescription(taskId, description);
    } catch (error) {
      console.error('Failed to update task description:', error);
      throw error;
    }
  }

  /**
   * 更新主任务完成日期
   */
  async function handleUpdateTaskDoneDate(taskId: number, doneDate: string) {
    try {
      await updateMainTaskDoneDate(taskId, doneDate);
    } catch (error) {
      console.error('Failed to update task done date:', error);
      throw error;
    }
  }

  // ============ 子任务操作 ============

  /**
   * 添加新子任务
   */
  async function handleAddSubTask(content: string) {
    if (!currentTask.value?.id) return;

    const trimmed = content.trim();
    if (!trimmed) return;

    try {
      await saveSubTask(trimmed, currentTask.value.id);
      await loadSubTasks();
    } catch (error) {
      console.error('Failed to add sub-task:', error);
      throw error;
    }
  }

  /**
   * 更新子任务内容
   */
  async function handleUpdateSubTaskContent(subTask: Schedule, newContent: string) {
    const trimmed = newContent.trim();
    if (!trimmed) return;

    try {
      await updateScheduleContent(subTask.id!, trimmed);
      subTask.content = trimmed;
    } catch (error) {
      console.error('Failed to update sub-task:', error);
      throw error;
    }
  }

  /**
   * 更新子任务日期
   */
  async function handleUpdateSubTaskDate(subTask: Schedule, newDate: string) {
    try {
      await updateScheduleDate(subTask.id!, 'create_date', newDate);
      subTask.create_date = newDate;
    } catch (error) {
      console.error('Failed to update sub-task date:', error);
      throw error;
    }
  }

  /**
   * 切换子任务完成状态
   */
  async function handleToggleSubTaskDone(subTask: Schedule) {
    if (!subTask.id) return;

    try {
      await toggleScheduleStatus(subTask.id, !subTask.is_done);
      subTask.is_done = !subTask.is_done;
    } catch (error) {
      console.error('Failed to toggle sub-task:', error);
      throw error;
    }
  }

  /**
   * 删除子任务
   */
  async function handleDeleteSubTask(subTaskId: number) {
    try {
      await deleteSchedule(subTaskId);
      await loadSubTasks();
    } catch (error) {
      console.error('Failed to delete sub-task:', error);
      throw error;
    }
  }

  /**
   * 更新子任务描述
   */
  async function handleUpdateSubTaskDescription(subTaskId: number, description: string) {
    try {
      await updateScheduleDescription(subTaskId, description);
    } catch (error) {
      console.error('Failed to update sub-task description:', error);
      throw error;
    }
  }

  /**
   * 返回所有公开接口
   */
  return {
    // 状态
    tasks,
    currentTask,
    subTasks,
    searchKeyword,

    // 任务加载
    loadTasks,
    loadSubTasks,
    selectTask,

    // 主任务操作
    handleAddTask,
    handleUpdateTask,
    handleUpdateTaskDate,
    handleToggleDone,
    handleDeleteTask,
    handleUpdateTaskDescription,
    handleUpdateTaskDoneDate,

    // 子任务操作
    handleAddSubTask,
    handleUpdateSubTaskContent,
    handleUpdateSubTaskDate,
    handleToggleSubTaskDone,
    handleDeleteSubTask,
    handleUpdateSubTaskDescription,
  };
}