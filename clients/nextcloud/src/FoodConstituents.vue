<script setup lang="ts">
import { computed, ref } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcTextField from './LabeledField.vue'
import Icon from './Icon.vue'
import { mdiPlus, mdiTrashCanOutline } from '@mdi/js'
import type { Catalogue, Constituent } from './contracts'
import { locale, t, exact, parseNonnegative, nutrientLabel } from './i18n'
const model = defineModel<Record<string, Constituent>>({ required: true })
const props = defineProps<{ catalogue: Catalogue }>()
const choice = ref('beta_glucan'),
  name = ref(''),
  unit = ref<Constituent['unit']>('g')
const definitions = computed(() => {
  const all = new Map<string, Constituent>([
    [
      'beta_glucan',
      { name: 'Beta-glucan', translations: { de: 'Beta-Glucan' }, unit: 'g', amount: null },
    ],
    [
      'soluble_fibre',
      {
        name: 'Soluble fibre',
        translations: { de: 'Lösliche Ballaststoffe' },
        unit: 'g',
        amount: null,
      },
    ],
  ])
  for (const item of props.catalogue.ingredients)
    for (const [id, value] of Object.entries(item.constituents)) all.set(id, value)
  return all
})
function add() {
  if (choice.value === 'custom') {
    if (!name.value.trim()) return
    const id = `constituent-${crypto.randomUUID()}`
    model.value[id] = {
      name: locale.value === 'en' ? name.value.trim() : 'Household constituent',
      translations: locale.value === 'de' ? { de: name.value.trim() } : {},
      unit: unit.value,
      amount: null,
    }
    name.value = ''
  } else {
    const definition = definitions.value.get(choice.value)
    if (definition) model.value[choice.value] = { ...definition, amount: null }
  }
}
function amount(id: string, event: Event) {
  if (!(event.target instanceof HTMLInputElement)) return
  const canonical = parseNonnegative(event.target.value)
  event.target.setCustomValidity(
    event.target.value && canonical === null ? t('nutrientQuantityError') : '',
  )
  const item = model.value[id]
  if (item && (!event.target.value || canonical !== null)) item.amount = canonical
}
</script>
<template>
  <div class="constituent-editor">
    <h3>{{ t('foodConstituents') }}</h3>
    <p class="muted">{{ t('constituentsHelp') }}</p>
    <div v-for="(item, id) in model" :key="id" class="constituent-row">
      <label class="field"
        >{{ item.translations[locale] ?? nutrientLabel(String(id), item.name) }} ({{
          item.unit
        }})<input
          :value="item.amount === null ? '' : exact(item.amount)"
          inputmode="decimal"
          maxlength="32"
          @input="amount(String(id), $event)"
      /></label>
      <NcButton
        variant="tertiary"
        :aria-label="`${t('remove')} ${item.translations[locale] ?? item.name}`"
        @click="delete model[id]"
        ><template #icon><Icon :path="mdiTrashCanOutline" /></template
      ></NcButton>
    </div>
    <div class="inline-row">
      <label class="field"
        >{{ t('chooseConstituent')
        }}<select v-model="choice">
          <option
            v-for="(item, id) in Object.fromEntries(definitions)"
            :key="id"
            :value="id"
            :disabled="Boolean(model[id])"
          >
            {{ item.translations[locale] ?? item.name }}
          </option>
          <option value="custom">{{ t('otherConstituent') }}</option>
        </select></label
      >
      <NcTextField
        v-if="choice === 'custom'"
        v-model="name"
        :label="t('constituentName')"
        maxlength="200"
      />
      <label v-if="choice === 'custom'" class="field"
        >{{ t('unit')
        }}<select v-model="unit">
          <option v-for="u in ['g', 'mg', 'µg']" :key="u" :value="u">{{ u }}</option>
        </select></label
      >
      <NcButton :disabled="choice === 'custom' ? !name.trim() : Boolean(model[choice])" @click="add"
        ><template #icon><Icon :path="mdiPlus" /></template>{{ t('addConstituent') }}</NcButton
      >
    </div>
  </div>
</template>
