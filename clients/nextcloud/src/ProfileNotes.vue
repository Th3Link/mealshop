<script setup lang="ts">
import { ref, watch } from 'vue'
import type { Member } from './contracts'
import { t } from './i18n'
import triggers from '../../../data/intolerance-triggers.json'
const props = defineProps<{ member: Member }>()
const emit = defineEmits<{ changed: [] }>()
const values = ref({ allergies: '', intolerances: '', excluded_tags: '' })
const open = ref(false)
const known = (id: string) => triggers.some((trigger) => trigger.id === id)
watch(
  () => props.member,
  (member) => {
    values.value = {
      allergies: member.allergies.join('\n'),
      intolerances: member.intolerances.filter((id) => !known(id)).join('\n'),
      excluded_tags: member.excluded_tags.join('\n'),
    }
    open.value = Object.values(values.value).some(Boolean)
  },
  { immediate: true },
)
function input(event: Event, key: keyof typeof values.value) {
  if (!(event.target instanceof HTMLTextAreaElement)) return
  values.value[key] = event.target.value
  const lines = event.target.value
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
  props.member[key] =
    key === 'intolerances'
      ? [...new Set([...props.member.intolerances.filter(known), ...lines])]
      : lines
  emit('changed')
}
</script>
<template>
  <details class="editor-disclosure profile-notes" :open="open">
    <summary>{{ t('profileNotes') }}</summary>
    <label class="field"
      >{{ t('allergies') }} · {{ t('onePerLine')
      }}<textarea :value="values.allergies" @input="input($event, 'allergies')" />
    </label>
    <label class="field"
      >{{ t('intolerances') }} · {{ t('onePerLine')
      }}<textarea :value="values.intolerances" @input="input($event, 'intolerances')" />
    </label>
    <label class="field"
      >{{ t('excludedTags') }} · {{ t('onePerLine')
      }}<textarea :value="values.excluded_tags" @input="input($event, 'excluded_tags')" />
    </label>
    <p class="muted">{{ t('excludedTagsHelp') }}</p>
  </details>
</template>
