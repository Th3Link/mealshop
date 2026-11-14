<script setup lang="ts">
import { computed } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import type { Ingredient, Member, PersonRules } from './contracts'
import IngredientPicker from './IngredientPicker.vue'
import { locale, t } from './i18n'
const props = defineProps<{ member: Member; ingredients: Ingredient[] }>()
const emit = defineEmits<{ changed: [] }>()
const defaults = (): PersonRules => ({
  version: 1,
  preset: 'unrestricted',
  exceptions: [],
  max_active_seconds: null,
  availability: [],
})
const rules = computed(() => props.member.rules ?? defaults())
function edit(action: (value: PersonRules) => void) {
  props.member.rules ??= defaults()
  action(props.member.rules)
  emit('changed')
}
function preset(event: Event) {
  if (!(event.target instanceof HTMLSelectElement)) return
  const value = event.target.value
  if (value === 'unrestricted' || value === 'vegan' || value === 'vegetarian')
    edit((r) => (r.preset = value))
}
const exceptionIds = computed({
  get: () => rules.value.exceptions.map((e) => e.ingredient_id),
  set: (ids: string[]) =>
    edit(
      (r) =>
        (r.exceptions = ids.map(
          (id) =>
            r.exceptions.find((e) => e.ingredient_id === id) ?? {
              ingredient_id: id,
              household_produced_only: false,
            },
        )),
    ),
})
function active(event: Event) {
  if (!(event.target instanceof HTMLInputElement)) return
  const value = event.target.value
  edit((r) => (r.max_active_seconds = value === '' ? null : Number(value)))
}
function meal(event: Event, index: number) {
  if (!(event.target instanceof HTMLSelectElement)) return
  const value = event.target.value
  if (value === 'breakfast' || value === 'lunch' || value === 'snack' || value === 'dinner')
    edit((r) => {
      const window = r.availability[index]
      if (window) window.meal = value
    })
}
function day(index: number, weekday: number, checked: boolean) {
  edit((r) => {
    const window = r.availability[index]
    if (window)
      window.weekdays = checked
        ? [...new Set([...window.weekdays, weekday])]
        : window.weekdays.filter((d) => d !== weekday)
  })
}
function name(id: string) {
  const item = props.ingredients.find((i) => i.id === id)
  return item?.translations[locale.value]?.name ?? item?.name ?? id
}
</script>
<template>
  <section class="editor-section">
    <h3>{{ t('personRules') }}</h3>
    <label class="field"
      >{{ t('dietClass')
      }}<select :value="rules.preset" @change="preset">
        <option value="unrestricted">{{ t('unrestricted') }}</option>
        <option value="vegetarian">{{ t('vegetarian') }}</option>
        <option value="vegan">{{ t('vegan') }}</option>
      </select></label
    >
    <NcNoteCard type="info">{{ t('rulePriority') }}</NcNoteCard>
    <IngredientPicker
      v-model="exceptionIds"
      :ingredients="ingredients"
      :label="t('ruleExceptions')"
    />
    <NcCheckboxRadioSwitch
      v-for="exception in rules.exceptions"
      :key="exception.ingredient_id"
      :model-value="exception.household_produced_only"
      @update:model-value="
        edit((r) => {
          const e = r.exceptions.find((e) => e.ingredient_id === exception.ingredient_id)
          if (e) e.household_produced_only = $event === true
        })
      "
      >{{ name(exception.ingredient_id) }} · {{ t('householdOnly') }}</NcCheckboxRadioSwitch
    >
    <label class="field"
      >{{ t('maxPersonActive')
      }}<input
        type="number"
        min="1"
        max="604800"
        step="1"
        :value="rules.max_active_seconds ?? ''"
        @input="active"
    /></label>
    <h4>{{ t('availabilityWindows') }}</h4>
    <p class="muted">{{ t('availabilityHelp') }}</p>
    <fieldset v-for="(window, index) in rules.availability" :key="index">
      <legend>{{ index + 1 }}</legend>
      <label class="field"
        >{{ t('meal')
        }}<select :value="window.meal" @change="meal($event, index)">
          <option
            v-for="key in ['breakfast', 'lunch', 'snack', 'dinner'] as const"
            :key="key"
            :value="key"
          >
            {{ t(key) }}
          </option>
        </select></label
      >
      <div class="allergen-options">
        <NcCheckboxRadioSwitch
          v-for="(label, weekday) in t('ruleDays').split(' · ')"
          :key="weekday"
          :model-value="window.weekdays.includes(weekday)"
          @update:model-value="day(index, weekday, $event === true)"
          >{{ label }}</NcCheckboxRadioSwitch
        >
      </div>
      <NcButton @click="edit((r) => r.availability.splice(index, 1))">{{ t('remove') }}</NcButton>
    </fieldset>
    <NcButton
      :disabled="rules.availability.length >= 28"
      @click="edit((r) => r.availability.push({ weekdays: [0, 1, 2, 3, 4, 5, 6], meal: 'dinner' }))"
      >{{ t('addAvailability') }}</NcButton
    >
  </section>
</template>
