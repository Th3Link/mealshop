<script setup lang="ts">
import { computed } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import { mdiChevronLeft, mdiChevronRight, mdiPlus } from '@mdi/js'
import Icon from './Icon.vue'
import { locale, t } from './i18n'
import type { CookingList } from './cooking-api'
import { localInput } from './cooking-format'
import type { Plan } from './planning-api'
const props = defineProps<{
  meals: Plan[]
  runs: CookingList['runs']
  home: string
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
const slots = ['preparation', 'breakfast', 'lunch', 'snack', 'dinner'] satisfies Array<
  'preparation' | 'breakfast' | 'lunch' | 'snack' | 'dinner'
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
        .flatMap((plan) => plan.allocations ?? [plan])
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
          label: slot === 'preparation' ? t('prepareSlot') : t(slot),
          plan: props.meals.find((item) =>
            item.allocations
              ? item.allocations.some((a) => a.date === iso(date) && a.meal === slot)
              : item.date === iso(date) &&
                item.meal === slot &&
                !(slot === 'preparation' && item.preparation),
          ),
          preparations:
            slot === 'preparation'
              ? props.meals.filter(
                  (p) =>
                    (p.preparation?.date ?? (p.allocations?.length === 0 ? p.date : '')) ===
                      iso(date) &&
                    !props.runs.some(
                      (r) =>
                        r.status === 'active' &&
                        r.plan_slot?.date === p.date &&
                        r.plan_slot?.meal === p.meal,
                    ),
                )
              : [],
        })),
        ...extraSlots.map((slot) => ({
          slot,
          label: slot,
          preparations: [],
          plan: props.meals.find((plan) =>
            (plan.allocations ?? [plan]).some((a) => a.date === iso(date) && a.meal === slot),
          ),
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
          <template v-if="cell.slot === 'preparation'">
            <template v-for="run in runs.filter((r) => r.status === 'active')" :key="run.id">
              <div
                v-for="entry in (run.calendar_entries ?? []).filter(
                  (e) => localInput(e.at, run.timezone).slice(0, 10) === day.iso,
                )"
                :key="entry.step_id"
                class="calendar-preparation"
              >
                <strong>{{ run.name }}</strong>
                <p>{{ localInput(entry.at, run.timezone).slice(11, 16) }} · {{ entry.name }}</p>
                <NcButton :to="`/h/${home}/cooking/${run.id}`">{{ t('details') }}</NcButton>
              </div>
            </template>
          </template>
          <div
            v-for="prep in cell.preparations"
            :key="`${prep.date}-${prep.meal}`"
            class="calendar-preparation"
          >
            <strong>{{ title(prep) }}</strong>
            <p>{{ prep.preparation?.time }} · {{ prep.servings }} {{ t('servings') }}</p>
            <p v-if="prep.allocations?.length === 0" class="muted">{{ t('unallocated') }}</p>
            <NcButton :disabled="disabled" @click="emit('select', prep.date, prep.meal, prep)">{{
              t('details')
            }}</NcButton>
          </div>
          <template v-if="cell.plan">
            <strong>{{ title(cell.plan) }}</strong>
            <p>
              {{
                cell.plan.allocations?.find((a) => a.date === day.iso && a.meal === cell.slot)
                  ?.servings ?? cell.plan.servings
              }}
              {{ t('servings') }}
            </p>
            <p>{{ t('eating') }}: {{ people(cell.plan, true).join(', ') }}</p>
            <details v-if="people(cell.plan, false).length" class="meal-skips">
              <summary>{{ t('skipping') }}: {{ people(cell.plan, false).length }}</summary>
              <p>{{ people(cell.plan, false).join(', ') }}</p>
            </details>
            <p
              v-if="
                reviewSlots.some(
                  (item) => item.date === cell.plan?.date && item.meal === cell.plan?.meal,
                )
              "
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
