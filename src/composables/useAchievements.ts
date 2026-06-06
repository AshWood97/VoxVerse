import { ref, watch } from 'vue';
import type { Ref } from 'vue';
import type { LearningStats } from '../services/tauri/stats';

export interface Achievement {
  id: string;
  titleKey: string;
  descriptionKey: string;
  icon: string;
}

const ACHIEVEMENTS_DB: Achievement[] = [
  { id: 'first_blood', titleKey: 'achievements.firstBlood.title', descriptionKey: 'achievements.firstBlood.description', icon: '🐣' },
  { id: 'chatterbox', titleKey: 'achievements.chatterbox.title', descriptionKey: 'achievements.chatterbox.description', icon: '🎤' },
  { id: 'vocab_master', titleKey: 'achievements.vocabMaster.title', descriptionKey: 'achievements.vocabMaster.description', icon: '📖' },
  { id: 'correction_addict', titleKey: 'achievements.correctionAddict.title', descriptionKey: 'achievements.correctionAddict.description', icon: '✨' },
  { id: 'streak_3', titleKey: 'achievements.streak3.title', descriptionKey: 'achievements.streak3.description', icon: '🔥' }
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
