import { invoke } from '@tauri-apps/api/core';

export interface DailyActivity {
  date: string;       // "YYYY-MM-DD"
  message_count: number;
}

export interface LearningStats {
  total_messages: number;
  total_vocabulary: number;
  total_corrections: number;
  active_days_30: number;
  streak_days: number;
  daily_activity: DailyActivity[];
  top_corrections: [string, number][];
  recent_corrections: RecentCorrection[];
  mode_distribution: ModeDistribution[];
}

export interface RecentCorrection {
  id: string;
  session_id: string | null;
  original_text: string;
  corrected_text: string;
  explanation: string | null;
  better_expression: string | null;
  created_at: string;
}

export interface ModeDistribution {
  mode_id: string;
  mode_name: string;
  session_count: number;
}

export async function getLearningStats(): Promise<LearningStats> {
  return invoke<LearningStats>('get_learning_stats');
}
