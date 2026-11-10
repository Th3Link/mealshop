<script setup lang="ts">
import NcButton from '@nextcloud/vue/components/NcButton'
import CookingDuration from './CookingDuration.vue'
import RecipeReferences from './RecipeReferences.vue'
import Icon from './Icon.vue'
import { mdiTrashCanOutline } from '@mdi/js'
import type { Recipe } from './contracts'
import { t } from './i18n'
const step = defineModel<Recipe['steps'][number]>({ required: true })
defineProps<{
  index: number
  parts: { id: string; label: string }[]
  ingredients: { id: string; label: string }[]
}>()
const emit = defineEmits<{ remove: [] }>()
</script>
<template>
  <fieldset class="step-editor">
    <legend class="visually-hidden">{{ t('instruction') }} {{ index + 1 }}</legend>
    <label class="field"
      >{{ index + 1 }}. {{ t('instruction')
      }}<textarea v-model="step.instruction" required maxlength="4000" />
    </label>
    <label v-if="parts.length" class="field"
      >{{ t('belongsTo')
      }}<select v-model="step.part_id">
        <option :value="null">{{ t('mainRecipe') }}</option>
        <option v-for="part in parts" :key="part.id" :value="part.id">{{ part.label }}</option>
      </select></label
    >
    <RecipeReferences
      v-model="step.ingredient_line_ids"
      :label="t('stepIngredients')"
      :options="ingredients"
    />
    <RecipeReferences
      v-if="parts.length"
      v-model="step.prepared_parts"
      :label="t('stepParts')"
      :options="parts.filter((p) => p.id !== step.part_id)"
    />
    <div class="form-row">
      <CookingDuration v-model="step.active_seconds" :label="t('cookActiveDuration')" />
      <CookingDuration v-model="step.passive_seconds" :label="t('cookPassiveDuration')" />
    </div>
    <details>
      <summary>{{ t('waitingRange') }}</summary>
      <CookingDuration
        :model-value="step.passive_max_seconds ?? step.passive_seconds"
        :label="t('maximumWait')"
        @update:model-value="step.passive_max_seconds = $event"
      />
    </details>
    <NcButton
      variant="tertiary"
      :aria-label="`${t('remove')} ${t('instruction')} ${index + 1}`"
      @click="emit('remove')"
      ><template #icon><Icon :path="mdiTrashCanOutline" /></template>{{ t('remove') }}</NcButton
    >
  </fieldset>
</template>
