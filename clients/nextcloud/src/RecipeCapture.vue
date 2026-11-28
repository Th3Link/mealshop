<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, useId } from 'vue'
import { z } from 'zod'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import {
  mdiCameraOutline,
  mdiInboxArrowDown,
  mdiLinkVariant,
  mdiMicrophone,
  mdiPencilOutline,
  mdiStop,
  mdiTextBoxEditOutline,
} from '@mdi/js'
import Icon from './Icon.vue'
import { api, domain, imageUrl } from './api'
import { useTask } from './state'
import { locale, servingCount, t } from './i18n'
import { importPreviewSchema, type ImportPreview } from './recipe-import'
import { parseRecipeText, type TextRecipe } from './recipe-text'
import { unitLabel } from './unit-label'
import { sourceHost, suggestedRecipe, suggestionsSchema, type Suggestion } from './suggestion-api'
const props = defineProps<{ home: string; listPath: string }>()
const emit = defineEmits<{
  captured: [recipe: TextRecipe, preview?: ImportPreview, suggestion?: string]
  changed: []
}>()
const mode = ref<'choose' | 'text' | 'photo'>('choose')
const text = ref(''),
  imageId = ref(''),
  textId = useId()
const { busy, error, run, errorMessage } = useTask()
const parsed = computed(() => parseRecipeText(text.value, locale.value))
const empty = computed(
  () => !parsed.value.title && !parsed.value.ingredients.length && !parsed.value.steps.length,
)
const suggestions = ref<Suggestion[]>([]),
  discarding = ref('')
async function loadSuggestions() {
  try {
    suggestions.value = (
      await api(domain(props.home, '/api/household/suggestions'), suggestionsSchema)
    ).suggestions
  } catch {
    // Readers and older servers have no inbox; adding a recipe works without it.
    suggestions.value = []
  }
}
onMounted(loadSuggestions)
function received(suggestion: Suggestion) {
  return new Date(suggestion.received_at * 1000).toLocaleDateString(locale.value)
}
async function review(suggestion: Suggestion) {
  const recipe = suggestedRecipe(suggestion, locale.value)
  await run(async () => {
    // A suggestion that names its source is saved as an import of that source.
    const preview = suggestion.source_url
      ? await api(domain(props.home, '/api/household/imports/preview'), importPreviewSchema, {
          method: 'POST',
          data: { mode: 'suggestion', id: suggestion.id },
        })
      : undefined
    emit('captured', recipe, preview, suggestion.id)
  })
}
async function discard(suggestion: Suggestion) {
  if (discarding.value !== suggestion.id) {
    discarding.value = suggestion.id
    return
  }
  await run(async () => {
    await api(
      domain(props.home, `/api/household/suggestions/${suggestion.id}`),
      z.object({ discarded: z.string() }),
      { method: 'DELETE' },
    )
    discarding.value = ''
    await loadSuggestions()
  })
}
async function open(next: 'text' | 'photo') {
  mode.value = next
  await nextTick()
  document.querySelector<HTMLElement>('.recipe-capture h1')?.focus()
}
function edited() {
  emit('changed')
}

type Recognition = {
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
function recognitionClass() {
  const scope: Record<string, unknown> = window as unknown as Record<string, unknown>
  const found = recognitionSchema.safeParse(
    scope.SpeechRecognition ?? scope.webkitSpeechRecognition,
  )
  return found.success ? found.data : undefined
}
const canDictate = Boolean(recognitionClass())
const listening = ref(false),
  dictationFailed = ref(false)
let recognition: Recognition | undefined
/** Spoken line breaks are the only command; everything else stays as recognised. */
function spoken(transcript: string) {
  return transcript
    .replace(/\s*\b(?:neue zeile|nächste zeile|new line|next line)\b[.,]?\s*/giu, '\n')
    .trim()
}
function dictate() {
  if (listening.value) {
    recognition?.stop()
    return
  }
  const Recognizer = recognitionClass()
  if (!Recognizer) return
  dictationFailed.value = false
  recognition = new Recognizer()
  recognition.lang = locale.value === 'de' ? 'de-DE' : 'en-US'
  recognition.continuous = true
  recognition.interimResults = false
  recognition.onresult = (event) => {
    for (let index = event.resultIndex; index < event.results.length; index += 1) {
      const result = event.results[index]
      if (!result?.isFinal || !result[0]) continue
      const addition = spoken(result[0].transcript)
      if (!addition) continue
      text.value += `${text.value && !text.value.endsWith('\n') ? '\n' : ''}${addition}`
      edited()
    }
  }
  recognition.onerror = () => {
    dictationFailed.value = true
  }
  recognition.onend = () => {
    listening.value = false
  }
  try {
    recognition.start()
    listening.value = true
  } catch {
    dictationFailed.value = true
  }
}
onBeforeUnmount(() => recognition?.stop())

async function choosePhoto(event: Event) {
  if (!(event.target instanceof HTMLInputElement)) return
  const file = event.target.files?.[0]
  event.target.value = ''
  if (!file) return
  await run(async () => {
    if (
      file.size > 5 * 1024 * 1024 ||
      !['image/jpeg', 'image/png', 'image/webp'].includes(file.type)
    )
      throw new Error('image')
    const result = await api(
      domain(props.home, '/api/household/images'),
      z.object({ id: z.string() }),
      {
        method: 'POST',
        data: file,
        contentType: file.type,
      },
    )
    imageId.value = result.id
    edited()
  })
}
async function proceed() {
  const recipe = parsed.value
  if (mode.value !== 'photo') {
    emit('captured', recipe)
    return
  }
  await run(async () => {
    const preview = await api(
      domain(props.home, '/api/household/imports/preview'),
      importPreviewSchema,
      {
        method: 'POST',
        data: {
          mode: 'photo',
          capture: {
            image_id: imageId.value,
            rotation: 0,
            text: text.value,
            engine: 'manual_transcription',
            score: null,
          },
          original: {
            title: recipe.title || null,
            language: locale.value,
            yield_text: recipe.servings === null ? null : String(recipe.servings),
            ingredients: recipe.ingredients.map((item) => item.raw),
            steps: recipe.steps.map((step) => ({ text: step, section: null })),
          },
        },
      },
    )
    emit('captured', recipe, preview)
  })
}
</script>
<template>
  <section v-if="mode === 'choose'" class="recipe-capture">
    <header class="page-header">
      <div>
        <h1 tabindex="-1">{{ t('addRecipe') }}</h1>
        <p>{{ t('addRecipeIntro') }}</p>
      </div>
      <NcButton :to="listPath">{{ t('back') }}</NcButton>
    </header>
    <NcNoteCard v-if="error" type="error" role="alert">{{ errorMessage(error) }}</NcNoteCard>
    <section v-if="suggestions.length" class="capture-inbox" :aria-label="t('suggestionInbox')">
      <h2><Icon :path="mdiInboxArrowDown" />{{ t('suggestionInbox') }}</h2>
      <p class="muted">{{ t('suggestionInboxHelp') }}</p>
      <ul class="suggestion-list">
        <li v-for="suggestion in suggestions" :key="suggestion.id" class="suggestion">
          <div>
            <strong>{{ suggestion.original.title || t('captureUntitled') }}</strong>
            <span class="muted">
              {{ received(suggestion) }} · {{ suggestion.original.ingredients.length }}
              {{ t('suggestionLines')
              }}<template v-if="suggestion.source_url">
                ·
                <a :href="suggestion.source_url" target="_blank" rel="noopener noreferrer">{{
                  sourceHost(suggestion.source_url) || t('source')
                }}</a></template
              >
            </span>
            <p v-if="suggestion.note" class="muted suggestion-note">{{ suggestion.note }}</p>
          </div>
          <div class="actions">
            <NcButton
              variant="primary"
              :disabled="busy"
              :aria-label="`${t('suggestionReview')}: ${suggestion.original.title || t('captureUntitled')}`"
              @click="review(suggestion)"
              >{{ t('suggestionReview') }}</NcButton
            >
            <NcButton
              :variant="discarding === suggestion.id ? 'error' : 'tertiary'"
              :disabled="busy"
              :aria-label="`${t(discarding === suggestion.id ? 'suggestionDiscardConfirm' : 'suggestionDiscard')}: ${suggestion.original.title || t('captureUntitled')}`"
              @click="discard(suggestion)"
              >{{
                t(discarding === suggestion.id ? 'suggestionDiscardConfirm' : 'suggestionDiscard')
              }}</NcButton
            >
          </div>
        </li>
      </ul>
    </section>
    <ul class="capture-choices">
      <li>
        <button type="button" @click="open('text')">
          <Icon :path="mdiTextBoxEditOutline" /><strong>{{ t('captureText') }}</strong
          ><span>{{ t('captureTextHelp') }}</span>
        </button>
      </li>
      <li>
        <button type="button" @click="open('photo')">
          <Icon :path="mdiCameraOutline" /><strong>{{ t('capturePhoto') }}</strong
          ><span>{{ t('capturePhotoHelp') }}</span>
        </button>
      </li>
      <li>
        <RouterLink :to="`${listPath}/import`" :aria-label="t('importRecipe')">
          <Icon :path="mdiLinkVariant" /><strong>{{ t('captureLink') }}</strong
          ><span>{{ t('captureLinkHelp') }}</span>
        </RouterLink>
      </li>
      <li>
        <RouterLink :to="`${listPath}/new`" :aria-label="t('newRecipe')">
          <Icon :path="mdiPencilOutline" /><strong>{{ t('captureBlank') }}</strong
          ><span>{{ t('captureBlankHelp') }}</span>
        </RouterLink>
      </li>
    </ul>
    <p class="muted">
      {{ t('captureAssistant') }}
      <RouterLink :to="`/h/${home}/assistants`">{{ t('captureAssistantLink') }}</RouterLink>
    </p>
  </section>
  <section v-else class="recipe-capture">
    <header class="page-header">
      <div>
        <h1 tabindex="-1">{{ t(mode === 'photo' ? 'capturePhoto' : 'captureText') }}</h1>
        <p>{{ t(mode === 'photo' ? 'capturePhotoHelp' : 'captureTextHelp') }}</p>
      </div>
      <NcButton :disabled="busy" @click="mode = 'choose'">{{ t('back') }}</NcButton>
    </header>
    <NcNoteCard v-if="error" type="error" role="alert">{{
      error instanceof Error && error.message === 'image' ? t('imageTooLarge') : errorMessage(error)
    }}</NcNoteCard>
    <div class="capture-workspace" :class="{ 'with-photo': mode === 'photo' }">
      <div v-if="mode === 'photo'" class="capture-photo">
        <img v-if="imageId" :src="imageUrl(home, imageId)" alt="" />
        <label class="capture-file"
          ><Icon :path="mdiCameraOutline" />{{
            t(imageId ? 'captureChangePhoto' : 'captureChoosePhoto')
          }}<input
            type="file"
            class="visually-hidden"
            accept="image/jpeg,image/png,image/webp"
            :disabled="busy"
            @change="choosePhoto"
        /></label>
        <p class="muted">{{ t('capturePhotoPrivate') }}</p>
      </div>
      <div class="capture-text">
        <label :for="textId">{{ t('captureTextLabel') }}</label>
        <textarea
          :id="textId"
          v-model="text"
          rows="14"
          maxlength="32000"
          :lang="locale"
          :placeholder="t('captureTextPlaceholder')"
          :disabled="busy"
          @input="edited"
        ></textarea>
        <p class="muted">{{ t('captureHint') }}</p>
        <div class="actions">
          <NcButton v-if="canDictate" :disabled="busy" :pressed="listening" @click="dictate"
            ><template #icon><Icon :path="listening ? mdiStop : mdiMicrophone" /></template
            >{{ t(listening ? 'captureDictateStop' : 'captureDictate') }}</NcButton
          >
        </div>
        <p v-if="listening" class="muted" role="status">{{ t('captureDictateHint') }}</p>
        <p v-if="canDictate" class="muted">{{ t('captureDictatePrivacy') }}</p>
        <p v-else class="muted">{{ t('captureDictateUnsupported') }}</p>
        <NcNoteCard v-if="dictationFailed" type="warning">{{
          t('captureDictateFailed')
        }}</NcNoteCard>
      </div>
      <aside class="capture-found" aria-live="polite">
        <h2>{{ t('captureFound') }}</h2>
        <p v-if="empty" class="muted">{{ t('captureNothing') }}</p>
        <template v-else>
          <p class="capture-title">
            <strong>{{ parsed.title || t('captureUntitled') }}</strong><span
              v-if="parsed.servings !== null"
            >
              · {{ servingCount(parsed.servings) }}</span
            >
          </p>
          <h3 v-if="parsed.ingredients.length">{{ t('lines') }}</h3>
          <ul class="plain-list">
            <li v-for="(item, index) in parsed.ingredients" :key="index">
              <template v-if="item.amount"
                ><span class="capture-amount"
                  >{{ item.amount.replace('.', locale === 'de' ? ',' : '.') }}
                  {{ item.unit === 'piece' || !item.unit ? '×' : unitLabel(item.unit) }}</span
                >
                {{ item.name }}</template
              ><template v-else>{{ item.raw }}</template>
            </li>
          </ul>
          <h3 v-if="parsed.steps.length">{{ t('steps') }}</h3>
          <ol class="plain-list">
            <li v-for="(step, index) in parsed.steps" :key="index">{{ step }}</li>
          </ol>
        </template>
        <NcLoadingIcon v-if="busy" :aria-label="t('loading')" />
        <NcButton
          variant="primary"
          :disabled="busy || empty || (mode === 'photo' && !imageId)"
          @click="proceed"
          >{{ t('captureContinue') }}</NcButton
        >
      </aside>
    </div>
  </section>
</template>
