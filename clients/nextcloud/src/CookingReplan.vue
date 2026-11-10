<script setup lang="ts">
import { nextTick, ref } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import CookingLocalTime from './CookingLocalTime.vue'
import CookingReview from './CookingReview.vue'
import { localInput } from './cooking-format'
import type { CookingRun, CookingPreview, PlanChange, Proposal, LocalTime } from './cooking-api'
import { t } from './i18n'
const props = defineProps<{
  run: CookingRun
  revision: number
  stepId?: string
  kind: 'target' | 'delay' | 'timer'
  disabled: boolean
  now: number
  preview: (proposal: Proposal, revision: number) => Promise<CookingPreview | undefined>
  apply: (preview: CookingPreview) => Promise<boolean>
}>()
const emit = defineEmits<{ close: []; dirty: [value: boolean] }>()
const current = props.run.steps.find((s) => s.stage.id === props.stepId)
const value = ref<LocalTime>(
  props.kind === 'target' && props.run.target
    ? { ...props.run.target }
    : {
        local: localInput(
          current?.timer.state === 'running' ? current.timer.deadline : props.now + 3600,
          props.run.timezone,
        ),
        timezone: props.run.timezone,
        fold: 'reject',
      },
)
const review = ref<CookingPreview>(),
  reviewed = ref(false),
  base = props.revision
async function preview() {
  emit('dirty', true)
  const change: PlanChange =
    props.kind === 'target'
      ? { kind: 'target', target: value.value }
      : props.kind === 'delay'
        ? { kind: 'delay', step_id: props.stepId ?? '', start_after: value.value }
        : { kind: 'timer', step_id: props.stepId ?? '', deadline: value.value }
  review.value = await props.preview({ kind: 'replan', run_id: props.run.id, change }, base)
  reviewed.value = false
  if (review.value) {
    await nextTick()
    document.getElementById('mealshop-cooking-review')?.focus()
  }
}
async function apply() {
  if (review.value && (await props.apply(review.value))) {
    emit('dirty', false)
    emit('close')
  }
}
</script>
<template>
  <section class="cooking-replan">
    <h2 tabindex="-1">
      {{
        t(
          kind === 'target'
            ? 'cookChangeTarget'
            : kind === 'delay'
              ? 'cookDelayStep'
              : 'cookChangeTimer',
        )
      }}
    </h2>
    <p v-if="current">{{ current.stage.name }}</p>
    <CookingReview
      v-if="review"
      v-model:reviewed="reviewed"
      :preview="review"
      :busy="disabled"
      :writable="true"
      @apply="apply"
      @back="review = undefined"
    />
    <form v-else @submit.prevent="preview" @input="emit('dirty', true)">
      <CookingLocalTime
        v-model="value"
        :label="
          t(kind === 'target' ? 'cookTarget' : kind === 'delay' ? 'cookStartAfter' : 'cookDeadline')
        "
      />
      <div class="actions">
        <NcButton type="submit" variant="primary" :disabled="disabled">{{
          t('cookPreview')
        }}</NcButton
        ><NcButton :disabled="disabled" @click="emit('close')">{{ t('cancel') }}</NcButton>
      </div>
    </form>
  </section>
</template>
