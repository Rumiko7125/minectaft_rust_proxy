import { useI18n } from 'vue-i18n'

/**
 * Convert UTC time to local time
 * Let browser handle timezone conversion automatically, no manual offset needed
 */
export function useTimezone() {
  const { t } = useI18n()

  /**
   * Convert UTC time to local time display
   * @param timestamp UTC time string or Unix timestamp
   * @returns Converted Date object
   */
  function toLocalTime(timestamp: string | number | undefined | null): Date | null {
    if (!timestamp) return null

    let date: Date

    // If Unix timestamp (number)
    if (typeof timestamp === 'number') {
      // timestamp is in seconds or milliseconds
      const ms = timestamp > 1e12 ? timestamp : timestamp * 1000
      date = new Date(ms)
    } else {
      // Try to parse as UTC time
      // If contains T or Z, JavaScript treats it as UTC
      // Otherwise manually add UTC marker
      const utcTimestamp = timestamp.includes('T') || timestamp.includes('Z')
        ? timestamp
        : timestamp.replace(' ', 'T') + 'Z'
      date = new Date(utcTimestamp)
    }

    if (isNaN(date.getTime())) return null

    console.log('input:', timestamp, '→ parsed UTC:', date.toISOString(), '→ local:', date.toLocaleString())
    // Return directly, let toLocaleString() handle timezone automatically
    return date
  }

  /**
   * Format relative time display
   * @param timestamp UTC time string or Unix timestamp
   * @returns Relative time string
   */
  function formatRelativeTime(timestamp: string | number | undefined | null): string {
    if (!timestamp) return '-'

    const date = toLocalTime(timestamp)
    if (!date) return '-'

    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const minutes = Math.floor(diff / 60000)
    const hours = Math.floor(diff / 3600000)
    const days = Math.floor(diff / 86400000)

    if (minutes < 1) return t('time.justNow') || 'just now'
    if (minutes < 60) return t('time.minutesAgo', { n: minutes }) || `${minutes}m`
    if (hours < 24) return t('time.hoursAgo', { n: hours }) || `${hours}h`
    if (days < 7) return t('time.daysAgo', { n: days }) || `${days}d`

    return date.toLocaleDateString()
  }

  /**
   * Format full time display
   * @param timestamp UTC time string or Unix timestamp
   * @returns Full time string
   */
  function formatTime(timestamp: string | number | undefined | null): string {
    if (!timestamp) return '-'

    const date = toLocalTime(timestamp)
    if (!date) return '-'

    return date.toLocaleString()
  }

  return {
    toLocalTime,
    formatRelativeTime,
    formatTime
  }
}
