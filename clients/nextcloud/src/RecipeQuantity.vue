<script setup lang="ts">
import EstimateBadge from './EstimateBadge.vue'
import { hasEstimates, hasMissingNutrition } from './nutrition-evidence'
import { computed, watch, toRaw, ref } from 'vue'
import { z } from 'zod'
import NcTextField from './LabeledField.vue'
import IngredientSelect from './IngredientSelect.vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcActions from '@nextcloud/vue/components/NcActions'
import NcActionButton from '@nextcloud/vue/components/NcActionButton'
import Icon from './Icon.vue'
import { mdiPlus, mdiThermometer, mdiChefHat, mdiClose } from '@mdi/js'
import { locale, exact, parseGrams, parseNonnegative, t } from './i18n'
import type { Ingredient } from './contracts'
import { conversionsSchema } from './quantity-api'
import type { LineDraft } from './quantity-api'
const model = defineModel<LineDraft>({ required: true })
const emit = defineEmits<{ changed: []; create: [name: string]; prepare: []; buy: [] }>()
const props = defineProps<{
  ingredients: Ingredient[]
  conversions: z.infer<typeof conversionsSchema>
  index: number
  invalid: boolean
  preparedName?: string
  canPrepare?: boolean
  parts: { id: string; label: string }[]
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
const temperature = computed(() => model.value.extensions?.find((e) => e.type === 'temperature'))
const temperatureInput = ref('')
watch(
  () => temperature.value?.value,
  (value) => {
    temperatureInput.value = value ? exact(value) : ''
  },
  { immediate: true },
)
const invalidTemperature = ref(model.value.invalidFields.includes('temperature'))
watch(locale, () => {
  if (!invalidTemperature.value && temperature.value)
    temperatureInput.value = exact(temperature.value.value)
})
function updateTemperature(value: string | number) {
  temperatureInput.value = String(value)
  const negative = temperatureInput.value.startsWith('-')
  const parsed = parseNonnegative(
    negative ? temperatureInput.value.slice(1) : temperatureInput.value,
  )
  const canonical = parsed ? `${negative ? '-' : ''}${parsed}` : null
  invalidTemperature.value = !canonical || Number(canonical) < -273.15 || Number(canonical) > 1000
  model.value.invalidFields = model.value.invalidFields.filter((key) => key !== 'temperature')
  if (invalidTemperature.value) model.value.invalidFields.push('temperature')
  else if (canonical)
    model.value.extensions = [
      {
        type: 'temperature',
        value: canonical,
        unit: 'celsius',
        original_input: `${temperatureInput.value} °C`,
        provenance: 'recipe_author',
      },
    ]
  emit('changed')
}
function addTemperature() {
  model.value.extensions = [
    {
      type: 'temperature',
      value: '',
      unit: 'celsius',
      original_input: '',
      provenance: 'recipe_author',
    },
  ]
  invalidTemperature.value = true
  model.value.invalidFields.push('temperature')
  emit('changed')
}
function removeTemperature() {
  model.value.extensions = []
  model.value.invalidFields = model.value.invalidFields.filter((key) => key !== 'temperature')
  invalidTemperature.value = false
  emit('changed')
}
function reset() {
  emit('changed')
  model.value.evidence = null
  model.value.mode = 'manual'
  model.value.density = ''
  model.value.spoon = ''
  model.value.invalidFields = model.value.invalidFields.filter((key) => key === 'temperature')
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
      <IngredientSelect
        v-model="model.ingredient_id"
        :ingredients="ingredients"
        allow-create
        hide-label
        @create="emit('create', $event)"
        @change="reset"
      />
      <NcTextField
        v-model="model.input"
        :label="model.unit === 'g' ? t('grams') : t('amount')"
        hide-label
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
        ><span class="visually-hidden">{{ t('unit') }}</span
        ><select v-model="model.unit" :aria-label="t('unit')" @change="reset">
          <option v-for="u in ['g', 'kg', 'ml', 'l', 'tsp', 'tbsp', 'piece']" :key="u" :value="u">
            {{ u }}
          </option>
        </select></label
      >
      <NcActions force-menu :aria-label="`${t('ingredientExtras')} ${index + 1}`">
        <template #icon><Icon :path="mdiPlus" /></template>
        <NcActionButton close-after-click v-if="!temperature" @click="addTemperature"
          ><template #icon><Icon :path="mdiThermometer" /></template
          >{{ t('ingredientTemperature') }}</NcActionButton
        >
        <NcActionButton close-after-click v-if="model.prepared_part" @click="emit('buy')">{{
          t('buyIngredient')
        }}</NcActionButton>
        <NcActionButton
          close-after-click
          v-else
          :disabled="!model.ingredient_id || canPrepare === false"
          @click="emit('prepare')"
          ><template #icon><Icon :path="mdiChefHat" /></template
          >{{ t('makeIngredient') }}</NcActionButton
        >
        <NcActionButton close-after-click v-if="model.part_id" @click="model.part_id = null"
          >{{ t('moveToPart') }} · {{ t('mainRecipe') }}</NcActionButton
        >
        <NcActionButton
          close-after-click
          v-for="part in parts.filter(
            (p) => p.id !== model.part_id && p.id !== model.prepared_part,
          )"
          :key="part.id"
          @click="model.part_id = part.id"
          >{{ t('moveToPart') }} · {{ part.label }}</NcActionButton
        >
      </NcActions>
    </div>
    <EstimateBadge
      v-if="
        ingredient &&
        !model.prepared_part &&
        (hasEstimates(ingredient) || hasMissingNutrition(ingredient))
      "
      :unknown="ingredient ? hasMissingNutrition(ingredient) : false"
    />
    <div v-if="temperature" class="ingredient-extension">
      <NcTextField
        :model-value="temperatureInput"
        :label="t('ingredientTemperature')"
        hide-label
        required
        :placeholder="t('ingredientTemperature')"
        inputmode="decimal"
        :error="invalidTemperature"
        :helper-text="invalidTemperature ? t('temperatureError') : undefined"
        @update:model-value="updateTemperature"
      />
      <span aria-hidden="true">°C</span>
      <NcButton variant="tertiary" :aria-label="t('removeTemperature')" @click="removeTemperature"
        ><template #icon><Icon :path="mdiClose" /></template
      ></NcButton>
    </div>
    <a
      v-if="model.prepared_part"
      class="prepared-reference"
      :href="`#recipe-part-${model.prepared_part}`"
      >{{ t('homemade') }} · {{ preparedName }}</a
    >
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
