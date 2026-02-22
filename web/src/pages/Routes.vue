<template>
  <div class="page-container">
    <div class="page-header">
      <div>
        <h1 class="page-title">{{ t('nav.routes') }}</h1>
        <p class="page-subtitle">{{ t('routes.subtitle') }}</p>
      </div>
      <div style="display:flex;gap:8px;align-items:center">
        <NButton @click="exportRoutes">{{ t('routes.exportRoutes') }}</NButton>
        <NButton @click="triggerImport">{{ t('routes.importRoutes') }}</NButton>
        <NButton type="primary" @click="openAdd">
          <template #icon><span>+</span></template>
          {{ t('routes.addDomainRoute') }}
        </NButton>
      </div>
      <input ref="routeFileInput" type="file" accept="application/json,.json" style="display:none" @change="onImportFile" />
    </div>

    <!-- Route List -->
    <div class="routes-table-wrap">
      <NDataTable
        :columns="columns"
        :data="routes"
        :loading="loading"
        :bordered="false"
        size="small"
      />
      <div class="pagination-row">
        <NPagination
          v-model:page="page"
          :page-count="Math.ceil(total / pageSize)"
          :page-size="pageSize"
          @update:page="loadData"
        />
      </div>
    </div>

    <!-- Empty State -->
    <div v-if="!loading && routes.length === 0" class="empty-state">
      <div class="empty-icon">🗺</div>
      <div class="empty-text">{{ t('routes.emptyHint') }}</div>
      <NButton type="primary" @click="openAdd">{{ t('routes.addDomainRoute') }}</NButton>
    </div>

    <!-- Add/Edit Route Modal -->
    <NModal v-model:show="showModal" preset="card" :title="editingRoute ? t('routes.editDomainRoute') : t('routes.addDomainRoute')" style="width: 520px">
      <NForm :model="form" label-placement="left" :label-width="120">
        <NFormItem :label="t('table.pattern')" required>
          <div v-if="trustedDomain" class="prefix-input-row">
            <NInput
              v-model:value="form.patternPrefix"
              :placeholder="t('routes.prefixPlaceholder')"
              style="flex: 1"
              @input="onPrefixInput"
            />
            <span class="domain-suffix">.{{ trustedDomain }}</span>
          </div>
          <template v-else>
            <NInput
              v-model:value="form.pattern"
              :placeholder="t('routes.patternPlaceholder')"
              :status="patternError ? 'error' : undefined"
              @input="validatePattern"
            />
            <div v-if="patternError" class="form-error">{{ t('routes.patternInvalid') }}</div>
          </template>
        </NFormItem>

        <NFormItem :label="t('routes.selectBackend')">
          <NSelect
            v-model:value="form.backendId"
            :options="backendOptions"
            clearable
            :placeholder="t('routes.selectBackendPlaceholder')"
            @update:value="onBackendSelect"
          />
        </NFormItem>

        <NFormItem v-if="!form.backendId" :label="t('routes.targetAddr')">
          <NInput v-model:value="form.targetAddr" placeholder="127.0.0.1" />
        </NFormItem>

        <NFormItem v-if="!form.backendId" :label="t('routes.targetPort')">
          <NInputNumber v-model:value="form.targetPort" :min="1" :max="65535" style="width: 100%" />
        </NFormItem>

        <NFormItem :label="t('table.priority')">
          <NInputNumber v-model:value="form.priority" style="width: 100%" />
          <div class="form-hint">{{ t('routes.priorityHelp') }}</div>
        </NFormItem>
      </NForm>

      <div v-if="form.backendId" class="backend-preview">
        <div class="backend-preview-label">{{ t('routes.routeTarget') }}</div>
        <div class="backend-preview-value">
          {{ selectedBackendName }} ({{ selectedBackendAddr }})
        </div>
      </div>

      <template #footer>
        <div style="display: flex; justify-content: flex-end; gap: 8px">
          <NButton @click="showModal = false">{{ t('common.cancel') }}</NButton>
          <NButton type="primary" :loading="saving" @click="saveRoute">{{ t('common.confirm') }}</NButton>
        </div>
      </template>
    </NModal>

    <!-- Delete Confirmation -->
    <NModal v-model:show="showDeleteConfirm" preset="card" :title="t('common.confirmDelete')" style="width: 400px">
      <p style="color: #8b949e">{{ t('common.confirmDeleteMsg', { target: deletingRoute?.pattern }) }}</p>
      <template #footer>
        <div style="display: flex; justify-content: flex-end; gap: 8px">
          <NButton @click="showDeleteConfirm = false">{{ t('common.cancel') }}</NButton>
          <NButton type="error" :loading="saving" @click="confirmDelete">{{ t('common.delete') }}</NButton>
        </div>
      </template>
    </NModal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h, computed, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  NDataTable, NDataTableColumns, NButton, NModal, NForm, NFormItem,
  NInput, NInputNumber, NSelect, NPagination, NTag, NTooltip, useMessage
} from 'naive-ui'
import { routesApi, backendApi, configApi, pingApi } from '../api'

const { t } = useI18n()
const message = useMessage()

const loading = ref(false)
const saving = ref(false)
const routes = ref<any[]>([])
const backends = ref<any[]>([])
const routeFileInput = ref<HTMLInputElement | null>(null)
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const trustedDomain = ref('')
// MOTD preview state
const pingCache = ref<Record<number, any>>({})
const pinging = ref<Record<number, boolean>>({})

// Modal state
const showModal = ref(false)
const editingRoute = ref<any>(null)
const patternError = ref(false)

const form = reactive({
  pattern: '',
  patternPrefix: '', // Only used in trustedDomain mode
  backendId: null as number | null,
  targetAddr: '',
  targetPort: 25565,
  priority: 0
})

// Delete confirm
const showDeleteConfirm = ref(false)
const deletingRoute = ref<any>(null)

const backendOptions = computed(() =>
  backends.value.map(b => ({
    label: b.name + (b.maintenance ? ` [${t('backend.maintenanceEnabled')}]` : ''),
    value: b.id,
    disabled: !b.enabled
  }))
)

const selectedBackendName = computed(() => {
  if (!form.backendId) return ''
  return backends.value.find(b => b.id === form.backendId)?.name || ''
})

const selectedBackendAddr = computed(() => {
  if (!form.backendId) return ''
  const b = backends.value.find(b => b.id === form.backendId)
  return b ? `${b.remote_address}:${b.remote_port}` : ''
})

function validatePattern() {
  if (!form.pattern) { patternError.value = false; return }
  try {
    new RegExp(form.pattern)
    patternError.value = false
  } catch {
    patternError.value = true
  }
}

function onPrefixInput() {
  // When using trustedDomain mode, automatically compose pattern
  if (trustedDomain.value) {
    form.pattern = form.patternPrefix
      ? `${form.patternPrefix}.${trustedDomain.value}`
      : trustedDomain.value
  }
}

async function pingServer(row: any) {
  const id = row.id
  if (pinging.value[id]) return
  const addr = row.target_addr || (backends.value.find((b: any) => b.id === row.backend_id)?.remote_address)
  const port = row.target_port || (backends.value.find((b: any) => b.id === row.backend_id)?.remote_port) || 25565
  if (!addr) { message.warning(t('routes.pingNoAddr')); return }
  pinging.value[id] = true
  try {
    const res: any = await pingApi.ping(addr, port)
    pingCache.value[id] = res
  } catch (e: any) {
    pingCache.value[id] = { error: e.message }
  } finally {
    pinging.value[id] = false
  }
}

function onBackendSelect(id: number | null) {
  if (id) {
    const b = backends.value.find(b => b.id === id)
    if (b) {
      form.targetAddr = b.remote_address
      form.targetPort = b.remote_port
    }
  }
}

function openAdd() {
  editingRoute.value = null
  form.pattern = ''
  form.patternPrefix = ''
  form.backendId = null
  form.targetAddr = ''
  form.targetPort = 25565
  form.priority = 0
  patternError.value = false
  showModal.value = true
}

function openEdit(row: any) {
  editingRoute.value = row
  form.pattern = row.pattern
  // If using trustedDomain mode, try to split prefix
  if (trustedDomain.value && row.pattern && row.pattern.endsWith(`.${trustedDomain.value}`)) {
    form.patternPrefix = row.pattern.slice(0, -(trustedDomain.value.length + 1))
  } else if (trustedDomain.value && row.pattern === trustedDomain.value) {
    form.patternPrefix = ''
  } else {
    form.patternPrefix = row.pattern
  }
  form.backendId = row.backend_id || null
  form.targetAddr = row.target_addr || ''
  form.targetPort = row.target_port || 25565
  form.priority = row.priority || 0
  patternError.value = false
  showModal.value = true
}

async function saveRoute() {
  if (!form.pattern || patternError.value) {
    message.error(t('routes.patternInvalid'))
    return
  }
  if (!form.backendId && !form.targetAddr) {
    message.error(t('routes.targetRequired'))
    return
  }
  saving.value = true
  try {
    const data = {
      pattern: form.pattern,
      backend_id: form.backendId,
      target_addr: form.backendId ? undefined : form.targetAddr,
      target_port: form.backendId ? undefined : form.targetPort,
      priority: form.priority
    }
    if (editingRoute.value) {
      await routesApi.updateDomainRoute(editingRoute.value.id, data)
    } else {
      await routesApi.addDomainRoute(data)
    }
    message.success(t('common.success'))
    showModal.value = false
    await loadData()
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  } finally {
    saving.value = false
  }
}

function openDelete(row: any) {
  deletingRoute.value = row
  showDeleteConfirm.value = true
}

async function confirmDelete() {
  if (!deletingRoute.value) return
  saving.value = true
  try {
    await routesApi.deleteDomainRoute(deletingRoute.value.id)
    message.success(t('common.deleteSuccess'))
    showDeleteConfirm.value = false
    await loadData()
  } catch (e: any) {
    message.error(e.message || t('common.deleteFailed'))
  } finally {
    saving.value = false
  }
}

const columns: NDataTableColumns<any> = [
  {
    title: () => t('table.pattern'),
    key: 'pattern',
    render: (row: any) => h('code', { class: 'pattern-code' }, row.pattern)
  },
  {
    title: () => t('routes.selectBackend'),
    key: 'backend',
    render: (row: any) => {
      if (row.backend_id) {
        const b = backends.value.find(b => b.id === row.backend_id)
        if (b) {
          return h(NTooltip, { trigger: 'hover', placement: 'top' }, {
            default: () => `${b.remote_address}:${b.remote_port}`,
            trigger: () => h(NTag, { type: 'info', size: 'small', style: 'cursor:default' }, { default: () => b.name })
          })
        }
        return h('span', { class: 'text-muted' }, `#${row.backend_id}`)
      }
      return h('span', { class: 'text-muted font-mono' }, `${row.target_addr || ''}:${row.target_port || ''}`)
    }
  },
  {
    title: () => t('table.priority'),
    key: 'priority',
    width: 80,
    render: (row: any) => h(NTag, { size: 'small', type: 'default' }, { default: () => row.priority ?? 0 })
  },
  {
    title: () => t('routes.motdPreview'),
    key: 'ping',
    width: 240,
    render: (row: any) => {
      const ping = pingCache.value[row.id]
      const loading = !!pinging.value[row.id]
      const btnEl = h(NButton, {
        size: 'small',
        loading,
        onClick: () => pingServer(row)
      }, { default: () => ping ? t('routes.pingRefresh') : t('routes.pingPreview') })
      if (!ping) return btnEl
      if (ping.error) return h('div', { style: 'display:flex;gap:6px;align-items:center' }, [
        btnEl,
        h('span', { style: 'color:#f85149;font-size:11px' }, t('routes.pingFailed'))
      ])
      return h('div', { class: 'ping-result' }, [
        btnEl,
        h('div', { class: 'ping-card' }, [
          ping.favicon ? h('img', { src: ping.favicon, class: 'ping-favicon' }) : null,
          h('div', { class: 'ping-info' }, [
            h('div', { class: 'ping-version' }, `${ping.version_name} · ${ping.online}/${ping.max} · ${ping.latency_ms}ms`),
            h('div', { class: 'ping-motd', innerHTML: renderMotdHtml(ping.description) }),
          ])
        ])
      ])
    }
  },
  {
    title: () => t('common.action'),
    key: 'action',
    width: 120,
    render: (row: any) => h('div', { style: 'display:flex;gap:6px' }, [
      h(NButton, { size: 'small', onClick: () => openEdit(row) }, { default: () => t('common.edit') }),
      h(NButton, { size: 'small', type: 'error', onClick: () => openDelete(row) }, { default: () => t('common.delete') })
    ])
  }
]

// Render MOTD (supports § color codes and JSON format)
const mcColorMap: Record<string, string> = {
  '0': '#000', '1': '#0000AA', '2': '#00AA00', '3': '#00AAAA',
  '4': '#AA0000', '5': '#AA00AA', '6': '#FFAA00', '7': '#AAAAAA',
  '8': '#555555', '9': '#5555FF', 'a': '#55FF55', 'b': '#55FFFF',
  'c': '#FF5555', 'd': '#FF55FF', 'e': '#FFFF55', 'f': '#FFFFFF',
}

function renderMotdHtml(motd: any): string {
  if (!motd) return ''
  if (typeof motd === 'object') {
    try { motd = JSON.stringify(motd) } catch { return '' }
  }
  // Try to parse as JSON chat component
  try {
    const obj = JSON.parse(motd)
    if (typeof obj === 'object') return renderChatComponent(obj)
  } catch { /* Plain text */ }
  return renderLegacyText(motd)
}

function renderChatComponent(comp: any): string {
  if (typeof comp === 'string') return renderLegacyText(comp)
  if (Array.isArray(comp)) return comp.map(renderChatComponent).join('')
  const color = comp.color ? (mcColorMap[comp.color] || comp.color) : ''
  let style = color ? `color:${color};` : ''
  if (comp.bold) style += 'font-weight:bold;'
  if (comp.italic) style += 'font-style:italic;'
  const text = comp.text ? renderLegacyText(comp.text) : ''
  const extra = (comp.extra || []).map(renderChatComponent).join('')
  return `<span style="${style}">${text}${extra}</span>`
}

function renderLegacyText(text: string): string {
  let html = ''
  let i = 0
  while (i < text.length) {
    if ((text[i] === '§' || text[i] === '&') && i + 1 < text.length) {
      const code = text[i + 1].toLowerCase()
      if (mcColorMap[code]) {
        html += `<span style="color:${mcColorMap[code]}">`
      }
      i += 2
    } else {
      html += text[i] === '<' ? '&lt;' : text[i] === '>' ? '&gt;' : text[i]
      i++
    }
  }
  return html || text
}

async function loadData() {
  loading.value = true
  try {
    const res = await routesApi.getDomainRoutes({ limit: pageSize.value, offset: (page.value - 1) * pageSize.value })
    routes.value = res.routes || []
    total.value = res.total || 0
  } catch (e) { console.error(e) }
  finally { loading.value = false }
}

async function loadBackends() {
  try {
    const [backendsRes, configRes] = await Promise.all([
      backendApi.list(),
      configApi.get()
    ])
    backends.value = (backendsRes as any).list || []
    trustedDomain.value = (configRes as any).trusted_domain || ''
  } catch (e) { console.error(e) }
}

async function exportRoutes() {
  try {
    const res = await routesApi.getDomainRoutes({ limit: 10000 })
    const allRoutes = res.routes || []
    const exportData = {
      version: 1,
      exported_at: new Date().toISOString(),
      routes: allRoutes.map((r: any) => {
        const b = backends.value.find((b: any) => b.id === r.backend_id)
        return {
          pattern: r.pattern,
          target_addr: r.target_addr || '',
          target_port: r.target_port || 25565,
          priority: r.priority ?? 0,
          backend_name: b?.name || null
        }
      })
    }
    const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = 'routes_config.json'
    link.click()
    message.success(t('common.success'))
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  }
}

function triggerImport() {
  routeFileInput.value?.click()
}

async function onImportFile(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  ;(e.target as HTMLInputElement).value = ''
  try {
    const text = await file.text()
    const data = JSON.parse(text)
    const routesToImport: any[] = data.routes || []
    if (!routesToImport.length) {
      message.warning(t('routes.importEmpty'))
      return
    }
    let imported = 0
    for (const r of routesToImport) {
      if (!r.pattern) continue
      // Try to find backend_id by name
      const b = r.backend_name ? backends.value.find((b: any) => b.name === r.backend_name) : null
      const payload: any = {
        pattern: r.pattern,
        priority: r.priority ?? 0,
      }
      if (b) {
        payload.backend_id = b.id
        payload.target_addr = b.remote_address
        payload.target_port = b.remote_port
      } else {
        payload.target_addr = r.target_addr || ''
        payload.target_port = r.target_port || 25565
      }
      try {
        await routesApi.addDomainRoute(payload)
        imported++
      } catch { /* skip duplicates */ }
    }
    message.success(t('routes.importSuccess', { n: imported }))
    await loadData()
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  }
}

onMounted(() => {
  loadData()
  loadBackends()
})
</script>

<style scoped>
.page-container {
  padding: 24px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
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

.routes-table-wrap {
  background: #161b22;
  border: 1px solid #21262d;
  border-radius: 8px;
  overflow: hidden;
}

.pagination-row {
  display: flex;
  justify-content: flex-end;
  padding: 12px 16px;
  border-top: 1px solid #21262d;
}

.empty-state {
  text-align: center;
  padding: 60px 24px;
  color: #8b949e;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

.empty-text {
  font-size: 15px;
  margin-bottom: 20px;
}

.pattern-code {
  font-family: 'Consolas', 'Monaco', monospace;
  background: #21262d;
  color: #79c0ff;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.backend-cell {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.backend-cell-name {
  font-weight: 500;
  color: #e6edf3;
  font-size: 13px;
}

.backend-cell-addr {
  font-family: monospace;
  color: #8b949e;
  font-size: 12px;
}

.text-muted {
  color: #8b949e;
  font-family: monospace;
  font-size: 12px;
}

.form-error {
  color: #f85149;
  font-size: 12px;
  margin-top: 4px;
}

.form-hint {
  color: #8b949e;
  font-size: 12px;
  margin-top: 4px;
}

.backend-preview {
  background: #0d1117;
  border: 1px solid #21262d;
  border-radius: 6px;
  padding: 10px 14px;
  margin-top: 4px;
}

.backend-preview-label {
  color: #8b949e;
  font-size: 12px;
  margin-bottom: 4px;
}

.backend-preview-value {
  color: #e6edf3;
  font-size: 14px;
  font-weight: 500;
}

.prefix-input-row {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
}

.domain-suffix {
  color: #8b949e;
  font-size: 13px;
  white-space: nowrap;
  flex-shrink: 0;
}

.ping-result {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.ping-card {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  background: #0d1117;
  border: 1px solid #21262d;
  border-radius: 6px;
  padding: 6px 8px;
}

.ping-favicon {
  width: 32px;
  height: 32px;
  border-radius: 3px;
  image-rendering: pixelated;
  flex-shrink: 0;
}

.ping-info {
  flex: 1;
  min-width: 0;
}

.ping-version {
  color: #6e7681;
  font-size: 10px;
  margin-bottom: 2px;
}

.ping-motd {
  font-family: 'Consolas', monospace;
  font-size: 11px;
  color: #aaa;
  line-height: 1.4;
  word-break: break-all;
}
</style>
