import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN.json'
import zhTW from './locales/zh-TW.json'
import en from './locales/en.json'
import ja from './locales/ja.json'
import ru from './locales/ru.json'
import de from './locales/de.json'
import fr from './locales/fr.json'
import ko from './locales/ko.json'

export const SUPPORTED_LOCALES = [
  { code: 'zh-CN', label: '简体中文' },
  { code: 'zh-TW', label: '繁體中文' },
  { code: 'en', label: 'English' },
  { code: 'ja', label: '日本語' },
  { code: 'ru', label: 'Русский' },
  { code: 'de', label: 'Deutsch' },
  { code: 'fr', label: 'Français' },
  { code: 'ko', label: '한국어' }
] as const

export type LocaleCode = typeof SUPPORTED_LOCALES[number]['code']

// Detect language
function detectLocale(): LocaleCode {
  // 1. localStorage (priority)
  const stored = localStorage.getItem('proxy_locale')
  if (stored && SUPPORTED_LOCALES.some(l => l.code === stored)) {
    return stored as LocaleCode
  }

  // 2. Browser language
  const browserLang = navigator.language
  if (browserLang.startsWith('zh-TW') || browserLang.startsWith('zh-HK') || browserLang.startsWith('zh-MO')) return 'zh-TW'
  if (browserLang.startsWith('zh')) return 'zh-CN'
  if (browserLang.startsWith('ja')) return 'ja'
  if (browserLang.startsWith('ru')) return 'ru'
  if (browserLang.startsWith('de')) return 'de'
  if (browserLang.startsWith('fr')) return 'fr'
  if (browserLang.startsWith('ko')) return 'ko'
  if (browserLang.startsWith('en')) return 'en'

  // 3. Default
  return 'en'
}

const i18n = createI18n({
  legacy: false,
  locale: detectLocale(),
  fallbackLocale: 'en',
  messages: {
    'zh-CN': zhCN,
    'zh-TW': zhTW,
    'en': en,
    'ja': ja,
    'ru': ru,
    'de': de,
    'fr': fr,
    'ko': ko
  }
})

export default i18n
