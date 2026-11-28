import { beforeEach, describe, expect, it, vi } from 'vitest'

const api = vi.fn()
vi.mock('../src/api', () => ({
  api: (...args: unknown[]) => api(...args) as unknown,
  ApiError: class ApiError extends Error {
    constructor(
      readonly code: string,
      readonly status: number,
    ) {
      super(code)
    }
  },
}))
const { ApiError } = await import('../src/api')
const { loadAssist, recordingType, transcribe } = await import('../src/assist-api')
const wait = () => Promise.resolve()
const blob = new Blob(['synthetic'], { type: 'audio/webm' })

describe('Assistant transcription is optional and never assumed', () => {
  beforeEach(() => api.mockReset())
  it('offers nothing when the Assistant cannot be asked', async () => {
    api.mockRejectedValueOnce(new Error('offline'))
    expect(await loadAssist()).toEqual({ speech: false, picture: false, handwriting: false })
  })
  it('uploads once, waits through the task and returns its text', async () => {
    api
      .mockResolvedValueOnce({ id: 7, kind: 'speech', status: 'waiting' })
      .mockResolvedValueOnce({ id: 7, kind: 'speech', status: 'running', progress: 0.5 })
      .mockRejectedValueOnce(new ApiError('assist_lookup_failed', 502))
      .mockResolvedValueOnce({ id: 7, kind: 'speech', status: 'done', text: '100 g Oats' })
    const seen: string[] = []
    expect(await transcribe('speech', blob, { wait, onStatus: (s) => seen.push(s) })).toBe(
      '100 g Oats',
    )
    expect(seen).toEqual(['waiting', 'running', 'done'])
    expect(api.mock.calls[0]?.[0]).toBe('/api/assist/transcriptions?kind=speech')
    expect(api.mock.calls[0]?.[2]).toMatchObject({ method: 'POST', contentType: 'audio/webm' })
    expect(api.mock.calls[1]?.[0]).toBe('/api/assist/transcriptions/7')
  })
  it('reports a failed or endless task instead of inventing text', async () => {
    api
      .mockResolvedValueOnce({ id: 8, kind: 'picture', status: 'waiting' })
      .mockResolvedValueOnce({ id: 8, kind: 'picture', status: 'failed' })
    await expect(transcribe('picture', blob, { wait })).rejects.toMatchObject({
      code: 'assist_failed',
    })
    api.mockReset()
    api.mockResolvedValue({ id: 9, kind: 'speech', status: 'waiting' })
    await expect(transcribe('speech', blob, { wait, attempts: 3 })).rejects.toMatchObject({
      code: 'assist_timeout',
    })
  })
  it('records only in a container the server accepts', () => {
    expect(recordingType((type) => type === 'audio/mp4')).toBe('audio/mp4')
    expect(recordingType(() => false)).toBeNull()
  })
})
