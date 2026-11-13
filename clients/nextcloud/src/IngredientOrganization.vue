<script setup lang="ts">
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import NcTextField from './LabeledField.vue'
import { computed, watch } from 'vue'
import NutrientInput from './NutrientInput.vue'
import FoodConstituents from './FoodConstituents.vue'
import type { Catalogue, Organization } from './contracts'
import { t, locale, nutrientLabel } from './i18n'
const model = defineModel<Organization>({ required: true })
const props = defineProps<{ catalogue: Catalogue; id?: string }>()
const groups = computed(() =>
  props.catalogue.ingredients.filter(
    (i) =>
      i.organization.is_group &&
      i.id !== props.id &&
      (!i.archived || i.id === model.value.group_id),
  ),
)
watch(
  () => [model.value.is_group, model.value.group_id],
  () => {
    if (model.value.is_group) model.value.group_id = null
    if (!model.value.group_id && model.value.nutrition.mode === 'inherit')
      model.value.nutrition = { mode: 'reference' }
  },
)
watch(
  () => model.value.nutrition,
  (profile) => {
    if (profile.mode === 'own')
      for (const key of ['beta_glucan', 'soluble_fibre'])
        if (profile.constituents[key]) delete profile.values[key]
  },
  { deep: true },
)
function production(event: Event) {
  if (!(event.target instanceof HTMLSelectElement)) return
  const value = event.target.value
  if (value === 'unknown' || value === 'purchased' || value === 'household_produced')
    model.value.production = value
}
function mode(event: Event) {
  if (!(event.target instanceof HTMLSelectElement)) return
  const value = event.target.value
  model.value.nutrition =
    value === 'own'
      ? { mode: 'own', values: {}, constituents: {}, note: '', source_url: '' }
      : value === 'inherit'
        ? { mode: 'inherit' }
        : { mode: 'reference' }
}
</script>
<template>
  <section class="editor-section nutrition-editor">
    <h3>{{ t('nutrients') }}</h3>
    <label class="field"
      >{{ t('nutritionMode')
      }}<select :value="model.nutrition.mode" @change="mode">
        <option value="reference">{{ t('referenceValues') }}</option>
        <option value="own">{{ t('ownValues') }}</option>
        <option v-if="model.group_id && !model.is_group" value="inherit">
          {{ t('inheritValues') }}
        </option>
      </select></label
    >
    <template v-if="model.nutrition.mode === 'own'">
      <p class="muted">{{ t('unknownValue') }} {{ t('rangeInputHelp') }}</p>
      <div class="nutrient-inputs">
        <NutrientInput
          v-for="n in catalogue.nutrient_definitions.filter(
            (n) => n.group === 'energy' || n.group === 'macro',
          )"
          :key="n.id"
          v-model="model.nutrition.values[n.id]"
          :label="`${nutrientLabel(n.id, n.label)} (${n.unit})`"
        />
      </div>
      <details>
        <summary>{{ t('moreNutrients') }}</summary>
        <div class="nutrient-inputs">
          <NutrientInput
            v-for="n in catalogue.nutrient_definitions.filter((n) => n.group === 'micro')"
            :key="n.id"
            v-model="model.nutrition.values[n.id]"
            :label="`${nutrientLabel(n.id, n.label)} (${n.unit})`"
          />
        </div>
      </details>
      <FoodConstituents v-model="model.nutrition.constituents" :catalogue="catalogue" />
      <NcTextField
        v-model="model.nutrition.note"
        :label="t('nutritionNote')"
        required
        maxlength="2000"
      />
      <NcTextField v-model="model.nutrition.source_url" :label="t('nutritionSource')" type="url" />
    </template>
  </section>
  <details class="editor-disclosure">
    <summary>{{ t('classificationSection') }}</summary>
    <div class="editor-fields">
      <NcCheckboxRadioSwitch v-model="model.is_group">{{
        t('groupIngredient')
      }}</NcCheckboxRadioSwitch>
      <label v-if="!model.is_group" class="field"
        >{{ t('ingredientGroup')
        }}<select v-model="model.group_id">
          <option :value="null">{{ t('noGroup') }}</option>
          <option v-for="group in groups" :key="group.id" :value="group.id">
            {{ group.translations[locale]?.name ?? group.name
            }}{{ group.archived ? ` · ${t('archived')}` : '' }}
          </option>
        </select></label
      >
      <label class="field"
        >{{ t('productionClass')
        }}<select :value="model.production ?? 'unknown'" @change="production">
          <option value="unknown">{{ t('unknown') }}</option>
          <option value="purchased">{{ t('purchasedDeclaration') }}</option>
          <option value="household_produced">{{ t('householdProduced') }}</option>
        </select></label
      >
      <div class="form-row">
        <label class="field"
          >{{ t('dietClass')
          }}<select v-model="model.diet">
            <option
              v-for="key in ['unknown', 'vegan', 'vegetarian', 'meat_fish'] as const"
              :key="key"
              :value="key"
            >
              {{ t(key) }}
            </option>
          </select></label
        >
        <label class="field"
          >{{ t('originClass')
          }}<select v-model="model.origin">
            <option
              v-for="key in ['unknown', 'plant', 'animal', 'mixed'] as const"
              :key="key"
              :value="key"
            >
              {{ t(key) }}
            </option>
          </select></label
        >
      </div>
    </div>
  </details>
</template>
