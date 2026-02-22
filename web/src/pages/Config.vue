<template>
  <div class="page-container">
    <div class="page-header">
      <div>
        <h1 class="page-title">{{ t('nav.config') }}</h1>
        <p class="page-subtitle">{{ t('config.subtitle') }}</p>
      </div>
      <div style="display: flex; gap: 8px">
        <NButton @click="handleReloadServer">{{ t('config.reload') }}</NButton>
        <NButton type="primary" :loading="saving" @click="handleSave">{{ t('config.saveAll') }}</NButton>
      </div>
    </div>

    <div class="config-grid">
      <!-- Proxy Network -->
      <div class="config-section">
        <div class="section-title">🌐 {{ t('config.groupNetwork') }}</div>
        <div class="config-card">
          <div class="config-row">
            <div class="config-label">{{ t('config.proxyPort') }}</div>
            <div class="config-control">
              <NInputNumber v-model:value="config.local_port" :min="1" :max="65535" style="width: 140px" />
            </div>
          </div>
          <div class="config-row">
            <div class="config-label">{{ t('config.trustedDomain') }}</div>
            <div class="config-control">
              <NInput v-model:value="config.trusted_domain" :placeholder="t('config.trustedDomainHint')" style="width: 240px" />
            </div>
          </div>
          <div class="config-row">
            <div class="config-label">{{ t('config.allowInput') }}</div>
            <div class="config-control">
              <NSwitch v-model:value="config.allow_input" />
            </div>
            <div class="config-hint">{{ t('config.allowInputHint') }}</div>
          </div>
        </div>
      </div>

      <!-- Web API -->
      <div class="config-section">
        <div class="section-title">🔌 {{ t('config.groupWebApi') }}</div>
        <div class="config-card">
          <div class="config-row">
            <div class="config-label">{{ t('config.webApiEnable') }}</div>
            <div class="config-control">
              <NSwitch v-model:value="config.web_api_enable" />
            </div>
          </div>
          <div class="config-row">
            <div class="config-label">{{ t('config.webApiPort') }}</div>
            <div class="config-control">
              <NInputNumber v-model:value="config.web_api_port" :min="1" :max="65535" style="width: 140px" />
            </div>
          </div>
        </div>
      </div>

      <!-- Two-Factor Auth -->
      <div class="config-section">
        <div class="section-title">🔐 {{ t('config.groupTwoFactor') }}</div>
        <div class="config-card">
          <div class="config-row">
            <div class="config-label">{{ t('config.twoFactorEnable') }}</div>
            <div class="config-control">
              <NSwitch v-model:value="config.enable_2fa" />
            </div>
          </div>
          <div class="config-row">
            <div class="config-label">{{ t('config.twoFactorSessionHours') }}</div>
            <div class="config-control">
              <NInputNumber v-model:value="config.two_factor_session_hours" :min="1" style="width: 140px" />
            </div>
            <div class="config-hint">{{ t('config.twoFactorSessionHint') }}</div>
          </div>
          <div class="config-row">
            <div class="config-label">{{ t('config.twoFactorIssuer') }}</div>
            <div class="config-control">
              <NInput v-model:value="config.two_factor_issuer" style="width: 240px" />
            </div>
          </div>
        </div>
      </div>

      <!-- Log Settings -->
      <div class="config-section">
        <div class="section-title">📋 {{ t('config.groupLog') }}</div>
        <div class="config-card">
          <div class="config-row">
            <div class="config-label">{{ t('config.logDir') }}</div>
            <div class="config-control">
              <NInput v-model:value="config.log_dir" :placeholder="t('config.logDirPlaceholder')" style="width: 240px" />
            </div>
          </div>
          <div class="config-row">
            <div class="config-label">{{ t('config.showLogLevel') }}</div>
            <div class="config-control">
              <NSelect v-model:value="config.show_log_level" :options="logLevelOptions" style="width: 140px" />
            </div>
          </div>
          <div class="config-row">
            <div class="config-label">{{ t('config.saveLogLevel') }}</div>
            <div class="config-control">
              <NSelect v-model:value="config.save_log_level" :options="logLevelOptions" style="width: 140px" />
            </div>
          </div>
        </div>
      </div>

      <!-- Backend Server Management Hint -->
      <div class="config-section full-width">
        <div class="config-card info-card">
          <div class="info-icon">🖥</div>
          <div class="info-content">
            <div class="info-title">{{ t('backend.title') }}</div>
            <div class="info-desc">{{ t('config.backendManagementHint') }}
              <router-link to="/backend" class="info-link">{{ t('nav.backend') }}</router-link>
              {{ t('config.backendManagementHint2') }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Web CLI Terminal -->
    <div class="cli-zone">
      <div class="cli-title">💻 {{ t('config.cliTitle') }}</div>
      <div class="cli-desc">{{ t('config.cliDesc') }}</div>
      <div class="terminal" ref="terminalRef">
        <div class="terminal-output">
          <div v-if="cliHistory.length === 0" class="terminal-welcome">
            {{ t('config.cliWelcome') }}
          </div>
          <div v-for="(entry, i) in cliHistory" :key="i" class="terminal-entry">
            <div class="terminal-cmd">
              <span class="terminal-prompt">❯</span>
              <span>{{ entry.command }}</span>
            </div>
            <pre v-if="entry.output" class="terminal-result" :class="{ 'terminal-error': entry.isError }">{{ entry.output }}</pre>
          </div>
          <!-- Scroll Anchor -->
          <div ref="terminalBottom"></div>
        </div>
        <div class="terminal-input-row">
          <span class="terminal-prompt">❯</span>
          <input
            ref="cliInputRef"
            v-model="cliInput"
            class="terminal-input"
            :placeholder="t('config.cliInputPlaceholder')"
            :disabled="cliRunning"
            @keydown="onCliKeydown"
            autocomplete="off"
            spellcheck="false"
          />
          <NButton size="small" :loading="cliRunning" @click="submitCli" style="flex-shrink:0">
            {{ t('config.cliRun') }}
          </NButton>
          <NButton size="small" quaternary @click="clearCli" style="flex-shrink:0">
            {{ t('config.cliClear') }}
          </NButton>
        </div>
      </div>
    </div>

    <!-- Data Migration -->
    <div class="migration-zone">
      <div class="migration-title">📦 {{ t('config.migrationTitle') }}</div>
      <div class="migration-desc">{{ t('config.migrationDesc') }}</div>
      <div style="display:flex;gap:8px;flex-wrap:wrap">
        <NButton @click="handleExportMigration" :loading="migrating">{{ t('config.exportMigration') }}</NButton>
        <NButton @click="triggerMigrationImport">{{ t('config.importMigration') }}</NButton>
      </div>
      <input ref="migrationFileInput" type="file" accept=".zip,application/zip" style="display:none" @change="onMigrationImportFile" />
    </div>

    <!-- Danger Zone -->
    <div class="danger-zone">
      <div class="danger-title">⚠️ {{ t('config.dangerZone') }}</div>
      <div class="danger-desc">{{ t('config.dangerZoneDesc') }}</div>
      <NButton type="error" ghost @click="showRestartConfirm = true">
        {{ t('config.restartServer') }}
      </NButton>
    </div>

    <!-- Restart Confirmation Modal -->
    <NModal v-model:show="showRestartConfirm" preset="card" :title="t('config.restartConfirm')" style="width: 420px">
      <div class="restart-warning">
        <div class="restart-warning-icon">⚠️</div>
        <div>{{ t('config.restartWarning') }}</div>
      </div>
      <NFormItem :label="t('auth.totpTitle')" style="margin-top: 16px">
        <NInput
          v-model:value="restartTotp"
          :placeholder="t('auth.totpPlaceholder')"
          maxlength="6"
        />
      </NFormItem>
      <template #footer>
        <div style="display: flex; justify-content: flex-end; gap: 8px">
          <NButton @click="showRestartConfirm = false">{{ t('common.cancel') }}</NButton>
          <NButton type="error" :loading="restarting" @click="handleRestart">{{ t('common.confirm') }}</NButton>
        </div>
      </template>
    </NModal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive, computed, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { NInput, NInputNumber, NSwitch, NButton, NSelect, NModal, NFormItem, useMessage } from 'naive-ui'
import { configApi, migrationApi, cliApi } from '../api'

const { t } = useI18n()
const message = useMessage()
const saving = ref(false)
const restarting = ref(false)
const migrating = ref(false)
const showRestartConfirm = ref(false)
const restartTotp = ref('')
const migrationFileInput = ref<HTMLInputElement | null>(null)

// CLI state
const cliInput = ref('')
const cliRunning = ref(false)
const cliHistory = ref<{ command: string; output: string; isError: boolean }[]>([])
const cliCmdHistory = ref<string[]>([])
const cliHistoryCursor = ref(-1)
const terminalBottom = ref<HTMLElement | null>(null)
const cliInputRef = ref<HTMLInputElement | null>(null)
const terminalRef = ref<HTMLElement | null>(null)

const config = reactive({
  local_port: 25565,
  allow_input: true,
  trusted_domain: '',
  web_api_enable: true,
  web_api_port: 8080,
  enable_2fa: false,
  two_factor_session_hours: 24,
  two_factor_issuer: 'MinecraftProxy',
  log_dir: '',
  show_log_level: 3,
  save_log_level: 2
})

const logLevelOptions = computed(() => [
  { label: t('backend.logLevel0'), value: 0 },
  { label: t('backend.logLevel1'), value: 1 },
  { label: t('backend.logLevel2'), value: 2 },
  { label: t('backend.logLevel3'), value: 3 },
  { label: t('backend.logLevel4'), value: 4 }
])

async function loadConfig() {
  try {
    const res = await configApi.get()
    Object.assign(config, res)
  } catch (e) { console.error(e) }
}

async function handleSave() {
  saving.value = true
  try {
    await configApi.update(config)
    message.success(t('common.saveSuccess'))
  } catch (e: any) {
    message.error(e.message || t('common.saveFailed'))
  } finally {
    saving.value = false
  }
}

async function handleReloadServer() {
  try {
    await configApi.reload()
    message.success(t('config.reloadSuccess'))
    await loadConfig()
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  }
}

async function handleRestart() {
  if (!restartTotp.value) {
    message.error(t('auth.totpPlaceholder'))
    return
  }
  restarting.value = true
  try {
    await configApi.restart(restartTotp.value)
    message.success(t('config.restartSuccess'))
    showRestartConfirm.value = false
    restartTotp.value = ''
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  } finally {
    restarting.value = false
  }
}

// CLI Tab completion command tree
const CLI_COMMAND_TREE: Record<string, string[]> = {
  'help': [],
  'list': ['players', 'whitelist', 'blacklist'],
  'kick': [],
  'ban': [],
  'pardon': [],
  'whitelist': ['on', 'off', 'add', 'remove'],
  'backend': ['list', 'info', 'add', 'remove', 'enable', 'disable', 'default', 'maintenance'],
  '2fa': ['list', 'remove'],
  'config': ['list', 'set', 'reload'],
  'modlog': []
}

// Tab completion state
const tabCompletions = ref<string[]>([])
const tabIndex = ref(0)
let lastWasTab = false

function handleTabComplete() {
  const input = cliInput.value
  const parts = input.split(' ')
  let completions: string[]
  let prefix: string

  if (parts.length <= 1) {
    // Complete first word (command name)
    const partial = parts[0] || ''
    completions = Object.keys(CLI_COMMAND_TREE).filter(cmd => cmd.startsWith(partial))
    prefix = ''
  } else {
    // Complete subcommand
    const cmd = parts[0]
    const partial = parts[parts.length - 1]
    const subCmds = CLI_COMMAND_TREE[cmd] || []
    completions = subCmds.filter(s => s.startsWith(partial))
    prefix = parts.slice(0, -1).join(' ') + ' '
  }

  if (completions.length === 0) return

  if (!lastWasTab) {
    // First Tab press: record completion list
    tabCompletions.value = completions
    tabIndex.value = 0
  } else {
    // Next Tab press: cycle to next
    tabIndex.value = (tabIndex.value + 1) % tabCompletions.value.length
  }

  const chosen = tabCompletions.value[tabIndex.value]
  // If unique match, append space for continued subcommand input
  cliInput.value = prefix + chosen + (tabCompletions.value.length === 1 ? ' ' : '')
  lastWasTab = true
}

function onCliKeydown(e: KeyboardEvent) {
  if (e.key === 'Tab') {
    e.preventDefault()
    handleTabComplete()
  } else if (e.key === 'Enter') {
    e.preventDefault()
    lastWasTab = false
    tabCompletions.value = []
    submitCli()
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    lastWasTab = false
    tabCompletions.value = []
    historyUp()
  } else if (e.key === 'ArrowDown') {
    e.preventDefault()
    lastWasTab = false
    tabCompletions.value = []
    historyDown()
  } else {
    lastWasTab = false
    tabCompletions.value = []
  }
}

// CLI functions
async function submitCli() {
  const cmd = cliInput.value.trim()
  if (!cmd || cliRunning.value) return

  cliInput.value = ''
  cliRunning.value = true

  // Record to command history
  if (cliCmdHistory.value[0] !== cmd) {
    cliCmdHistory.value.unshift(cmd)
    if (cliCmdHistory.value.length > 100) cliCmdHistory.value.pop()
  }
  cliHistoryCursor.value = -1

  try {
    const res = await cliApi.execute(cmd)
    cliHistory.value.push({ command: cmd, output: res.output, isError: false })
  } catch (e: any) {
    cliHistory.value.push({ command: cmd, output: e.message || 'Error', isError: true })
  } finally {
    cliRunning.value = false
    await nextTick()
    terminalBottom.value?.scrollIntoView({ behavior: 'smooth' })
    cliInputRef.value?.focus()
  }
}

function historyUp() {
  if (cliCmdHistory.value.length === 0) return
  cliHistoryCursor.value = Math.min(cliHistoryCursor.value + 1, cliCmdHistory.value.length - 1)
  cliInput.value = cliCmdHistory.value[cliHistoryCursor.value]
}

function historyDown() {
  if (cliHistoryCursor.value <= 0) {
    cliHistoryCursor.value = -1
    cliInput.value = ''
    return
  }
  cliHistoryCursor.value--
  cliInput.value = cliCmdHistory.value[cliHistoryCursor.value]
}

function clearCli() {
  cliHistory.value = []
}

async function handleExportMigration() {
  migrating.value = true
  try {
    const blob = await migrationApi.exportData()
    const ts = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob as Blob)
    link.download = `proxy_migration_${ts}.zip`
    link.click()
    message.success(t('common.success'))
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  } finally {
    migrating.value = false
  }
}

function triggerMigrationImport() {
  migrationFileInput.value?.click()
}

async function onMigrationImportFile(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  ;(e.target as HTMLInputElement).value = ''
  migrating.value = true
  try {
    await migrationApi.importData(file)
    message.success(t('config.migrationImportSuccess'))
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  } finally {
    migrating.value = false
  }
}

onMounted(loadConfig)
</script>

<style scoped>
.page-container {
  padding: 24px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 28px;
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

.config-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.config-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.config-section.full-width {
  grid-column: 1 / -1;
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  color: #8b949e;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.config-card {
  background: #161b22;
  border: 1px solid #21262d;
  border-radius: 8px;
  overflow: hidden;
}

.config-row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
  border-bottom: 1px solid #21262d;
  flex-wrap: wrap;
}

.config-row:last-child {
  border-bottom: none;
}

.config-label {
  color: #e6edf3;
  font-size: 14px;
  min-width: 160px;
  flex-shrink: 0;
}

.config-control {
  flex-shrink: 0;
}

.config-hint {
  color: #8b949e;
  font-size: 12px;
  margin-left: auto;
}

/* Info card */
.info-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
}

.info-icon {
  font-size: 28px;
  flex-shrink: 0;
}

.info-title {
  font-weight: 600;
  color: #e6edf3;
  font-size: 14px;
  margin-bottom: 4px;
}

.info-desc {
  color: #8b949e;
  font-size: 13px;
  line-height: 1.5;
}

.info-link {
  color: #58a6ff;
  text-decoration: none;
}

.info-link:hover {
  text-decoration: underline;
}

/* CLI Zone */
.cli-zone {
  background: #161b22;
  border: 1px solid #21262d;
  border-radius: 8px;
  padding: 20px 24px;
  margin-bottom: 16px;
}

.cli-title {
  font-size: 15px;
  font-weight: 600;
  color: #e6edf3;
  margin-bottom: 4px;
}

.cli-desc {
  color: #8b949e;
  font-size: 13px;
  margin-bottom: 14px;
}

.terminal {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 6px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  overflow: hidden;
}

.terminal-output {
  min-height: 180px;
  max-height: 360px;
  overflow-y: auto;
  padding: 12px 14px 4px;
  color: #c9d1d9;
}

.terminal-welcome {
  color: #484f58;
  font-style: italic;
}

.terminal-entry {
  margin-bottom: 8px;
}

.terminal-cmd {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #58a6ff;
  margin-bottom: 2px;
}

.terminal-prompt {
  color: #3fb950;
  font-weight: bold;
  flex-shrink: 0;
}

.terminal-result {
  margin: 0;
  padding: 4px 0 4px 22px;
  white-space: pre-wrap;
  word-break: break-word;
  color: #c9d1d9;
  line-height: 1.5;
}

.terminal-result.terminal-error {
  color: #f85149;
}

.terminal-input-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  border-top: 1px solid #21262d;
  background: #0d1117;
}

.terminal-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: #e6edf3;
  font-family: inherit;
  font-size: 13px;
  caret-color: #3fb950;
}

.terminal-input::placeholder {
  color: #484f58;
}

/* Migration Zone */
.migration-zone {
  background: #161b22;
  border: 1px solid #21262d;
  border-radius: 8px;
  padding: 20px 24px;
  margin-bottom: 16px;
}

.migration-title {
  font-size: 15px;
  font-weight: 600;
  color: #e6edf3;
  margin-bottom: 6px;
}

.migration-desc {
  color: #8b949e;
  font-size: 13px;
  margin-bottom: 16px;
}

/* Danger Zone */
.danger-zone {
  background: #161b22;
  border: 1px solid #f85149;
  border-radius: 8px;
  padding: 20px 24px;
}

.danger-title {
  font-size: 15px;
  font-weight: 600;
  color: #f85149;
  margin-bottom: 6px;
}

.danger-desc {
  color: #8b949e;
  font-size: 13px;
  margin-bottom: 16px;
}

/* Restart warning */
.restart-warning {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  background: rgba(248, 81, 73, 0.1);
  border: 1px solid rgba(248, 81, 73, 0.3);
  border-radius: 6px;
  padding: 12px 16px;
  color: #f85149;
  font-size: 14px;
}

.restart-warning-icon {
  font-size: 18px;
  flex-shrink: 0;
}
</style>
