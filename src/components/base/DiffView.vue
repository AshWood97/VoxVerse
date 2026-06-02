<script setup lang="ts">
/**
 * DiffView: displays original vs corrected text with inline highlights.
 * Red strikethrough for removed/changed words, green for corrections.
 */
const props = defineProps<{
  original: string;
  corrected: string;
}>();

// Simple word-level diff
function computeDiff() {
  const origWords = props.original.split(/\s+/);
  const corrWords = props.corrected.split(/\s+/);
  const result: { text: string; type: 'same' | 'removed' | 'added' }[] = [];

  let i = 0, j = 0;
  while (i < origWords.length || j < corrWords.length) {
    if (i < origWords.length && j < corrWords.length) {
      if (origWords[i].toLowerCase() === corrWords[j].toLowerCase()) {
        result.push({ text: corrWords[j], type: 'same' });
        i++; j++;
      } else {
        // Check if original word was removed (exists later in corrected)
        const futureJ = corrWords.indexOf(origWords[i], j + 1);
        const futureI = origWords.indexOf(corrWords[j], i + 1);

        if (futureJ !== -1 && (futureI === -1 || futureJ - j <= futureI - i)) {
          // Words were inserted in corrected
          while (j < futureJ) {
            result.push({ text: corrWords[j], type: 'added' });
            j++;
          }
        } else if (futureI !== -1) {
          // Words were removed from original
          while (i < futureI) {
            result.push({ text: origWords[i], type: 'removed' });
            i++;
          }
        } else {
          // Simple replacement
          result.push({ text: origWords[i], type: 'removed' });
          result.push({ text: corrWords[j], type: 'added' });
          i++; j++;
        }
      }
    } else if (i < origWords.length) {
      result.push({ text: origWords[i], type: 'removed' });
      i++;
    } else {
      result.push({ text: corrWords[j], type: 'added' });
      j++;
    }
  }
  return result;
}
</script>

<template>
  <div class="diff-view">
    <template v-for="(part, idx) in computeDiff()" :key="idx">
      <span
        :class="{
          'diff-same': part.type === 'same',
          'diff-removed': part.type === 'removed',
          'diff-added': part.type === 'added',
        }"
      >{{ part.text }}</span>
      <span v-if="idx < computeDiff().length - 1"> </span>
    </template>
  </div>
</template>

<style scoped>
.diff-view {
  font-size: var(--font-size-sm);
  line-height: 1.8;
  padding: var(--space-sm) var(--space-md);
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
}

.diff-same {
  color: var(--text-primary);
}

.diff-removed {
  color: #e74c3c;
  text-decoration: line-through;
  opacity: 0.7;
}

.diff-added {
  color: #2ecc71;
  font-weight: 600;
  background: rgba(46, 204, 113, 0.1);
  border-radius: 3px;
  padding: 0 2px;
}
</style>
