<script setup lang="ts">
import { computed, onBeforeUnmount, ref, useId } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import { mdiMicrophone, mdiStop } from '@mdi/js'
import Icon from './Icon.vue'
import { calendarDate, locale, mealName, servingCount, t } from './i18n'
import { readPlanCommand, type PlanCommand, type RecipeChoice } from './plan-command'
import { recognitionClass, type Recognition } from './speech'
const props = defineProps<{
  recipes: RecipeChoice[]
  disabled: boolean
  /** What the planner answered to the last command, already in the reader's language. */
  note?: string
}>()
const emit = defineEmits<{ run: [command: PlanCommand] }>()
const inputId = useId()
const text = ref('')
const understood = ref<PlanCommand>()
const unclear = ref(false)
const today = () => new Date().toLocaleDateString('en-CA')
const summary = computed(() => {
  const command = understood.value
  if (!command) return ''
  const intent = {
    plan: 'planIntentPlan',
    remove: 'planIntentRemove',
    suggest: 'planIntentSuggest',
    show: 'planIntentShow',
  } as const
  return [
    t(intent[command.intent]),
    command.recipes.length === 1 ? command.recipes[0]?.title : undefined,
    command.date ? calendarDate(command.date) : undefined,
    command.meal ? mealName(command.meal) : undefined,
    command.servings ? servingCount(command.servings) : undefined,
  ]
    .filter(Boolean)
    .join(' · ')
})
function submit() {
  const command = readPlanCommand(text.value, today(), props.recipes)
  // A sentence that names neither a day, a meal, a recipe nor an action was not understood.
  unclear.value =
    !command ||
    (command.intent === 'plan' &&
      !command.date &&
      !command.meal &&
      !command.recipes.length &&
      !command.recipeText)
  understood.value = unclear.value ? undefined : (command ?? undefined)
  if (!command || unclear.value) return
  // Several matching titles are offered; nothing runs before one is picked.
  if (command.intent === 'plan' && command.recipes.length > 1) return
  emit('run', command)
}
function choose(recipe: RecipeChoice) {
  if (!understood.value) return
  understood.value = { ...understood.value, recipes: [recipe] }
  emit('run', understood.value)
}
const canListen = Boolean(recognitionClass())
const listening = ref(false),
  failed = ref(false)
let recognition: Recognition | undefined
function listen() {
  if (listening.value) {
    recognition?.stop()
    return
  }
  const Recognizer = recognitionClass()
  if (!Recognizer) return
  failed.value = false
  recognition = new Recognizer()
  recognition.lang = locale.value === 'de' ? 'de-DE' : 'en-US'
  recognition.continuous = false
  recognition.interimResults = false
  recognition.onresult = (event) => {
    const heard = event.results[event.resultIndex]?.[0]?.transcript.trim()
    if (!heard) return
    text.value = heard
    submit()
  }
  recognition.onerror = () => {
    failed.value = true
  }
  recognition.onend = () => {
    listening.value = false
  }
  try {
    recognition.start()
    listening.value = true
  } catch {
    failed.value = true
  }
}
onBeforeUnmount(() => recognition?.stop())
</script>
<template>
  <form class="plan-command" @submit.prevent="submit">
    <label :for="inputId">{{ t('planCommand') }}</label>
    <div class="plan-command-row">
      <input
        :id="inputId"
        v-model="text"
        type="text"
        maxlength="300"
        autocomplete="off"
        :lang="locale"
        :placeholder="t('planCommandPlaceholder')"
        :disabled="disabled"
        :aria-describedby="`${inputId}-help`"
      />
      <NcButton type="submit" variant="primary" :disabled="disabled || !text.trim()">{{
        t('planCommandGo')
      }}</NcButton>
      <NcButton
        v-if="canListen"
        :disabled="disabled"
        :pressed="listening"
        :aria-label="t(listening ? 'planCommandStop' : 'planCommandSpeak')"
        :title="t(listening ? 'planCommandStop' : 'planCommandSpeak')"
        @click="listen"
        ><template #icon><Icon :path="listening ? mdiStop : mdiMicrophone" /></template
      ></NcButton>
    </div>
    <p :id="`${inputId}-help`" class="muted">
      {{ t(canListen ? 'planCommandPrivacy' : 'planCommandHelp') }}
    </p>
    <div class="plan-command-answer" role="status">
      <p v-if="listening">{{ t('planCommandListening') }}</p>
      <p v-if="unclear">{{ t('planNotUnderstood') }}</p>
      <p v-if="summary">
        <strong>{{ t('planUnderstood') }}:</strong> {{ summary }}
      </p>
      <template v-if="understood && understood.intent === 'plan'">
        <div v-if="understood.recipes.length > 1" class="plan-command-choices">
          <p>{{ t('planWhichRecipe') }}</p>
          <NcButton
            v-for="recipe in understood.recipes"
            :key="`${recipe.id}@${recipe.revision}`"
            :disabled="disabled"
            @click="choose(recipe)"
            >{{ recipe.title }}</NcButton
          >
        </div>
        <p v-else-if="!understood.recipes.length && understood.recipeText">
          {{ t('planNoRecipe') }} “{{ understood.recipeText }}”. {{ t('planNoRecipeHelp') }}
        </p>
      </template>
      <p v-if="note">{{ note }}</p>
    </div>
    <NcNoteCard v-if="failed" type="warning">{{ t('planCommandFailed') }}</NcNoteCard>
  </form>
</template>
