<script setup lang="ts">
import { computed, watch, toRaw } from 'vue'
import { z } from 'zod'
import NcTextField from '@nextcloud/vue/components/NcTextField'
import IngredientSelect from './IngredientSelect.vue'
import { locale, exact, parseGrams, t } from './i18n'
import type { Ingredient } from './contracts'
import { conversionsSchema } from './quantity-api'
import type { LineDraft } from './quantity-api'
const model = defineModel<LineDraft>({ required: true })
const emit = defineEmits<{ changed: [] }>()
const props = defineProps<{
  ingredients: Ingredient[]
  conversions: z.infer<typeof conversionsSchema>
  index: number
  invalid: boolean
}>()
const ingredient = computed(() => props.ingredients.find((i) => i.id === model.value.ingredient_id))
const mass = computed(() => model.value.unit === 'g' || model.value.unit === 'kg')
const options = computed(() =>
  props.conversions.entries.filter(
    (e) =>
      e.unit === model.value.unit &&
      e.evidence.ingredient_id === ingredient.value?.id &&
      e.evidence.form === ingredient.value?.form &&
      e.evidence.source_record_id === ingredient.value?.source.record_id,
  ),
)
function reset() {
  emit('changed')
  model.value.evidence = null
  model.value.mode = 'manual'
  model.value.density = ''
  model.value.spoon = ''
  model.value.invalidFields = []
}
function choose() {
  emit('changed')
  const evidence = options.value.find((e) => e.id === model.value.mode)?.evidence
  if (evidence) {
    model.value.evidence = structuredClone(toRaw(evidence))
    model.value.density = exact(evidence.grams_per_base_unit)
    model.value.spoon = evidence.millilitres_per_spoon ? exact(evidence.millilitres_per_spoon) : ''
  } else if (model.value.mode === 'manual') {
    model.value.evidence = {
      ingredient_id: model.value.ingredient_id,
      form: ingredient.value?.form ?? '',
      source_url: '',
      version: '',
      uncertainty: '',
      grams_per_base_unit: '',
      spoon_standard: null,
      millilitres_per_spoon: null,
    }
  }
}
function updateEvidence(
  key: 'source_url' | 'version' | 'uncertainty' | 'spoon_standard',
  value: string | number,
) {
  if (!model.value.evidence) choose()
  if (model.value.evidence) model.value.evidence[key] = String(value)
}
watch(locale, (_, previous) => {
  for (const key of ['input', 'density', 'spoon'] satisfies ('input' | 'density' | 'spoon')[]) {
    const value = model.value[key]
    if (!value) continue
    const canonical = parseGrams(value, previous)
    if (canonical) model.value[key] = exact(canonical)
    else model.value.invalidFields.push(key)
  }
})
</script>
<template>
  <div class="quantity-line">
    <div class="line">
      <IngredientSelect v-model="model.ingredient_id" :ingredients="ingredients" @change="reset" />
      <NcTextField
        v-model="model.input"
        :label="model.unit === 'g' ? t('grams') : t('amount')"
        inputmode="decimal"
        maxlength="32"
        required
        :error="invalid && (!parseGrams(model.input) || model.invalidFields.includes('input'))"
        :helper-text="
          invalid && (!parseGrams(model.input) || model.invalidFields.includes('input'))
            ? t('quantityError')
            : undefined
        "
        @update:model-value="model.invalidFields = model.invalidFields.filter((k) => k !== 'input')"
      /><label class="field"
        >{{ t('unit')
        }}<select v-model="model.unit" :aria-label="t('unit')" @change="reset">
          <option v-for="u in ['g', 'kg', 'ml', 'l', 'tsp', 'tbsp', 'piece']" :key="u" :value="u">
            {{ u }}
          </option>
        </select></label
      >
    </div>
    <template v-if="!mass"
      ><p class="muted">{{ t('conversionHelp') }}</p>
      <label class="field"
        >{{ t('conversion')
        }}<select v-model="model.mode" :aria-label="t('conversion')" @change="choose">
          <option value="manual">{{ t('manual') }}</option>
          <option v-if="model.evidence && model.mode === 'saved'" value="saved">
            {{ t('source') }}
          </option>
          <option v-for="option in options" :key="option.id" :value="option.id">
            {{ option.label }}
          </option>
        </select></label
      >
      <template v-if="model.mode === 'manual'"
        ><NcTextField
          v-model="model.density"
          @update:model-value="
            model.invalidFields = model.invalidFields.filter((k) => k !== 'density')
          "
          :label="t('density')"
          inputmode="decimal"
          maxlength="32"
          required /><NcTextField
          :model-value="model.evidence?.source_url ?? ''"
          :label="t('sourceUrl')"
          type="url"
          required
          @update:model-value="updateEvidence('source_url', $event)" /><NcTextField
          :model-value="model.evidence?.version ?? ''"
          :label="t('version')"
          required
          @update:model-value="updateEvidence('version', $event)" /><NcTextField
          :model-value="model.evidence?.uncertainty ?? ''"
          :label="t('uncertainty')"
          required
          @update:model-value="updateEvidence('uncertainty', $event)" /><template
          v-if="model.unit === 'tsp' || model.unit === 'tbsp'"
          ><NcTextField
            :model-value="model.evidence?.spoon_standard ?? ''"
            :label="t('spoonStandard')"
            required
            @update:model-value="updateEvidence('spoon_standard', $event)" /><NcTextField
            v-model="model.spoon"
            @update:model-value="
              model.invalidFields = model.invalidFields.filter((k) => k !== 'spoon')
            "
            :label="t('spoonVolume')"
            inputmode="decimal"
            maxlength="32"
            required /></template
      ></template>
      <p v-else-if="model.evidence" lang="en">
        {{ model.evidence.uncertainty }} · {{ model.evidence.version }} ·
        <a :href="model.evidence.source_url" target="_blank" rel="noopener noreferrer">{{
          t('source')
        }}</a>
      </p>
    </template>
  </div>
</template>
