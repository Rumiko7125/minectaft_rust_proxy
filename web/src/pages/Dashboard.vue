<template>
  <div class="page-container">
    <div class="page-header">
      <h1 class="page-title">{{ t('dashboard.title') }}</h1>
      <NButton size="small" @click="manualRefresh" :loading="loading">
        <template #icon>
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
        </template>
        {{ t('common.refresh') }} ({{ countdown }}s)
      </NButton>
    </div>

    <!-- Top Stats Cards -->
    <div class="stats-row">
      <div class="stat-card">
        <div class="stat-icon-wrap icon-blue">
          <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
        </div>
        <div class="stat-body">
          <div class="stat-label">{{ t('dashboard.onlinePlayers') }}</div>
          <div class="stat-num">{{ stats.online_count }}</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon-wrap icon-green">
          <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 6 13.5 15.5 8.5 10.5 1 18"/><polyline points="17 6 23 6 23 12"/></svg>
        </div>
        <div class="stat-body">
          <div class="stat-label">{{ t('dashboard.totalTraffic') }} ↑</div>
          <div class="stat-num green">{{ formatBytes(stats.today_upload) }}</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon-wrap icon-red">
          <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 18 13.5 8.5 8.5 13.5 1 6"/><polyline points="17 18 23 18 23 12"/></svg>
        </div>
        <div class="stat-body">
          <div class="stat-label">{{ t('dashboard.totalTraffic') }} ↓</div>
          <div class="stat-num red">{{ formatBytes(stats.today_download) }}</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon-wrap icon-purple">
          <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
        </div>
        <div class="stat-body">
          <div class="stat-label">{{ t('dashboard.serverStatus') }}</div>
          <div class="stat-num green small">{{ t('common.online') }}</div>
          <div class="stat-uptime">{{ formatUptime(uptimeSeconds) }}</div>
        </div>
      </div>
    </div>

    <!-- Backend Server Status -->
    <div class="section-title">{{ t('dashboard.backendStatus') }}</div>
    <div v-if="backends.length === 0" class="no-backends">
      {{ t('backend.noDefaultWarning') }}
    </div>
    <div v-else class="backends-grid">
      <div v-for="b in backends" :key="b.id" class="backend-card" :class="{ 'maintenance': b.maintenance, 'disabled': !b.enabled }">
        <div class="bc-header">
          <span class="bc-name">{{ b.name }}</span>
          <div class="bc-badges">
            <span v-if="b.is_default" class="badge-default">{{ t('backend.default') }}</span>
            <span v-if="b.maintenance" class="badge-maint">{{ t('backend.maintenanceEnabled') }}</span>
            <span v-else-if="!b.enabled" class="badge-disabled">{{ t('backend.disabled') }}</span>
            <span v-else class="badge-ok">{{ t('backend.enabled') }}</span>
          </div>
        </div>
        <div class="bc-addr">
          <span class="addr-dot"></span>
          <code>{{ b.remote_address }}:{{ b.remote_port }}</code>
        </div>
        <div class="bc-players">
          <span class="player-dot"></span>
          <span>{{ getBackendOnline(b.name) }}
            <span v-if="b.max_player >= 0"> / {{ b.max_player }}</span>
            <span v-else> / ∞</span>
            {{ t('dashboard.onlinePlayers') }}
          </span>
        </div>
      </div>
    </div>

    <!-- Charts -->
    <div class="charts-row">
      <div class="chart-card">
        <div class="chart-title">{{ t('dashboard.playerCountTrend') }}</div>
        <div ref="playerChartRef" class="chart-container"></div>
      </div>
      <div class="chart-card">
        <div class="chart-title">{{ t('dashboard.trafficTrendChart') }}</div>
        <div ref="trafficChartRef" class="chart-container"></div>
      </div>
    </div>

    <!-- Recent Activity -->
    <div class="activity-row">
      <div class="activity-card">
        <div class="activity-title">{{ t('dashboard.recentModeration') }}</div>
        <div v-if="recentModLogs.length === 0" class="no-data">{{ t('common.noData') }}</div>
        <div v-else class="mod-list">
          <div v-for="log in recentModLogs" :key="log.id" class="mod-item">
            <NTag :type="log.action === 'kick' ? 'warning' : log.action === 'ban' ? 'error' : 'success'" size="small" class="mod-tag">
              {{ log.action }}
            </NTag>
            <span class="mod-target">{{ log.target }}</span>
            <span class="mod-time">{{ formatRelativeTime(log.timestamp) }}</span>
          </div>
        </div>
      </div>

      <div class="activity-card">
        <div class="activity-title">{{ t('dashboard.recentSessions') }}</div>
        <div v-if="recentSessions.length === 0" class="no-data">{{ t('common.noData') }}</div>
        <div v-else class="session-list">
          <div v-for="s in recentSessions" :key="s.id || s.username" class="session-item">
            <div class="session-user">
              <span v-if="s.online" class="online-dot-sm"></span>
              <span class="session-name">{{ s.username }}</span>
            </div>
            <span class="session-backend">{{ s.backend_addr }}</span>
            <span class="session-time">{{ formatRelativeTime(s.login_at) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { NButton, NTag } from 'naive-ui'
import * as echarts from 'echarts'
import { playersApi, backendApi, api } from '../api'
import { useTimezone } from '../composables/useTimezone'

const { t } = useI18n()
const { formatRelativeTime } = useTimezone()

const loading = ref(false)
const countdown = ref(5)
let refreshTimer: ReturnType<typeof setInterval> | null = null

const serverStartTime = ref(0)   // Unix seconds, server start time
const uptimeSeconds = ref(0)     // Current running seconds, updated every second
let uptimeTimer: ReturnType<typeof setInterval> | null = null

const stats = ref<any>({ online_count: 0, max_players: 0, today_upload: 0, today_download: 0, hourly_stats: [], hourly_traffic: [], backend_counts: {} })
const recentModLogs = ref<any[]>([])
const recentSessions = ref<any[]>([])
const backends = ref<any[]>([])
const playerChartRef = ref<HTMLElement>()
const trafficChartRef = ref<HTMLElement>()

function formatUptime(seconds: number): string {
  if (seconds <= 0) return '--'
  const d = Math.floor(seconds / 86400)
  const h = Math.floor((seconds % 86400) / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (d > 0) return `${d}d ${h}h ${m}m`
  if (h > 0) return `${h}h ${m}m ${s}s`
  return `${m}m ${s}s`
}

function formatBytes(bytes: number): string {
  if (!bytes) return '0 B'
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1048576) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1073741824) return (bytes / 1048576).toFixed(1) + ' MB'
  return (bytes / 1073741824).toFixed(2) + ' GB'
}

function getBackendOnline(name: string): number {
  return (stats.value.backend_counts || {})[name] || 0
}

function initCharts() {
  const chartOpts = {
    tooltip: { trigger: 'axis' },
    grid: { left: '3%', right: '4%', bottom: '3%', top: '8%', containLabel: true },
  }

  if (playerChartRef.value) {
    const chart = echarts.init(playerChartRef.value)
    const hours = Array.from({ length: 24 }, (_, i) => `${i}:00`)
    const data = hours.map(h => {
      const found = stats.value.hourly_stats.find((s: any) => s.hour === parseInt(h))
      return found ? found.count : 0
    })
    chart.setOption({
      ...chartOpts,
      xAxis: { type: 'category', data: hours, axisLabel: { color: '#6e7681', fontSize: 11 } },
      yAxis: { type: 'value', axisLabel: { color: '#6e7681' }, splitLine: { lineStyle: { color: '#21262d' } }, minInterval: 1 },
      series: [{ data, type: 'line', smooth: true, areaStyle: { color: 'rgba(88,166,255,0.15)' }, lineStyle: { color: '#58a6ff' }, itemStyle: { color: '#58a6ff' }, symbol: 'none' }]
    })
  }

  if (trafficChartRef.value) {
    const chart = echarts.init(trafficChartRef.value)
    const hours = Array.from({ length: 24 }, (_, i) => `${i}:00`)
    const ht = stats.value.hourly_traffic || []
    const up = hours.map(h => { const d = ht.find((t: any) => t.hour === parseInt(h)); return d ? d.upload : 0 })
    const dn = hours.map(h => { const d = ht.find((t: any) => t.hour === parseInt(h)); return d ? d.download : 0 })
    chart.setOption({
      ...chartOpts,
      legend: { data: ['↑', '↓'], textStyle: { color: '#8b949e' }, top: 0, right: 0 },
      xAxis: { type: 'category', data: hours, axisLabel: { color: '#6e7681', fontSize: 11 } },
      yAxis: { type: 'value', axisLabel: { color: '#6e7681', formatter: (v: number) => formatBytes(v) }, splitLine: { lineStyle: { color: '#21262d' } } },
      series: [
        { name: '↑', type: 'line', data: up, smooth: true, lineStyle: { color: '#3fb950' }, itemStyle: { color: '#3fb950' }, symbol: 'none', areaStyle: { color: 'rgba(63,185,80,0.1)' } },
        { name: '↓', type: 'line', data: dn, smooth: true, lineStyle: { color: '#f85149' }, itemStyle: { color: '#f85149' }, symbol: 'none', areaStyle: { color: 'rgba(248,81,73,0.1)' } }
      ]
    })
  }
}

async function loadData() {
  loading.value = true
  try {
    const [statsRes, recentRes, backendsRes] = await Promise.all([
      playersApi.getStats(),
      api.get('/dashboard/recent'),
      backendApi.list()
    ])
    stats.value = statsRes
    recentModLogs.value = recentRes.recent_mod_logs || []
    if (recentRes.server_start_time) {
      serverStartTime.value = recentRes.server_start_time
    }
    // Merge online players + recent history sessions (online first)
    const online = (recentRes.online_sessions || []).map((s: any) => ({ ...s, online: true }))
    const hist = recentRes.recent_sessions || []
    recentSessions.value = [...online, ...hist].slice(0, 10)
    backends.value = (backendsRes as any).list || []
    await nextTick()
    initCharts()
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

function manualRefresh() {
  countdown.value = 5
  loadData()
}

function startAutoRefresh() {
  refreshTimer = setInterval(() => {
    countdown.value--
    if (countdown.value <= 0) {
      loadData()
      countdown.value = 5
    }
  }, 1000)
}

onMounted(() => {
  loadData()
  startAutoRefresh()
  uptimeTimer = setInterval(() => {
    if (serverStartTime.value > 0) {
      uptimeSeconds.value = Math.floor(Date.now() / 1000) - serverStartTime.value
    }
  }, 1000)
})

onUnmounted(() => {
  if (refreshTimer) clearInterval(refreshTimer)
  if (uptimeTimer) clearInterval(uptimeTimer)
})
</script>

<style scoped>
.page-container { padding: 24px; }

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.page-title {
  color: #e6edf3;
  font-size: 22px;
  font-weight: 600;
  margin: 0;
}

/* ===== Stats Cards ===== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  background: #161b22;
  border: 1px solid #21262d;
  border-radius: 10px;
  padding: 18px 20px;
  display: flex;
  align-items: center;
  gap: 16px;
  transition: border-color 0.2s;
}

.stat-card:hover { border-color: #30363d; }

.stat-icon-wrap {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.icon-blue { background: rgba(88,166,255,0.12); color: #58a6ff; }
.icon-green { background: rgba(63,185,80,0.12); color: #3fb950; }
.icon-red { background: rgba(248,81,73,0.12); color: #f85149; }
.icon-purple { background: rgba(188,140,255,0.12); color: #bc8cff; }

.stat-body { flex: 1; min-width: 0; }
.stat-label { color: #6e7681; font-size: 12px; margin-bottom: 4px; }
.stat-num { color: #e6edf3; font-size: 22px; font-weight: 700; }
.stat-num.small { font-size: 16px; }
.stat-num.green { color: #3fb950; }
.stat-num.red { color: #f85149; }
.stat-uptime { color: #6e7681; font-size: 11px; margin-top: 2px; }

/* ===== Section Title ===== */
.section-title {
  color: #8b949e;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  margin-bottom: 12px;
}

/* ===== Backend Cards ===== */
.backends-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 12px;
  margin-bottom: 24px;
}

.no-backends {
  background: #161b22;
  border: 1px solid #21262d;
  border-radius: 8px;
  padding: 24px;
  text-align: center;
  color: #6e7681;
  font-size: 13px;
  margin-bottom: 24px;
}

.backend-card {
  background: #161b22;
  border: 1px solid #21262d;
  border-radius: 8px;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.backend-card.maintenance { border-color: #e3b341; }
.backend-card.disabled { opacity: 0.5; }

.bc-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.bc-name {
  font-weight: 600;
  color: #e6edf3;
  font-size: 14px;
}

.bc-badges { display: flex; gap: 4px; flex-wrap: wrap; }

.badge-default, .badge-maint, .badge-disabled, .badge-ok {
  padding: 1px 7px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
}

.badge-default { background: rgba(227,179,65,0.15); color: #e3b341; border: 1px solid rgba(227,179,65,0.3); }
.badge-maint { background: rgba(248,81,73,0.12); color: #f85149; border: 1px solid rgba(248,81,73,0.3); }
.badge-disabled { background: rgba(110,118,129,0.12); color: #6e7681; border: 1px solid #30363d; }
.badge-ok { background: rgba(63,185,80,0.12); color: #3fb950; border: 1px solid rgba(63,185,80,0.3); }

.bc-addr {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #6e7681;
}

.bc-addr code { font-family: monospace; color: #8b949e; }

.addr-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #30363d;
  flex-shrink: 0;
}

.bc-players {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #8b949e;
}

.player-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #3fb950;
  flex-shrink: 0;
}

/* ===== Charts ===== */
.charts-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-bottom: 24px;
}

.chart-card {
  background: #161b22;
  border: 1px solid #21262d;
  border-radius: 10px;
  padding: 16px 20px;
}

.chart-title {
  color: #8b949e;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  margin-bottom: 12px;
}

.chart-container { width: 100%; height: 200px; }

/* ===== Recent Activity ===== */
.activity-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.activity-card {
  background: #161b22;
  border: 1px solid #21262d;
  border-radius: 10px;
  padding: 16px 20px;
}

.activity-title {
  color: #8b949e;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  margin-bottom: 12px;
}

.no-data { color: #6e7681; font-size: 13px; text-align: center; padding: 24px 0; }

.mod-list, .session-list { display: flex; flex-direction: column; gap: 8px; }

.mod-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
  border-bottom: 1px solid #21262d;
}
.mod-item:last-child { border-bottom: none; }

.mod-tag { flex-shrink: 0; }
.mod-target { color: #e6edf3; font-size: 13px; flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.mod-time { color: #6e7681; font-size: 11px; flex-shrink: 0; }

.session-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
  border-bottom: 1px solid #21262d;
}
.session-item:last-child { border-bottom: none; }

.session-user {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.online-dot-sm {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #3fb950;
  flex-shrink: 0;
}

.session-name { color: #e6edf3; font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.session-backend { color: #6e7681; font-size: 11px; font-family: monospace; flex-shrink: 0; max-width: 120px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.session-time { color: #6e7681; font-size: 11px; flex-shrink: 0; }
</style>
