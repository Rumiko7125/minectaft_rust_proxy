<template>
  <div class="page-container">
    <div class="page-header">
      <h1 class="page-title">{{ t('logs.title') }}</h1>
    </div>

    <NTabs v-model:value="tab" type="line" @update:value="onTabChange">
      <NTabPane name="moderation" :tab="t('logs.tabModeration')">
        <div class="toolbar">
          <div class="toolbar-left">
            <NSelect v-model:value="modFilter.action" :options="actionOptions" :placeholder="t('logs.filterAction')" clearable style="width: 130px" />
            <NInput v-model:value="modFilter.target" :placeholder="t('logs.filterPlayer')" clearable style="width: 160px" @keyup.enter="loadModerationLogs" />
            <NButton @click="loadModerationLogs" size="small">{{ t('common.search') }}</NButton>
          </div>
          <div class="toolbar-right">
            <NButton size="small" @click="loadModerationLogs" :loading="loading">
              <template #icon><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg></template>
              {{ t('common.refresh') }}
            </NButton>
            <NButton size="small" @click="exportModerationLogs">{{ t('common.export') }}</NButton>
          </div>
        </div>
        <NDataTable :columns="modColumns" :data="modLogs" :loading="loading" :bordered="false" size="small" />
        <div class="pagination">
          <NPagination v-model:page="modPage" :page-count="Math.ceil(modTotal / modPageSize)" :page-size="modPageSize" @update:page="loadModerationLogs" @update:page-size="handleModPageSizeChange" />
        </div>
      </NTabPane>

      <NTabPane name="session" :tab="t('logs.tabSession')">
        <div class="toolbar">
          <div class="toolbar-left">
            <NInput v-model:value="sessionSearch" :placeholder="t('logs.sessionSearch')" clearable style="width: 220px" @keyup.enter="loadSessionLogs" />
            <NButton @click="loadSessionLogs" size="small">{{ t('common.search') }}</NButton>
          </div>
          <div class="toolbar-right">
            <NButton size="small" @click="loadSessionLogs" :loading="loading">
              <template #icon><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg></template>
              {{ t('common.refresh') }}
            </NButton>
            <NButton size="small" @click="exportSessionLogs">{{ t('common.export') }}</NButton>
          </div>
        </div>
        <NDataTable :columns="sessionColumns" :data="sessionLogs" :loading="loading" :bordered="false" size="small" />
        <div class="pagination">
          <NPagination v-model:page="sessionPage" :page-count="Math.ceil(sessionTotal / sessionPageSize)" :page-size="sessionPageSize" @update:page="loadSessionLogs" @update:page-size="handleSessionPageSizeChange" />
        </div>
      </NTabPane>
    </NTabs>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import { NTabs, NTabPane, NDataTable, NDataTableColumns, NTag, NSelect, NInput, NButton, NPagination, useMessage } from 'naive-ui'
import { logsApi } from '../api'
import { useTimezone } from '../composables/useTimezone'

const { t } = useI18n()
const { formatRelativeTime } = useTimezone()
const message = useMessage()
const loading = ref(false)
const tab = ref('moderation')
const modLogs = ref<any[]>([])
const sessionLogs = ref<any[]>([])

// Moderation filters
const modFilter = reactive({
  action: null as string | null,
  target: ''
})
const modPage = ref(1)
const modPageSize = ref(20)
const modTotal = ref(0)

// Session filters
const sessionSearch = ref('')
const sessionPage = ref(1)
const sessionPageSize = ref(20)
const sessionTotal = ref(0)

const actionOptions = [
  { label: 'Kick', value: 'kick' },
  { label: 'Ban', value: 'ban' },
  { label: 'Pardon', value: 'pardon' }
]

const modColumns: NDataTableColumns<any> = [
  { title: () => t('common.time'), key: 'timestamp', width: 180, render: (row: any) => formatRelativeTime(row.timestamp) },
  {
    title: () => t('common.action'),
    key: 'action',
    width: 80,
    render: (row: any) => {
      const tagType = row.action === 'kick' ? 'warning' : row.action === 'ban' ? 'error' : 'success'
      return h(NTag, { type: tagType, size: 'small' }, { default: () => row.action })
    }
  },
  { title: () => t('table.username'), key: 'target' },
  { title: () => t('table.admin'), key: 'operator' },
  { title: () => t('table.reason'), key: 'reason' }
]

const sessionColumns: NDataTableColumns<any> = [
  { title: () => t('table.username'), key: 'username', width: 150 },
  { title: () => t('table.uuid'), key: 'uuid', width: 150 },
  { title: () => t('table.backend'), key: 'backend_addr' },
  { title: () => t('table.loginTime'), key: 'login_at', width: 160, render: (row: any) => formatRelativeTime(row.login_at) },
  { title: () => t('table.logoutTime'), key: 'logout_at', width: 160, render: (row: any) => formatRelativeTime(row.logout_at) }
]

function handleModPageSizeChange(newSize: number) {
  modPageSize.value = newSize
  loadModerationLogs()
}

function handleSessionPageSizeChange(newSize: number) {
  sessionPageSize.value = newSize
  loadSessionLogs()
}

async function loadModerationLogs() {
  loading.value = true
  try {
    const res = await logsApi.getModeration({
      action: modFilter.action || undefined,
      target: modFilter.target || undefined,
      limit: modPageSize.value,
      offset: (modPage.value - 1) * modPageSize.value
    })
    modLogs.value = res.logs || []
    modTotal.value = res.total || 0
  } catch (e) { console.error(e) }
  finally { loading.value = false }
}

async function loadSessionLogs() {
  loading.value = true
  try {
    const res = await logsApi.getSessions({
      search: sessionSearch.value || undefined,
      limit: sessionPageSize.value,
      offset: (sessionPage.value - 1) * sessionPageSize.value
    })
    sessionLogs.value = res.sessions || []
    sessionTotal.value = res.total || 0
  } catch (e) { console.error(e) }
  finally { loading.value = false }
}

async function exportModerationLogs() {
  try {
    const res = await logsApi.exportModeration()
    downloadCSV(res.csv, 'moderation_logs.csv')
    message.success(t('common.success'))
  } catch (e: any) {
    message.error(e.response?.data?.message || 'Export failed')
  }
}

async function exportSessionLogs() {
  try {
    const res = await logsApi.exportSessions({
      search: sessionSearch.value || undefined
    })
    downloadCSV(res.csv, 'session_logs.csv')
    message.success(t('common.success'))
  } catch (e: any) {
    message.error(e.response?.data?.message || 'Export failed')
  }
}

function downloadCSV(csv: string, filename: string) {
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' })
  const link = document.createElement('a')
  link.href = URL.createObjectURL(blob)
  link.download = filename
  link.click()
}

function onTabChange(newTab: string) {
  if (newTab === 'moderation') loadModerationLogs()
  else loadSessionLogs()
}

onMounted(() => {
  loadModerationLogs()
})
</script>

<style scoped>
.page-container { padding: 24px; }

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.page-title {
  color: #e6edf3;
  font-size: 22px;
  font-weight: 600;
  margin: 0;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
  padding: 8px 0;
}

.toolbar-left { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.toolbar-right { display: flex; gap: 8px; align-items: center; flex-shrink: 0; }

.pagination {
  display: flex;
  justify-content: flex-end;
  margin-top: 16px;
}
</style>
