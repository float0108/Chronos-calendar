import { getDatabase } from './connection';

/**
 * 备忘录类型
 */
export interface Note {
  id?: number;
  title: string;
  content: string;
  create_date: string;
}

/**
 * 加载所有备忘录
 */
export async function loadNotes(): Promise<Note[]> {
  const db = getDatabase();
  if (!db) return [];

  try {
    const notes = await db.select<Note[]>(`
      SELECT * FROM notes
      ORDER BY id DESC
    `);
    return notes;
  } catch (error) {
    console.error('Failed to load notes:', error);
    return [];
  }
}

/**
 * 搜索备忘录（根据标题和内容）
 */
export async function searchNotes(keyword: string): Promise<Note[]> {
  const db = getDatabase();
  if (!db) return [];

  if (!keyword.trim()) {
    return loadNotes();
  }

  try {
    const searchTerm = `%${keyword.trim()}%`;
    const notes = await db.select<Note[]>(`
      SELECT * FROM notes
      WHERE title LIKE $1 OR content LIKE $1
      ORDER BY id DESC
    `, [searchTerm]);
    return notes;
  } catch (error) {
    console.error('Failed to search notes:', error);
    return [];
  }
}

/**
 * 获取单个备忘录
 */
export async function getNote(noteId: number): Promise<Note | null> {
  const db = getDatabase();
  if (!db) return null;

  try {
    const notes = await db.select<Note[]>(`
      SELECT * FROM notes WHERE id = $1
    `, [noteId]);
    return notes[0] || null;
  } catch (error) {
    console.error('Failed to get note:', error);
    return null;
  }
}

/**
 * 创建备忘录
 */
export async function createNote(title: string = '', content: string = ''): Promise<number | null> {
  const db = getDatabase();
  if (!db) return null;

  try {
    const now = new Date();
    const createDate = now.getFullYear() + '-' +
      String(now.getMonth() + 1).padStart(2, '0') + '-' +
      String(now.getDate()).padStart(2, '0');

    const result = await db.execute(
      'INSERT INTO notes (title, content, create_date) VALUES ($1, $2, $3)',
      [title.trim() || '无标题', content, createDate]
    );

    return result.lastInsertId ?? null;
  } catch (error) {
    console.error('Failed to create note:', error);
    throw error;
  }
}

/**
 * 更新备忘录
 */
export async function updateNote(noteId: number, title: string, content: string): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute(
      'UPDATE notes SET title = $1, content = $2 WHERE id = $3',
      [title.trim() || '无标题', content, noteId]
    );
  } catch (error) {
    console.error('Failed to update note:', error);
    throw error;
  }
}

/**
 * 更新备忘录标题
 */
export async function updateNoteTitle(noteId: number, title: string): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute(
      'UPDATE notes SET title = $1 WHERE id = $2',
      [title.trim() || '无标题', noteId]
    );
  } catch (error) {
    console.error('Failed to update note title:', error);
    throw error;
  }
}

/**
 * 更新备忘录内容
 */
export async function updateNoteContent(noteId: number, content: string): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute(
      'UPDATE notes SET content = $1 WHERE id = $2',
      [content, noteId]
    );
  } catch (error) {
    console.error('Failed to update note content:', error);
    throw error;
  }
}

/**
 * 删除备忘录
 */
export async function deleteNote(noteId: number): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute('DELETE FROM notes WHERE id = $1', [noteId]);
  } catch (error) {
    console.error('Failed to delete note:', error);
    throw error;
  }
}
