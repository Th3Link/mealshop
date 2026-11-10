import { describe, it, expect } from 'vitest'
import { duration, localInput, localDate } from '../src/cooking-format'
import { localTimeSchema, timerSchema } from '../src/cooking-api'
import { locale } from '../src/i18n'
describe('cooking presentation preserves authoritative seconds and civil-time ambiguity', () => {
  it('formats long, paused and overdue durations without changing deadlines', () => {
    locale.value = 'en'
    expect(duration(172861)).toBe('2 d 00:01:01')
    expect(duration(0)).toBe('00:00:00')
    expect(duration(-1)).toBe('00:00:00')
    locale.value = 'de'
    expect(duration(86400)).toBe('1 T 00:00:00')
  })
  it('uses the selected IANA zone over the device zone across both DST transitions', () => {
    expect(localInput(Date.parse('2027-03-28T00:30:00Z') / 1000, 'Europe/Berlin')).toBe(
      '2027-03-28T01:30:00',
    )
    expect(localInput(Date.parse('2027-03-28T01:30:00Z') / 1000, 'Europe/Berlin')).toBe(
      '2027-03-28T03:30:00',
    )
    for (const utc of ['2027-10-31T00:30:00Z', '2027-10-31T01:30:00Z'])
      expect(localInput(Date.parse(utc) / 1000, 'Europe/Berlin')).toBe('2027-10-31T02:30:00')
    expect(localDate(1800000000, 'Europe/Berlin')).toBeTruthy()
  })
  it('keeps fold choices and persisted running or paused timer states explicit', () => {
    expect(
      localTimeSchema.parse({ local: '2027-10-31T02:30', timezone: 'Europe/Berlin', fold: 'later' })
        .fold,
    ).toBe('later')
    expect(
      timerSchema.parse({
        state: 'paused',
        started_at: 1800000000,
        paused_at: 1800000001,
        active_remaining: 0,
        remaining: 30,
      }),
    ).toMatchObject({ state: 'paused', remaining: 30 })
    expect(timerSchema.safeParse({ state: 'running', remaining: 30 }).success).toBe(false)
  })
})
