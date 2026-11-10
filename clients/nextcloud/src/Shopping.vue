<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcTextField from './LabeledField.vue'
import IngredientSelect from './IngredientSelect.vue'
import ShoppingQuantityInput from './ShoppingQuantityInput.vue'
import ShoppingProduct from './ShoppingProduct.vue'
import ShoppingShops from './ShoppingShops.vue'
import ShoppingSources from './ShoppingSources.vue'
import { plansSchema } from './planning-api'
import { recipesSchema } from './contracts'
import type { z } from 'zod'
import { api, domain } from './api'
import { homes, loadCatalogue, useTask } from './state'
import { t } from './i18n'
import ShoppingItems from './ShoppingItems.vue'
import { useUnsavedChanges } from './useUnsavedChanges'
import type { Catalogue } from './contracts'
import { shoppingSchema, productLookupSchema } from './shopping-api'
import type { Shopping, ShoppingCommand, ShoppingQuantity } from './shopping-api'
const route = useRoute(),
  home = computed(() => String(route.params.home))
const writable = computed(() => homes.value.some((h) => h.id === home.value && h.role !== 'reader'))
const data = ref<Shopping>(),
  catalogue = ref<Catalogue>()
const recipes = ref<z.infer<typeof recipesSchema>>(),
  plans = ref<z.infer<typeof plansSchema>>()
const recipeDirty = ref(false),
  manualId = ref<string>(),
  manualForm = ref<HTMLFormElement>()
const productDirty = ref(false)
const shoppingMode = ref(false),
  selectedShop = ref('all'),
  shopDirty = ref(false)
const filtered = computed(
  () =>
    data.value?.items.filter(
      (item) =>
        selectedShop.value === 'all' ||
        (selectedShop.value === 'none'
          ? item.shop_id === null
          : item.shop_id === selectedShop.value),
    ) ?? [],
)
const activeSession = computed(() => data.value?.shops.sessions.find((s) => !s.completed))
const baseline = ref(
  JSON.stringify({ label: '', ingredient: '', quantity: { amount: '1', unit: 'piece' } }),
)
const { busy, error, run, errorMessage } = useTask()
const label = ref(''),
  ingredient = ref(''),
  quantity = ref<ShoppingQuantity>({ amount: '1', unit: 'piece' }),
  message = ref('')
const open = computed(
  () => filtered.value.filter((item) => !/^0(?:\.0+)?$/.test(item.remaining)) ?? [],
)
const purchased = computed(
  () => filtered.value.filter((item) => /^0(?:\.0+)?$/.test(item.remaining)) ?? [],
)
watch(
  () => data.value?.shops.entries,
  (shops) => {
    if (
      selectedShop.value !== 'all' &&
      selectedShop.value !== 'none' &&
      !shops?.some((s) => s.id === selectedShop.value)
    )
      selectedShop.value = 'all'
  },
)
async function load() {
  await run(async () => {
    const [list, foods, recipeList, planList] = await Promise.all([
      api(domain(home.value, '/api/household/shopping'), shoppingSchema),
      loadCatalogue(home.value),
      api(domain(home.value, '/api/recipes'), recipesSchema),
      api(domain(home.value, '/api/household/mealplan'), plansSchema),
    ])
    data.value = list
    catalogue.value = foods
    recipes.value = recipeList
    plans.value = planList
  })
}
async function lookupProduct(barcode: string) {
  return run(async () => {
    let result
    try {
      result = await api(
        domain(home.value, '/api/household/shopping/lookup'),
        productLookupSchema,
        {
          method: 'POST',
          data: { expected_revision: data.value?.revision, barcode, consent: true },
        },
      )
    } catch (error) {
      try {
        data.value = await api(domain(home.value, '/api/household/shopping'), shoppingSchema)
      } catch {
        /* Keep the recoverable original error. */
      }
      throw error
    }
    data.value = result.shopping
    return result.candidate
  })
}
async function command(command: ShoppingCommand) {
  if (!data.value || busy.value) return false
  message.value = ''
  const result = await run(async () => {
    data.value = await api(domain(home.value, '/api/household/shopping'), shoppingSchema, {
      method: 'POST',
      data: { expected_revision: data.value?.revision, command },
    })
    message.value = t('changesSaved')
    return true
  })
  return result === true
}
const manualChanged = computed(
  () =>
    JSON.stringify({
      label: label.value,
      ingredient: ingredient.value,
      quantity: quantity.value,
    }) !== baseline.value,
)
function resetManual() {
  label.value = ''
  ingredient.value = ''
  quantity.value = { amount: '1', unit: 'piece' }
  manualId.value = undefined
  baseline.value = JSON.stringify({
    label: label.value,
    ingredient: ingredient.value,
    quantity: quantity.value,
  })
}
async function add() {
  if (
    await command({
      operation: 'add_manual',
      source_id: manualId.value,
      label: label.value,
      ingredient_id: ingredient.value || null,
      quantity: quantity.value,
    })
  ) {
    resetManual()
    await nextTick()
    manualForm.value?.querySelector('input')?.focus()
  }
}
async function checkItem(key: string, checked: boolean) {
  const focused =
    document.activeElement?.closest('[data-item-key]')?.getAttribute('data-item-key') === key
  const index = open.value.findIndex((item) => item.key === key)
  if ((await command({ operation: 'check', key, checked })) && focused) {
    await nextTick()
    const next = open.value[Math.min(Math.max(index, 0), open.value.length - 1)]
    const target = next
      ? document.querySelector<HTMLInputElement>(`[data-item-key="${CSS.escape(next.key)}"] input`)
      : document.querySelector<HTMLElement>('#mealshop-purchased > summary')
    target?.focus()
  }
}
async function editManual(source: Shopping['sources'][number]) {
  if (source.origin.kind !== 'manual' || (manualChanged.value && !window.confirm(t('unsaved'))))
    return
  manualId.value = source.id
  label.value = source.origin.label
  ingredient.value = source.requirements[0]?.item.ingredient_id ?? ''
  quantity.value = { ...source.origin.quantity }
  baseline.value = JSON.stringify({
    label: label.value,
    ingredient: ingredient.value,
    quantity: quantity.value,
  })
  await nextTick()
  document.getElementById('mealshop-manual-title')?.focus()
}
function cancelManual() {
  if (!manualChanged.value || window.confirm(t('unsaved'))) resetManual()
}
useUnsavedChanges(
  () => manualChanged.value || recipeDirty.value || shopDirty.value || productDirty.value,
)
onMounted(load)
</script>
<template>
  <header class="page-header">
    <div>
      <h1 tabindex="-1">{{ t('shoppingList') }}</h1>
      <p>{{ t('shoppingIntro') }}</p>
    </div>
    <NcButton
      v-if="data?.items.length"
      :variant="shoppingMode ? 'primary' : 'secondary'"
      :aria-pressed="shoppingMode"
      :disabled="busy || manualChanged || recipeDirty || shopDirty || productDirty"
      @click="shoppingMode = !shoppingMode"
      >{{ t(shoppingMode ? 'shoppingManage' : 'shoppingMode') }}</NcButton
    >
  </header>
  <NcNoteCard v-if="error" type="error" role="alert"
    >{{ errorMessage(error)
    }}<NcButton :disabled="busy" @click="load">{{ t('retry') }}</NcButton></NcNoteCard
  >
  <NcLoadingIcon v-if="busy" :aria-label="t('loading')" />
  <p v-if="message" role="status">{{ message }}</p>
  <template v-if="data">
    <NcEmptyContent
      v-if="!data.items.length"
      :name="t('shoppingEmpty')"
      :description="t('shoppingEmptyHelp')"
    />
    <div v-if="data.shops.entries.length" class="shopping-filter">
      <label class="field"
        >{{ t('shopFilter')
        }}<select :aria-label="t('shopFilter')" v-model="selectedShop" :disabled="busy">
          <option value="all">{{ t('shopAll') }}</option>
          <option value="none">{{ t('shopUnassigned') }}</option>
          <option v-for="shop in data.shops.entries" :key="shop.id" :value="shop.id">
            {{ shop.name }}
          </option>
        </select></label
      >
      <div v-if="writable && data.shops.learning" class="actions">
        <template v-if="activeSession"
          ><p role="status">
            {{ t('shopSessionActive') }}:
            {{ data.shops.entries.find((s) => s.id === activeSession?.shop_id)?.name }}
          </p>
          <NcButton
            :disabled="busy"
            @click="command({ operation: 'finish_session', id: activeSession.id, discard: false })"
            >{{ t('shopFinish') }}</NcButton
          ><NcButton
            :disabled="busy"
            @click="command({ operation: 'finish_session', id: activeSession.id, discard: true })"
            >{{ t('shopDiscard') }}</NcButton
          ></template
        >
        <NcButton
          v-else
          :disabled="busy || selectedShop === 'all' || selectedShop === 'none'"
          @click="command({ operation: 'start_session', shop_id: selectedShop })"
          >{{ t('shopStart') }}</NcButton
        >
      </div>
      <NcButton
        v-if="
          writable &&
          selectedShop !== 'all' &&
          selectedShop !== 'none' &&
          data.shops.manual_order[selectedShop]?.length
        "
        :disabled="busy"
        @click="command({ operation: 'order', shop_id: selectedShop, keys: [] })"
        >{{ t('shopClearOrder') }}</NcButton
      >
      <p v-if="!filtered.length">{{ t('shopEmptyList') }}</p>
    </div>
    <section v-if="open.length">
      <h2>{{ t('shoppingToBuy') }}</h2>
      <ShoppingItems
        :items="open"
        :writable="writable"
        :busy="busy"
        :shops="data.shops"
        :selected-shop="selectedShop"
        @command="command"
        @check="checkItem"
      />
    </section>
    <details v-if="purchased.length" id="mealshop-purchased">
      <summary>{{ t('shoppingBought') }} · {{ purchased.length }}</summary>
      <ShoppingItems
        :items="purchased"
        :writable="writable"
        :busy="busy"
        :shops="data.shops"
        :selected-shop="selectedShop"
        @command="command"
        @check="checkItem"
      />
    </details>
    <section v-if="writable && !shoppingMode" class="narrow">
      <h2 id="mealshop-manual-title" tabindex="-1">
        {{ t(manualId ? 'shoppingEditItem' : 'shoppingAddManual') }}
      </h2>
      <form ref="manualForm" @submit.prevent="add">
        <fieldset class="editor-fields" :disabled="busy">
          <NcTextField v-model="label" :label="t('name')" required maxlength="200" />
          <details>
            <summary>{{ t('shoppingLinkIngredient') }}</summary>
            <IngredientSelect
              v-model="ingredient"
              :ingredients="catalogue?.ingredients ?? []"
              :allow-create="false"
            />
          </details>
          <ShoppingQuantityInput v-model="quantity" :disabled="busy" /><NcButton
            type="submit"
            variant="primary"
            :disabled="busy || !label.trim() || !quantity.amount"
            >{{ t(manualId ? 'save' : 'shoppingAddManual') }}</NcButton
          >
          <NcButton v-if="manualId || manualChanged" :disabled="busy" @click="cancelManual">{{
            t('cancel')
          }}</NcButton>
        </fieldset>
      </form>
    </section>
    <ShoppingProduct
      v-if="catalogue && writable && !shoppingMode"
      :data="data"
      :catalogue="catalogue"
      :busy="busy"
      :perform="command"
      :lookup="lookupProduct"
      :error="error"
      @dirty="productDirty = $event"
    />
    <ShoppingShops
      v-if="catalogue && !shoppingMode"
      :data="data"
      :catalogue="catalogue"
      :busy="busy"
      :writable="writable"
      :perform="command"
      @dirty="shopDirty = $event"
    />
    <ShoppingSources
      v-if="recipes && plans && !shoppingMode"
      :data="data"
      :recipes="recipes"
      :plans="plans"
      :busy="busy"
      :writable="writable"
      :perform="command"
      @dirty="recipeDirty = $event"
      @manual="editManual"
    />
  </template>
</template>
