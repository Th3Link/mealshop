<script setup lang="ts">
import { ref } from 'vue'
import NcDialog from '@nextcloud/vue/components/NcDialog'
import NcTextField from '@nextcloud/vue/components/NcTextField'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import { z } from 'zod'
import { api, domain } from './api'
import { locale, t, allergenName } from './i18n'
import { groups } from '../../../data/allergens.json'
import { useTask, loadCatalogue } from './state'
import type { Catalogue, Ingredient } from './contracts'
import IngredientPicker from './IngredientPicker.vue'
import IngredientOrganization from './IngredientOrganization.vue'
import { organizationSchema } from './contracts'
const props = defineProps<{ home: string; catalogue: Catalogue; ingredient?: Ingredient }>()
const catalogue = ref(props.catalogue)
const currentIngredient = ref(props.ingredient)
const organization = ref(organizationSchema.parse(props.ingredient?.organization ?? {}))
const emit = defineEmits<{ close: []; saved: [id: string] }>()
const name = ref(''),
  form = ref(''),
  source = ref('')
const recorded = ref(new Date().toLocaleDateString('en-CA'))
const tags = ref(props.ingredient?.tags.join('\n') ?? '')
const components = ref([...(props.ingredient?.components ?? [])])
const allergens = ref([...(props.ingredient?.declared_allergens ?? [])])
const { busy, error, run, errorMessage } = useTask()
async function reload() {
  if (props.ingredient && !window.confirm(t('unsaved'))) return
  await run(async () => {
    const latest = await loadCatalogue(props.home)
    catalogue.value = latest
    if (props.ingredient) {
      const item = latest.ingredients.find((item) => item.id === props.ingredient?.id)
      if (!item) throw new Error('Ingredient not found')
      currentIngredient.value = item
      organization.value = structuredClone(item.organization)
      tags.value = item.tags.join('\n')
      components.value = [...item.components]
      allergens.value = [...item.declared_allergens]
    }
  })
}
async function save() {
  await run(async () => {
    const annotations = {
      organization: organization.value,
      tags: [
        ...new Set(
          tags.value
            .split('\n')
            .map((tag) => tag.trim())
            .filter(Boolean),
        ),
      ],
      components: components.value,
      declared_allergens: allergens.value,
      archived: currentIngredient.value?.archived ?? false,
      favorite: currentIngredient.value?.favorite ?? false,
    }
    const result = await api(
      domain(
        props.home,
        `/api/household/ingredients${props.ingredient ? `/${props.ingredient.id}` : ''}`,
      ),
      z.object({ id: z.string().optional(), revision: z.number().int() }),
      {
        method: props.ingredient ? 'PUT' : 'POST',
        data: props.ingredient
          ? { expected_revision: catalogue.value.revision, annotations }
          : {
              expected_revision: catalogue.value.revision,
              ingredient: {
                name: name.value,
                form: form.value,
                language: locale.value,
                source_url: source.value,
                recorded_on: recorded.value,
                annotations,
              },
            },
      },
    )
    emit('saved', result.id ?? props.ingredient?.id ?? '')
  })
}
</script>
<template>
  <NcDialog
    size="large"
    content-classes="mealshop ingredient-dialog"
    :name="ingredient ? t('editIngredient') : t('addIngredient')"
    @close="emit('close')"
  >
    <form class="ingredient-editor" @submit.prevent="save">
      <NcNoteCard v-if="error" type="error" role="alert"
        >{{ errorMessage(error)
        }}<NcButton :disabled="busy" @click="reload">{{ t('reloadSaved') }}</NcButton></NcNoteCard
      >
      <template v-if="!ingredient">
        <NcNoteCard type="info">{{ t('sourceDeclaration') }}</NcNoteCard>
        <NcTextField v-model="name" :label="t('ingredientName')" required maxlength="200" />
        <NcTextField v-model="form" :label="t('ingredientForm')" required maxlength="200" />
        <NcTextField v-model="source" :label="t('sourceOptional')" type="url" />
        <label class="field"
          >{{ t('recordedOn') }}<input v-model="recorded" type="date" required
        /></label>
      </template>
      <IngredientOrganization v-model="organization" :catalogue="catalogue" :id="ingredient?.id" />
      <label class="field"
        >{{ t('tags') }}<textarea v-model="tags" maxlength="2400" :aria-label="t('tags')" />
      </label>
      <p class="muted">{{ t('tagsHelp') }}</p>
      <IngredientPicker
        v-model="components"
        :ingredients="
          catalogue.ingredients.filter(
            (item) =>
              item.id !== ingredient?.id && (!item.archived || components.includes(item.id)),
          )
        "
        :label="t('components')"
      />
      <details :open="Boolean(ingredient?.declared_allergens.length)">
        <summary>{{ t('declaredAllergens') }}</summary>
        <fieldset>
          <legend>{{ t('declaredAllergens') }}</legend>
          <div class="allergen-options">
            <NcCheckboxRadioSwitch
              v-for="group in groups"
              :key="group.id"
              v-model="allergens"
              :value="group.id"
              >{{ allergenName(group.id) }}</NcCheckboxRadioSwitch
            >
          </div>
        </fieldset>
      </details>
      <div class="actions">
        <NcButton :disabled="busy" @click="emit('close')">{{ t('cancel') }}</NcButton
        ><NcButton variant="primary" type="submit" :disabled="busy">{{ t('save') }}</NcButton>
      </div>
    </form>
  </NcDialog>
</template>
