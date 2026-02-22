<template>
  <div class="page-container">
    <div class="page-header">
      <div>
        <h1 class="page-title">{{ t('players.title') }}</h1>
      </div>
    </div>

    <NTabs v-model:value="tab" type="line" @update:value="onTabChange">
      <!-- Online Players -->
      <NTabPane name="online" :tab="t('players.tabOnline')">
        <div class="tab-toolbar">
          <div class="player-count-badge">
            <span class="dot online-dot"></span>
            {{ players.length }} {{ t('dashboard.onlinePlayers') }}
          </div>
          <NButton size="small" @click="loadOnline" :loading="loading">
            <template #icon>
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
            </template>
            {{ t('common.refresh') }}
          </NButton>
        </div>

        <div v-if="!loading && players.length === 0" class="empty-state">
          <div class="empty-icon">🎮</div>
          <div class="empty-text">{{ t('players.noOnline') }}</div>
        </div>

        <NDataTable v-else :columns="columns" :data="players" :loading="loading" :bordered="false" size="small" />
        <div class="pagination">
          <NPagination v-model:page="onlinePage" :page-count="Math.ceil(onlineTotal / onlinePageSize)" :page-size="onlinePageSize" @update:page="loadOnline" />
        </div>
      </NTabPane>

      <!-- History -->
      <NTabPane name="history" :tab="t('players.tabHistory')">
        <div class="tab-toolbar">
          <div style="display:flex;gap:8px;align-items:center">
            <NInput v-model:value="search" :placeholder="t('common.search')" clearable style="width: 220px" @keyup.enter="loadHistory" />
            <NButton @click="loadHistory" size="small">{{ t('common.search') }}</NButton>
          </div>
          <div style="display:flex;gap:8px">
            <NButton size="small" @click="exportHistory">{{ t('common.export') }}</NButton>
            <NButton size="small" @click="loadHistory" :loading="loading">
              <template #icon>
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
              </template>
              {{ t('common.refresh') }}
            </NButton>
          </div>
        </div>
        <NDataTable :columns="historyColumns" :data="history" :loading="loading" :bordered="false" size="small" />
        <div class="pagination">
          <NPagination v-model:page="historyPage" :page-count="Math.ceil(historyTotal / historyPageSize)" :page-size="historyPageSize" @update:page="loadHistory" />
        </div>
      </NTabPane>
    </NTabs>

    <!-- Action Confirmation Dialog -->
    <NModal v-model:show="showActionModal" preset="card" :title="actionTitle" style="width: 420px;">
      <NFormItem :label="t('table.reason')">
        <NInput v-model:value="actionReason" type="textarea" :placeholder="t('players.reasonPlaceholder')" :rows="3" />
      </NFormItem>
      <div style="display: flex; justify-content: flex-end; gap: 8px; margin-top: 8px">
        <NButton @click="showActionModal = false">{{ t('common.cancel') }}</NButton>
        <NButton :type="actionType === 'kick' ? 'warning' : 'error'" @click="confirmAction">{{ t('common.confirm') }}</NButton>
      </div>
    </NModal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h } from 'vue'
import { useI18n } from 'vue-i18n'
import { NTabs, NTabPane, NDataTable, NDataTableColumns, NButton, NInput, NAvatar, NTooltip, NPagination, NModal, NFormItem, NTag, useMessage } from 'naive-ui'
import { playersApi, accessApi, logsApi } from '../api'
import { useTimezone } from '../composables/useTimezone'

const { t } = useI18n()
const { formatRelativeTime } = useTimezone()
const message = useMessage()
const loading = ref(false)
const tab = ref('online')
const players = ref<any[]>([])
const history = ref<any[]>([])
const search = ref('')

const showActionModal = ref(false)
const actionTitle = ref('')
const actionReason = ref('')
const actionType = ref<'kick' | 'ban'>('kick')
const actionTarget = ref('')

const onlinePage = ref(1)
const onlinePageSize = ref(20)
const onlineTotal = ref(0)

const historyPage = ref(1)
const historyPageSize = ref(20)
const historyTotal = ref(0)

function onTabChange(val: string) {
  if (val === 'online') loadOnline()
  else loadHistory()
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1048576) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1073741824) return (bytes / 1048576).toFixed(1) + ' MB'
  return (bytes / 1073741824).toFixed(2) + ' GB'
}

function getAvatarUrl(uuid: string | undefined): string {
  if (!uuid) return ''
  return `https://api.mineatar.io/face/${uuid.replace(/-/g, '')}?scale=6`
}

function shortenUuid(uuid: string | undefined): string {
  if (!uuid) return '-'
  return uuid.substring(0, 8) + '...'
}

function handleKick(row: any) {
  actionTitle.value = t('players.kickTitle')
  actionType.value = 'kick'
  actionTarget.value = row.username
  actionReason.value = ''
  showActionModal.value = true
}

function handleBan(row: any) {
  actionTitle.value = t('players.banTitle')
  actionType.value = 'ban'
  actionTarget.value = row.username
  actionReason.value = ''
  showActionModal.value = true
}

async function confirmAction() {
  try {
    if (actionType.value === 'kick') {
      await playersApi.kick(actionTarget.value, actionReason.value || undefined)
      message.success(t('players.kickSuccess', { name: actionTarget.value }))
    } else {
      await accessApi.addBlacklist(actionTarget.value, actionReason.value || undefined)
      message.success(t('players.addBlacklistSuccess'))
      loadOnline()
    }
    showActionModal.value = false
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  }
}

const columns: NDataTableColumns<any> = [
  {
    title: '',
    key: 'avatar',
    width: 46,
    render: (row: any) => h(NAvatar, {
      src: getAvatarUrl(row.uuid),
      round: true,
      size: 'small',
      fallbackSrc: 'https://crafatar.com/avatars/8667ba71-b85a-4004-af54-28e122c77270?size=32'
    })
  },
  { title: () => t('table.username'), key: 'username', width: 140 },
  {
    title: () => t('table.uuid'), key: 'uuid', width: 110,
    render: (row: any) => h(NTooltip, { trigger: 'hover' }, {
      default: () => row.uuid || '-',
      trigger: () => h('span', { class: 'uuid-short' }, shortenUuid(row.uuid))
    })
  },
  {
    title: () => t('table.backend'), key: 'backend_name',
    render: (row: any) => h(NTag, { type: 'default', size: 'small' }, { default: () => row.backend_name || '-' })
  },
  { title: () => t('table.version'), key: 'version', width: 110 },
  {
    title: () => t('table.loginTime'), key: 'login_time', width: 100,
    render: (row: any) => formatRelativeTime(row.login_time * 1000)
  },
  {
    title: () => t('table.upload'), key: 'upload_bytes', width: 90,
    render: (row: any) => h('span', { class: 'traffic-up' }, formatBytes(row.upload_bytes))
  },
  {
    title: () => t('table.download'), key: 'download_bytes', width: 90,
    render: (row: any) => h('span', { class: 'traffic-down' }, formatBytes(row.download_bytes))
  },
  {
    title: () => t('table.action'), key: 'action', width: 160,
    render: (row: any) => h('div', { style: 'display:flex;gap:6px' }, [
      h(NButton, { size: 'small', type: 'warning', onClick: () => handleKick(row) }, { default: () => t('players.kick') }),
      h(NButton, { size: 'small', type: 'error', onClick: () => handleBan(row) }, { default: () => t('players.addBlacklist') })
    ])
  }
]

const historyColumns: NDataTableColumns<any> = [
  { title: () => t('table.username'), key: 'username', width: 140 },
  {
    title: () => t('table.uuid'), key: 'uuid', width: 110,
    render: (row: any) => h(NTooltip, { trigger: 'hover' }, {
      default: () => row.uuid || '-',
      trigger: () => h('span', { class: 'uuid-short' }, shortenUuid(row.uuid))
    })
  },
  { title: () => t('table.backend'), key: 'backend_addr' },
  { title: () => t('table.version'), key: 'version', width: 110 },
  {
    title: () => t('table.loginTime'), key: 'login_at', width: 100,
    render: (row: any) => formatRelativeTime(row.login_at)
  },
  {
    title: () => t('table.logoutTime'), key: 'logout_at', width: 100,
    render: (row: any) => row.logout_at ? formatRelativeTime(row.logout_at) : h('span', { class: 'text-muted' }, '-')
  },
  {
    title: () => t('table.upload'), key: 'upload_bytes', width: 90,
    render: (row: any) => h('span', { class: 'traffic-up' }, formatBytes(row.upload_bytes || 0))
  },
  {
    title: () => t('table.download'), key: 'download_bytes', width: 90,
    render: (row: any) => h('span', { class: 'traffic-down' }, formatBytes(row.download_bytes || 0))
  }
]

async function loadOnline() {
  loading.value = true
  try {
    const res = await playersApi.getOnline({ limit: onlinePageSize.value, offset: (onlinePage.value - 1) * onlinePageSize.value })
    players.value = res.players || []
    onlineTotal.value = res.total || 0
  } catch (e) { console.error(e) }
  finally { loading.value = false }
}

async function loadHistory() {
  loading.value = true
  try {
    const res = await playersApi.getHistory({ search: search.value || undefined, limit: historyPageSize.value, offset: (historyPage.value - 1) * historyPageSize.value })
    history.value = res.sessions || []
    historyTotal.value = res.total || 0
  } catch (e) { console.error(e) }
  finally { loading.value = false }
}

async function exportHistory() {
  try {
    const res = await logsApi.exportSessions({ search: search.value || undefined })
    const blob = new Blob([res.csv], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = 'session_history.csv'
    link.click()
    message.success(t('common.success'))
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  }
}

onMounted(async () => {
  await loadOnline()
  await loadHistory()
})
</script>

<style scoped>
.page-container { padding: 24px; }

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.page-title {
  color: #e6edf3;
  font-size: 22px;
  font-weight: 600;
  margin: 0;
}

.tab-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  margin-bottom: 8px;
}

.player-count-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #8b949e;
  font-size: 13px;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.online-dot { background: #3fb950; box-shadow: 0 0 6px #3fb950; }

.empty-state {
  text-align: center;
  padding: 60px 24px;
  color: #8b949e;
}

.empty-icon { font-size: 48px; margin-bottom: 12px; }
.empty-text { font-size: 14px; }

.pagination {
  display: flex;
  justify-content: flex-end;
  margin-top: 16px;
}

.uuid-short {
  font-family: monospace;
  font-size: 12px;
  color: #8b949e;
  cursor: pointer;
}

.traffic-up { color: #3fb950; font-size: 12px; }
.traffic-down { color: #f85149; font-size: 12px; }
.text-muted { color: #8b949e; }
</style>
