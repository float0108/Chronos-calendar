import { getCurrentWindow } from '@tauri-apps/api/window';
import { exit } from '@tauri-apps/plugin-process';
import type { ResizeDirection } from '../types';

let isWindowLocked = false;

export function setWindowLocked(locked: boolean) {
  isWindowLocked = locked;
}

export async function startWindowDrag(event: MouseEvent): Promise<void> {
  if (isWindowLocked) return;
  if ((event.target as HTMLElement).closest('.no-drag')) return;

  try {
    const window = getCurrentWindow();
    await window.startDragging();
  } catch (error) {
    console.error('Drag failed:', error);
  }
}

export async function startWindowResize(direction: ResizeDirection): Promise<void> {
  if (isWindowLocked) return;

  try {
    const window = getCurrentWindow();
    await window.startResizeDragging(direction);
  } catch (error) {
    console.error('Resize failed:', error);
  }
}

export async function closeWindow(): Promise<void> {
  try {
    const window = getCurrentWindow();
    await window.close();
  } catch (error) {
    console.error('Close failed:', error);
  }
}

export async function hideWindow(): Promise<void> {
  try {
    const window = getCurrentWindow();
    await window.hide();
  } catch (error) {
    console.error('Hide failed:', error);
  }
}

export async function exitApp(): Promise<void> {
  try {
    await exit(0);
  } catch (error) {
    console.error('Exit failed:', error);
  }
}
