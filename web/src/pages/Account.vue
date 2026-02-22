<template>
  <div class="page-container">
    <h1 class="page-title">{{ t('nav.account') }}</h1>
    <NCard :title="t('account.profile')">
      <NForm :model="profile" label-placement="top">
        <NFormItem :label="t('auth.username')">
          <NInput v-model:value="profile.username" disabled />
        </NFormItem>
        <NFormItem :label="t('account.preferredLocale')">
          <NSelect v-model:value="profile.preferred_locale" :options="localeOptions" />
        </NFormItem>
        <NFormItem>
          <NButton type="primary" @click="handleSaveProfile">{{ t('common.save') }}</NButton>
        </NFormItem>
      </NForm>
    </NCard>

    <NCard :title="t('account.changePassword')" style="margin-top: 16px">
      <NForm ref="pwdFormRef" :model="passwords" :rules="pwdRules" label-placement="top">
        <NFormItem :label="t('account.currentPassword')" path="old">
          <NInput v-model:value="passwords.old" type="password" show-password-on="click" />
        </NFormItem>
        <NFormItem :label="t('account.newPassword')" path="new">
          <NInput v-model:value="passwords.new" type="password" show-password-on="click" />
        </NFormItem>
        <NFormItem :label="t('account.confirmNewPassword')" path="confirm">
          <NInput v-model:value="passwords.confirm" type="password" show-password-on="click" />
        </NFormItem>
        <NFormItem>
          <NButton type="primary" @click="handleChangePassword" :loading="changing">{{ t('account.changePassword') }}</NButton>
        </NFormItem>
      </NForm>
    </NCard>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { NCard, NForm, NFormItem, NInput, NButton, NSelect, useMessage } from 'naive-ui'
import { useAppStore } from '../store/app'
import { adminApi } from '../api'

const { t, locale } = useI18n()
const message = useMessage()
const appStore = useAppStore()
const changing = ref(false)

const profile = reactive({
  username: appStore.username || '',
  preferred_locale: appStore.preferredLocale || 'zh-CN'
})

const passwords = reactive({
  old: '',
  new: '',
  confirm: ''
})

const localeOptions = [
  { label: '简体中文', value: 'zh-CN' },
  { label: 'English', value: 'en' },
  { label: '繁體中文', value: 'zh-TW' },
  { label: '日本語', value: 'ja' },
  { label: 'Русский', value: 'ru' },
  { label: 'Deutsch', value: 'de' },
  { label: 'Français', value: 'fr' },
  { label: '한국어', value: 'ko' }
]

const pwdRules = computed(() => ({
  old: { required: true, message: t('validation.required'), trigger: 'blur' },
  new: { required: true, message: t('validation.required'), trigger: 'blur' },
  confirm: [
    { required: true, message: t('validation.required'), trigger: 'blur' },
    {
      validator: () => passwords.new === passwords.confirm,
      message: t('account.passwordMismatch'),
      trigger: 'blur'
    }
  ]
}))

async function handleSaveProfile() {
  try {
    await adminApi.updateLocale(profile.preferred_locale)
    locale.value = profile.preferred_locale
    appStore.setLocale(profile.preferred_locale)
    message.success(t('common.success'))
  } catch (e: any) {
    message.error(e.message || t('common.operationFailed'))
  }
}

async function handleChangePassword() {
  if (passwords.new !== passwords.confirm) {
    message.error(t('account.passwordMismatch'))
    return
  }
  changing.value = true
  try {
    await adminApi.changePassword(passwords.old, passwords.new)
    message.success(t('common.success'))
    passwords.old = ''
    passwords.new = ''
    passwords.confirm = ''
  } catch (e: any) {
    message.error(e.response?.data?.message || 'Change failed')
  } finally {
    changing.value = false
  }
}
</script>

<style scoped>
.page { padding: 20px; }
h1 { color: #e6edf3; margin-bottom: 20px; }
</style>
