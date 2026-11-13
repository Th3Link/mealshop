<script setup lang="ts">
import type { BrewRecipe } from './brewing-api'
import { t } from './i18n'
defineProps<{ sources: BrewRecipe['template']['sources'] }>()
</script>
<template>
  <section class="brew-sources">
    <h3>{{ t('brewSource') }}</h3>
    <p class="muted">{{ t('brewSourceHelp') }}</p>
    <ul>
      <li v-for="s in sources" :key="s.id">
        <a :href="s.url" target="_blank" rel="noopener noreferrer"
          >{{ s.publisher }} · {{ s.title }}</a
        ><span class="muted">
          · {{ s.document_version ?? t('unknown') }} · {{ s.retrieved_on }}</span
        >
      </li>
    </ul>
  </section>
</template>
