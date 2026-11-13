<script setup lang="ts">
import { computed, ref, useId, watch } from 'vue'
import { z } from 'zod'
import NcSelect from '@nextcloud/vue/components/NcSelect'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcButton from '@nextcloud/vue/components/NcButton'
import LabeledField from './LabeledField.vue'
import IngredientSelect from './IngredientSelect.vue'
import IngredientEditor from './IngredientEditor.vue'
import { t } from './i18n'
import { loadCatalogue, useTask } from './state'
import type { Catalogue } from './contracts'
import type { Shopping } from './shopping-api'
import type { StockTarget } from './inventory-api'
const model = defineModel<StockTarget>({ required: true })
const props = defineProps<{
  catalogue: Catalogue
  products: Shopping['products']
  home: string
  disabled: boolean
  allowUnmatched?: boolean
}>()
const emit = defineEmits<{ nested: [open: boolean] }>()
const catalogue = ref(props.catalogue),
  creating = ref<string>(),
  id = useId()
watch(
  () => props.catalogue,
  (value) => {
    catalogue.value = value
  },
)
const options = computed(() =>
  Object.values(props.products).map((p) => ({
    barcode: p.barcode,
    label: [p.name, p.brand, p.barcode].filter(Boolean).join(' · '),
  })),
)
const selected = computed(() =>
  model.value.kind === 'product'
    ? (options.value.find(
        (p) => model.value.kind === 'product' && p.barcode === model.value.barcode,
      ) ?? null)
    : null,
)
const { busy, error, run, errorMessage } = useTask()
function kind(event: Event) {
  if (!(event.target instanceof HTMLSelectElement) || props.disabled) return
  switch (event.target.value) {
    case 'ingredient':
      model.value = { kind: 'ingredient', ingredient_id: '' }
      break
    case 'product':
      model.value = { kind: 'product', barcode: '' }
      break
    case 'unmatched':
      if (props.allowUnmatched) model.value = { kind: 'unmatched', label: '' }
  }
}
function product(value: unknown) {
  const result = z.object({ barcode: z.string() }).nullable().safeParse(value)
  if (result.success && !props.disabled)
    model.value = { kind: 'product', barcode: result.data?.barcode ?? '' }
}
function create(name: string) {
  if (props.disabled) return
  creating.value = name
  emit('nested', true)
}
function close() {
  creating.value = undefined
  emit('nested', false)
}
async function refresh() {
  await run(async () => {
    catalogue.value = await loadCatalogue(props.home)
  })
}
async function saved(id: string) {
  model.value = { kind: 'ingredient', ingredient_id: id }
  close()
  await refresh()
}
</script>
<template>
  <label class="field"
    >{{ t('stockBinding')
    }}<select
      :aria-label="t('stockBinding')"
      :value="model.kind"
      :disabled="disabled"
      @change="kind"
    >
      <option value="ingredient">{{ t('stockIngredient') }}</option>
      <option value="product">{{ t('stockProduct') }}</option>
      <option v-if="allowUnmatched" value="unmatched">{{ t('stockUnlinked') }}</option>
    </select></label
  >
  <IngredientSelect
    v-if="model.kind === 'ingredient'"
    v-model="model.ingredient_id"
    :ingredients="catalogue.ingredients"
    :label="t('stockIngredient')"
    allow-create
    :disabled="disabled || busy"
    @create="create"
  />
  <div v-else-if="model.kind === 'product'" class="labeled-field">
    <label :for="id">{{ t('stockProduct') }}</label
    ><NcSelect
      :input-id="id"
      :input-label="t('stockProduct')"
      label-outside
      :model-value="selected"
      :options="options"
      label="label"
      :disabled="disabled"
      @update:model-value="product"
    />
  </div>
  <template v-else
    ><LabeledField
      v-model="model.label"
      :label="t('stockName')"
      required
      maxlength="200"
      :disabled="disabled"
    />
    <p class="muted">{{ t('stockUnmatchedHelp') }}</p></template
  >
  <NcNoteCard v-if="error" type="error" role="alert"
    >{{ errorMessage(error)
    }}<NcButton :disabled="busy" @click="refresh">{{ t('retry') }}</NcButton></NcNoteCard
  >
  <IngredientEditor
    v-if="creating !== undefined"
    :home="home"
    :catalogue="catalogue"
    :initial-name="creating"
    parent-dirty
    @close="close"
    @saved="saved"
  />
</template>
