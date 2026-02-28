export interface Schedule {
  id?: number;
  date: string;
  content: string;
  is_done: boolean;
  priority: number;
  created_at?: string;
}

export interface EditingCell {
  date: string;
  index: number;
}
