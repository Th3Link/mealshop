<script setup lang="ts">
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import NcTextField from '@nextcloud/vue/components/NcTextField'
import { computed, watch } from 'vue'
import type { Catalogue, Organization } from './contracts'
import { t, locale, exact, parseGrams, nutrientLabel } from './i18n'
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
function mode(event: Event) {
  if (!(event.target instanceof HTMLSelectElement)) return
  const value = event.target.value
  model.value.nutrition =
    value === 'own'
      ? { mode: 'own', values: {}, note: '', source_url: '' }
      : value === 'inherit'
        ? { mode: 'inherit' }
        : { mode: 'reference' }
}
function amount(id: string, event: Event) {
  if (!(event.target instanceof HTMLInputElement) || model.value.nutrition.mode !== 'own') return
  const input = event.target.value
  const zero = locale.value === 'de' ? /^0(?:,0{1,6})?$/ : /^0(?:\.0{1,6})?$/
  const canonical = parseGrams(input) ?? (zero.test(input) ? input.replace(',', '.') : null)
  event.target.setCustomValidity(input && canonical === null ? t('quantityError') : '')
  if (!input || canonical !== null)
    model.value.nutrition.values[id] = {
      amount: canonical,
      uncertainty: 'Household declared value; verify the supplied evidence',
    }
}
</script>
<template>
  <fieldset>
    <legend>{{ t('ingredientGroup') }}</legend>
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
      ><label class="field"
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
      <p>{{ t('unknownValue') }}</p>
      <NcTextField
        v-model="model.nutrition.note"
        :label="t('nutritionNote')"
        required
        maxlength="2000"
      />
      <NcTextField v-model="model.nutrition.source_url" :label="t('nutritionSource')" type="url" />
      <div class="nutrient-inputs">
        <label v-for="nutrient in catalogue.nutrient_definitions" :key="nutrient.id" class="field"
          >{{ nutrientLabel(nutrient.id, nutrient.label) }} ({{ nutrient.unit }})<input
            :value="
              model.nutrition.values[nutrient.id]?.amount == null
                ? ''
                : exact(model.nutrition.values[nutrient.id]?.amount ?? null)
            "
            inputmode="decimal"
            maxlength="32"
            @input="amount(nutrient.id, $event)"
        /></label>
      </div>
    </template>
  </fieldset>
</template>
