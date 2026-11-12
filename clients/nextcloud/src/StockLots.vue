<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import LabeledField from './LabeledField.vue'
import Icon from './Icon.vue'
import { mdiPencilOutline, mdiScale, mdiArchiveOutline } from '@mdi/js'
import { t, compactExact, shoppingUnit, calendarDate } from './i18n'
import { stockName, stockToday } from './inventory-format'
import { foldSearch } from './ingredient-search'
import type { InventoryCommand, StockLot } from './inventory-api'
const props = defineProps<{ lots: StockLot[]; writable: boolean; busy: boolean }>()
const emit = defineEmits<{ action: [command: InventoryCommand] }>()
const query = ref(''),
  archived = ref(false),
  page = ref(0)
const matching = computed(() =>
  props.lots.filter(
    (l) =>
      (archived.value || !l.archived) &&
      foldSearch(
        [
          l.binding.name,
          ...Object.values(l.binding.translations),
          l.metadata.location,
          l.metadata.batch,
          l.binding.product?.barcode ?? '',
        ].join(' '),
      ).includes(foldSearch(query.value)),
  ),
)
const visible = computed(() => matching.value.slice(page.value * 50, (page.value + 1) * 50))
watch([query, archived], () => {
  page.value = 0
})
</script>
<template>
  <div class="toolbar">
    <LabeledField v-model="query" :label="t('stockSearch')" type="search" /><NcCheckboxRadioSwitch
      v-model="archived"
      >{{ t('stockShowArchived') }}</NcCheckboxRadioSwitch
    >
  </div>
  <NcEmptyContent
    v-if="!matching.length"
    :name="lots.length ? t('stockNoMatches') : t('stockEmpty')"
    :description="lots.length ? undefined : t('stockEmptyHelp')"
  />
  <ul class="stock-lots">
    <li v-for="lot in visible" :key="lot.id">
      <div class="stock-lot-name">
        <strong>{{ stockName(lot.binding) }}</strong
        ><span
          >{{ lot.metadata.location
          }}<template v-if="lot.metadata.batch"> · {{ lot.metadata.batch }}</template></span
        ><span
          :class="{
            'stock-date-passed': lot.metadata.expiry && lot.metadata.expiry < stockToday(),
          }"
          >{{ lot.metadata.expiry ? calendarDate(lot.metadata.expiry) : t('stockDateUnknown')
          }}<template v-if="lot.metadata.expiry && lot.metadata.expiry < stockToday()">
            · {{ t('stockDatePassed') }}</template
          ><template v-if="lot.archived"> · {{ t('stockArchived') }}</template></span
        ><span v-if="!lot.binding.ingredient_id && !lot.binding.product" class="muted">{{
          t('stockUnlinked')
        }}</span>
      </div>
      <dl class="stock-lot-amounts">
        <div>
          <dt>{{ t('stockRemaining') }}</dt>
          <dd>{{ compactExact(lot.remaining) }} {{ shoppingUnit(lot.unit, lot.remaining) }}</dd>
        </div>
        <div>
          <dt>{{ t('stockReserved') }}</dt>
          <dd>{{ compactExact(lot.reserved) }} {{ shoppingUnit(lot.unit, lot.reserved) }}</dd>
        </div>
        <div>
          <dt>{{ t('stockAvailable') }}</dt>
          <dd>{{ compactExact(lot.available) }} {{ shoppingUnit(lot.unit, lot.available) }}</dd>
        </div>
      </dl>
      <div v-if="writable && !lot.archived" class="actions">
        <NcButton
          variant="tertiary"
          :disabled="busy"
          :aria-label="`${t('stockMetadata')}: ${stockName(lot.binding)}`"
          @click="
            emit('action', {
              operation: 'update_lot',
              lot_id: lot.id,
              metadata: { ...lot.metadata },
            })
          "
          ><template #icon><Icon :path="mdiPencilOutline" /></template></NcButton
        ><NcButton
          variant="tertiary"
          :disabled="busy"
          :aria-label="`${t('stockCount')}: ${stockName(lot.binding)}`"
          @click="
            emit('action', {
              operation: 'adjust_lot',
              lot_id: lot.id,
              quantity: { amount: lot.remaining, unit: lot.unit },
              reason: '',
            })
          "
          ><template #icon><Icon :path="mdiScale" /></template></NcButton
        ><NcButton
          v-if="/^0(?:\.0+)?$/.test(lot.remaining)"
          variant="tertiary"
          :disabled="busy"
          :aria-label="`${t('stockArchive')}: ${stockName(lot.binding)}`"
          @click="emit('action', { operation: 'archive_lot', lot_id: lot.id })"
          ><template #icon><Icon :path="mdiArchiveOutline" /></template
        ></NcButton>
      </div>
    </li>
  </ul>
  <div v-if="matching.length > 50" class="actions">
    <NcButton :disabled="page === 0" @click="page--">{{ t('stockPagesPrevious') }}</NcButton
    ><NcButton :disabled="(page + 1) * 50 >= matching.length" @click="page++">{{
      t('stockPagesNext')
    }}</NcButton>
  </div>
</template>
