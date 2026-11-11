<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { z } from 'zod'
import NcSelect from '@nextcloud/vue/components/NcSelect'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import LabeledField from './LabeledField.vue'
import Icon from './Icon.vue'
import { mdiPlus, mdiArrowUp, mdiArrowDown, mdiTrashCanOutline } from '@mdi/js'
import type { recipesSchema } from './contracts'
import type { CollectionDraft } from './library-api'
import { foldSearch } from './ingredient-search'
import { t } from './i18n'
const model = defineModel<CollectionDraft>({ required: true })
const props = defineProps<{ recipes: z.infer<typeof recipesSchema>['recipes']; busy: boolean }>()
const emit = defineEmits<{ changed: [] }>()
const query = ref(''),
  selectedId = ref(''),
  picker = ref<HTMLElement>()
const selected = computed(() => props.recipes.find((r) => r.id === selectedId.value) ?? null)
const options = computed(() =>
  props.recipes.filter(
    (r) =>
      !model.value.entries.some((entry) => entry.recipe_id === r.id) &&
      foldSearch(r.title).includes(foldSearch(query.value)),
  ),
)
function choose(value: unknown) {
  const result = z.object({ id: z.string() }).nullable().safeParse(value)
  if (result.success) selectedId.value = result.data?.id ?? ''
}
async function add() {
  if (!selected.value) return
  model.value = {
    ...model.value,
    entries: [
      ...model.value.entries,
      { recipe_id: selected.value.id, revision: selected.value.revision },
    ],
  }
  selectedId.value = ''
  query.value = ''
  emit('changed')
  await nextTick()
  picker.value?.querySelector('input')?.focus()
}
function move(index: number, direction: number) {
  const entries = [...model.value.entries],
    current = entries[index],
    other = entries[index + direction]
  if (!current || !other) return
  entries[index] = other
  entries[index + direction] = current
  model.value = { ...model.value, entries }
  emit('changed')
}
function remove(index: number) {
  model.value = { ...model.value, entries: model.value.entries.filter((_, at) => at !== index) }
  emit('changed')
}
function name(id: string) {
  return props.recipes.find((r) => r.id === id)?.title ?? id
}
function latest(id: string) {
  return props.recipes.find((r) => r.id === id)?.revision
}
</script>
<template>
  <fieldset class="editor-fields" :disabled="props.busy">
    <LabeledField
      v-model="model.name"
      :label="t('collectionName')"
      maxlength="200"
      required
      @update:model-value="emit('changed')"
    />
    <label class="field"
      >{{ t('collectionDescription')
      }}<textarea v-model="model.description" maxlength="2000" rows="2" @input="emit('changed')" />
    </label>
    <section class="editor-section">
      <h2>{{ t('recipes') }}</h2>
      <p class="muted">{{ t('collectionRevisionHelp') }}</p>
      <div v-if="model.entries.length" class="collection-entry-head" aria-hidden="true">
        <span>{{ t('collectionRecipe') }}</span
        ><span>{{ t('revision') }}</span>
      </div>
      <ol class="collection-editor-entries">
        <li v-for="(entry, index) in model.entries" :key="entry.recipe_id">
          <strong>{{ name(entry.recipe_id) }}</strong>
          <div class="collection-revision">
            <span class="collection-revision-caption" aria-hidden="true">{{ t('revision') }}</span
            ><LabeledField
              v-model="entry.revision"
              type="number"
              :label="`${t('revision')} · ${name(entry.recipe_id)}`"
              :hide-label="true"
              min="1"
              :max="latest(entry.recipe_id)"
              step="1"
              required
              @update:model-value="emit('changed')"
            />
          </div>
          <div class="collection-entry-actions">
            <NcButton
              type="button"
              variant="tertiary"
              :aria-label="`${t('moveRecipeUp')} · ${name(entry.recipe_id)}`"
              :disabled="props.busy || index === 0"
              @click="move(index, -1)"
              ><template #icon><Icon :path="mdiArrowUp" /></template
            ></NcButton>
            <NcButton
              type="button"
              variant="tertiary"
              :aria-label="`${t('moveRecipeDown')} · ${name(entry.recipe_id)}`"
              :disabled="props.busy || index === model.entries.length - 1"
              @click="move(index, 1)"
              ><template #icon><Icon :path="mdiArrowDown" /></template
            ></NcButton>
            <NcButton
              type="button"
              variant="tertiary"
              :aria-label="`${t('removeCollectionRecipe')} · ${name(entry.recipe_id)}`"
              :disabled="props.busy"
              @click="remove(index)"
              ><template #icon><Icon :path="mdiTrashCanOutline" /></template
            ></NcButton>
          </div>
        </li>
      </ol>
      <p v-if="!model.entries.length" class="muted">{{ t('emptyCollection') }}</p>
      <div ref="picker" class="collection-add-recipe">
        <div class="labeled-field">
          <NcSelect
            :model-value="selected"
            :options="options"
            :filterable="false"
            label="title"
            :input-label="t('chooseCollectionRecipe')"
            label-outside
            :placeholder="t('chooseCollectionRecipe')"
            :disabled="props.busy || model.entries.length >= 200"
            @search="query = $event"
            @update:model-value="choose"
            ><template #option="recipe"
              ><span>{{ recipe.title }}</span></template
            ><template #no-options>{{ t('noResults') }}</template></NcSelect
          >
        </div>
        <NcButton type="button" :disabled="props.busy || !selected" @click="add"
          ><template #icon><Icon :path="mdiPlus" /></template
          >{{ t('addCollectionRecipe') }}</NcButton
        >
      </div>
      <p class="muted">{{ t('collectionRecipeHelp') }}</p>
      <NcNoteCard v-if="!props.recipes.length" type="info">{{ t('noRecipesText') }}</NcNoteCard>
    </section>
  </fieldset>
</template>
