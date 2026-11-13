<script setup lang="ts">
import { computed, ref, watch, toRaw } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import { mdiPlus, mdiTrashCanOutline } from '@mdi/js'
import Icon from './Icon.vue'
import LabeledField from './LabeledField.vue'
import BrewPhase from './BrewPhase.vue'
import BrewQuantity from './BrewQuantity.vue'
import BrewSources from './BrewSources.vue'
import IngredientSelect from './IngredientSelect.vue'
import type { Ingredient } from './contracts'
import type { BrewTemplates, BrewRecord, BrewRecipe, BrewCommand } from './brewing-api'
import {
  seedRecipe,
  localized,
  newPhase,
  methodOptions,
  fermentationProcesses,
  beverages,
} from './brewing-format'
import { t } from './i18n'
const props = defineProps<{
  catalogue: BrewTemplates
  record?: BrewRecord
  disabled: boolean
  ingredients: Ingredient[]
}>()
const emit = defineEmits<{ review: [value: BrewCommand]; dirty: [value: boolean] }>()
const originalId = props.record?.id ?? null
const originalRevision = props.record?.revision ?? null
const draft = ref<BrewRecipe | undefined>(
  props.record ? structuredClone(toRaw(props.record.recipe)) : undefined,
)
const template = ref(draft.value?.template.id ?? '')
const options = computed(
  () =>
    draft.value?.phases.map((p) => ({ id: p.stage.id, label: p.stage.name || t('brewPhase') })) ??
    [],
)
function choose(event: Event) {
  if (!(event.target instanceof HTMLSelectElement)) return
  const next = event.target.value
  if (draft.value && !window.confirm(t('brewTemplateReplace'))) {
    event.target.value = template.value
    return
  }
  template.value = next
  draft.value = seedRecipe(props.catalogue, next)
  emit('dirty', true)
}
function addIngredient() {
  draft.value?.ingredients.push({
    id: `line-${crypto.randomUUID()}`,
    name: '',
    ingredient_id: null,
    quantity: null,
    phase_id: null,
    note: '',
  })
  emit('dirty', true)
}
function removePhase(id: string) {
  if (!draft.value) return
  if (!window.confirm(t('brewRemovePhaseRefs'))) return
  draft.value.phases = draft.value.phases.filter((p) => p.stage.id !== id)
  for (const p of draft.value.phases)
    p.stage.dependencies = p.stage.dependencies.filter((v) => v !== id)
  for (const line of draft.value.ingredients) if (line.phase_id === id) line.phase_id = null
  emit('dirty', true)
}
function selectIngredient(line: BrewRecipe['ingredients'][number]) {
  const i = props.ingredients.find((i) => i.id === line.ingredient_id)
  if (i) line.name = i.translations[draft.value?.language ?? 'en']?.name ?? i.name
}
watch(
  () => (draft.value && 'strategy' in draft.value.method ? draft.value.method.strategy : undefined),
  (strategy) => {
    if (!draft.value || !('strategy' in draft.value.method)) return
    const mash = strategy === 'partial_mash' || strategy === 'all_grain'
    if (!mash) draft.value.method.mash = 'none'
    else if (draft.value.method.mash === 'none') draft.value.method.mash = 'infusion'
  },
)
function review() {
  if (draft.value)
    emit('review', {
      operation: 'save_recipe',
      id: originalId,
      expected_recipe_revision: originalRevision,
      recipe: structuredClone(toRaw(draft.value)),
    })
}
watch(draft, () => emit('dirty', true), { deep: true })
</script>
<template>
  <form
    class="brew-editor"
    @submit.prevent="review"
    @input="emit('dirty', true)"
    @change="emit('dirty', true)"
  >
    <fieldset :disabled="disabled">
      <legend class="visually-hidden">{{ t(record ? 'brewEdit' : 'brewNewRecipe') }}</legend>
      <label class="field"
        >{{ t('brewTemplate')
        }}<select :value="template" required @change="choose">
          <option value="" disabled>{{ t('brewMethod') }}</option>
          <optgroup v-for="(label, beverage) in beverages" :key="beverage" :label="t(label)">
            <option
              v-for="item in catalogue.templates.filter((t) => t.beverage === beverage)"
              :key="item.id"
              :value="item.id"
            >
              {{ localized(item) }}
            </option>
          </optgroup>
        </select></label
      >
      <p class="muted">{{ t('brewTemplateHelp') }}</p>
      <template v-if="draft"
        ><div class="form-row">
          <LabeledField v-model="draft.title" :label="t('title')" required maxlength="200" /><label
            class="field"
            >{{ t('brewRecipeLanguage')
            }}<select v-model="draft.language">
              <option value="en">English</option>
              <option value="de">Deutsch</option>
            </select></label
          ><BrewQuantity v-model="draft.volume" :label="t('brewVolume')" volume />
        </div>
        <div v-if="'strategy' in draft.method" class="form-row">
          <label v-for="(choices, field) in methodOptions" :key="field" class="field"
            >{{
              t(
                (
                  {
                    strategy: 'brewStrategy',
                    layout: 'brewLayout',
                    heat: 'brewHeat',
                    sparge: 'brewSparge',
                    mash: 'brewMash',
                  } as const
                )[field],
              )
            }}<select v-model="draft.method[field]">
              <option v-for="(key, value) in choices" :key="value" :value="value">
                {{ t(key) }}
              </option>
            </select></label
          >
        </div>
        <p v-else>{{ t('brewMethod') }}: {{ t(fermentationProcesses[draft.method.process]) }}</p>
        <section>
          <h2>{{ t('ingredients') }}</h2>
          <p class="muted">{{ t('brewIngredientHelp') }}</p>
          <div v-for="line in draft.ingredients" :key="line.id" class="brew-input-line">
            <div class="brew-line-heading">
              <LabeledField
                v-model="line.name"
                :label="t('ingredientColumn')"
                required
                maxlength="200"
              /><NcButton
                variant="tertiary"
                :aria-label="`${t('remove')}: ${line.name}`"
                @click="draft.ingredients = draft.ingredients.filter((l) => l.id !== line.id)"
                ><template #icon><Icon :path="mdiTrashCanOutline" /></template
              ></NcButton>
            </div>
            <div class="form-row">
              <BrewQuantity
                v-if="line.quantity"
                v-model="line.quantity"
                :label="t('brewAmount')"
              /><NcButton
                v-else
                variant="tertiary"
                @click="line.quantity = { amount: '', unit: 'g' }"
                ><template #icon><Icon :path="mdiPlus" /></template>{{ t('brewAmount') }}</NcButton
              ><label class="field"
                >{{ t('brewPhase')
                }}<select v-model="line.phase_id">
                  <option :value="null">{{ t('brewNone') }}</option>
                  <option v-for="p in options" :key="p.id" :value="p.id">{{ p.label }}</option>
                </select></label
              ><LabeledField v-model="line.note" :label="t('brewNotes')" maxlength="1000" />
            </div>
            <div v-if="line.ingredient_id !== null" class="form-row">
              <IngredientSelect
                :model-value="line.ingredient_id"
                :ingredients="ingredients"
                :disabled="disabled"
                :label="t('ingredientColumn')"
                @update:model-value="
                  (value) => {
                    line.ingredient_id = value
                    selectIngredient(line)
                  }
                "
              /><NcButton variant="tertiary" @click="line.ingredient_id = null">{{
                t('remove')
              }}</NcButton>
            </div>
            <div class="actions">
              <NcButton v-if="line.quantity" variant="tertiary" @click="line.quantity = null"
                >{{ t('remove') }} · {{ t('brewAmount') }}</NcButton
              ><NcButton
                v-if="line.ingredient_id === null"
                variant="tertiary"
                @click="line.ingredient_id = ''"
                ><template #icon><Icon :path="mdiPlus" /></template>{{ t('ingredients') }}</NcButton
              >
            </div>
          </div>
          <NcButton @click="addIngredient"
            ><template #icon><Icon :path="mdiPlus" /></template
            >{{ t('brewAddIngredient') }}</NcButton
          >
        </section>
        <section>
          <h2>{{ t('cookSteps') }}</h2>
          <BrewPhase
            v-for="(phase, index) in draft.phases"
            :key="phase.stage.id"
            v-model="draft.phases[index]!"
            :index="index"
            :disabled="disabled"
            :options="options"
            :catalogue="catalogue"
            :beverage="draft.beverage"
            @remove="removePhase(phase.stage.id)"
          /><NcButton @click="draft.phases.push(newPhase())"
            ><template #icon><Icon :path="mdiPlus" /></template>{{ t('brewAddPhase') }}</NcButton
          >
        </section>
        <section>
          <h2>{{ t('brewYeast') }}</h2>
          <p class="muted">{{ t('brewYeastHelp') }}</p>
          <template v-if="draft.yeast_preparation"
            ><div class="form-row">
              <LabeledField
                v-model.number="draft.yeast_preparation.days_before"
                type="number"
                min="0"
                max="30"
                :label="t('brewDaysBefore')"
                required
              /><LabeledField
                v-model="draft.yeast_preparation.local_time"
                type="time"
                :label="t('brewLocalTime')"
                required
              /><label class="field"
                >{{ t('cookFoldChoice')
                }}<select v-model="draft.yeast_preparation.fold">
                  <option value="reject">{{ t('cookFoldReject') }}</option>
                  <option value="earlier">{{ t('cookFoldEarlier') }}</option>
                  <option value="later">{{ t('cookFoldLater') }}</option>
                </select></label
              >
            </div>
            <LabeledField
              v-model="draft.yeast_preparation.instruction"
              :label="t('instruction')"
              required
              maxlength="4000"
            /><NcButton variant="tertiary" @click="delete draft.yeast_preparation">{{
              t('remove')
            }}</NcButton></template
          ><NcButton
            v-else
            variant="tertiary"
            @click="
              draft.yeast_preparation = {
                days_before: 1,
                local_time: '08:00',
                fold: 'reject',
                instruction: '',
              }
            "
            ><template #icon><Icon :path="mdiPlus" /></template>{{ t('brewAddYeast') }}</NcButton
          >
        </section>
        <label class="field"
          >{{ t('brewNotes') }}<textarea v-model="draft.notes" maxlength="4000" rows="2" /></label
        ><BrewSources :sources="draft.template.sources" />
        <div class="actions">
          <NcButton type="submit" variant="primary">{{ t('brewReview') }}</NcButton>
        </div></template
      >
    </fieldset>
  </form>
</template>
