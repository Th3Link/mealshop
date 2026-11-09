<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import { t } from './i18n'
import { canonicalGtin } from './shopping-api'
interface Detector {
  detect: (source: HTMLVideoElement) => Promise<{ rawValue: string }[]>
}
interface DetectorConstructor {
  new (options: { formats: string[] }): Detector
  getSupportedFormats: () => Promise<string[]>
}
const emit = defineEmits<{ barcode: [value: string] }>()
const video = ref<HTMLVideoElement>(),
  running = ref(false),
  starting = ref(false),
  error = ref(false)
let stream: MediaStream | undefined,
  frame = 0,
  generation = 0
function stop() {
  generation += 1
  running.value = false
  starting.value = false
  cancelAnimationFrame(frame)
  stream?.getTracks().forEach((track) => track.stop())
  stream = undefined
  if (video.value) video.value.srcObject = null
}
function visibility() {
  if (document.hidden) stop()
}
async function start() {
  stop()
  const attempt = generation
  error.value = false
  starting.value = true
  try {
    const Constructor = (window as Window & { BarcodeDetector?: DetectorConstructor })
      .BarcodeDetector
    if (!window.isSecureContext || !Constructor || !navigator.mediaDevices?.getUserMedia)
      throw new Error('unsupported')
    const supported = await Constructor.getSupportedFormats()
    const formats = ['ean_8', 'ean_13', 'upc_a'].filter((format) => supported.includes(format))
    if (!formats.length || generation !== attempt) throw new Error('unsupported')
    const detector = new Constructor({ formats })
    const media = await navigator.mediaDevices.getUserMedia({
      audio: false,
      video: { facingMode: { ideal: 'environment' } },
    })
    if (generation !== attempt) {
      media.getTracks().forEach((track) => track.stop())
      return
    }
    stream = media
    starting.value = false
    running.value = true
    await nextTick()
    if (!video.value || generation !== attempt) {
      stop()
      return
    }
    video.value.srcObject = media
    await video.value.play()
    async function scan() {
      if (generation !== attempt || !video.value) return
      try {
        const found = (await detector.detect(video.value)).find((result) =>
          canonicalGtin(result.rawValue),
        )
        if (generation !== attempt) return
        if (found) {
          emit('barcode', found.rawValue)
          stop()
          return
        }
        frame = requestAnimationFrame(() => void scan())
      } catch {
        if (generation === attempt) {
          error.value = true
          stop()
        }
      }
    }
    await scan()
  } catch {
    if (generation === attempt) {
      error.value = true
      stop()
    }
  }
}
document.addEventListener('visibilitychange', visibility)
onBeforeUnmount(() => {
  stop()
  document.removeEventListener('visibilitychange', visibility)
})
</script>
<template>
  <div class="barcode-camera">
    <NcButton v-if="!running && !starting" @click="start">{{ t('barcodeScan') }}</NcButton>
    <p v-if="starting" role="status">{{ t('barcodePermission') }}</p>
    <video v-if="running" ref="video" muted playsinline :aria-label="t('barcodeCamera')" />
    <NcButton v-if="running || starting" @click="stop">{{ t('barcodeStop') }}</NcButton>
    <p v-if="error" role="status">{{ t('barcodeCameraUnavailable') }}</p>
  </div>
</template>
