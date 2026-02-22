<template>
  <NConfigProvider :locale="naiveLocale" :theme="darkTheme">
    <NMessageProvider>
      <NDialogProvider>
        <NNotificationProvider>
          <RouterView />
        </NNotificationProvider>
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
  darkTheme
} from 'naive-ui'
import { useAppStore } from './store/app'

const { locale } = useI18n()
const appStore = useAppStore()

// Sync store locale settings to i18n
watch(() => appStore.preferredLocale, (newLocale) => {
  if (newLocale) {
    locale.value = newLocale
  }
}, { immediate: true })

const NAIVE_LOCALE_MAP: Record<string, any> = {
  'zh-CN': null,
  'en': null
}

const naiveLocale = computed(() => NAIVE_LOCALE_MAP[locale.value] || NAIVE_LOCALE_MAP['zh-CN'])
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
  width: 100%;
  background: #0d1117;
}

/* Unified page container styles */
.page-container {
  padding: 24px;
}

.page-title {
  color: #e6edf3;
  font-size: 20px;
  font-weight: 600;
  margin-bottom: 20px;
}

/* Unified card styles */
.n-card {
  background: #161b22 !important;
  border: 1px solid #30363d !important;
}

/* Unified table styles */
.n-data-table {
  background: #161b22 !important;
}

/* Unified tab styles */
.n-tabs .n-tabs-tab {
  color: #8b949e !important;
}

.n-tabs .n-tabs-tab:hover {
  color: #e6edf3 !important;
}

.n-tabs .n-tabs-tab.n-tabs-tab--active {
  color: #58a6ff !important;
}
</style>
