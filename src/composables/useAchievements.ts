import { ref, watch } from 'vue';
import type { Ref } from 'vue';
import type { LearningStats } from '../services/tauri/stats';

export interface Achievement {
  id: string;
  title: string;
  description: string;
  icon: string;
}

const ACHIEVEMENTS_DB: Achievement[] = [
  { id: 'first_blood', title: 'First Words 🗣️', description: 'Sent your very first message', icon: '🐣' },
  { id: 'chatterbox', title: 'Chatterbox 💬', description: 'Sent 50 messages total', icon: '🎤' },
  { id: 'vocab_master', title: 'Vocab Hunter 📚', description: 'Saved your first vocabulary word', icon: '📖' },
  { id: 'correction_addict', title: 'Perfectionist ✏️', description: 'Used grammar correction 5 times', icon: '✨' },
  { id: 'streak_3', title: 'On fire! 🔥', description: '3-day learning streak', icon: '🔥' }
];

// Global state
const unlockedIds = ref<Set<string>>(new Set(JSON.parse(localStorage.getItem('unlocked_achievements') || '[]')));
const recentUnlock = ref<Achievement | null>(null);

export function useAchievements() {
  function unlock(id: string) {
    if (unlockedIds.value.has(id)) return;
    
    // Unlock and persist
    unlockedIds.value.add(id);
    localStorage.setItem('unlocked_achievements', JSON.stringify([...unlockedIds.value]));
    
    // Trigger popup
    const achievement = ACHIEVEMENTS_DB.find(a => a.id === id);
    if (achievement) {
      recentUnlock.value = achievement;
      // Auto-hide popup after 5 seconds
      setTimeout(() => {
        if (recentUnlock.value?.id === id) {
          recentUnlock.value = null;
        }
      }, 5000);
    }
  }

  // Hook to monitor stats/states
  function monitor(statsRef: Ref<LearningStats | null>) {
    watch(statsRef, (val) => {
      if (!val) return;
      if (val.total_messages >= 1) unlock('first_blood');
      if (val.total_messages >= 50) unlock('chatterbox');
      if (val.total_vocabulary >= 1) unlock('vocab_master');
      if (val.total_corrections >= 5) unlock('correction_addict');
      if (val.streak_days >= 3) unlock('streak_3');
    }, { deep: true });
  }

  return {
    unlockedIds,
    recentUnlock,
    unlock,
    monitor
  };
}
