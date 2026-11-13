<script setup lang="ts">
import NcButton from '@nextcloud/vue/components/NcButton'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import { mdiBeerOutline } from '@mdi/js'
import Icon from './Icon.vue'
import type { BrewState, BrewTemplates, BrewCommand } from './brewing-api'
import { localized, equipmentKinds } from './brewing-format'
import { t, compactExact } from './i18n'
const props = defineProps<{
  data: BrewState
  section: 'recipes' | 'equipment' | 'batches'
  base: string
  writable: boolean
  disabled: boolean
  catalogue?: BrewTemplates
}>()
const showArchived = defineModel<boolean>('showArchived', { required: true })
const emit = defineEmits<{ review: [command: BrewCommand] }>()
function methodName(id: string) {
  const item = props.catalogue?.templates.find((tmp) => tmp.id === id)
  return item ? localized(item) : t('brewMethod')
}
</script>
<template>
  <NcCheckboxRadioSwitch v-model="showArchived">{{ t('brewShowArchived') }}</NcCheckboxRadioSwitch>
  <ul v-if="section === 'recipes'" class="plain-list">
    <li v-for="r in data.recipes.filter((r) => showArchived || !r.archived)" :key="r.record.id">
      <div class="section-heading">
        <RouterLink :to="`${base}/recipes/${r.record.id}`"
          ><strong>{{ r.record.recipe.title }}</strong></RouterLink
        ><NcButton
          v-if="writable"
          variant="tertiary"
          :disabled="disabled"
          @click="
            emit('review', { operation: 'archive_recipe', id: r.record.id, archived: !r.archived })
          "
          >{{ t(r.archived ? 'brewRestore' : 'brewArchive') }}</NcButton
        >
      </div>
      <p class="muted">
        {{ compactExact(r.record.recipe.volume.amount) }} {{ r.record.recipe.volume.unit }} ·
        {{ methodName(r.record.recipe.template.id) }} · {{ t('revision') }}
        {{ r.record.revision }}
      </p>
    </li>
  </ul>
  <ul v-else-if="section === 'equipment'" class="plain-list">
    <li v-for="e in data.equipment.filter((e) => showArchived || !e.archived)" :key="e.id">
      <div class="section-heading">
        <strong>{{ e.details.name }}</strong>
        <div class="actions">
          <NcButton
            v-if="writable"
            variant="tertiary"
            :to="`${base}/equipment/${e.id}?edit=1`"
            :disabled="disabled"
            >{{ t('edit') }}</NcButton
          ><NcButton
            v-if="writable"
            variant="tertiary"
            :disabled="disabled"
            @click="
              emit('review', { operation: 'archive_equipment', id: e.id, archived: !e.archived })
            "
            >{{ t(e.archived ? 'brewRestore' : 'brewArchive') }}</NcButton
          >
        </div>
      </div>
      <p>{{ t(equipmentKinds[e.details.kind]) }} · {{ e.details.model }}</p>
      <p class="muted">
        {{ t('brewCapacity') }}:
        {{
          e.details.capacity
            ? `${compactExact(e.details.capacity.amount)} ${e.details.capacity.unit}`
            : t('unknown')
        }}
      </p>
    </li>
  </ul>
  <ul v-else class="plain-list">
    <li
      v-for="b in data.batches
        .filter((b) => showArchived || !b.batch.archived)
        .sort((a, b) => b.run.created_at - a.run.created_at)"
      :key="b.run.id"
    >
      <div class="section-heading">
        <RouterLink :to="`${base}/batches/${b.run.id}`"
          ><strong>{{ b.run.name }}</strong></RouterLink
        ><span>{{
          t(
            b.run.status === 'active'
              ? 'cookRunActive'
              : b.run.status === 'completed'
                ? 'cookCompleted'
                : 'cookCancelled',
          )
        }}</span>
      </div>
      <p>
        {{ b.run.brew_recipe.title }} · {{ compactExact(b.run.brew_recipe.volume.amount) }}
        {{ b.run.brew_recipe.volume.unit }}
      </p>
      <p class="muted">
        {{ b.batch.draft.brew_day.local }} · {{ b.run.timezone }} · {{ b.reading_count }}
        {{ t('brewMeasurements') }}
      </p>
      <p v-if="b.run.steps.some((s) => s.timer.state === 'running')">
        {{ t('cookRunningTimers') }}:
        {{
          b.run.steps
            .filter((s) => s.timer.state === 'running')
            .map((s) => s.stage.name)
            .join(', ')
        }}
      </p>
    </li>
  </ul>
  <NcEmptyContent
    class="brew-overview-empty"
    v-if="
      section === 'recipes'
        ? !data.recipes.some((r) => showArchived || !r.archived)
        : section === 'equipment'
          ? !data.equipment.some((e) => showArchived || !e.archived)
          : !data.batches.some((b) => showArchived || !b.batch.archived)
    "
    :name="
      t(
        section === 'recipes'
          ? 'brewEmptyRecipes'
          : section === 'equipment'
            ? 'brewEmptyEquipment'
            : 'brewEmptyBatches',
      )
    "
    ><template #icon><Icon :path="mdiBeerOutline" /></template
    ><template #description
      ><p>
        {{
          t(
            section === 'equipment' ||
              (section === 'batches' && data.recipes.some((r) => !r.archived))
              ? 'brewEquipmentHelp'
              : section === 'recipes'
                ? 'brewTemplateHelp'
                : 'brewFirstRecipe',
          )
        }}
      </p>
      <NcButton
        v-if="writable"
        :to="`${base}/${section === 'equipment' ? 'equipment' : section === 'batches' && data.recipes.some((r) => !r.archived) ? 'batches' : 'recipes'}/new`"
        >{{
          t(
            section === 'equipment'
              ? 'brewNewEquipment'
              : section === 'batches' && data.recipes.some((r) => !r.archived)
                ? 'brewNewBatch'
                : 'brewNewRecipe',
          )
        }}</NcButton
      ></template
    ></NcEmptyContent
  >
</template>
