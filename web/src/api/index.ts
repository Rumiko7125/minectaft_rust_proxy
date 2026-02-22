import axios from 'axios'
import { useAppStore } from '../store/app'

const api = axios.create({
  baseURL: '/api/v1',
  timeout: 15000
})

// Request interceptor - Add Token
api.interceptors.request.use(
  config => {
    const appStore = useAppStore()
    if (appStore.token) {
      config.headers.Authorization = `Bearer ${appStore.token}`
    }
    return config
  },
  error => Promise.reject(error)
)

// Response interceptor - Handle errors
api.interceptors.response.use(
  response => response.data,
  error => {
    const appStore = useAppStore()
    if (error.response?.status === 401) {
      appStore.logout()
      window.location.href = '/login'
    }
    // Extract backend error message
    if (error.response?.data?.message) {
      error.message = error.response.data.message
    }
    return Promise.reject(error)
  }
)

export default api
export { api }

// Auth API
export const authApi = {
  getStatus: () => api.get('/auth/status'),
  login: (username: string, password: string) =>
    api.post('/auth/login', { username, password }),
  totpSetup: (setupToken: string) => api.post('/auth/totp/setup', { setup_token: setupToken }),
  totpConfirm: (username: string, setupToken: string, totpCode: string) =>
    api.post('/auth/totp/confirm', { username, setup_token: setupToken, totp_code: totpCode }),
  totpVerify: (username: string, sessionToken: string, totpCode: string) =>
    api.post('/auth/totp/verify', { username, session_token: sessionToken, totp_code: totpCode }),
  logout: () => api.post('/auth/logout'),
  setup: (username: string, password: string) =>
    api.post('/auth/setup', { username, password })
}

// Players API
export const playersApi = {
  getOnline: (params?: { limit?: number; offset?: number }) => api.get('/players', { params }),
  getHistory: (params?: { search?: string; limit?: number; offset?: number }) =>
    api.get('/players/history', { params }),
  getStats: () => api.get('/players/stats'),
  kick: (username: string, reason?: string) => api.post('/kick', { username, reason })
}

// Access API
export const accessApi = {
  getWhitelist: (params?: { limit?: number; offset?: number }) => api.get('/whitelist', { params }),
  addWhitelist: (username: string) => api.post('/whitelist', { username }),
  removeWhitelist: (username: string) => api.delete(`/whitelist/${username}`),
  toggleWhitelist: (enabled: boolean) => api.patch('/whitelist', { enabled }),
  getBlacklist: (params?: { limit?: number; offset?: number }) => api.get('/blacklist', { params }),
  addBlacklist: (username: string, reason?: string) =>
    api.post('/blacklist', { username, reason }),
  removeBlacklist: (username: string) => api.delete(`/blacklist/${username}`)
}

// Routes API
export const routesApi = {
  getDomainRoutes: (params?: { limit?: number; offset?: number }) => api.get('/routes/domain', { params }),
  addDomainRoute: (data: { pattern: string; target_addr?: string; target_port?: number; priority?: number; backend_id?: number | null }) =>
    api.post('/routes/domain', data),
  updateDomainRoute: (id: number, data: any) => api.patch(`/routes/domain/${id}`, data),
  deleteDomainRoute: (id: number) => api.delete(`/routes/domain/${id}`)
}

// Backend API
export const backendApi = {
  list: () => api.get('/backend'),
  get: (id: number) => api.get(`/backend/${id}`),
  create: (data: any) => api.post('/backend', data),
  update: (id: number, data: any) => api.patch(`/backend/${id}`, data),
  delete: (id: number) => api.delete(`/backend/${id}`),
  enable: (id: number) => api.post(`/backend/${id}/enable`),
  disable: (id: number) => api.post(`/backend/${id}/disable`),
  setDefault: (id: number) => api.post(`/backend/${id}/set-default`),
  unsetDefault: () => api.post('/backend/unset-default'),
  getDefault: () => api.get('/backend/default'),
  toggleMaintenance: (id: number, maintenance: boolean, maintenanceMessage?: string) =>
    api.post(`/backend/${id}/maintenance`, { maintenance, maintenance_message: maintenanceMessage })
}

// TwoFactor API
export const twoFactorApi = {
  list: (params?: { limit?: number; offset?: number }) => api.get('/2fa/players', { params }),
  unbind: (username: string) => api.delete(`/2fa/players/${username}`),
  getQR: (username: string) => api.get(`/2fa/players/${username}/qr`)
}

// Logs API
export const logsApi = {
  getModeration: (params?: { action?: string; operator?: string; target?: string; limit?: number; offset?: number }) =>
    api.get('/logs/moderation', { params }),
  getSessions: (params?: { search?: string; limit?: number; offset?: number }) =>
    api.get('/logs/sessions', { params }),
  exportSessions: (params?: { search?: string }) =>
    api.get('/logs/sessions/export', { params }),
  exportModeration: () =>
    api.get('/logs/moderation/export')
}

// Config API
export const configApi = {
  get: () => api.get('/config'),
  update: (data: any) => api.patch('/config', data),
  reload: () => api.post('/config/reload'),
  restart: (totpCode: string) => api.post('/server/restart', { totp_code: totpCode })
}

// Admin API
export const adminApi = {
  list: () => api.get('/admin/accounts'),
  create: (username: string, password: string) =>
    api.post('/admin/accounts', { username, password }),
  delete: (id: number) => api.delete(`/admin/accounts/${id}`),
  reset2fa: (id: number) => api.post(`/admin/accounts/${id}/reset-2fa`),
  changePassword: (oldPassword: string, newPassword: string) =>
    api.post('/admin/me/password', { old_password: oldPassword, new_password: newPassword }),
  updateLocale: (locale: string) =>
    api.patch('/admin/me/locale', { locale })
}

// CLI API
export const cliApi = {
  execute: (command: string): Promise<{ output: string; command: string }> =>
    api.post('/cli', { command })
}

// Dashboard API
export const dashboardApi = {
  getRecent: () => api.get('/dashboard/recent'),
  getStats: () => api.get('/players/stats')
}

// Migration API
export const migrationApi = {
  exportData: () =>
    api.get('/migration/export', { responseType: 'blob' }),
  importData: (file: File) => {
    const formData = new FormData()
    formData.append('file', file)
    return api.post('/migration/import', formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
    })
  }
}

// Ping API
export const pingApi = {
  ping: (addr: string, port: number) =>
    api.get('/ping', { params: { addr, port } })
}
