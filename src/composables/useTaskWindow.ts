import { ref } from 'vue';
import {
  loadSchedulesByFatherTask,
  saveSubTask,
  deleteSchedule,
  toggleScheduleStatus,
  toggleMainTaskStatus,
  updateScheduleContent,
  updateScheduleDescription,
  updateScheduleDate,
  loadMainTasks,
  updateMainTaskContent,
  updateMainTaskDescription,
  updateMainTaskCreateDate,
  updateMainTaskDoneDate,
  type Schedule,
  type MainTask,
} from '../api/database';

export function useTaskWindow() {
  const tasks = ref<MainTask[]>([]);
  const currentTask = ref<MainTask | null>(null);
  const subTasks = ref<Schedule[]>([]);

  async function loadSubTasks() {
    if (!currentTask.value?.id) {
      subTasks.value = [];
      return;
    }
    subTasks.value = await loadSchedulesByFatherTask(currentTask.value.id);
  }

  async function addSubTask(content: string) {
    if (!currentTask.value?.id) return;
    await saveSubTask(content.trim(), currentTask.value.id);
    await loadSubTasks();
  }

  async function toggleSubTaskDone(subTask: Schedule) {
    if (!subTask.id) return;
    await toggleScheduleStatus(subTask.id, !subTask.is_done);
    await loadSubTasks();
  }

  async function removeSubTask(subTaskId: number) {
    await deleteSchedule(subTaskId);
    await loadSubTasks();
  }

  async function updateSubTaskContent(subTask: Schedule, newContent: string) {
    if (!subTask.id) return;
    const trimmed = newContent.trim();
    if (!trimmed) {
      await removeSubTask(subTask.id);
      return;
    }
    if (trimmed === subTask.content) return;
    await updateScheduleContent(subTask.id, trimmed);
    await loadSubTasks();
  }

  async function updateSubTaskDate(subTask: Schedule, newDate: string) {
    if (!subTask.id) return;
    await updateScheduleDate(subTask.id, 'create_date', newDate);
    await loadSubTasks();
  }

  async function saveSubTaskDetail(subTask: Schedule, description: string, createDate: string, doneDate: string) {
    if (!subTask.id) return;

    if (description !== (subTask.description || '')) {
      await updateScheduleDescription(subTask.id, description);
    }
    if (createDate !== (subTask.create_date || '')) {
      await updateScheduleDate(subTask.id, 'create_date', createDate);
    }
    if (doneDate !== (subTask.done_date || '')) {
      await updateScheduleDate(subTask.id, 'done_date', doneDate);
    }
    if (doneDate && !subTask.is_done) {
      await toggleScheduleStatus(subTask.id, true);
    }

    await loadSubTasks();
  }

  async function updateTaskTitle(taskId: number, newTitle: string) {
    const trimmed = newTitle.trim();
    if (!trimmed) return;
    await updateMainTaskContent(taskId, trimmed);
  }

  async function saveMainTaskDetail(
    taskId: number,
    description: string,
    createDate: string,
    doneDate: string,
    originalTask: MainTask
  ) {
    if (description !== (originalTask.description || '')) {
      await updateMainTaskDescription(taskId, description);
    }
    if (createDate !== (originalTask.create_date || '')) {
      await updateMainTaskCreateDate(taskId, createDate);
    }
    const currentDoneDate = originalTask.done_date || '';
    if (doneDate !== currentDoneDate) {
      await updateMainTaskDoneDate(taskId, doneDate || null);
    }
    if (doneDate && !originalTask.is_done) {
      await toggleMainTaskStatus(taskId, true);
    }

    tasks.value = await loadMainTasks();
    const updatedTask = tasks.value.find(t => t.id === taskId);
    if (updatedTask) {
      currentTask.value = updatedTask;
    }
  }

  async function selectTask(taskId: number) {
    tasks.value = await loadMainTasks();
    const task = tasks.value.find(t => t.id === taskId);
    if (task) {
      currentTask.value = task;
      await loadSubTasks();
    }
  }

  return {
    tasks,
    currentTask,
    subTasks,
    loadMainTasks: () => loadMainTasks(),
    loadSubTasks,
    addSubTask,
    toggleSubTaskDone,
    removeSubTask,
    updateSubTaskContent,
    updateSubTaskDate,
    saveSubTaskDetail,
    updateTaskTitle,
    saveMainTaskDetail,
    selectTask,
  };
}
