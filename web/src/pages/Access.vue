<template>
  <div class="page-container">
    <div class="page-header">
      <div>
        <h1 class="page-title">{{ t('access.title') }}</h1>
        <p class="page-subtitle">{{ t('access.subtitle') }}</p>
      </div>
      <NButton size="small" @click="loadData" :loading="loading">
        <template #icon><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg></template>
        {{ t('common.refresh') }}
      </NButton>
    </div>

    <NTabs v-model:value="tab" type="line" class="access-tabs">
      <!-- Whitelist -->
      <NTabPane name="whitelist" :tab="t('access.tabWhitelist')">
        <div class="tab-toolbar">
          <div class="whitelist-status" :class="{ active: whitelistEnabled }">
            <span class="status-dot"></span>
            <span>{{ whitelistEnabled ? t('access.whitelistEnabled') : t('access.whitelistDisabled') }}</span>
            <NButton size="small" :type="whitelistEnabled ? 'default' : 'primary'" @click="toggleWhitelist">
              {{ whitelistEnabled ? t('access.disableWhitelist') : t('access.enableWhitelist') }}
            </NButton>
          </div>
          <div style="display: flex; gap: 8px;">
            <NButton @click="showBatchAdd = true">{{ t('access.batchImport') }}</NButton>
            <NButton type="primary" @click="showAddWhitelist = true">{{ t('common.add') }}</NButton>
          </div>
        </div>
        <NDataTable :columns="wlColumns" :data="whitelist" :loading="loading" :bordered="false" />
        <div class="pagination-row">
          <NPagination v-model:page="wlPage" :page-count="Math.ceil(wlTotal / wlPageSize)" :page-size="wlPageSize" @update:page="loadData" @update:page-size="s => { wlPageSize = s; loadData() }" />
        </div>
      </NTabPane>

      <!-- Blacklist -->
      <NTabPane name="blacklist" :tab="t('access.tabBlacklist')">
        <div class="tab-toolbar">
          <div></div>
          <NButton type="error" @click="showBan = true">{{ t('access.addToBlacklist') }}</NButton>
        </div>
        <NDataTable :columns="blColumns" :data="blacklist" :loading="loading" :bordered="false" />
        <div class="pagination-row">
          <NPagination v-model:page="blPage" :page-count="Math.ceil(blTotal / blPageSize)" :page-size="blPageSize" @update:page="loadData" @update:page-size="s => { blPageSize = s; loadData() }" />
        </div>
      </NTabPane>
    </NTabs>

    <!-- Add Whitelist Modal -->
    <NModal v-model:show="showAddWhitelist" preset="card" :title="t('access.addToWhitelist')" style="width: 400px">
      <NFormItem :label="t('auth.username')">
        <NInput v-model:value="newUsername" :placeholder="t('access.usernamePlaceholder')" @keyup.enter="addToWhitelist" />
      </NFormItem>
      <template #footer>
        <div style="display: flex; justify-content: flex-end; gap: 8px">
          <NButton @click="showAddWhitelist = false">{{ t('common.cancel') }}</NButton>
          <NButton type="primary" :loading="saving" @click="addToWhitelist">{{ t('common.confirm') }}</NButton>
        </div>
      </template>
    </NModal>

    <!-- Batch Add Modal -->
    <NModal v-model:show="showBatchAdd" preset="card" :title="t('access.batchImport')" style="width: 500px">
      <p style="color: #8b949e; margin-bottom: 12px; font-size: 13px">{{ t('access.batchImportHint') }}</p>
      <NInput v-model:value="batchUsernames" type="textarea" :rows="6" :placeholder="t('access.batchImportPlaceholder')" />
      <template #footer>
        <div style="display: flex; justify-content: flex-end; gap: 8px">
          <NButton @click="showBatchAdd = false">{{ t('common.cancel') }}</NButton>
          <NButton type="primary" :loading="saving" @click="batchAddWhitelist">{{ t('common.confirm') }}</NButton>
        </div>
      </template>
    </NModal>

    <!-- Ban Modal -->
    <NModal v-model:show="showBan" preset="card" :title="t('access.addToBlacklist')" style="width: 400px">
      <NForm label-placement="left" :label-width="80">
        <NFormItem :label="t('auth.username')">
          <NInput v-model:value="banUsername" :placeholder="t('access.usernamePlaceholder')" />
        </NFormItem>
        <NFormItem :label="t('access.banReason')">
          <NInput v-model:value="banReason" type="textarea" :rows="2" :placeholder="t('access.banReasonPlaceholder')" />
        </NFormItem>
      </NForm>
      <template #footer>
        <div style="display: flex; justify-content: flex-end; gap: 8px">
          <NButton @click="showBan = false">{{ t('common.cancel') }}</NButton>
          <NButton type="error" :loading="saving" @click="confirmBan">{{ t('common.confirm') }}</NButton>
        </div>
      </template>
    </NModal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h } from 'vue'
import { useI18n } from 'vue-i18n'
import { NTabs, NTabPane, NButton, NDataTable, NDataTableColumns, NModal, NForm, NFormItem, NInput, NPagination, useMessage } from 'naive-ui'
import { accessApi } from '../api'
import { useTimezone } from '../composables/useTimezone'

const { t } = useI18n()
const { formatRelativeTime } = useTimezone()
const message = useMessage()
const loading = ref(false)
const saving = ref(false)
const tab = ref('whitelist')
const whitelist = ref<any[]>([])
const blacklist = ref<any[]>([])
const whitelistEnabled = ref(false)

const wlPage = ref(1)
const wlPageSize = ref(20)
const wlTotal = ref(0)
const blPage = ref(1)
const blPageSize = ref(20)
const blTotal = ref(0)

const showAddWhitelist = ref(false)
const newUsername = ref('')
const showBatchAdd = ref(false)
const batchUsernames = ref('')
const showBan = ref(false)
const banUsername = ref('')
const banReason = ref('')

const wlColumns: NDataTableColumns<any> = [
  { title: () => t('table.username'), key: 'username', width: 200 },
  { title: () => t('table.addedAt'), key: 'added_at', render: (row: any) => formatRelativeTime(row.added_at) },
  {
    title: () => t('common.action'),
    key: 'action',
    width: 120,
    render: (row: any) => h(NButton, {
      size: 'small',
      type: 'error',
      ghost: true,
      onClick: () => removeFromWhitelist(row)
    }, { default: () => t('common.delete') })
  }
]

const blColumns: NDataTableColumns<any> = [
  { title: () => t('table.username'), key: 'username', width: 160 },
  {
    title: () => t('table.reason'),
    key: 'reason',
    render: (row: any) => row.reason || h('span', { style: 'color: #8b949e' }, '-')
  },
  { title: () => t('table.addedAt'), key: 'added_at', width: 180, render: (row: any) => formatRelativeTime(row.added_at) },
  {
    title: () => t('common.action'),
    key: 'action',
    width: 120,
    render: (row: any) => h(NButton, {
      size: 'small',
      type: 'success',
      ghost: true,
      onClick: () => unbanPlayer(row)
    }, { default: () => t('access.removeFromBlacklist') })
  }
]

async function toggleWhitelist() {
  const newState = !whitelistEnabled.value
  try {
    await accessApi.toggleWhitelist(newState)
    whitelistEnabled.value = newState
    message.success(newState ? t('access.whitelistEnabled') : t('access.whitelistDisabled'))
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  }
}

async function addToWhitelist() {
  if (!newUsername.value) return
  saving.value = true
  try {
    await accessApi.addWhitelist(newUsername.value)
    message.success(t('common.addSuccess'))
    showAddWhitelist.value = false
    newUsername.value = ''
    loadData()
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  } finally {
    saving.value = false
  }
}

async function batchAddWhitelist() {
  const usernames = batchUsernames.value.split('\n').map(s => s.trim()).filter(Boolean)
  if (!usernames.length) return
  saving.value = true
  try {
    for (const username of usernames) {
      await accessApi.addWhitelist(username)
    }
    message.success(t('common.addSuccess'))
    showBatchAdd.value = false
    batchUsernames.value = ''
    loadData()
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  } finally {
    saving.value = false
  }
}

async function removeFromWhitelist(row: any) {
  try {
    await accessApi.removeWhitelist(row.username)
    message.success(t('common.deleteSuccess'))
    loadData()
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  }
}

async function confirmBan() {
  if (!banUsername.value) return
  saving.value = true
  try {
    await accessApi.addBlacklist(banUsername.value, banReason.value || undefined)
    message.success(t('common.addSuccess'))
    showBan.value = false
    banUsername.value = ''
    banReason.value = ''
    loadData()
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  } finally {
    saving.value = false
  }
}

async function unbanPlayer(row: any) {
  try {
    await accessApi.removeBlacklist(row.username)
    message.success(t('common.deleteSuccess'))
    loadData()
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  }
}

async function loadData() {
  loading.value = true
  try {
    const [wl, bl] = await Promise.all([
      accessApi.getWhitelist({ limit: wlPageSize.value, offset: (wlPage.value - 1) * wlPageSize.value }),
      accessApi.getBlacklist({ limit: blPageSize.value, offset: (blPage.value - 1) * blPageSize.value })
    ])
    whitelist.value = wl.list || []
    blacklist.value = bl.list || []
    wlTotal.value = wl.total || 0
    blTotal.value = bl.total || 0
    if (wl.whitelist_enabled !== undefined) {
      whitelistEnabled.value = wl.whitelist_enabled
    }
  } catch (e) { console.error(e) }
  finally { loading.value = false }
}

onMounted(loadData)
</script>

<style scoped>
.page-container {
  padding: 24px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.page-title {
  color: #e6edf3;
  font-size: 22px;
  font-weight: 600;
  margin: 0 0 4px;
}

.page-subtitle {
  color: #8b949e;
  font-size: 13px;
  margin: 0;
}

.access-tabs {
  background: #161b22;
  border: 1px solid #21262d;
  border-radius: 8px;
  overflow: hidden;
}

:deep(.n-tabs-nav) {
  padding: 0 16px;
  border-bottom: 1px solid #21262d;
}

:deep(.n-tab-pane) {
  padding: 16px !important;
}

.tab-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.whitelist-status {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  color: #8b949e;
}

.whitelist-status.active {
  color: #3fb950;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #8b949e;
  flex-shrink: 0;
}

.whitelist-status.active .status-dot {
  background: #3fb950;
}

.pagination-row {
  display: flex;
  justify-content: flex-end;
  margin-top: 12px;
}
</style>
