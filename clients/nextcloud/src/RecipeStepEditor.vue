<script setup lang="ts">
import NcButton from '@nextcloud/vue/components/NcButton'
import NcTextField from './LabeledField.vue'
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
      <NcTextField
        v-model="step.active_seconds"
        type="number"
        :label="t('activeSeconds')"
        min="0"
        max="604800"
        step="1"
      /><NcTextField
        v-model="step.passive_seconds"
        type="number"
        :label="t('passiveSeconds')"
        min="0"
        max="604800"
        step="1"
      />
    </div>
    <NcButton
      variant="tertiary"
      :aria-label="`${t('remove')} ${t('instruction')} ${index + 1}`"
      @click="emit('remove')"
      ><template #icon><Icon :path="mdiTrashCanOutline" /></template>{{ t('remove') }}</NcButton
    >
  </fieldset>
</template>
