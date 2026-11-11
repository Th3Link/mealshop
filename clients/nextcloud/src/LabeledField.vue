<script setup lang="ts">
import { useId } from 'vue'
import NcTextField from '@nextcloud/vue/components/NcTextField'
defineOptions({ inheritAttrs: false })
const props = defineProps<{ label: string; hideLabel?: boolean }>()
const model = defineModel<string | number>({ default: '' })
const id = useId()
</script>
<template>
  <div class="labeled-field">
    <label :for="id" :class="{ 'visually-hidden': hideLabel }">{{ props.label }}</label>
    <NcTextField v-bind="$attrs" :id="id" v-model="model" :label="label" label-outside />
  </div>
</template>
