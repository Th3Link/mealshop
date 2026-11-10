<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import NcTextField from './LabeledField.vue'
import IngredientSelect from './IngredientSelect.vue'
import type { Catalogue } from './contracts'
import type { Shopping, ShoppingCommand } from './shopping-api'
import { locale, t } from './i18n'
const props = defineProps<{
  data: Shopping
  catalogue: Catalogue
  busy: boolean
  writable: boolean
  perform: (command: ShoppingCommand) => Promise<boolean>
}>()
const emit = defineEmits<{ dirty: [value: boolean] }>()
const name = ref(''),
  id = ref<string>(),
  ingredient = ref(''),
  first = ref(''),
  second = ref('')
const baseline = ref(''),
  preferenceDirty = ref(false)
const dirty = computed(() => name.value !== baseline.value || preferenceDirty.value)
watch(dirty, (value) => emit('dirty', value))
function chooseIngredient(value: string) {
  if (preferenceDirty.value && !window.confirm(t('unsaved'))) return
  ingredient.value = value
  const p = props.data.shops.preferences[value]
  first.value = p?.first ?? ''
  second.value = p?.second ?? ''
  preferenceDirty.value = false
}

function edit(shop: Shopping['shops']['entries'][number]) {
  if (dirty.value && !window.confirm(t('unsaved'))) return
  id.value = shop.id
  name.value = shop.name
  baseline.value = shop.name
}
function cancel() {
  if (!dirty.value || window.confirm(t('unsaved'))) {
    name.value = ''
    id.value = undefined
    baseline.value = ''
    preferenceDirty.value = false
    const preference = props.data.shops.preferences[ingredient.value]
    first.value = preference?.first ?? ''
    second.value = preference?.second ?? ''
  }
}
async function save() {
  if (await props.perform({ operation: 'save_shop', id: id.value, name: name.value })) {
    name.value = ''
    id.value = undefined
    baseline.value = ''
  }
}
async function preference() {
  if (
    await props.perform({
      operation: 'preference',
      ingredient_id: ingredient.value,
      first: first.value || null,
      second: second.value || null,
    })
  )
    preferenceDirty.value = false
}
async function reset() {
  if (window.confirm(t('shopResetHelp'))) await props.perform({ operation: 'reset_history' })
}
async function remove(shop: Shopping['shops']['entries'][number]) {
  if (window.confirm(t('shopRemoveHelp')))
    await props.perform({ operation: 'remove_shop', id: shop.id })
}
const active = computed(() => props.data.shops.sessions.find((s) => !s.completed))
function itemName(key: string) {
  const item = props.data.items.find((i) => i.key === key)
  return item?.translations[locale.value] ?? item?.name ?? key
}
</script>
<template>
  <details class="shopping-settings">
    <summary>{{ t('shopSettings') }}</summary>
    <h2>{{ t('shops') }}</h2>
    <p v-if="!data.shops.entries.length">{{ t('shopsEmpty') }}</p>
    <ul class="plain-list">
      <li v-for="shop in data.shops.entries" :key="shop.id">
        <strong>{{ shop.name }}</strong>
        <div v-if="writable" class="actions">
          <NcButton :disabled="busy" @click="edit(shop)">{{ t('shopEdit') }}</NcButton
          ><NcButton :disabled="busy || active?.shop_id === shop.id" @click="remove(shop)">{{
            t('remove')
          }}</NcButton>
        </div>
      </li>
    </ul>
    <form v-if="writable" class="narrow" @submit.prevent="save">
      <fieldset class="editor-fields" :disabled="busy">
        <NcTextField v-model="name" :label="t('shopName')" required maxlength="120" />
        <div class="actions">
          <NcButton type="submit" :disabled="busy || !name.trim()">{{
            t(id ? 'save' : 'shopAdd')
          }}</NcButton
          ><NcButton v-if="id || dirty" :disabled="busy" @click="cancel">{{
            t('cancel')
          }}</NcButton>
        </div>
      </fieldset>
    </form>
    <form v-if="writable && data.shops.entries.length" class="narrow" @submit.prevent="preference">
      <h3>{{ t('shopPreferences') }}</h3>
      <fieldset class="editor-fields" :disabled="busy">
        <IngredientSelect
          :model-value="ingredient"
          @update:model-value="chooseIngredient"
          :ingredients="catalogue.ingredients"
          :allow-create="false"
        />
        <div class="form-row">
          <label class="field"
            >{{ t('shopFirst')
            }}<select :aria-label="t('shopFirst')" v-model="first" @change="preferenceDirty = true">
              <option value="">{{ t('shopUnassigned') }}</option>
              <option v-for="shop in data.shops.entries" :key="shop.id" :value="shop.id">
                {{ shop.name }}
              </option>
            </select></label
          ><label class="field"
            >{{ t('shopSecond')
            }}<select
              :aria-label="t('shopSecond')"
              v-model="second"
              @change="preferenceDirty = true"
            >
              <option value="">{{ t('shopUnassigned') }}</option>
              <option
                v-for="shop in data.shops.entries.filter((s) => s.id !== first)"
                :key="shop.id"
                :value="shop.id"
              >
                {{ shop.name }}
              </option>
            </select></label
          >
        </div>
        <NcButton
          type="submit"
          :disabled="busy || !ingredient || !preferenceDirty || Boolean(first && first === second)"
          >{{ t('save') }}</NcButton
        >
      </fieldset>
    </form>
    <h2>{{ t('shopLearning') }}</h2>
    <p>{{ t('shopLearningHelp') }}</p>
    <NcCheckboxRadioSwitch
      :model-value="data.shops.learning"
      :disabled="busy || !writable"
      @update:model-value="perform({ operation: 'learning', enabled: !data.shops.learning })"
      >{{ t('shopLearningConsent') }}</NcCheckboxRadioSwitch
    >
    <p v-if="active" role="status">
      {{ t('shopSessionActive') }}:
      {{ data.shops.entries.find((s) => s.id === active?.shop_id)?.name }}
    </p>
    <details>
      <summary>{{ t('shopHistory') }} · {{ data.shops.sessions.length }}</summary>
      <p v-if="!data.shops.sessions.length">{{ t('shopHistoryEmpty') }}</p>
      <ol>
        <li v-for="session in data.shops.sessions" :key="session.id">
          {{ data.shops.entries.find((s) => s.id === session.shop_id)?.name }} ·
          {{ t(session.completed ? 'shopSessionCompleted' : 'shopSessionActive') }}
          <ol>
            <li v-for="key in session.events" :key="key">
              {{ itemName(key) }}
              <span v-if="session.corrected.length">· {{ t('shopCorrected') }}</span>
            </li>
          </ol>
        </li>
      </ol>
    </details>
    <NcButton v-if="writable && data.shops.sessions.length" :disabled="busy" @click="reset">{{
      t('shopReset')
    }}</NcButton>
  </details>
</template>
