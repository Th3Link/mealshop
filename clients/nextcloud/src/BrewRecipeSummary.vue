<script setup lang="ts">
import type { BrewRecipe } from './brewing-api'
import { methodOptions, readingUnits } from './brewing-format'
import { t, compactExact, exact, shoppingUnit } from './i18n'
import { duration } from './cooking-format'
import BrewSources from './BrewSources.vue'
defineProps<{ recipe: BrewRecipe }>()
</script>
<template>
  <div>
    <dl class="brew-facts">
      <div>
        <dt>{{ t('brewVolume') }}</dt>
        <dd>{{ compactExact(recipe.volume.amount) }} {{ recipe.volume.unit }}</dd>
      </div>
      <div v-for="(choices, key) in methodOptions" :key="key">
        <dt>
          {{
            t(
              (
                {
                  strategy: 'brewStrategy',
                  layout: 'brewLayout',
                  heat: 'brewHeat',
                  sparge: 'brewSparge',
                  mash: 'brewMash',
                } as const
              )[key],
            )
          }}
        </dt>
        <dd>{{ t(choices[recipe.method[key] as keyof typeof choices]) }}</dd>
      </div>
    </dl>
    <section>
      <h2>{{ t('ingredients') }}</h2>
      <p v-if="!recipe.ingredients.length" class="muted">{{ t('brewIngredientHelp') }}</p>
      <ul class="plain-list">
        <li v-for="line in recipe.ingredients" :key="line.id">
          <strong :lang="recipe.language">{{ line.name }}</strong> ·
          <template v-if="line.quantity"
            >{{ compactExact(line.quantity.amount) }}
            {{ shoppingUnit(line.quantity.unit, line.quantity.amount) }}</template
          ><template v-else>{{ t('unknown') }}</template
          ><span v-if="line.phase_id" class="muted">
            · {{ recipe.phases.find((p) => p.stage.id === line.phase_id)?.stage.name }}</span
          >
          <p v-if="line.note" :lang="recipe.language">{{ line.note }}</p>
        </li>
      </ul>
    </section>
    <section>
      <h2>{{ t('cookSteps') }}</h2>
      <ol class="brew-phase-list">
        <li v-for="p in recipe.phases" :key="p.stage.id">
          <h3 :lang="recipe.language">{{ p.stage.name }}</h3>
          <p class="brew-instruction" :lang="recipe.language">{{ p.stage.instruction }}</p>
          <p class="muted">
            {{
              p.timing_known
                ? duration(p.stage.active_seconds + p.stage.passive_seconds)
                : t('brewUnknownTiming')
            }}<template v-if="p.temperature">
              · {{ exact(p.temperature.value) }} {{ t(readingUnits[p.temperature.unit]) }}</template
            ><template v-if="p.volume">
              · {{ compactExact(p.volume.amount) }} {{ p.volume.unit }}</template
            >
          </p>
          <p v-if="p.stage.dependencies.length" class="muted">
            {{ t('cookDependencies') }}:
            {{
              p.stage.dependencies
                .map((id) => recipe.phases.find((p) => p.stage.id === id)?.stage.name ?? id)
                .join(', ')
            }}
          </p>
          <p v-if="p.stage.uncertainty">{{ p.stage.uncertainty }}</p>
        </li>
      </ol>
    </section>
    <section v-if="recipe.yeast_preparation">
      <h3>{{ t('brewYeast') }}</h3>
      <p>
        {{ recipe.yeast_preparation.instruction }} · {{ recipe.yeast_preparation.days_before }}
        {{ t('brewDaysBefore') }} · {{ recipe.yeast_preparation.local_time }}
      </p>
    </section>
    <p v-if="recipe.notes" class="brew-instruction" :lang="recipe.language">{{ recipe.notes }}</p>
    <BrewSources :sources="recipe.template.sources" />
  </div>
</template>
