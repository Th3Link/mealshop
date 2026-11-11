<script setup lang="ts">
import NcButton from '@nextcloud/vue/components/NcButton'
import LabeledField from './LabeledField.vue'
import RecipeContents from './RecipeContents.vue'
import Nutrition from './Nutrition.vue'
import type { SharedRecipe } from './library-api'
import { locale, servingCount, t } from './i18n'
const props = defineProps<{ value: SharedRecipe; imageUrl: string; busy: boolean }>()
const servings = defineModel<string | number>('servings', { required: true })
const unit = defineModel<'g' | 'kg'>('unit', { required: true })
const emit = defineEmits<{ preview: [] }>()
</script>
<template>
  <header class="page-header">
    <div>
      <h1 tabindex="-1" :lang="props.value.record.recipe.language">
        {{ props.value.record.recipe.title }}
      </h1>
      <p>
        <template v-if="props.value.record.recipe.author"
          >{{ props.value.record.recipe.author }} · </template
        >{{ t('revision') }} {{ props.value.record.revision }} ·
        {{ servingCount(props.value.record.recipe.servings) }}
      </p>
      <p v-if="props.value.record.recipe.language !== locale" class="muted">
        {{ t('originalLanguage') }}:
        {{ props.value.record.recipe.language === 'de' ? 'Deutsch' : 'English' }}
      </p>
    </div>
  </header>
  <img
    v-if="props.value.record.recipe.image_id"
    :src="props.imageUrl"
    :alt="props.value.record.recipe.title"
    class="recipe-image"
  />
  <div class="recipe-reading-toolbar">
    <form class="recipe-scale" @submit.prevent="emit('preview')">
      <LabeledField
        v-model="servings"
        type="number"
        :label="t('previewServings')"
        min="1"
        max="1000000"
        step="1"
        required
      /><NcButton type="submit" :disabled="props.busy">{{ t('preview') }}</NcButton>
    </form>
    <label class="field"
      >{{ t('displayUnit')
      }}<select v-model="unit" :disabled="props.busy" @change="emit('preview')">
        <option value="g">g</option>
        <option value="kg">kg</option>
      </select></label
    >
  </div>
  <RecipeContents
    :record="props.value.record"
    :lines="props.value.scaled?.display_lines ?? props.value.record.display_lines"
  />
  <Nutrition :value="props.value.scaled?.nutrition ?? props.value.record.nutrition" />
</template>
