<template>
  <div class="app-layout">
    <!-- Sidebar -->
    <aside class="sidebar" :class="{ collapsed: sidebarCollapsed }">
      <div class="sidebar-header">
        <div class="logo">
          <span class="logo-icon">🦀</span>
          <span class="logo-text" v-show="!sidebarCollapsed">Rust Proxy</span>
        </div>
        <button class="collapse-btn" @click="sidebarCollapsed = !sidebarCollapsed">
          <span>{{ sidebarCollapsed ? '›' : '‹' }}</span>
        </button>
      </div>

      <nav class="sidebar-nav">
        <router-link v-for="item in navItems" :key="item.path" :to="item.path" class="nav-item" :title="sidebarCollapsed ? t(item.label) : ''">
          <span class="nav-icon">{{ item.icon }}</span>
          <span class="nav-label" v-show="!sidebarCollapsed">{{ t(item.label) }}</span>
        </router-link>
      </nav>

      <div class="sidebar-footer">
        <div class="user-info" v-show="!sidebarCollapsed">
          <span class="user-avatar">{{ appStore.username?.charAt(0)?.toUpperCase() }}</span>
          <span class="user-name">{{ appStore.username }}</span>
        </div>
        <NDropdown :options="localeOptions" @select="handleLocaleSelect" placement="top-start" :show-arrow="false">
          <button class="lang-btn" :title="currentLocaleLabel">
            <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
          </button>
        </NDropdown>
        <button class="logout-btn" @click="handleLogout" :title="t('common.exit')">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
            <polyline points="16,17 21,12 16,7"/>
            <line x1="21" y1="12" x2="9" y2="12"/>
          </svg>
        </button>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { NDropdown } from 'naive-ui'
import { useAppStore } from '../store/app'
import { authApi } from '../api'
import { SUPPORTED_LOCALES } from '../i18n'

const router = useRouter()
const { t, locale } = useI18n()
const appStore = useAppStore()
const sidebarCollapsed = ref(false)

const localeOptions = SUPPORTED_LOCALES.map(l => ({ label: l.label, key: l.code }))
const currentLocaleLabel = computed(() => SUPPORTED_LOCALES.find(l => l.code === locale.value)?.label ?? locale.value)

function handleLocaleSelect(key: string) {
  locale.value = key
  localStorage.setItem('proxy_locale', key)
}

const navItems = [
  { path: '/dashboard', label: 'nav.dashboard', icon: '📊' },
  { path: '/players', label: 'nav.players', icon: '👤' },
  { path: '/access', label: 'nav.access', icon: '🛡' },
  { path: '/routes', label: 'nav.routes', icon: '🗺' },
  { path: '/backend', label: 'nav.backend', icon: '🖥' },
  { path: '/twofactor', label: 'nav.twoFactor', icon: '🔐' },
  { path: '/logs', label: 'nav.logs', icon: '📋' },
  { path: '/config', label: 'nav.config', icon: '⚙' },
  { path: '/account', label: 'nav.account', icon: '👑' },
]

async function handleLogout() {
  try { await authApi.logout() } catch {}
  appStore.logout()
  router.push('/login')
}
</script>

<style scoped>
.app-layout {
  display: flex;
  min-height: 100vh;
  background: #0d1117;
}

/* ===== Sidebar ===== */
.sidebar {
  width: 220px;
  background: #161b22;
  display: flex;
  flex-direction: column;
  border-right: 1px solid #21262d;
  flex-shrink: 0;
  transition: width 0.25s ease;
  overflow: hidden;
}

.sidebar.collapsed {
  width: 56px;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 12px;
  border-bottom: 1px solid #21262d;
  min-height: 60px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  overflow: hidden;
}

.logo-icon {
  font-size: 22px;
  flex-shrink: 0;
  display: inline-block;
  animation: crab-float 3s ease-in-out infinite;
  cursor: default;
}

.logo:hover .logo-icon {
  animation: crab-dance 0.7s ease-in-out forwards;
}

@keyframes crab-float {
  0%, 100% { transform: translateY(0) rotate(0deg); }
  30% { transform: translateY(-3px) rotate(-8deg); }
  70% { transform: translateY(-2px) rotate(8deg); }
}

@keyframes crab-dance {
  0%   { transform: scale(1)    rotate(0deg); }
  15%  { transform: scale(1.3)  rotate(-20deg); }
  35%  { transform: scale(1.3)  rotate(20deg); }
  55%  { transform: scale(1.2)  rotate(-12deg); }
  75%  { transform: scale(1.1)  rotate(12deg); }
  100% { transform: scale(1)    rotate(0deg); }
}

.logo-text {
  font-size: 15px;
  font-weight: 800;
  white-space: nowrap;
  background: linear-gradient(90deg,
    #f74c00 0%,
    #ff8c42 20%,
    #ffe0c2 40%,
    #ff8c42 60%,
    #f74c00 80%,
    #ff8c42 100%
  );
  background-size: 250% 100%;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: rust-shimmer 4s linear infinite;
}

@keyframes rust-shimmer {
  0%   { background-position: 100% center; }
  100% { background-position: -150% center; }
}

.collapse-btn {
  background: none;
  border: none;
  color: #6e7681;
  cursor: pointer;
  padding: 4px 6px;
  border-radius: 6px;
  font-size: 16px;
  line-height: 1;
  transition: all 0.2s;
  flex-shrink: 0;
}

.collapse-btn:hover {
  background: #21262d;
  color: #e6edf3;
}

/* ===== Navigation ===== */
.sidebar-nav {
  flex: 1;
  padding: 8px 0;
  overflow-y: auto;
  overflow-x: hidden;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  color: #8b949e;
  text-decoration: none;
  transition: all 0.18s;
  border-left: 3px solid transparent;
  white-space: nowrap;
  font-size: 14px;
}

.nav-item:hover {
  background: #21262d;
  color: #c9d1d9;
}

.nav-item.router-link-active {
  background: #1f2937;
  color: #58a6ff;
  border-left-color: #58a6ff;
}

.nav-icon {
  font-size: 16px;
  flex-shrink: 0;
  width: 20px;
  text-align: center;
}

.nav-label {
  font-size: 13.5px;
}

/* ===== Footer ===== */
.sidebar-footer {
  padding: 12px;
  border-top: 1px solid #21262d;
  display: flex;
  align-items: center;
  gap: 8px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  overflow: hidden;
}

.user-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: linear-gradient(135deg, #58a6ff, #3fb950);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 700;
  color: #fff;
  flex-shrink: 0;
}

.user-name {
  font-size: 13px;
  color: #8b949e;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.lang-btn {
  background: none;
  border: none;
  color: #6e7681;
  cursor: pointer;
  padding: 6px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  flex-shrink: 0;
}

.lang-btn:hover {
  background: #21262d;
  color: #e6edf3;
}

.logout-btn {
  background: none;
  border: none;
  color: #6e7681;
  cursor: pointer;
  padding: 6px;
  border-radius: 6px;
  font-size: 16px;
  transition: all 0.2s;
  flex-shrink: 0;
}

.logout-btn:hover {
  background: #3d1f1f;
  color: #f85149;
}

/* ===== Main Content ===== */
.main-content {
  flex: 1;
  overflow-y: auto;
  background: #0d1117;
  min-width: 0;
}
</style>
