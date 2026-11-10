<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import NcDialog from '@nextcloud/vue/components/NcDialog'
import NcTextField from './LabeledField.vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import { z } from 'zod'
import { api, domain } from './api'
import { locale, t, allergenName } from './i18n'
import { groups } from '../../../data/allergens.json'
import intoleranceTriggers from '../../../data/intolerance-triggers.json'
import { useTask, loadCatalogue } from './state'
import type { Catalogue, Ingredient } from './contracts'
import IngredientPicker from './IngredientPicker.vue'
import IngredientOrganization from './IngredientOrganization.vue'
import { organizationSchema } from './contracts'
import { useUnsavedChanges } from './useUnsavedChanges'
const props = defineProps<{
  home: string
  catalogue: Catalogue
  ingredient?: Ingredient
  initialName?: string
  parentDirty?: boolean
}>()
const catalogue = ref(props.catalogue)
const currentIngredient = ref(props.ingredient)
const organization = ref(organizationSchema.parse(props.ingredient?.organization ?? {}))
const emit = defineEmits<{ close: []; saved: [id: string] }>()
const name = ref(props.initialName ?? ''),
  form = ref(''),
  source = ref('')
const recorded = ref(new Date().toLocaleDateString('en-CA'))
const tags = ref(props.ingredient?.tags.join('\n') ?? '')
const components = ref([...(props.ingredient?.components ?? [])])
const allergens = ref([...(props.ingredient?.declared_allergens ?? [])])
const { busy, error, run, errorMessage } = useTask()
const changed = ref(false)
const fields = ref<HTMLFieldSetElement>()
const savedId = ref<string>()
const returnFocus =
  document.activeElement instanceof HTMLElement ? document.activeElement : undefined
useUnsavedChanges(() => changed.value || Boolean(props.parentDirty))
watch(
  [name, form, source, recorded, tags, components, allergens, organization],
  () => {
    changed.value = true
  },
  { deep: true },
)
function canClose() {
  return !busy.value && (!changed.value || window.confirm(t('unsaved')))
}
async function closed(open: boolean) {
  if (open) return
  if (savedId.value !== undefined) emit('saved', savedId.value)
  else emit('close')
  await nextTick()
  if (returnFocus?.isConnected) returnFocus.focus()
}
function revealInvalid(event: Event) {
  if (event.target instanceof HTMLElement) {
    let parent = event.target.parentElement
    while (parent) {
      if (parent instanceof HTMLDetailsElement) parent.open = true
      parent = parent.parentElement
    }
  }
}
async function reload() {
  if (props.ingredient && changed.value && !window.confirm(t('unsaved'))) return
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
  if (!error.value) changed.value = false
}
async function save() {
  if (busy.value || !fields.value?.form?.reportValidity()) return false
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
    changed.value = false
    savedId.value = result.id ?? props.ingredient?.id ?? ''
  })
  return !error.value
}
</script>
<template>
  <NcDialog
    size="large"
    content-classes="mealshop ingredient-dialog"
    :name="ingredient ? t('editIngredient') : t('addIngredient')"
    is-form
    :set-return-focus="returnFocus"
    :no-close="busy || changed"
    :buttons="[
      { label: t('cancel'), disabled: busy, callback: canClose },
      {
        label: busy ? t('ingredientPending') : t('save'),
        variant: 'primary',
        type: 'submit',
        disabled: busy,
        callback: save,
      },
    ]"
    @update:open="closed"
  >
    <div class="ingredient-editor" @invalid.capture="revealInvalid">
      <fieldset ref="fields" class="editor-fields" :disabled="busy">
        <NcNoteCard v-if="error" type="error" role="alert"
          >{{ errorMessage(error)
          }}<NcButton :disabled="busy" @click="reload">{{ t('reloadSaved') }}</NcButton></NcNoteCard
        >
        <section v-if="!ingredient" class="editor-section">
          <h3>{{ t('identitySection') }}</h3>
          <p class="muted">{{ t('identityHelp') }}</p>
          <NcTextField v-model="name" :label="t('ingredientName')" required maxlength="200" />
          <NcTextField v-model="form" :label="t('ingredientForm')" required maxlength="200" />
        </section>
        <IngredientOrganization
          v-model="organization"
          :catalogue="catalogue"
          :id="ingredient?.id"
        />
        <details class="editor-disclosure">
          <summary>{{ t('ingredientMetadata') }}</summary>
          <div class="editor-fields">
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
          </div>
        </details>
        <details class="editor-disclosure" :open="Boolean(ingredient?.declared_allergens.length)">
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
        <details class="editor-disclosure">
          <summary>
            {{ t('intoleranceTriggers')
            }}<span v-if="organization.intolerance_triggers.length"> · {{ t('sorbitol') }}</span>
          </summary>
          <p class="muted">{{ t('allergenHelp') }}</p>
          <NcCheckboxRadioSwitch
            v-for="trigger in intoleranceTriggers"
            :key="trigger.id"
            v-model="organization.intolerance_triggers"
            :value="trigger.id"
            >{{ t('sorbitol') }}</NcCheckboxRadioSwitch
          >
        </details>
        <details v-if="!ingredient" class="editor-disclosure">
          <summary>{{ t('provenanceSection') }}</summary>
          <div class="editor-fields">
            <NcTextField v-model="source" :label="t('sourceOptional')" type="url" />
            <label class="field"
              >{{ t('recordedOn') }}<input v-model="recorded" type="date" required
            /></label>
          </div>
        </details>
      </fieldset>
    </div>
  </NcDialog>
</template>
