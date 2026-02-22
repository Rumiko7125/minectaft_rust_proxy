<template>
  <div class="page">
    <div class="page-header">
      <div>
        <h1 class="page-title">{{ t('backend.title') }}</h1>
        <p class="page-desc">{{ t('backend.domainRulesHint') }}</p>
      </div>
      <div style="display:flex;gap:8px">
        <NButton size="small" @click="loadBackends" :loading="loading">
          <template #icon><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg></template>
          {{ t('common.refresh') }}
        </NButton>
        <NButton type="primary" @click="openAdd">
          <template #icon><span>+</span></template>
          {{ t('backend.add') }}
        </NButton>
      </div>
    </div>

    <!-- Empty State -->
    <NEmpty v-if="!loading && backends.length === 0" :description="t('backend.emptyHint')" class="empty-state">
      <template #extra>
        <NButton type="primary" @click="openAdd">{{ t('backend.add') }}</NButton>
      </template>
    </NEmpty>

    <!-- Backend Server Card Grid -->
    <div v-else class="backend-grid">
      <div v-for="b in backends" :key="b.id" class="backend-card" :class="{ 'is-default': b.is_default, 'is-maintenance': b.maintenance, 'is-disabled': !b.enabled }">
        <!-- Card Header -->
        <div class="card-header">
          <div class="card-title-row">
            <div class="backend-name">{{ b.name }}</div>
            <div class="badge-row">
              <span v-if="b.is_default" class="badge badge-default">{{ t('backend.default') }}</span>
              <span class="badge" :class="b.enabled ? 'badge-enabled' : 'badge-disabled'">
                {{ b.enabled ? t('backend.enabled') : t('backend.disabled') }}
              </span>
              <span v-if="b.maintenance" class="badge badge-maintenance">{{ t('backend.maintenanceEnabled') }}</span>
            </div>
          </div>
          <div class="backend-addr">
            <span class="addr-icon">→</span>
            <code>{{ b.remote_address }}:{{ b.remote_port }}</code>
          </div>
        </div>

        <!-- Stats -->
        <div class="card-stats">
          <div class="stat-item">
            <span class="stat-label">{{ t('backend.onlinePlayers') }}</span>
            <span class="stat-value">{{ getOnlineCount(b.name) }}</span>
            <span class="stat-suffix" v-if="b.max_player >= 0">/ {{ b.max_player }}</span>
            <span class="stat-suffix" v-else>/ ∞</span>
          </div>
          <div class="stat-item" v-if="b.log_dir">
            <span class="stat-label">{{ t('backend.logDir') }}</span>
            <code class="stat-code">{{ b.log_dir }}</code>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="card-actions">
          <NButton size="small" @click="edit(b)">{{ t('common.edit') }}</NButton>
          <NButton size="small" :type="b.enabled ? 'warning' : 'success'" @click="toggleEnabled(b)">
            {{ b.enabled ? t('backend.disable') : t('backend.enable') }}
          </NButton>
          <NButton size="small" :type="b.maintenance ? 'default' : 'error'" @click="openMaintenance(b)">
            {{ b.maintenance ? t('backend.maintenanceOff') : t('backend.maintenanceOn') }}
          </NButton>
          <NButton v-if="!b.is_default" size="small" @click="setDefault(b)">{{ t('backend.setDefault') }}</NButton>
          <NButton v-if="b.is_default" size="small" type="warning" @click="unsetDefault()">{{ t('backend.unsetDefault') }}</NButton>
          <NButton v-if="!b.is_default" size="small" type="error" @click="confirmDelete(b)">{{ t('common.delete') }}</NButton>
        </div>
      </div>
    </div>

    <!-- Add/Edit Modal -->
    <NModal v-model:show="showModal" preset="card" :title="editingId ? t('backend.edit') : t('backend.add')" style="width: 640px; max-height: 90vh; overflow-y: auto">
      <NTabs type="line" animated>
        <!-- Basic Info -->
        <NTabPane name="basic" :tab="t('backend.basicInfo')">
          <NForm :model="form" label-placement="left" :label-width="120">
            <NFormItem :label="t('backend.name')" required>
              <NInput v-model:value="form.name" :placeholder="t('backend.namePlaceholder')" />
            </NFormItem>
            <NFormItem :label="t('backend.remoteAddress')" required>
              <NInput v-model:value="form.remote_address" :placeholder="t('backend.remoteAddressPlaceholder')" />
            </NFormItem>
            <NFormItem :label="t('backend.remotePort')">
              <NInputNumber v-model:value="form.remote_port" style="width: 100%" :min="1" :max="65535" />
            </NFormItem>
            <NFormItem :label="t('backend.maxPlayer')">
              <NInputNumber v-model:value="form.max_player" style="width: 100%" :min="-1" />
              <template #feedback>{{ t('backend.maxPlayerHint') }}</template>
            </NFormItem>
            <NFormItem :label="t('backend.limboMessage')">
              <NInput v-model:value="form.limbo_message" :placeholder="t('backend.limboMessagePlaceholder')" />
            </NFormItem>
            <NFormItem :label="t('backend.limboLanguage')">
              <NSelect v-model:value="form.language" :options="languageOptions" style="width: 100%" />
              <template #feedback>{{ t('backend.limboLanguageHint') }}</template>
            </NFormItem>
          </NForm>
        </NTabPane>

        <!-- MOTD Editor -->
        <NTabPane name="motd" :tab="t('backend.motdEditor')">
          <div class="motd-section">
            <!-- Live Preview -->
            <div class="motd-preview-box">
              <div class="motd-preview-header">{{ t('backend.motdPreview') }}</div>
              <div class="motd-preview-content">
                <div class="mc-preview-server">
                  <div class="mc-server-icon">
                    <img v-if="motdFields.favicon" :src="motdFields.favicon" class="favicon-preview" />
                    <div v-else class="favicon-placeholder">?</div>
                  </div>
                  <div class="mc-server-info">
                    <div class="mc-server-toprow">
                      <span class="mc-server-name">{{ form.name || 'Server' }}</span>
                      <span class="mc-server-version">Auto</span>
                    </div>
                    <div class="mc-server-motd">
                      <div v-html="renderLegacyText(motdFields.line1)" class="motd-line"></div>
                      <div v-html="renderLegacyText(motdFields.line2)" class="motd-line" v-if="motdFields.line2"></div>
                    </div>
                    <div class="mc-server-players">
                      <span class="players-count">{{ form.max_player < 0 ? '∞' : form.max_player }} max</span>
                      <span v-if="motdFields.sampleText" class="players-hover" :title="motdFields.sampleText">▲ hover</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Server Icon -->
            <div class="motd-field-row">
              <div class="motd-field-label">{{ t('backend.favicon') }}</div>
              <div class="favicon-upload-row">
                <img v-if="motdFields.favicon" :src="motdFields.favicon" class="favicon-thumb" />
                <NButton size="small" @click="triggerFaviconUpload">{{ t('backend.uploadFavicon') }}</NButton>
                <NButton v-if="motdFields.favicon" size="small" type="error" @click="motdFields.favicon = ''">{{ t('common.delete') }}</NButton>
                <span class="motd-hint">{{ t('backend.faviconHint') }}</span>
                <input ref="faviconInput" type="file" accept="image/png,image/gif" style="display:none" @change="onFaviconUpload" />
              </div>
            </div>

            <!-- Description Line 1 -->
            <div class="motd-field-row">
              <div class="motd-field-label">{{ t('backend.motdLine1') }}</div>
              <div class="motd-input-with-colors">
                <NInput v-model:value="motdFields.line1" :placeholder="t('backend.motdLine1Placeholder')" />
                <div class="color-btn-row">
                  <span
                    v-for="cc in colorCodes" :key="cc.code"
                    class="color-btn" :style="{ background: cc.hex === '#000000' ? '#333' : cc.hex }"
                    :title="`§${cc.code} ${cc.name}`"
                    @click="insertColor('line1', cc.code)"
                  >§</span>
                  <span class="color-btn fmt-btn" title="§l Bold" @click="insertColor('line1', 'l')">B</span>
                  <span class="color-btn fmt-btn" title="§o Italic" @click="insertColor('line1', 'o')" style="font-style:italic">I</span>
                  <span class="color-btn fmt-btn" title="§r Reset" @click="insertColor('line1', 'r')">R</span>
                </div>
              </div>
            </div>

            <!-- Description Line 2 -->
            <div class="motd-field-row">
              <div class="motd-field-label">{{ t('backend.motdLine2') }}</div>
              <div class="motd-input-with-colors">
                <NInput v-model:value="motdFields.line2" :placeholder="t('backend.motdLine2Placeholder')" />
                <div class="color-btn-row">
                  <span
                    v-for="cc in colorCodes" :key="cc.code"
                    class="color-btn" :style="{ background: cc.hex === '#000000' ? '#333' : cc.hex }"
                    :title="`§${cc.code} ${cc.name}`"
                    @click="insertColor('line2', cc.code)"
                  >§</span>
                  <span class="color-btn fmt-btn" title="§l Bold" @click="insertColor('line2', 'l')">B</span>
                  <span class="color-btn fmt-btn" title="§o Italic" @click="insertColor('line2', 'o')" style="font-style:italic">I</span>
                  <span class="color-btn fmt-btn" title="§r Reset" @click="insertColor('line2', 'r')">R</span>
                </div>
              </div>
            </div>

            <!-- Hover Tooltip (sample players) -->
            <div class="motd-field-row motd-field-row--top">
              <div class="motd-field-label">{{ t('backend.hoverTooltip') }}</div>
              <div style="flex:1">
                <NInput
                  v-model:value="motdFields.sampleText"
                  type="textarea"
                  :rows="3"
                  :placeholder="t('backend.hoverTooltipPlaceholder')"
                />
                <div class="motd-hint">{{ t('backend.hoverTooltipHint') }}</div>
              </div>
            </div>
          </div>
        </NTabPane>

        <!-- Advanced Settings -->
        <NTabPane name="advanced" :tab="t('backend.advancedSettings')">
          <NForm :model="form" label-placement="left" :label-width="140">
            <NFormItem :label="t('backend.logDir')">
              <NInput v-model:value="form.log_dir" :placeholder="t('backend.logDirPlaceholder')" />
            </NFormItem>
            <NFormItem :label="t('backend.showLogLevel')">
              <NSelect v-model:value="form.show_log_level" :options="logLevelOptions" style="width: 100%" />
            </NFormItem>
            <NFormItem :label="t('backend.saveLogLevel')">
              <NSelect v-model:value="form.save_log_level" :options="logLevelOptions" style="width: 100%" />
            </NFormItem>
            <NFormItem :label="t('backend.pingPassthrough')">
              <NSwitch v-model:value="form.ping_passthrough" />
              <span style="color:#8b949e;font-size:12px;margin-left:8px">{{ t('backend.pingPassthroughHint') }}</span>
            </NFormItem>
            <NFormItem :label="t('backend.motdPassthrough')">
              <NSwitch v-model:value="form.motd_passthrough" />
              <span style="color:#8b949e;font-size:12px;margin-left:8px">{{ t('backend.motdPassthroughHint') }}</span>
            </NFormItem>
          </NForm>
        </NTabPane>
      </NTabs>

      <template #footer>
        <div style="display: flex; justify-content: flex-end; gap: 8px">
          <NButton @click="showModal = false">{{ t('common.cancel') }}</NButton>
          <NButton type="primary" @click="save" :loading="saving">{{ t('common.confirm') }}</NButton>
        </div>
      </template>
    </NModal>

    <!-- Maintenance Mode Modal -->
    <NModal v-model:show="showMaintenanceModal" preset="card" :title="t('backend.maintenance')" style="width: 440px">
      <div v-if="maintenanceTarget">
        <NAlert v-if="!maintenanceTarget.maintenance" type="warning" style="margin-bottom: 16px">
          {{ t('backend.maintenanceWarning') }}
        </NAlert>
        <NFormItem :label="t('backend.maintenanceMsg')">
          <NInput
            v-model:value="maintenanceMsg"
            :placeholder="t('backend.maintenanceMsgPlaceholder')"
            type="textarea"
            :rows="3"
          />
        </NFormItem>
        <div style="color: #8b949e; font-size: 12px">{{ t('backend.motdColorCodeHint') }}</div>
      </div>
      <template #footer>
        <div style="display: flex; justify-content: flex-end; gap: 8px">
          <NButton @click="showMaintenanceModal = false">{{ t('common.cancel') }}</NButton>
          <NButton
            :type="maintenanceTarget?.maintenance ? 'success' : 'error'"
            @click="applyMaintenance"
            :loading="maintenanceSaving"
          >
            {{ maintenanceTarget?.maintenance ? t('backend.maintenanceOff') : t('backend.maintenanceOn') }}
          </NButton>
        </div>
      </template>
    </NModal>

    <!-- Delete Confirmation -->
    <NModal v-model:show="showDeleteModal" preset="dialog" :title="t('common.confirmDelete')">
      <p>{{ t('backend.deleteConfirm', { name: deletingBackend?.name }) }}</p>
      <template #action>
        <NButton @click="showDeleteModal = false">{{ t('common.cancel') }}</NButton>
        <NButton type="error" @click="remove" :loading="deleting">{{ t('common.delete') }}</NButton>
      </template>
    </NModal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  NButton, NModal, NForm, NFormItem, NInput, NInputNumber, NSelect,
  NSwitch, NTabs, NTabPane, NAlert, NEmpty, useMessage
} from 'naive-ui'
import { backendApi, playersApi } from '../api'
import { useAppStore } from '../store/app'

const { t } = useI18n()
const message = useMessage()
const appStore = useAppStore()
const loading = ref(false)
const saving = ref(false)
const deleting = ref(false)
const backends = ref<any[]>([])
const showModal = ref(false)
const showDeleteModal = ref(false)
const showMaintenanceModal = ref(false)
const editingId = ref<number | null>(null)
const deletingBackend = ref<any>(null)
const maintenanceTarget = ref<any>(null)
const maintenanceMsg = ref('')
const maintenanceSaving = ref(false)
const motdJsonValid = ref(true)
const onlinePlayers = ref<any[]>([])
const faviconInput = ref<HTMLInputElement | null>(null)

const motdFields = reactive({
  favicon: '',
  line1: '',
  line2: '',
  sampleText: '',
})

const form = reactive({
  name: '',
  remote_address: '',
  remote_port: 25565,
  max_player: -1,
  motd_json: '',
  limbo_message: '',
  language: 'en',
  log_dir: '',
  show_log_level: 0,
  save_log_level: 0,
  ping_passthrough: false,
  motd_passthrough: false,
})

const previewName = computed(() => form.name)

const logLevelOptions = computed(() => [
  { label: t('backend.logLevel0'), value: 0 },
  { label: t('backend.logLevel1'), value: 1 },
  { label: t('backend.logLevel2'), value: 2 },
  { label: t('backend.logLevel3'), value: 3 },
  { label: t('backend.logLevel4'), value: 4 },
])

const languageOptions = [
  { label: '简体中文', value: 'zh-CN' },
  { label: 'English', value: 'en' },
  { label: '日本語', value: 'ja' },
  { label: 'Русский', value: 'ru' },
  { label: 'Deutsch', value: 'de' },
  { label: 'Français', value: 'fr' },
  { label: '한국어', value: 'ko' },
  { label: '繁體中文', value: 'zh-TW' },
]

const colorCodes = [
  { code: '0', name: 'Black', hex: '#000000' },
  { code: '1', name: 'Dark Blue', hex: '#0000AA' },
  { code: '2', name: 'Dark Green', hex: '#00AA00' },
  { code: '3', name: 'Dark Cyan', hex: '#00AAAA' },
  { code: '4', name: 'Dark Red', hex: '#AA0000' },
  { code: '5', name: 'Dark Purple', hex: '#AA00AA' },
  { code: '6', name: 'Gold', hex: '#FFAA00' },
  { code: '7', name: 'Gray', hex: '#AAAAAA' },
  { code: '8', name: 'Dark Gray', hex: '#555555' },
  { code: '9', name: 'Blue', hex: '#5555FF' },
  { code: 'a', name: 'Green', hex: '#55FF55' },
  { code: 'b', name: 'Aqua', hex: '#55FFFF' },
  { code: 'c', name: 'Red', hex: '#FF5555' },
  { code: 'd', name: 'Pink', hex: '#FF55FF' },
  { code: 'e', name: 'Yellow', hex: '#FFFF55' },
  { code: 'f', name: 'White', hex: '#FFFFFF' },
]

const mcColorMap: Record<string, string> = {
  '0': '#000000', '1': '#0000AA', '2': '#00AA00', '3': '#00AAAA',
  '4': '#AA0000', '5': '#AA00AA', '6': '#FFAA00', '7': '#AAAAAA',
  '8': '#555555', '9': '#5555FF', 'a': '#55FF55', 'b': '#55FFFF',
  'c': '#FF5555', 'd': '#FF55FF', 'e': '#FFFF55', 'f': '#FFFFFF',
}

function renderMotdHtml(motdJson: string): string {
  if (!motdJson) return '<span style="color:#aaa">A Minecraft Server</span>'
  try {
    const obj = JSON.parse(motdJson)
    return renderComponent(obj)
  } catch {
    // May be plain text with § color codes
    return renderLegacyText(motdJson)
  }
}

function renderComponent(comp: any): string {
  if (typeof comp === 'string') return renderLegacyText(comp)
  if (Array.isArray(comp)) return comp.map(renderComponent).join('')

  let style = ''
  const color = comp.color ? (mcColorMap[comp.color] || comp.color) : ''
  if (color) style += `color:${color};`
  if (comp.bold) style += 'font-weight:bold;'
  if (comp.italic) style += 'font-style:italic;'
  if (comp.underlined) style += 'text-decoration:underline;'
  if (comp.strikethrough) style += 'text-decoration:line-through;'

  const text = comp.text ? renderLegacyText(comp.text) : ''
  const extra = (comp.extra || []).map(renderComponent).join('')
  return `<span style="${style}">${text}${extra}</span>`
}

function renderLegacyText(text: string): string {
  let html = ''
  let i = 0
  let currentColor = ''
  let styles = ''

  while (i < text.length) {
    if ((text[i] === '§' || text[i] === '&') && i + 1 < text.length) {
      const code = text[i + 1].toLowerCase()
      if (mcColorMap[code]) {
        if (html || currentColor) html += `</span>`
        currentColor = mcColorMap[code]
        styles = `color:${currentColor}`
        html += `<span style="${styles}">`
      } else if (code === 'l') {
        styles += ';font-weight:bold'
      } else if (code === 'o') {
        styles += ';font-style:italic'
      } else if (code === 'n') {
        styles += ';text-decoration:underline'
      } else if (code === 'r') {
        html += '</span>'
        currentColor = ''
        styles = ''
        html += '<span>'
      }
      i += 2
    } else {
      html += text[i] === '<' ? '&lt;' : text[i] === '>' ? '&gt;' : text[i]
      i++
    }
  }
  if (currentColor) html += '</span>'
  return html || text
}

function validateMotdJson() {
  if (!form.motd_json) {
    motdJsonValid.value = true
    return
  }
  try {
    JSON.parse(form.motd_json)
    motdJsonValid.value = true
  } catch {
    motdJsonValid.value = false
  }
}

function parseMotdToFields(json: string | null) {
  motdFields.favicon = ''
  motdFields.line1 = ''
  motdFields.line2 = ''
  motdFields.sampleText = ''
  if (!json) return
  try {
    const obj = JSON.parse(json)
    motdFields.favicon = obj.favicon || ''
    const descText = typeof obj.description === 'string'
      ? obj.description
      : (obj.description?.text || obj.description?.translate || '')
    const lines = descText.split('\n')
    motdFields.line1 = lines[0] || ''
    motdFields.line2 = lines[1] || ''
    motdFields.sampleText = (obj.players?.sample || []).map((p: any) => p.name).join('\n')
  } catch { /* ignore */ }
}

function buildMotdJson(): string | null {
  if (!motdFields.line1 && !motdFields.line2 && !motdFields.favicon && !motdFields.sampleText) {
    return null
  }
  const descText = motdFields.line2 ? `${motdFields.line1}\n${motdFields.line2}` : motdFields.line1
  const sample = motdFields.sampleText.split('\n')
    .map(s => s.trim()).filter(Boolean)
    .map((name, i) => ({ name, id: `00000000-0000-0000-0000-${String(i).padStart(12, '0')}` }))
  const motd: any = {
    description: { text: descText },
    players: { max: form.max_player < 0 ? 100 : form.max_player, online: 0, sample }
  }
  if (motdFields.favicon) {
    motd.favicon = motdFields.favicon
  }
  return JSON.stringify(motd)
}

function insertColor(field: 'line1' | 'line2', code: string) {
  motdFields[field] += `§${code}`
}

function triggerFaviconUpload() {
  faviconInput.value?.click()
}

function onFaviconUpload(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  const reader = new FileReader()
  reader.onload = () => { motdFields.favicon = reader.result as string }
  reader.readAsDataURL(file)
  // Reset input so same file can be re-selected
  ;(e.target as HTMLInputElement).value = ''
}

function getOnlineCount(backendName: string): number {
  return onlinePlayers.value.filter((p: any) => p.backend_name === backendName).length
}

async function loadData() {
  loading.value = true
  try {
    const [backendsRes, playersRes] = await Promise.all([
      backendApi.list(),
      playersApi.getOnline({ limit: 200 }).catch(() => ({ players: [] }))
    ])
    backends.value = (backendsRes as any).list || []
    onlinePlayers.value = (playersRes as any).players || []
  } catch (e: any) {
    message.error(e.message || t('common.loadFailed'))
  } finally {
    loading.value = false
  }
}

function openAdd() {
  editingId.value = null
  resetForm()
  showModal.value = true
}

function edit(row: any) {
  editingId.value = row.id
  form.name = row.name
  form.remote_address = row.remote_address
  form.remote_port = row.remote_port
  form.max_player = row.max_player
  form.motd_json = row.motd_json || ''
  form.limbo_message = row.limbo_message || ''
  form.language = row.language || 'en'
  form.log_dir = row.log_dir || ''
  form.show_log_level = row.show_log_level || 0
  form.save_log_level = row.save_log_level || 0
  form.ping_passthrough = row.ping_passthrough || false
  form.motd_passthrough = row.motd_passthrough || false
  parseMotdToFields(row.motd_json)
  showModal.value = true
}

async function save() {
  if (!form.name.trim() || !form.remote_address.trim()) {
    message.warning(t('backend.fillRequired'))
    return
  }
  saving.value = true
  try {
    const payload = {
      ...form,
      motd_json: buildMotdJson(),
      limbo_message: form.limbo_message || null,
      log_dir: form.log_dir || null,
    }
    if (editingId.value) {
      await backendApi.update(editingId.value, payload)
      message.success(t('common.saveSuccess'))
    } else {
      await backendApi.create(payload)
      message.success(t('common.addSuccess'))
    }
    showModal.value = false
    loadData()
  } catch (e: any) {
    message.error(e.message || t('common.saveFailed'))
  } finally {
    saving.value = false
  }
}

async function toggleEnabled(row: any) {
  try {
    if (row.enabled) {
      await backendApi.disable(row.id)
    } else {
      await backendApi.enable(row.id)
    }
    message.success(t('common.operationSuccess'))
    loadData()
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  }
}

async function setDefault(row: any) {
  try {
    await backendApi.setDefault(row.id)
    message.success(t('common.operationSuccess'))
    loadData()
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  }
}

async function unsetDefault() {
  try {
    await backendApi.unsetDefault()
    message.success(t('common.operationSuccess'))
    loadData()
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  }
}

function openMaintenance(row: any) {
  if (row.maintenance) {
    // Maintenance already enabled -> close directly, no modal needed
    backendApi.toggleMaintenance(row.id, false, undefined)
      .then(() => { message.success(t('common.operationSuccess')); loadData() })
      .catch((e: any) => message.error(e.message || t('common.operationFailed')))
  } else {
    // Maintenance not enabled -> show modal to input maintenance message
    maintenanceTarget.value = row
    maintenanceMsg.value = row.maintenance_message || ''
    showMaintenanceModal.value = true
  }
}

async function applyMaintenance() {
  if (!maintenanceTarget.value) return
  maintenanceSaving.value = true
  try {
    // Modal only shows when enabling maintenance, so here we always enable maintenance
    await backendApi.toggleMaintenance(maintenanceTarget.value.id, true, maintenanceMsg.value || undefined)
    message.success(t('common.operationSuccess'))
    showMaintenanceModal.value = false
    loadData()
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  } finally {
    maintenanceSaving.value = false
  }
}

function confirmDelete(row: any) {
  deletingBackend.value = row
  showDeleteModal.value = true
}

async function remove() {
  if (!deletingBackend.value) return
  deleting.value = true
  try {
    await backendApi.delete(deletingBackend.value.id)
    message.success(t('common.deleteSuccess'))
    showDeleteModal.value = false
    loadData()
  } catch (e: any) {
    message.error(e.message || t('common.deleteFailed'))
  } finally {
    deleting.value = false
  }
}

function resetForm() {
  form.name = ''
  form.remote_address = ''
  form.remote_port = 25565
  form.max_player = -1
  form.motd_json = ''
  form.limbo_message = ''
  form.language = 'en'
  form.log_dir = ''
  form.show_log_level = 0
  form.save_log_level = 0
  form.ping_passthrough = false
  form.motd_passthrough = false
  motdJsonValid.value = true
  parseMotdToFields(null)
}

onMounted(loadData)
</script>

<style scoped>
.page {
  padding: 28px;
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 28px;
}

.page-title {
  font-size: 22px;
  font-weight: 700;
  color: #e6edf3;
  margin: 0 0 4px;
}

.page-desc {
  color: #8b949e;
  font-size: 13px;
  margin: 0;
}

.empty-state {
  margin-top: 80px;
}

/* ===== Card Grid ===== */
.backend-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 16px;
}

.backend-card {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 12px;
  padding: 20px;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.backend-card:hover {
  border-color: #58a6ff;
  box-shadow: 0 0 0 3px rgba(88, 166, 255, 0.08);
}

.backend-card.is-default {
  border-color: #3fb950;
}

.backend-card.is-maintenance {
  border-color: #f85149;
  background: #1a1012;
}

.backend-card.is-disabled {
  opacity: 0.6;
}

.card-header {
  margin-bottom: 16px;
}

.card-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
  gap: 8px;
}

.backend-name {
  font-size: 16px;
  font-weight: 600;
  color: #e6edf3;
}

.badge-row {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 20px;
  font-weight: 500;
}

.badge-default { background: rgba(63, 185, 80, 0.15); color: #3fb950; border: 1px solid rgba(63, 185, 80, 0.4); }
.badge-enabled { background: rgba(88, 166, 255, 0.1); color: #58a6ff; border: 1px solid rgba(88, 166, 255, 0.3); }
.badge-disabled { background: rgba(139, 148, 158, 0.1); color: #8b949e; border: 1px solid #30363d; }
.badge-maintenance { background: rgba(248, 81, 73, 0.15); color: #f85149; border: 1px solid rgba(248, 81, 73, 0.4); }

.backend-addr {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #8b949e;
  font-size: 13px;
}

.addr-icon {
  color: #58a6ff;
}

.backend-addr code {
  background: #0d1117;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 13px;
  color: #79c0ff;
}

/* ===== Stats ===== */
.card-stats {
  display: flex;
  gap: 16px;
  padding: 12px 0;
  border-top: 1px solid #21262d;
  border-bottom: 1px solid #21262d;
  margin-bottom: 14px;
  flex-wrap: wrap;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}

.stat-label {
  color: #8b949e;
}

.stat-value {
  color: #e6edf3;
  font-weight: 600;
}

.stat-suffix {
  color: #6e7681;
}

.stat-code {
  background: #0d1117;
  padding: 1px 5px;
  border-radius: 3px;
  font-size: 12px;
  color: #79c0ff;
}

/* ===== Action Buttons ===== */
.card-actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

/* ===== MOTD Editor ===== */
.motd-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.motd-preview-box {
  border: 1px solid #30363d;
  border-radius: 8px;
  overflow: hidden;
}

.motd-preview-header {
  background: #21262d;
  padding: 6px 12px;
  font-size: 11px;
  color: #8b949e;
}

.motd-preview-content {
  padding: 14px 16px;
  background: #1a1f26;
}

.mc-preview-server {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.favicon-preview, .favicon-placeholder {
  width: 48px;
  height: 48px;
  border-radius: 4px;
  image-rendering: pixelated;
  flex-shrink: 0;
}
.favicon-placeholder {
  background: #30363d;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #555;
  font-size: 18px;
}

.mc-server-info { flex: 1; min-width: 0; }

.mc-server-toprow {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 3px;
}

.mc-server-name {
  color: #fff;
  font-weight: 600;
  font-size: 14px;
  font-family: 'Minecraft', monospace;
}

.mc-server-version {
  color: #8b949e;
  font-size: 11px;
}

.mc-server-motd {
  font-size: 13px;
  font-family: 'Minecraft', monospace;
  line-height: 1.5;
  color: #aaa;
  min-height: 18px;
}

.motd-line { min-height: 18px; }

.mc-server-players {
  color: #666;
  font-size: 11px;
  margin-top: 4px;
  display: flex;
  gap: 10px;
}

.players-hover {
  color: #58a6ff;
  cursor: default;
  font-size: 11px;
}

/* ===== MOTD Field Row ===== */
.motd-field-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.motd-field-row--top {
  align-items: flex-start;
}

.motd-field-label {
  font-size: 12px;
  color: #8b949e;
  width: 76px;
  flex-shrink: 0;
  text-align: right;
}

.favicon-upload-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.favicon-thumb {
  width: 32px;
  height: 32px;
  border-radius: 4px;
  image-rendering: pixelated;
  border: 1px solid #30363d;
}

.motd-hint {
  color: #6e7681;
  font-size: 11px;
}

.motd-input-with-colors {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.color-btn-row {
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
}

.color-btn {
  width: 18px;
  height: 18px;
  font-size: 10px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  cursor: pointer;
  color: #fff;
  font-weight: bold;
  user-select: none;
  border: 1px solid rgba(255,255,255,0.1);
  flex-shrink: 0;
}

.fmt-btn {
  background: #21262d;
  color: #aaa;
}
</style>
