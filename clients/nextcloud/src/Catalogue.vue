<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import NcTextField from './LabeledField.vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { z } from 'zod'
import { catalogueSchema, recipesSchema } from './contracts'
import type { Ingredient } from './contracts'
import { homes, useTask, loadCatalogue } from './state'
import { exact, locale, t, allergenName, nutrientLabel } from './i18n'
import { api, domain } from './api'
import { foldSearch } from './ingredient-search'
import IngredientEditor from './IngredientEditor.vue'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import { mdiPlus, mdiStar, mdiStarOutline, mdiChevronRight, mdiChevronDown } from '@mdi/js'
import Icon from './Icon.vue'
const route = useRoute(),
  router = useRouter()
const data = ref<z.infer<typeof catalogueSchema>>()
const home = computed(() => String(route.params.home))
const writable = computed(() =>
  homes.value.some((item) => item.id === home.value && item.role !== 'reader'),
)
const favoritesOnly = ref(false),
  showArchived = ref(false)
function resetFilters() {
  query.value = ''
  favoritesOnly.value = false
  showArchived.value = false
}
const editor = ref<'create' | 'metadata' | null>(null)
const related = ref<z.infer<typeof recipesSchema>>()
const relatedError = ref<unknown>(null)
let relatedGeneration = 0
watch(
  () => route.params.id,
  () => {
    void loadRelated()
  },
)
async function loadRelated() {
  const generation = ++relatedGeneration
  related.value = undefined
  relatedError.value = null
  if (!route.params.id) return
  try {
    const result = await api(
      domain(
        home.value,
        `/api/household/ingredients/${encodeURIComponent(String(route.params.id))}/recipes`,
      ),
      recipesSchema,
    )
    if (generation === relatedGeneration) related.value = result
  } catch (error) {
    if (generation === relatedGeneration) relatedError.value = error
  }
}
async function saved(id: string) {
  data.value = await loadCatalogue(home.value)
  editor.value = null
  await router.push(`/h/${home.value}/ingredients/${id}`)
  await loadRelated()
}
async function annotate(item: Ingredient, changes: { favorite?: boolean; archived?: boolean }) {
  if (!data.value) return
  await run(async () => {
    await api(
      domain(home.value, `/api/household/ingredients/${item.id}`),
      z.object({ revision: z.number() }),
      {
        method: 'PUT',
        data: {
          expected_revision: data.value?.revision,
          annotations: {
            organization: item.organization,
            tags: item.tags,
            components: item.components,
            declared_allergens: item.declared_allergens,
            archived: item.archived,
            favorite: item.favorite,
            ...changes,
          },
        },
      },
    )
    data.value = await loadCatalogue(home.value)
  })
}
function archive(item: Ingredient) {
  if (item.archived || window.confirm(t('archiveHelp')))
    void annotate(item, { archived: !item.archived })
}
const { busy, error, run, errorMessage } = useTask()
const query = computed({
  get: () => (typeof route.query.q === 'string' ? route.query.q : ''),
  set: (q: string) => {
    void router.replace({ query: { ...route.query, q: q || undefined } })
  },
})
const name = (item: Ingredient) => item.translations[locale.value]?.name ?? item.name
const form = (item: Ingredient) => item.translations[locale.value]?.form ?? item.form
const selected = computed(() => data.value?.ingredients.find((i) => i.id === route.params.id))
const expandedGroups = ref<string[]>([])
const filtered = computed(() => {
  const terms = query.value.trim().toLocaleLowerCase().split(/\s+/)
  return (
    data.value?.ingredients
      .filter((i) => (showArchived.value || !i.archived) && (!favoritesOnly.value || i.favorite))
      .filter(
        (i) =>
          !i.organization.group_id ||
          (!showArchived.value &&
            data.value?.ingredients.some((p) => p.id === i.organization.group_id && p.archived)) ||
          favoritesOnly.value ||
          Boolean(query.value.trim()) ||
          expandedGroups.value.includes(i.organization.group_id),
      )
      .filter((i) => {
        const text = [
          i.name,
          ...i.aliases,
          ...(data.value?.ingredients.find((p) => p.id === i.organization.group_id)
            ? [name(data.value.ingredients.find((p) => p.id === i.organization.group_id) ?? i)]
            : []),
          ...i.tags,
          ...Object.values(i.translations).flatMap((tr) => [tr.name, ...tr.aliases]),
        ]
          .join(' ')
          .toLocaleLowerCase()
        return terms.every((term) => foldSearch(text).includes(foldSearch(term)))
      })
      .sort((a, b) => {
        const parent = (i: Ingredient) =>
          data.value?.ingredients.find((p) => p.id === i.organization.group_id) ?? i
        return (
          name(parent(a)).localeCompare(name(parent(b)), locale.value) ||
          Number(Boolean(a.organization.group_id)) - Number(Boolean(b.organization.group_id)) ||
          name(a).localeCompare(name(b), locale.value)
        )
      }) ?? []
  )
})
const foodCount = computed(() =>
  query.value.trim()
    ? filtered.value.filter((i) => !i.organization.is_group).length
    : (data.value?.ingredients.filter(
        (i) =>
          !i.organization.is_group &&
          (showArchived.value || !i.archived) &&
          (!favoritesOnly.value || i.favorite),
      ).length ?? 0),
)
const groupCount = computed(() => filtered.value.filter((i) => i.organization.is_group).length)
async function load() {
  await run(async () => {
    data.value = await loadCatalogue(String(route.params.home))
    await loadRelated()
  })
}
onMounted(load)
</script>
<template>
  <template v-if="!route.params.id">
    <header class="page-header">
      <div>
        <h1 tabindex="-1">{{ t('ingredients') }}</h1>
        <p>{{ t('catalogueIntro') }}</p>
      </div>
      <NcButton v-if="writable" variant="primary" @click="editor = 'create'"
        ><template #icon><Icon :path="mdiPlus" /></template>{{ t('addIngredient') }}</NcButton
      >
    </header>
    <div class="actions">
      <NcCheckboxRadioSwitch v-model="favoritesOnly">{{
        t('favoritesOnly')
      }}</NcCheckboxRadioSwitch>
      <NcCheckboxRadioSwitch v-model="showArchived">{{ t('showArchived') }}</NcCheckboxRadioSwitch>
    </div>
    <div class="toolbar">
      <NcTextField
        v-model="query"
        type="search"
        :label="t('search')"
        :show-trailing-button="Boolean(query)"
        trailing-button-icon="close"
        :trailing-button-label="t('clearSearch')"
        @trailing-button-click="query = ''"
      />
      <p v-if="data" role="status">
        {{ t('ingredients') }}: {{ foodCount }} · {{ t('ingredientGroups') }}: {{ groupCount }}
      </p>
    </div>
  </template>
  <NcNoteCard v-if="error" type="error" role="alert"
    >{{ errorMessage(error) }}<NcButton @click="load">{{ t('retry') }}</NcButton></NcNoteCard
  >
  <NcLoadingIcon v-if="busy" :aria-label="t('loading')" />
  <template v-if="data && !busy">
    <template v-if="route.params.id && selected">
      <NcButton :to="{ path: `/h/${route.params.home}/ingredients`, query: route.query }">{{
        t('back')
      }}</NcButton>
      <header class="page-header">
        <div>
          <h1 tabindex="-1">{{ name(selected) }}</h1>
          <p>{{ form(selected) }}</p>
          <p v-if="locale !== 'en' && !selected.translations[locale]?.name" class="muted">
            {{ t('original') }}
          </p>
        </div>
      </header>
      <div class="actions" v-if="writable">
        <NcButton
          :aria-pressed="selected.favorite"
          :disabled="busy"
          @click="annotate(selected, { favorite: !selected.favorite })"
          ><template #icon><Icon :path="selected.favorite ? mdiStar : mdiStarOutline" /></template
          >{{ t('favorite') }}</NcButton
        >
        <NcButton :disabled="busy" @click="editor = 'metadata'">{{ t('editIngredient') }}</NcButton>
        <NcButton :disabled="busy" @click="archive(selected)">{{
          selected.archived ? t('restoreIngredient') : t('archiveIngredient')
        }}</NcButton>
      </div>
      <p>
        {{ t('dietClass') }}: {{ t(selected.organization.diet) }} · {{ t('originClass') }}:
        {{ t(selected.organization.origin) }}
      </p>
      <p v-if="selected.organization.intolerance_triggers.length">
        {{ t('intoleranceTriggers') }}: {{ t('sorbitol') }}
      </p>
      <p v-if="selected.nutrition_inherited_from">
        {{ t('inheritedFrom') }}:
        {{
          name(
            data.ingredients.find((i) => i.id === selected?.nutrition_inherited_from) ?? selected,
          )
        }}
      </p>
      <ul v-if="selected.organization.is_group" class="record-list">
        <li
          v-for="child in data.ingredients.filter((i) => i.organization.group_id === selected?.id)"
          :key="child.id"
        >
          <RouterLink :to="`/h/${home}/ingredients/${child.id}`">{{ name(child) }}</RouterLink>
        </li>
      </ul>
      <NcNoteCard v-if="selected.archived" type="warning">{{ t('archived') }}</NcNoteCard>
      <NcNoteCard type="info">{{
        selected.source.kind === 'household_declaration'
          ? t('sourceDeclaration')
          : t('sourceNotice')
      }}</NcNoteCard>
      <section
        v-if="
          selected.tags.length || selected.components.length || selected.declared_allergens.length
        "
      >
        <h2>{{ t('ingredientMetadata') }}</h2>
        <p v-if="selected.tags.length">{{ t('tags') }}: {{ selected.tags.join(' · ') }}</p>
        <p v-if="selected.declared_allergens.length">
          {{ t('declaredAllergens') }}:
          {{ selected.declared_allergens.map(allergenName).join(' · ') }}
        </p>
        <h3 v-if="selected.components.length">{{ t('components') }}</h3>
        <ul v-if="selected.components.length" class="plain-list">
          <li v-for="id in selected.components" :key="id">
            <RouterLink :to="`/h/${home}/ingredients/${id}`">{{
              name(data.ingredients.find((item) => item.id === id) ?? selected)
            }}</RouterLink>
          </li>
        </ul>
      </section>
      <section>
        <h2>{{ t('recipesUsingIngredient') }}</h2>
        <NcNoteCard v-if="relatedError" type="error"
          >{{ errorMessage(relatedError)
          }}<NcButton @click="loadRelated">{{ t('retry') }}</NcButton></NcNoteCard
        >
        <p v-else-if="related && !related.recipes.length">{{ t('noRecipesForIngredient') }}</p>
        <ul v-else-if="related" class="record-list">
          <li v-for="recipe in related.recipes" :key="recipe.id">
            <RouterLink :to="`/h/${home}/recipes/${recipe.id}`">{{ recipe.title }}</RouterLink>
          </li>
        </ul>
        <NcLoadingIcon v-else :aria-label="t('loading')" />
      </section>
      <div class="detail-columns">
        <section>
          <h2>{{ t('nutrients') }}</h2>
          <p>{{ t('basis') }}</p>
          <table>
            <thead>
              <tr>
                <th scope="col">{{ t('nutrient') }}</th>
                <th scope="col" class="numeric">{{ t('amount') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="nutrient in data.nutrient_definitions.filter((n) => n.group !== 'component')"
                :key="nutrient.id"
              >
                <th scope="row">
                  {{ nutrientLabel(nutrient.id, nutrient.label) }}
                  <small>{{ nutrient.unit }}</small>
                </th>
                <td class="numeric">
                  {{ exact(selected.nutrients[nutrient.id]?.amount ?? null) }}
                </td>
              </tr>
            </tbody>
          </table>
          <template v-if="Object.keys(selected.constituents).length"
            ><h3>{{ t('foodConstituents') }}</h3>
            <ul class="plain-list">
              <li v-for="(c, id) in selected.constituents" :key="id">
                {{ c.translations[locale] ?? c.name }}: {{ exact(c.amount) }} {{ c.unit }} / 100 g
              </li>
            </ul></template
          >
        </section>
        <section>
          <h2>{{ t('source') }}</h2>
          <dl>
            <dt>{{ t('source') }}</dt>
            <dd>{{ selected.source.name }}</dd>
            <dt>{{ t('record') }}</dt>
            <dd>{{ selected.source.record_id }}</dd>
            <dt>{{ t('version') }}</dt>
            <dd>{{ selected.source.version }}</dd>
            <dt>{{ t('licence') }}</dt>
            <dd>{{ selected.source.license }}</dd>
            <dt>{{ t('retrieved') }}</dt>
            <dd>{{ selected.source.retrieved_on }}</dd>
          </dl>
          <a
            v-if="selected.source.url"
            :href="selected.source.url"
            target="_blank"
            rel="noopener noreferrer"
            >{{ t('openSource') }}</a
          >
          <details v-if="selected.reference_source">
            <summary>{{ t('referenceValues') }}</summary>
            <p>
              {{ selected.reference_source.name }} · {{ selected.reference_source.record_id }} ·
              {{ selected.reference_source.version }}
            </p>
            <a
              v-if="selected.reference_source.url"
              :href="selected.reference_source.url"
              target="_blank"
              rel="noopener noreferrer"
              >{{ t('openSource') }}</a
            >
          </details>
          <h3>{{ t('uncertainty') }}</h3>
          <p class="muted">{{ t('original') }}</p>
          <p :lang="selected.source.kind === 'primary' ? 'en' : undefined">
            {{ selected.uncertainty }}
          </p>
          <details v-for="note in selected.education" :key="note.component">
            <summary lang="en">{{ note.title }}</summary>
            <p lang="en">{{ note.summary }}</p>
            <p lang="en">{{ note.conditions }}</p>
            <p lang="en">{{ note.limitations }}</p>
            <a :href="note.source_url" target="_blank" rel="noopener noreferrer" lang="en">{{
              note.evidence_reference
            }}</a>
          </details>
        </section>
      </div>
    </template>
    <NcEmptyContent v-else-if="route.params.id" :name="t('noResults')"
      ><template #action
        ><NcButton :to="`/h/${route.params.home}/ingredients`">{{ t('back') }}</NcButton></template
      ></NcEmptyContent
    >
    <template v-else
      ><NcEmptyContent v-if="!filtered.length" :name="t('noResults')"
        ><template #action
          ><NcButton @click="resetFilters">{{ t('resetFilters') }}</NcButton></template
        ></NcEmptyContent
      >
      <ul v-else class="record-list">
        <li
          v-for="item in filtered"
          :key="item.id"
          class="ingredient-list-row"
          :class="{
            subtype: item.organization.group_id,
            'ingredient-group-row': item.organization.is_group,
          }"
        >
          <NcButton
            v-if="item.organization.is_group && !query.trim()"
            variant="tertiary"
            :aria-expanded="expandedGroups.includes(item.id)"
            :aria-label="`${name(item)}: ${t('subtypes')}`"
            @click="
              expandedGroups = expandedGroups.includes(item.id)
                ? expandedGroups.filter((id) => id !== item.id)
                : [...expandedGroups, item.id]
            "
            ><template #icon
              ><Icon
                :path="
                  expandedGroups.includes(item.id) ? mdiChevronDown : mdiChevronRight
                " /></template
          ></NcButton>
          <RouterLink
            :aria-label="item.organization.is_group ? name(item) : undefined"
            :to="{ path: `/h/${route.params.home}/ingredients/${item.id}`, query: route.query }"
            ><div>
              <strong
                >{{ name(item)
                }}<small
                  v-if="item.organization.is_group"
                  class="ingredient-group-count"
                  aria-hidden="true"
                  >{{
                    data.ingredients.filter((i) => i.organization.group_id === item.id).length
                  }}</small
                ></strong
              ><span v-if="!item.organization.is_group">{{ form(item) }}</span>
            </div>
            <span v-if="!item.organization.is_group" class="source-label"
              >{{ item.source.name }} <small>{{ item.source.record_id }}</small></span
            ></RouterLink
          >
          <NcButton
            v-if="writable"
            variant="tertiary"
            :aria-label="`${t('favorite')} ${name(item)}`"
            :aria-pressed="item.favorite"
            :disabled="busy"
            @click="annotate(item, { favorite: !item.favorite })"
            ><template #icon><Icon :path="item.favorite ? mdiStar : mdiStarOutline" /></template
          ></NcButton>
        </li></ul
    ></template>
  </template>
  <IngredientEditor
    v-if="editor && data"
    :home="home"
    :catalogue="data"
    :ingredient="editor === 'metadata' ? selected : undefined"
    @close="editor = null"
    @saved="saved"
  />
</template>
