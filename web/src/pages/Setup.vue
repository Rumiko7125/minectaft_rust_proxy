<template>
  <div class="setup-page">
    <div class="setup-lang">
      <NDropdown :options="localeOptions" @select="handleLocaleSelect" placement="bottom-end" :show-arrow="false">
        <button class="setup-lang-btn">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
          {{ currentLocaleLabel }}
        </button>
      </NDropdown>
    </div>
    <NCard class="setup-card">
      <h1>{{ t('auth.setupTitle') }}</h1>
      <NSteps :current="step">
        <NStep :title="t('auth.setupStep1')" />
        <NStep :title="t('auth.setupStep2')" />
      </NSteps>

      <NForm v-if="step === 1" ref="formRef" :model="form" :rules="rules">
        <NFormItem path="username" :label="t('auth.username')">
          <NInput v-model:value="form.username" />
        </NFormItem>
        <NFormItem path="password" :label="t('auth.password')">
          <NInput v-model:value="form.password" type="password" show-password-on="click" />
        </NFormItem>
        <NFormItem path="confirmPassword" :label="t('account.confirmPassword')">
          <NInput v-model:value="form.confirmPassword" type="password" show-password-on="click" />
        </NFormItem>
        <NButton type="primary" block :loading="loading" @click="handleCreate">
          {{ t('common.confirm') }}
        </NButton>
      </NForm>

      <div v-else-if="step === 2" class="totp-step">
        <p>{{ t('auth.bindDesc') }}</p>
        <p class="scan-hint">{{ t('auth.scanQR') }}</p>
        <!-- QR Code Image -->
        <div class="qr-code" v-if="qrCodeDataUrl">
          <img :src="qrCodeDataUrl" alt="TOTP QR Code" />
        </div>
        <div class="otpauth">{{ otpauth }}</div>
        <NForm>
          <NFormItem :label="t('auth.totpTitle')">
            <NInput v-model:value="form.totp" :placeholder="t('auth.totpPlaceholder')" />
          </NFormItem>
          <NButton type="primary" block :loading="loading" @click="handleBind">
            {{ t('auth.totpVerify') }}
          </NButton>
        </NForm>
      </div>
    </NCard>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { NCard, NSteps, NStep, NForm, NFormItem, NInput, NButton, NDropdown, useMessage } from 'naive-ui'
import { useAppStore } from '../store/app'
import { authApi } from '../api'
import { SUPPORTED_LOCALES } from '../i18n'
import QRCode from 'qrcode'

const router = useRouter()
const { t, locale } = useI18n()

const localeOptions = SUPPORTED_LOCALES.map(l => ({ label: l.label, key: l.code }))
const currentLocaleLabel = computed(() => SUPPORTED_LOCALES.find(l => l.code === locale.value)?.label ?? locale.value)

function handleLocaleSelect(key: string) {
  locale.value = key
  localStorage.setItem('proxy_locale', key)
}
const message = useMessage()
const appStore = useAppStore()

const loading = ref(false)
const step = ref(1)
const secret = ref('')
const otpauth = ref('')
const qrCodeDataUrl = ref('')

// Generate QR code image
async function generateQRCode(otpauthUrl: string) {
  try {
    qrCodeDataUrl.value = await QRCode.toDataURL(otpauthUrl, {
      width: 200,
      margin: 2,
      color: {
        dark: '#000000',
        light: '#ffffff'
      }
    })
  } catch (err) {
    console.error('QR code generation failed:', err)
  }
}

const form = reactive({
  username: '',
  password: '',
  confirmPassword: '',
  totp: ''
})

const rules = computed(() => ({
  username: { required: true, message: t('validation.required'), trigger: 'blur' },
  password: { required: true, message: t('validation.required'), trigger: 'blur' },
  confirmPassword: [
    { required: true, message: t('validation.required'), trigger: 'blur' },
    {
      validator: () => form.password === form.confirmPassword,
      message: t('account.passwordMismatch'),
      trigger: 'blur'
    }
  ]
}))

async function handleCreate() {
  if (form.password !== form.confirmPassword) {
    message.error(t('account.passwordMismatch'))
    return
  }
  loading.value = true
  try {
    const res = await authApi.setup(form.username, form.password)
    // Use returned setup_token to get TOTP settings
    const totpRes = await authApi.totpSetup(res.setup_token)
    secret.value = totpRes.secret
    otpauth.value = totpRes.qr_data_url || `otpauth://totp/admin?secret=${secret.value}&issuer=MinecraftProxy`
    // Generate QR code
    await generateQRCode(otpauth.value)
    step.value = 2
  } catch (e: any) {
    message.error(e.response?.data?.message || 'Setup failed')
  } finally {
    loading.value = false
  }
}

async function handleBind() {
  loading.value = true
  try {
    await authApi.totpConfirm(form.username, secret.value, form.totp)
    message.success(t('auth.bindSuccess'))
    router.push('/login')
  } catch (e: any) {
    message.error(e.response?.data?.message || 'Bind failed')
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.setup-page {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background: #0d1117;
  padding: 20px;
  position: relative;
}

.setup-lang {
  position: absolute;
  top: 20px;
  right: 24px;
}

.setup-lang-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: #161b22;
  border: 1px solid #30363d;
  color: #8b949e;
  cursor: pointer;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 13px;
  transition: all 0.2s;
}

.setup-lang-btn:hover {
  border-color: #58a6ff;
  color: #e6edf3;
}

.setup-card {
  width: 420px;
  background: #161b22;
  border: 1px solid #30363d;
}

h1 {
  text-align: center;
  margin-bottom: 24px;
  color: #e6edf3;
  font-size: 20px;
  font-weight: 600;
}

.totp-step {
  margin-top: 24px;
  text-align: center;
}

.scan-hint {
  color: #18a058;
  font-weight: 500;
  margin-bottom: 16px;
}

.qr-code {
  margin: 20px auto;
  padding: 16px;
  background: #ffffff;
  border-radius: 8px;
  display: inline-block;
}

.qr-code img {
  display: block;
  width: 200px;
  height: 200px;
}

.qr-placeholder {
  margin: 20px auto;
}

.otpauth {
  font-family: monospace;
  font-size: 12px;
  color: #8b949e;
  word-break: break-all;
  background: #1c2128;
  padding: 10px;
  border-radius: 6px;
  margin: 10px 0;
}

.secret {
  font-family: monospace;
  font-size: 14px;
  color: #8b949e;
  word-break: break-all;
}
</style>
