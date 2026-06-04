<script setup lang="ts">
import { computed } from 'vue';
import type { LearningStats } from '../../services/tauri/stats';

const props = defineProps<{
  stats: LearningStats | null;
  isLoading: boolean;
  error: string | null;
}>();

const emit = defineEmits<{ close: [] }>();

// ── 7-Day Chart ────────────────────────────────────────────────────────────
const CHART_W = 420;
const CHART_H = 80;
const PADDING = { left: 8, right: 8, top: 8, bottom: 24 };

const chartPoints = computed(() => {
  if (!props.stats?.daily_activity?.length) return [];
  const data = props.stats.daily_activity;
  const max = Math.max(...data.map(d => d.message_count), 1);
  const innerW = CHART_W - PADDING.left - PADDING.right;
  const innerH = CHART_H - PADDING.top - PADDING.bottom;
  const step = innerW / Math.max(data.length - 1, 1);

  return data.map((d, i) => ({
    x: PADDING.left + i * step,
    y: PADDING.top + innerH - (d.message_count / max) * innerH,
    count: d.message_count,
    date: d.date,
    label: d.date.slice(5), // "MM-DD"
  }));
});

const polylinePoints = computed(() =>
  chartPoints.value.map(p => `${p.x},${p.y}`).join(' ')
);

const areaPath = computed(() => {
  if (!chartPoints.value.length) return '';
  const pts = chartPoints.value;
  const bottom = CHART_H - PADDING.bottom;
  const first = pts[0];
  const last = pts[pts.length - 1];
  return `M${first.x},${bottom} ` +
    pts.map(p => `L${p.x},${p.y}`).join(' ') +
    ` L${last.x},${bottom} Z`;
});

// ── Formatters ───────────────────────────────────────────────────────────
function streakLabel(n: number): string {
  if (n === 0) return 'No streak yet';
  if (n === 1) return '1 day streak 🔥';
  return `${n} day streak 🔥`;
}

function overlayClick(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains('stats-overlay')) {
    emit('close');
  }
}
</script>

<template>
  <div class="stats-overlay" @click="overlayClick">
    <div class="stats-panel">
      <!-- Header -->
      <div class="stats-header">
        <div class="stats-title">
          <span class="stats-icon">📊</span>
          <h2>My Learning Progress</h2>
        </div>
        <button class="close-btn" @click="emit('close')">✕</button>
      </div>

      <!-- Loading -->
      <div v-if="isLoading" class="stats-loading">
        <div class="spinner"></div>
        <span>Loading stats…</span>
      </div>

      <!-- Error -->
      <div v-else-if="error" class="stats-error">
        ⚠️ {{ error }}
      </div>

      <!-- Content -->
      <div v-else-if="stats" class="stats-body">

        <!-- Streak banner -->
        <div class="streak-banner" :class="{ active: stats.streak_days > 0 }">
          <span class="streak-num">{{ stats.streak_days }}</span>
          <div class="streak-info">
            <span class="streak-label">{{ streakLabel(stats.streak_days) }}</span>
            <span class="streak-sub">{{ stats.active_days_30 }} active days in the last 30 days</span>
          </div>
        </div>

        <!-- Stat cards -->
        <div class="stat-cards">
          <div class="stat-card">
            <span class="stat-icon">💬</span>
            <span class="stat-value">{{ stats.total_messages.toLocaleString() }}</span>
            <span class="stat-label">Messages Sent</span>
          </div>
          <div class="stat-card">
            <span class="stat-icon">📚</span>
            <span class="stat-value">{{ stats.total_vocabulary.toLocaleString() }}</span>
            <span class="stat-label">Words Saved</span>
          </div>
          <div class="stat-card">
            <span class="stat-icon">✏️</span>
            <span class="stat-value">{{ stats.total_corrections.toLocaleString() }}</span>
            <span class="stat-label">Corrections Run</span>
          </div>
        </div>

        <!-- 7-Day Activity Chart -->
        <div class="chart-section">
          <h3 class="section-title">7-Day Activity</h3>
          <div class="chart-wrap">
            <svg :width="CHART_W" :height="CHART_H" class="chart-svg" viewBox="0 0 420 80" preserveAspectRatio="xMidYMid meet">
              <!-- Grid lines -->
              <line x1="8" y1="8" :x2="CHART_W - 8" y2="8" stroke="var(--border-subtle)" stroke-width="0.5" />
              <line x1="8" y1="44" :x2="CHART_W - 8" y2="44" stroke="var(--border-subtle)" stroke-width="0.5" />

              <!-- Area fill -->
              <defs>
                <linearGradient id="chartGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stop-color="var(--accent-primary)" stop-opacity="0.35" />
                  <stop offset="100%" stop-color="var(--accent-primary)" stop-opacity="0" />
                </linearGradient>
              </defs>
              <path :d="areaPath" fill="url(#chartGrad)" />

              <!-- Line -->
              <polyline
                v-if="chartPoints.length"
                :points="polylinePoints"
                fill="none"
                stroke="var(--accent-primary)"
                stroke-width="2"
                stroke-linejoin="round"
                stroke-linecap="round"
              />

              <!-- Dots + tooltips -->
              <g v-for="p in chartPoints" :key="p.date">
                <circle
                  :cx="p.x" :cy="p.y" r="3.5"
                  fill="var(--accent-primary)"
                  stroke="var(--bg-secondary)"
                  stroke-width="1.5"
                />
                <!-- X labels -->
                <text :x="p.x" :y="CHART_H - 4" text-anchor="middle" fill="var(--text-tertiary)" font-size="7">
                  {{ p.label }}
                </text>
              </g>
            </svg>
          </div>
          <p class="chart-caption">Daily messages sent (user only)</p>
        </div>

        <!-- Top Corrections -->
        <div v-if="stats.top_corrections.length" class="corrections-section">
          <h3 class="section-title">Most Corrected Phrases</h3>
          <div class="correction-tags">
            <span
              v-for="([phrase, count]) in stats.top_corrections"
              :key="phrase"
              class="correction-tag"
              :style="{ fontSize: `${Math.min(14, 10 + count)}px` }"
            >
              {{ phrase }}
              <em>×{{ count }}</em>
            </span>
          </div>
        </div>
        <div v-else class="empty-corrections">
          <span>🎉 No corrections yet — keep chatting!</span>
        </div>

        <div v-if="stats.mode_distribution.length" class="mode-section">
          <h3 class="section-title">Practice Mode Mix</h3>
          <div class="mode-bars">
            <div
              v-for="mode in stats.mode_distribution"
              :key="mode.mode_id"
              class="mode-row"
            >
              <span class="mode-name">{{ mode.mode_name }}</span>
              <span class="mode-count">{{ mode.session_count }}</span>
            </div>
          </div>
        </div>

        <div v-if="stats.recent_corrections.length" class="recent-section">
          <h3 class="section-title">Recent Corrections</h3>
          <div class="recent-list">
            <div
              v-for="correction in stats.recent_corrections"
              :key="correction.id"
              class="recent-item"
            >
              <div class="recent-before">{{ correction.original_text }}</div>
              <div class="recent-after">{{ correction.corrected_text }}</div>
              <div v-if="correction.explanation" class="recent-explanation">
                Why: {{ correction.explanation }}
              </div>
              <div v-if="correction.better_expression" class="recent-better">
                Better: {{ correction.better_expression }}
              </div>
            </div>
          </div>
        </div>

      </div>

      <!-- No data -->
      <div v-else class="stats-empty">
        Start chatting to see your learning stats here!
      </div>
    </div>
  </div>
</template>

<style scoped>
.stats-overlay {
  position: fixed;
  inset: 0;
  background: var(--overlay-bg);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
  animation: fadeIn 0.2s ease-out;
}

.stats-panel {
  background: var(--bg-secondary);
  border: 1px solid var(--border-visible);
  border-radius: var(--radius-xl);
  width: 520px;
  max-width: 95vw;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: var(--shadow-lg);
  animation: slideUp 0.25s ease-out;
}

@keyframes slideUp {
  from { transform: translateY(20px); opacity: 0; }
  to   { transform: translateY(0);    opacity: 1; }
}

/* ── Header ── */
.stats-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-lg);
  border-bottom: 1px solid var(--border-subtle);
  position: sticky;
  top: 0;
  background: var(--bg-secondary);
  z-index: 1;
}
.stats-title {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}
.stats-icon { font-size: 1.4rem; }
.stats-title h2 {
  font-size: var(--font-size-lg);
  font-weight: 700;
  background: var(--accent-gradient);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
.close-btn {
  width: 32px; height: 32px;
  border-radius: var(--radius-sm);
  color: var(--text-tertiary);
  display: flex; align-items: center; justify-content: center;
  transition: background var(--transition-fast);
}
.close-btn:hover { background: var(--bg-hover); color: var(--text-primary); }

/* ── Loading / Error / Empty ── */
.stats-loading, .stats-error, .stats-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-md);
  padding: var(--space-xl);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}
.spinner {
  width: 28px; height: 28px;
  border: 3px solid var(--border-subtle);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
.stats-error { color: var(--status-danger); }

/* ── Body ── */
.stats-body {
  padding: var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

/* ── Streak Banner ── */
.streak-banner {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  padding: var(--space-md) var(--space-lg);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  transition: all var(--transition-fast);
}
.streak-banner.active {
  background: var(--progress-warm);
  border-color: var(--progress-warm-border);
}
.streak-num {
  font-size: 2.5rem;
  font-weight: 800;
  line-height: 1;
  background: var(--accent-gradient);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  min-width: 3rem;
  text-align: center;
}
.streak-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.streak-label {
  font-size: var(--font-size-base);
  font-weight: 600;
  color: var(--text-primary);
}
.streak-sub {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
}

/* ── Stat Cards ── */
.stat-cards {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-md);
}
.stat-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-md);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  transition: all var(--transition-fast);
}
.stat-card:hover {
  border-color: var(--border-visible);
  transform: translateY(-2px);
}
.stat-icon { font-size: 1.5rem; }
.stat-value {
  font-size: 1.8rem;
  font-weight: 700;
  color: var(--text-primary);
}
.stat-label {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
  text-align: center;
}

/* ── Chart ── */
.section-title {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: var(--space-sm);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.chart-wrap {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  padding: var(--space-sm);
  overflow: hidden;
}
.chart-svg {
  width: 100%;
  height: auto;
  display: block;
}
.chart-caption {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
  text-align: center;
  margin-top: var(--space-xs);
}

/* ── Corrections Tag Cloud ── */
.correction-tags {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-sm);
}
.correction-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: var(--accent-soft);
  border: 1px solid var(--accent-border);
  border-radius: 999px;
  color: var(--accent-primary);
  font-weight: 500;
  transition: all var(--transition-fast);
}
.correction-tag:hover {
  background: var(--accent-soft);
}
.correction-tag em {
  font-style: normal;
  font-size: 0.75em;
  opacity: 0.6;
}

.empty-corrections {
  padding: var(--space-md);
  text-align: center;
  color: var(--text-tertiary);
  font-size: var(--font-size-sm);
}

.mode-section,
.recent-section {
  display: flex;
  flex-direction: column;
}

.mode-bars,
.recent-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.mode-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-md);
  padding: var(--space-sm) var(--space-md);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
}

.mode-name {
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  font-weight: 600;
}

.mode-count {
  min-width: 28px;
  padding: 2px 8px;
  border-radius: var(--radius-full);
  background: var(--accent-secondary-soft);
  color: var(--accent-secondary);
  text-align: center;
  font-size: var(--font-size-xs);
  font-weight: 700;
}

.recent-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: var(--space-sm) var(--space-md);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
}

.recent-before {
  color: var(--status-danger);
  font-size: var(--font-size-sm);
  text-decoration: line-through;
  opacity: 0.8;
}

.recent-after {
  color: var(--status-success);
  font-size: var(--font-size-sm);
  font-weight: 600;
}

.recent-explanation {
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
}

.recent-better {
  color: var(--text-tertiary);
  font-size: var(--font-size-xs);
}
</style>
