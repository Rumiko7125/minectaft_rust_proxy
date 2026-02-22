<template>
  <div class="page-container">
    <div class="page-header">
      <div>
        <h1 class="page-title">{{ t('twoFactor.title') }}</h1>
        <p class="page-subtitle">{{ t('twoFactor.desc') }}</p>
      </div>
      <NButton size="small" @click="loadData" :loading="loading">
        <template #icon><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg></template>
        {{ t('common.refresh') }}
      </NButton>
    </div>

    <div class="toolbar">
      <NInput v-model:value="searchText" :placeholder="t('twoFactor.searchPlaceholder')" clearable style="width: 220px" @keyup.enter="loadData" />
      <NButton @click="loadData" size="small">{{ t('common.search') }}</NButton>
    </div>

    <NDataTable :columns="columns" :data="players" :loading="loading" :bordered="false" size="small" />
    <div class="pagination">
      <NPagination v-model:page="page" :page-count="Math.ceil(total / pageSize)" :page-size="pageSize" @update:page="loadData" @update:page-size="handlePageSizeChange" />
    </div>

    <!-- QR Code Modal -->
    <NModal v-model:show="showQR" preset="card" :title="t('twoFactor.viewQR')" style="width: 350px">
      <div class="qr-container">
        <img v-if="qrDataUrl" :src="qrDataUrl" alt="QR Code" class="qr-image" />
        <p class="secret-text">{{ qrSecret }}</p>
      </div>
    </NModal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h } from 'vue'
import { useI18n } from 'vue-i18n'
import { NDataTable, NDataTableColumns, NButton, NModal, NInput, useMessage, NPagination } from 'naive-ui'
import { twoFactorApi } from '../api'
import QRCode from 'qrcode'
import { useTimezone } from '../composables/useTimezone'

const { t } = useI18n()
const { formatRelativeTime } = useTimezone()
const message = useMessage()
const loading = ref(false)
const players = ref<any[]>([])
const searchText = ref('')

// Pagination
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)

function handlePageSizeChange(newSize: number) {
  pageSize.value = newSize
  loadData()
}

// QR Code
const showQR = ref(false)
const qrSecret = ref('')
const qrDataUrl = ref('')

const columns: NDataTableColumns<any> = [
  { title: () => t('auth.username'), key: 'username' },
  {
    title: () => t('account.bound'),
    key: 'bound',
    render: (row: any) => row.bound ? t('common.enabled') : t('common.disabled')
  },
  {
    title: () => t('table.boundTime'),
    key: 'created_at',
    render: (row: any) => formatRelativeTime(row.created_at)
  },
  {
    title: () => t('common.action'),
    key: 'action',
    width: 200,
    render: (row: any) => {
      return h('div', { style: 'display: flex; gap: 8px;' }, [
        h(NButton, {
          size: 'small',
          onClick: () => viewQR(row)
        }, { default: () => t('twoFactor.viewQR') }),
        h(NButton, {
          size: 'small',
          type: 'error',
          onClick: () => handleUnbind(row)
        }, { default: () => t('twoFactor.unbind') })
      ])
    }
  }
]

async function viewQR(row: any) {
  try {
    const res = await twoFactorApi.getQR(row.username)
    qrSecret.value = res.secret
    // Generate QR code image
    qrDataUrl.value = await QRCode.toDataURL(res.qr_data_url, {
      width: 200,
      margin: 2
    })
    showQR.value = true
  } catch (e: any) {
    message.error(e.response?.data?.message || 'Failed to get QR code')
  }
}

async function handleUnbind(row: any) {
  try {
    await twoFactorApi.unbind(row.username)
    message.success(t('twoFactor.unbindSuccess'))
    loadData()
  } catch (e: any) {
    message.error(e.response?.data?.message || 'Unbind failed')
  }
}

async function loadData() {
  loading.value = true
  try {
    const res = await twoFactorApi.list({ limit: pageSize.value, offset: (page.value - 1) * pageSize.value })
    let users = res.users || []
    if (searchText.value) {
      const q = searchText.value.toLowerCase()
      users = users.filter((u: any) => u.username?.toLowerCase().includes(q))
    }
    players.value = users
    total.value = res.total || 0
  } catch (e) { console.error(e) }
  finally { loading.value = false }
}

onMounted(loadData)
</script>

<style scoped>
.page-container { padding: 24px; }

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

.toolbar {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 16px;
}

.pagination {
  display: flex;
  justify-content: flex-end;
  margin-top: 16px;
}

.qr-container { text-align: center; }
.qr-image { display: block; margin: 0 auto 16px; }
.secret-text { color: #8b949e; font-family: monospace; word-break: break-all; }
</style>
