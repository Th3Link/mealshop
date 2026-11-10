<script setup lang="ts">
import { computed, watch } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import type { CookingRun, CookingCommand } from './cooking-api'
import { duration, localDate } from './cooking-format'
import { t } from './i18n'
const props = defineProps<{ run: CookingRun; now: number; disabled: boolean }>()
const emit = defineEmits<{
  command: [value: CookingCommand]
  replan: [stepId: string, kind: 'delay' | 'timer']
  announce: [value: string]
}>()
const ordered = computed(() => [...props.run.steps].sort((a, b) => rank(a) - rank(b)))
function rank(step: CookingRun['steps'][number]) {
  return step.timer.state === 'running'
    ? step.timer.deadline <= props.now
      ? 0
      : 1
    : step.timer.state === 'paused'
      ? 2
      : step.ready
        ? 3
        : step.timer.state === 'pending'
          ? 4
          : 5
}
function phase(step: CookingRun['steps'][number]) {
  const timer = step.timer
  return timer.state === 'running'
    ? timer.deadline <= props.now
      ? 'cookElapsed'
      : timer.active_until > props.now
        ? 'cookActivePhase'
        : 'cookWaitingPhase'
    : (
        {
          pending: step.ready ? 'cookReady' : 'cookBlocked',
          paused: 'cookPaused',
          completed: 'cookCompleted',
          cancelled: 'cookCancelled',
        } as const
      )[timer.state]
}
const elapsed = computed(() =>
  props.run.steps
    .filter((s) => s.timer.state === 'running' && s.timer.deadline <= props.now)
    .map((s) => s.stage.id),
)
watch(elapsed, (value, previous) => {
  const added = value.filter((id) => !previous.includes(id))
  if (added.length)
    emit(
      'announce',
      `${t('cookElapsed')}: ${added.map((id) => props.run.steps.find((s) => s.stage.id === id)?.stage.name).join(', ')}`,
    )
})
const phaseEnded = computed(() =>
  props.run.steps
    .filter(
      (s) =>
        s.timer.state === 'running' &&
        s.timer.active_until > s.timer.started_at &&
        s.timer.active_until <= props.now &&
        s.timer.deadline > props.now,
    )
    .map((s) => s.stage.id),
)
watch(phaseEnded, (value, previous) => {
  const added = value.filter((id) => !previous.includes(id))
  if (added.length)
    emit(
      'announce',
      `${t('cookEventActive')}: ${added.map((id) => props.run.steps.find((s) => s.stage.id === id)?.stage.name).join(', ')}`,
    )
})
function action(stepId: string, action: 'start' | 'pause' | 'resume' | 'complete') {
  emit('command', { operation: 'timer', run_id: props.run.id, step_id: stepId, action })
}
</script>
<template>
  <section>
    <h2>{{ t('cookSteps') }}</h2>
    <ol class="cooking-timeline">
      <li v-for="step in ordered" :key="step.stage.id" :data-cooking-step="step.stage.id">
        <div class="section-heading">
          <h3 :lang="run.recipe.language">
            {{ step.stage.name
            }}<template v-if="step.stage.repeat > 1">
              · {{ step.cycle }}/{{ step.stage.repeat }}</template
            >
          </h3>
          <span class="cooking-state">{{ t(phase(step)) }}</span>
        </div>
        <p v-if="step.stage.instruction !== step.stage.name">{{ step.stage.instruction }}</p>
        <p v-if="step.timer.state === 'running'" class="cooking-countdown">
          <span v-if="step.timer.deadline < now">{{ t('cookOverdueBy') }} </span
          ><span role="timer" aria-live="off">{{
            duration(Math.abs(step.timer.deadline - now))
          }}</span>
        </p>
        <p v-if="step.timer.state === 'paused'" class="cooking-countdown">
          {{ duration(step.timer.remaining) }} <span class="muted">{{ t('cookRemaining') }}</span>
        </p>
        <p v-if="step.timer.state === 'pending'" class="muted">
          {{ t('cookPreset') }}:
          {{ duration(step.stage.active_seconds + step.stage.passive_seconds) }} ·
          {{ t('cookPlannedStart') }}:
          {{
            localDate(run.schedule.find((s) => s.id === step.stage.id)?.start ?? now, run.timezone)
          }}
        </p>
        <p v-if="step.timer.state === 'pending' && !step.ready && step.available_at" class="muted">
          {{ t('cookWaitUntil') }}: {{ localDate(step.available_at, run.timezone) }}
        </p>
        <p v-if="step.stage.dependencies.length" class="muted">
          {{ t('cookDependencies') }}:
          {{
            step.stage.dependencies
              .map((id) => run.steps.find((s) => s.stage.id === id)?.stage.name ?? id)
              .join(', ')
          }}
        </p>
        <p v-if="step.stage.equipment.length">
          {{ t('cookEquipment') }}: {{ step.stage.equipment.join(', ') }}
        </p>
        <div v-if="run.status === 'active'" class="actions">
          <NcButton
            v-if="step.timer.state === 'pending'"
            :disabled="disabled || !step.ready"
            :aria-label="`${t('cookStartStep')}: ${step.stage.name}`"
            @click="action(step.stage.id, 'start')"
            >{{
              t(
                step.stage.active_seconds + step.stage.passive_seconds === 0
                  ? 'cookCompleteStep'
                  : 'cookStartStep',
              )
            }}</NcButton
          ><NcButton
            v-if="step.timer.state === 'running'"
            :disabled="disabled"
            @click="action(step.stage.id, 'pause')"
            >{{ t('cookPause') }}</NcButton
          ><NcButton
            v-if="step.timer.state === 'paused'"
            :disabled="disabled"
            @click="action(step.stage.id, 'resume')"
            >{{ t('cookResume') }}</NcButton
          ><NcButton
            v-if="step.timer.state === 'running' || step.timer.state === 'paused'"
            :disabled="disabled"
            @click="action(step.stage.id, 'complete')"
            >{{ t('cookCompleteStep') }}</NcButton
          ><NcButton
            v-if="step.timer.state === 'pending'"
            variant="tertiary"
            :disabled="disabled"
            @click="emit('replan', step.stage.id, 'delay')"
            >{{ t('cookDelayStep') }}</NcButton
          ><NcButton
            v-if="step.timer.state === 'running' || step.timer.state === 'paused'"
            variant="tertiary"
            :disabled="disabled"
            @click="emit('replan', step.stage.id, 'timer')"
            >{{ t('cookChangeTimer') }}</NcButton
          >
        </div>
        <details>
          <summary>{{ t('cookTimingDetails') }}</summary>
          <dl>
            <dt>{{ t('cookActiveDuration') }}</dt>
            <dd>{{ duration(step.stage.active_seconds) }}</dd>
            <dt>{{ t('cookPassiveDuration') }}</dt>
            <dd>{{ duration(step.stage.passive_seconds) }}</dd>
            <dt>{{ t('cookUncertainty') }}</dt>
            <dd>{{ step.stage.uncertainty || t('unknown') }}</dd>
            <dt>{{ t('cookPlannedFinish') }}</dt>
            <dd>{{ run.local_schedule.find((s) => s.id === step.stage.id)?.finish }}</dd>
          </dl>
        </details>
      </li>
    </ol>
  </section>
</template>
