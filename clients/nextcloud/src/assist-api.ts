import { z } from 'zod'
import { api, ApiError } from './api'

/** What the instance's Nextcloud Assistant can do for capture right now. */
export const assistSchema = z.object({
  speech: z.boolean(),
  picture: z.boolean(),
  handwriting: z.boolean(),
})
export type Assist = z.infer<typeof assistSchema>
export type AssistKind = keyof Assist
export const transcriptionSchema = z.object({
  id: z.number().int(),
  kind: z.enum(['speech', 'picture', 'handwriting']),
  status: z.enum(['waiting', 'running', 'done', 'failed']),
  text: z.string().optional(),
  progress: z.number().nullable().optional(),
})
export type Transcription = z.infer<typeof transcriptionSchema>

export async function loadAssist(): Promise<Assist> {
  try {
    return await api('/api/assist', assistSchema)
  } catch {
    // No Assistant, an older server or no connection: capture works without it.
    return { speech: false, picture: false, handwriting: false }
  }
}

/** Sends a recording or picture to the Assistant and waits for the words it finds.
 * The text is unreviewed source text; the caller puts it in front of the cook. */
export async function transcribe(
  kind: AssistKind,
  data: Blob,
  options: {
    onStatus?: (status: Transcription['status']) => void
    signal?: AbortSignal
    wait?: (milliseconds: number) => Promise<void>
    attempts?: number
  } = {},
): Promise<string> {
  const wait =
    options.wait ?? ((ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms)))
  const started = await api(`/api/assist/transcriptions?kind=${kind}`, transcriptionSchema, {
    method: 'POST',
    data,
    contentType: data.type,
    signal: options.signal,
  })
  options.onStatus?.(started.status)
  let lookups = 0
  for (let attempt = 0; attempt < (options.attempts ?? 300); attempt += 1) {
    await wait(2000)
    if (options.signal?.aborted) throw new ApiError('assist_cancelled', 0)
    let current: Transcription
    try {
      current = await api(`/api/assist/transcriptions/${started.id}`, transcriptionSchema, {
        signal: options.signal,
      })
      lookups = 0
    } catch (error) {
      // One failed lookup says nothing about the task; several in a row end the wait.
      if (!(error instanceof ApiError) || error.code !== 'assist_lookup_failed' || ++lookups > 4)
        throw error
      continue
    }
    options.onStatus?.(current.status)
    if (current.status === 'done') return current.text ?? ''
    if (current.status === 'failed') throw new ApiError('assist_failed', 0)
  }
  throw new ApiError('assist_timeout', 0)
}

/** The first container the browser can record that the server accepts. */
export function recordingType(supported: (type: string) => boolean): string | null {
  return ['audio/webm', 'audio/ogg', 'audio/mp4'].find(supported) ?? null
}
