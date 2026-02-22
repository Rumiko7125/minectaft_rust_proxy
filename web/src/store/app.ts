import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

const TOKEN_KEY = 'proxy_token'
const USERNAME_KEY = 'proxy_username'
const LOCALE_KEY = 'proxy_locale'

// Default locale
const DEFAULT_LOCALE = 'zh-CN'

export const useAppStore = defineStore('app', () => {
  const darkMode = ref(true)
  const isLoggedIn = ref(false)
  const username = ref('')
  const token = ref('')
  const preferredLocale = ref('zh-CN')
  const onlinePlayers = ref<any[]>([])

  // Restore login state from localStorage
  function initFromStorage() {
    const savedToken = localStorage.getItem(TOKEN_KEY)
    const savedUsername = localStorage.getItem(USERNAME_KEY)
    const savedLocale = localStorage.getItem(LOCALE_KEY)

    if (savedToken) {
      token.value = savedToken
      username.value = savedUsername || ''
      isLoggedIn.value = true
    }

    preferredLocale.value = savedLocale || DEFAULT_LOCALE
  }

  // Restore from storage on init
  initFromStorage()

  // Watch for token changes, auto-save to localStorage
  watch(token, (newToken) => {
    if (newToken) {
      localStorage.setItem(TOKEN_KEY, newToken)
    } else {
      localStorage.removeItem(TOKEN_KEY)
    }
  })

  watch(username, (newUsername) => {
    if (newUsername) {
      localStorage.setItem(USERNAME_KEY, newUsername)
    } else {
      localStorage.removeItem(USERNAME_KEY)
    }
  })

  watch(preferredLocale, (newLocale) => {
    localStorage.setItem(LOCALE_KEY, newLocale)
  })

  function login(user: string, t: string) {
    isLoggedIn.value = true
    username.value = user
    token.value = t
  }

  function logout() {
    isLoggedIn.value = false
    username.value = ''
    token.value = ''
    localStorage.removeItem(TOKEN_KEY)
    localStorage.removeItem(USERNAME_KEY)
  }

  function setOnlinePlayers(players: any[]) {
    onlinePlayers.value = players
  }

  function toggleDarkMode() {
    darkMode.value = !darkMode.value
  }

  function setLocale(locale: string) {
    preferredLocale.value = locale
  }

  return {
    darkMode,
    isLoggedIn,
    username,
    token,
    preferredLocale,
    onlinePlayers,
    login,
    logout,
    setOnlinePlayers,
    toggleDarkMode,
    setLocale,
    initFromStorage
  }
})
