<template>
  <div class="login-page">
    <div class="login-lang">
      <NDropdown :options="localeOptions" @select="handleLocaleSelect" placement="bottom-end" :show-arrow="false">
        <button class="login-lang-btn">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
          {{ currentLocaleLabel }}
        </button>
      </NDropdown>
    </div>
    <NCard class="login-card">
      <h1>{{ t('auth.title') }}</h1>
      <NForm ref="formRef" :model="form" :rules="rules" @submit.prevent="handleLogin">
        <NFormItem path="username" :label="t('auth.username')">
          <NInput v-model:value="form.username" @keyup.enter="handleLogin" />
        </NFormItem>
        <NFormItem path="password" :label="t('auth.password')">
          <NInput
            v-model:value="form.password"
            type="password"
            show-password-on="click"
            @keyup.enter="handleLogin"
          />
        </NFormItem>
        <NFormItem v-if="needTotp" path="totp" :label="t('auth.totpTitle')">
          <NInput v-model:value="form.totp" :placeholder="t('auth.totpPlaceholder')" @keyup.enter="handleLogin" />
        </NFormItem>
        <NButton type="primary" block :loading="loading" @click="handleLogin">
          {{ needTotp ? t('auth.totpVerify') : t('auth.login') }}
        </NButton>
      </NForm>
    </NCard>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { NCard, NForm, NFormItem, NInput, NButton, NDropdown, useMessage } from 'naive-ui'
import { useAppStore } from '../store/app'
import { authApi } from '../api'
import { SUPPORTED_LOCALES } from '../i18n'

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

// Check if initialization is needed
onMounted(async () => {
  try {
    const status = await authApi.getStatus()
    if (status.needs_setup) {
      router.replace('/setup')
    }
  } catch (e) {
    // Ignore error, stay on login page
  }
})

const needTotp = ref(false)
const sessionToken = ref('')

const form = reactive({
  username: '',
  password: '',
  totp: ''
})

const rules = {
  username: { required: true, message: t('validation.required') },
  password: { required: true, message: t('validation.required') }
}

async function handleLogin() {
  if (!form.username || !form.password) {
    message.error(t('validation.required'))
    return
  }

  loading.value = true
  try {
    if (!needTotp.value) {
      const res = await authApi.login(form.username, form.password)
      if (res.status === 'need_bind_2fa') {
        message.info(t('auth.bindTitle'))
      } else if (res.status === 'need_totp') {
        needTotp.value = true
        sessionToken.value = res.session_token
      }
    } else {
      if (!form.totp) {
        message.error(t('validation.required'))
        loading.value = false
        return
      }
      const res = await authApi.totpVerify(form.username, sessionToken.value, form.totp)
      appStore.login(form.username, res.access_token)
      router.push('/dashboard')
    }
  } catch (e: any) {
    message.error(e.response?.data?.message || t('auth.loginFailed'))
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-page {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background: #0d1117;
  position: relative;
}

.login-lang {
  position: absolute;
  top: 20px;
  right: 24px;
}

.login-lang-btn {
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

.login-lang-btn:hover {
  border-color: #58a6ff;
  color: #e6edf3;
}

.login-card {
  width: 400px;
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
</style>
