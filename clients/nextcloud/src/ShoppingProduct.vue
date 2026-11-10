<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import { errorMessage } from './api'
import NcDialog from '@nextcloud/vue/components/NcDialog'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import NcTextField from './LabeledField.vue'
import IngredientSelect from './IngredientSelect.vue'
import ShoppingQuantityInput from './ShoppingQuantityInput.vue'
import BarcodeCamera from './BarcodeCamera.vue'
import ProductEvidence from './ProductEvidence.vue'
import { canonicalGtin } from './shopping-api'
import type {
  Shopping,
  ShoppingCommand,
  ProductCandidate,
  ShoppingQuantity,
  Product,
} from './shopping-api'
import type { Catalogue } from './contracts'
import { t } from './i18n'
const props = defineProps<{
  error: unknown
  data: Shopping
  catalogue: Catalogue
  busy: boolean
  perform: (command: ShoppingCommand) => Promise<boolean>
  lookup: (barcode: string) => Promise<ProductCandidate | null | undefined>
}>()
const emit = defineEmits<{ dirty: [value: boolean] }>()
const open = ref(false),
  barcode = ref(''),
  name = ref(''),
  brand = ref(''),
  labelQuantity = ref(''),
  ingredient = ref(''),
  confirmed = ref(false),
  consent = ref(false),
  resolved = ref(false),
  candidate = ref<ProductCandidate>(),
  packages = ref(1),
  notice = ref(''),
  evidence = ref<Product['evidence']>(null)
const resolvedBarcode = ref('')
const quantity = ref<ShoppingQuantity>({ amount: '', unit: 'g' })
const state = computed(() =>
  JSON.stringify({
    barcode: barcode.value,
    name: name.value,
    brand: brand.value,
    label: labelQuantity.value,
    ingredient: ingredient.value,
    quantity: quantity.value,
    packages: packages.value,
  }),
)
const baseline = ref(state.value)
const dirty = computed(() => open.value && baseline.value !== state.value)
watch(dirty, (value) => emit('dirty', value))
watch(ingredient, () => (confirmed.value = false))
const validCode = computed(() => canonicalGtin(barcode.value))
const valid = computed(
  () =>
    resolved.value &&
    validCode.value &&
    name.value.trim() &&
    quantity.value.amount &&
    (!ingredient.value || confirmed.value),
)
function close() {
  if (!props.busy && (!dirty.value || window.confirm(t('unsaved')))) {
    open.value = false
    reset()
  }
}
function reset() {
  barcode.value = ''
  name.value = ''
  brand.value = ''
  labelQuantity.value = ''
  ingredient.value = ''
  confirmed.value = false
  consent.value = false
  resolved.value = false
  resolvedBarcode.value = ''
  candidate.value = undefined
  packages.value = 1
  notice.value = ''
  evidence.value = null
  quantity.value = { amount: '', unit: 'g' }
  baseline.value = state.value
}
function resolve() {
  if (resolvedBarcode.value && dirty.value && !window.confirm(t('unsaved'))) {
    barcode.value = resolvedBarcode.value
    resolved.value = true
    return
  }
  if (!validCode.value) return
  const product = props.data.products[validCode.value]
  name.value = product?.name ?? ''
  brand.value = product?.brand ?? ''
  labelQuantity.value = product?.label_quantity ?? ''
  ingredient.value = product?.ingredient_id ?? ''
  confirmed.value = false
  quantity.value = product ? { ...product.quantity } : { amount: '', unit: 'g' }
  evidence.value = product?.evidence ?? null
  candidate.value = undefined
  consent.value = false
  resolved.value = true
  resolvedBarcode.value = barcode.value
  notice.value = product ? '' : t('productUnresolved')
  baseline.value = state.value
}
function code(value: string) {
  barcode.value = value
  resolve()
}
function changedBarcode() {
  resolved.value = false
  candidate.value = undefined
  evidence.value = null
}
async function remove(product: Product) {
  if (window.confirm(t('productRemoveHelp')))
    await props.perform({ operation: 'remove_product', barcode: product.barcode })
}
function saved(product: Product) {
  reset()
  barcode.value = product.barcode
  resolve()
  open.value = true
}
async function lookup() {
  if (!consent.value || !validCode.value) return
  if (dirty.value && !window.confirm(t('unsaved'))) return
  const found = await props.lookup(barcode.value)
  if (found === undefined) return
  if (found === null) {
    notice.value = t('productUnresolved')
    return
  }
  candidate.value = found
  evidence.value = found.evidence
  name.value = found.evidence.original_name
  brand.value = found.evidence.original_brand
  labelQuantity.value = found.evidence.original_quantity
  quantity.value = { amount: '', unit: 'g' }
  notice.value = t('productVerifyPackage')
}
async function save(add: boolean) {
  if (!valid.value) return
  if (
    await props.perform({
      operation: 'save_product',
      product: {
        barcode: barcode.value,
        name: name.value,
        brand: brand.value,
        label_quantity: labelQuantity.value,
        quantity: quantity.value,
        ingredient_id: ingredient.value || null,
        match_confirmed: confirmed.value,
        candidate_id: candidate.value?.id,
      },
    })
  ) {
    baseline.value = state.value
    if (
      !add ||
      (await props.perform({
        operation: 'add_product',
        barcode: barcode.value,
        packages: Number(packages.value),
      }))
    ) {
      notice.value = t(add ? 'productAdded' : 'changesSaved')
      if (add) {
        open.value = false
        reset()
      }
    }
  }
}
</script>
<template>
  <NcButton :disabled="busy" @click="open = true">{{ t('productAdd') }}</NcButton>
  <details v-if="Object.keys(data.products).length">
    <summary>{{ t('productSaved') }} · {{ Object.keys(data.products).length }}</summary>
    <ul class="plain-list">
      <li v-for="product in data.products" :key="product.barcode">
        <div class="actions">
          <NcButton :disabled="busy" @click="saved(product)"
            >{{ product.name }} · {{ product.barcode }}</NcButton
          ><NcButton :disabled="busy" @click="remove(product)">{{ t('remove') }}</NcButton>
        </div>
      </li>
    </ul>
  </details>
  <NcDialog
    v-if="open"
    size="normal"
    content-classes="mealshop product-dialog"
    :name="t('productAdd')"
    @close="close"
  >
    <form @submit.prevent="save(true)">
      <fieldset class="editor-fields" :disabled="busy">
        <NcNoteCard v-if="error" type="error" role="alert">{{ errorMessage(error) }}</NcNoteCard>
        <p>{{ t('productHelp') }}</p>
        <NcTextField
          v-model="barcode"
          :label="t('barcode')"
          inputmode="numeric"
          autocomplete="off"
          maxlength="14"
          :aria-invalid="Boolean(barcode && !validCode)"
          :helper-text="barcode && !validCode ? t('barcodeInvalid') : undefined"
          @update:model-value="changedBarcode"
        />
        <div class="actions">
          <NcButton :disabled="busy || !validCode" @click="resolve">{{
            t('barcodeResolve')
          }}</NcButton
          ><BarcodeCamera v-if="!busy" @barcode="code" />
        </div>
        <p v-if="notice" role="status">{{ notice }}</p>
        <template v-if="resolved">
          <details>
            <summary>{{ t('productExternal') }}</summary>
            <p>{{ t('productExternalHelp') }}</p>
            <a
              href="https://opendatacommons.org/licenses/odbl/1-0/"
              target="_blank"
              rel="noreferrer"
              >Open Food Facts · ODbL 1.0</a
            ><NcCheckboxRadioSwitch v-model="consent">{{
              t('productConsent')
            }}</NcCheckboxRadioSwitch
            ><NcButton :disabled="busy || !consent" @click="lookup">{{
              t('productLookup')
            }}</NcButton>
          </details>
          <ProductEvidence v-if="evidence" :evidence="evidence" />
          <p v-else class="muted">{{ t('productHouseholdSource') }}</p>
          <NcTextField v-model="name" :label="t('productName')" required maxlength="200" />
          <NcTextField v-model="brand" :label="t('productBrand')" maxlength="200" />
          <NcTextField v-model="labelQuantity" :label="t('productLabelQuantity')" maxlength="200" />
          <h3>{{ t('productPackageQuantity') }}</h3>
          <p class="muted">{{ t('productVerifyPackage') }}</p>
          <ShoppingQuantityInput v-model="quantity" :disabled="busy" />
          <details>
            <summary>{{ t('shoppingLinkIngredient') }}</summary>
            <p>{{ t('productMatchHelp') }}</p>
            <IngredientSelect
              v-model="ingredient"
              :ingredients="catalogue.ingredients"
              :allow-create="false"
            /><NcCheckboxRadioSwitch v-if="ingredient" v-model="confirmed">{{
              t('productMatchConfirm')
            }}</NcCheckboxRadioSwitch>
          </details>
          <NcTextField
            v-model="packages"
            type="number"
            :label="t('productPackageCount')"
            min="1"
            max="1000000"
            step="1"
            required
          />
        </template>
        <div class="actions">
          <NcButton type="submit" variant="primary" :disabled="busy || !valid">{{
            t('productSaveAdd')
          }}</NcButton
          ><NcButton v-if="resolved" :disabled="busy || !valid" @click="save(false)">{{
            t('productSaveOnly')
          }}</NcButton
          ><NcButton :disabled="busy" @click="close">{{ t('cancel') }}</NcButton>
        </div>
      </fieldset>
    </form>
  </NcDialog>
</template>
