<script setup lang="ts">
import { computed } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import { mdiChevronLeft, mdiChevronRight, mdiPlus } from '@mdi/js'
import Icon from './Icon.vue'
import { locale, t } from './i18n'
import type { Plan } from './planning-api'
const props = defineProps<{
  meals: Plan[]
  members: { id: string; name: string }[]
  title: (plan: Plan) => string
  writable: boolean
  disabled: boolean
  reviewSlots: { date: string; meal: string }[]
}>()
const emit = defineEmits<{ select: [date: string, meal: string, plan?: Plan] }>()
const selectedDate = defineModel<string>('date', { required: true })
const anchor = computed({
  get: () => new Date(`${selectedDate.value}T12:00:00`),
  set: (value: Date) => {
    selectedDate.value = iso(value)
  },
})
const slots = ['breakfast', 'lunch', 'snack', 'dinner'] satisfies Array<
  'breakfast' | 'lunch' | 'snack' | 'dinner'
>
const iso = (date: Date) =>
  `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
const days = computed(() => {
  const monday = new Date(
    anchor.value.getFullYear(),
    anchor.value.getMonth(),
    anchor.value.getDate(),
    12,
  )
  monday.setDate(monday.getDate() - ((monday.getDay() + 6) % 7))
  const end = new Date(monday)
  end.setDate(end.getDate() + 6)
  const extraSlots = [
    ...new Set(
      props.meals
        .filter(
          (plan) =>
            plan.date >= iso(monday) &&
            plan.date <= iso(end) &&
            !slots.some((slot) => slot === plan.meal),
        )
        .map((plan) => plan.meal),
    ),
  ]
  return Array.from({ length: 7 }, (_, index) => {
    const date = new Date(monday)
    date.setDate(date.getDate() + index)
    return {
      cells: [
        ...slots.map((slot) => ({
          slot,
          label: t(slot),
          plan: props.meals.find((item) => item.date === iso(date) && item.meal === slot),
        })),
        ...extraSlots.map((slot) => ({
          slot,
          label: slot,
          plan: props.meals.find((plan) => plan.date === iso(date) && plan.meal === slot),
        })),
      ],
      iso: iso(date),
      label: new Intl.DateTimeFormat(locale.value, {
        weekday: 'long',
        day: 'numeric',
        month: 'short',
      }).format(date),
    }
  })
})
const range = computed(() => `${days.value[0]?.label} – ${days.value[6]?.label}`)
function move(offset: number) {
  const date = new Date(anchor.value)
  date.setDate(date.getDate() + offset)
  anchor.value = date
}
function people(plan: Plan, present: boolean) {
  const ids = plan.attendees ?? plan.calendar?.member_ids ?? []
  return props.members
    .filter((member) => ids.includes(member.id) === present)
    .map((member) => member.name)
}
</script>
<template>
  <section class="meal-calendar" :aria-label="t('plan')">
    <div class="calendar-toolbar">
      <h2 id="mealshop-calendar-title" tabindex="-1" aria-live="polite">{{ range }}</h2>
      <div class="actions">
        <NcButton :aria-label="t('previousWeek')" @click="move(-7)"
          ><template #icon><Icon :path="mdiChevronLeft" /></template
        ></NcButton>
        <NcButton @click="anchor = new Date()">{{ t('today') }}</NcButton>
        <NcButton :aria-label="t('nextWeek')" @click="move(7)"
          ><template #icon><Icon :path="mdiChevronRight" /></template
        ></NcButton>
      </div>
    </div>
    <div
      class="calendar-days"
      :style="{
        '--meal-slot-count': days[0]?.cells.length ?? 4,
        '--meal-day-tracks': (days[0]?.cells.length ?? 4) + 1,
      }"
    >
      <section v-for="day in days" :key="day.iso" class="calendar-day" :aria-label="day.label">
        <h3>
          <time :datetime="day.iso">{{ day.label }}</time>
        </h3>
        <div
          v-for="cell in day.cells"
          :key="cell.slot"
          class="meal-slot"
          :data-date="day.iso"
          :data-meal="cell.slot"
        >
          <h4>{{ cell.label }}</h4>
          <template v-if="cell.plan">
            <strong>{{ title(cell.plan) }}</strong>
            <p>{{ t('eating') }}: {{ people(cell.plan, true).join(', ') }}</p>
            <details v-if="people(cell.plan, false).length" class="meal-skips">
              <summary>{{ t('skipping') }}: {{ people(cell.plan, false).length }}</summary>
              <p>{{ people(cell.plan, false).join(', ') }}</p>
            </details>
            <p
              v-if="reviewSlots.some((item) => item.date === day.iso && item.meal === cell.slot)"
              class="muted"
            >
              {{ t('reviewRequired') }}
            </p>
            <NcButton
              :aria-label="`${t('details')} ${day.iso} ${cell.label}`"
              @click="emit('select', day.iso, cell.slot, cell.plan)"
              >{{ t('details') }}</NcButton
            >
          </template>
          <NcButton
            v-else-if="writable"
            variant="tertiary"
            :disabled="disabled"
            :aria-label="`${t('emptySlot')} ${day.iso} ${cell.label}`"
            @click="emit('select', day.iso, cell.slot)"
            ><template #icon><Icon :path="mdiPlus" /></template
          ></NcButton>
          <span v-else>–</span>
        </div>
      </section>
    </div>
  </section>
</template>
