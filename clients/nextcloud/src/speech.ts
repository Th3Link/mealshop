import { z } from 'zod'
/** The browser's speech recognition, where it exists. Its vendor may process the audio. */
export type Recognition = {
  lang: string
  continuous: boolean
  interimResults: boolean
  onresult:
    | ((event: {
        resultIndex: number
        results: ArrayLike<ArrayLike<{ transcript: string }> & { isFinal: boolean }>
      }) => void)
    | null
  onerror: (() => void) | null
  onend: (() => void) | null
  start(): void
  stop(): void
}
const recognitionSchema = z.custom<new () => Recognition>((value) => typeof value === 'function')
export function recognitionClass() {
  const scope: Record<string, unknown> = window as unknown as Record<string, unknown>
  const found = recognitionSchema.safeParse(
    scope.SpeechRecognition ?? scope.webkitSpeechRecognition,
  )
  return found.success ? found.data : undefined
}
