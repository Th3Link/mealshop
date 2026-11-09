<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import NcTextField from '@nextcloud/vue/components/NcTextField'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { z } from 'zod'
import { catalogueSchema } from './contracts'
import type { Ingredient } from './contracts'
import { useTask, loadCatalogue } from './state'
import { exact, locale, t } from './i18n'
const route = useRoute(),
  router = useRouter()
const data = ref<z.infer<typeof catalogueSchema>>()
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
const filtered = computed(() => {
  const terms = query.value.trim().toLocaleLowerCase().split(/\s+/)
  return (
    data.value?.ingredients
      .filter((i) => {
        const text = [
          i.name,
          ...i.aliases,
          ...Object.values(i.translations).flatMap((tr) => [tr.name, ...tr.aliases]),
        ]
          .join(' ')
          .toLocaleLowerCase()
        return terms.every((term) => text.includes(term))
      })
      .sort((a, b) => name(a).localeCompare(name(b), locale.value)) ?? []
  )
})
async function load() {
  await run(async () => {
    data.value = await loadCatalogue()
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
    </header>
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
      <p v-if="data" role="status">{{ filtered.length }} {{ t('results') }}</p>
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
      <NcNoteCard type="info">{{ t('sourceNotice') }}</NcNoteCard>
      <div class="detail-columns">
        <section>
          <h2>{{ t('nutrients') }}</h2>
          <p>{{ t('basis') }} · {{ t('original') }}</p>
          <table>
            <thead>
              <tr>
                <th scope="col">{{ t('nutrient') }}</th>
                <th scope="col" class="numeric">{{ t('amount') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="nutrient in data.nutrient_definitions" :key="nutrient.id">
                <th scope="row" lang="en">
                  {{ nutrient.label }} <small>{{ nutrient.unit }}</small>
                </th>
                <td class="numeric">
                  {{ exact(selected.nutrients[nutrient.id]?.amount ?? null) }}
                </td>
              </tr>
            </tbody>
          </table>
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
          <a :href="selected.source.url" target="_blank" rel="noopener noreferrer">{{
            t('openSource')
          }}</a>
          <h3>{{ t('uncertainty') }}</h3>
          <p class="muted">{{ t('original') }}</p>
          <p lang="en">{{ selected.uncertainty }}</p>
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
          ><NcButton @click="query = ''">{{ t('clearSearch') }}</NcButton></template
        ></NcEmptyContent
      >
      <ul v-else class="record-list">
        <li v-for="item in filtered" :key="item.id">
          <RouterLink
            :to="{ path: `/h/${route.params.home}/ingredients/${item.id}`, query: route.query }"
            ><div>
              <strong>{{ name(item) }}</strong
              ><span>{{ form(item) }}</span>
            </div>
            <span class="source-label"
              >{{ item.source.name }} <small>{{ item.source.record_id }}</small></span
            ></RouterLink
          >
        </li>
      </ul></template
    >
  </template>
</template>
