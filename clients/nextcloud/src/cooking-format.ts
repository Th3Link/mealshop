import { locale, t } from './i18n'
export function duration(seconds: number): string {
  const value = Math.max(0, Math.floor(seconds)),
    days = Math.floor(value / 86400),
    hours = Math.floor((value % 86400) / 3600),
    minutes = Math.floor((value % 3600) / 60),
    rest = value % 60
  return `${days ? `${days} ${t('cookDaysShort')} ` : ''}${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}:${String(rest).padStart(2, '0')}`
}
export function localDate(seconds: number, timezone: string): string {
  try {
    return new Intl.DateTimeFormat(locale.value, {
      timeZone: timezone,
      dateStyle: 'medium',
      timeStyle: 'medium',
    }).format(new Date(seconds * 1000))
  } catch {
    return new Date(seconds * 1000).toISOString()
  }
}
export function localInput(seconds: number, timezone: string): string {
  const parts = new Intl.DateTimeFormat('en-CA', {
    timeZone: timezone,
    calendar: 'iso8601',
    numberingSystem: 'latn',
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hourCycle: 'h23',
  }).formatToParts(new Date(seconds * 1000))
  const get = (type: Intl.DateTimeFormatPartTypes) =>
    parts.find((p) => p.type === type)?.value ?? ''
  return `${get('year')}-${get('month')}-${get('day')}T${get('hour')}:${get('minute')}:${get('second')}`
}
export function zone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone
}
