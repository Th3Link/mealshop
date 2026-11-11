<script setup lang="ts">
import { z } from 'zod'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcUserBubble from '@nextcloud/vue/components/NcUserBubble'
import Icon from './Icon.vue'
import { mdiContentCopy, mdiTrashCanOutline } from '@mdi/js'
import { grantsSchema } from './library-api'
import type { ShareResource } from './library-api'
import { t } from './i18n'
const props = defineProps<{
  items: z.infer<typeof grantsSchema>['shares']
  resource: ShareResource
  busy: boolean
}>()
const emit = defineEmits<{ revoke: [id: string]; publish: [id: string]; copy: [id: string] }>()
function needsPublication(item: z.infer<typeof grantsSchema>['shares'][number]) {
  return (
    props.resource.kind === 'recipe' &&
    item.grant.resource.kind === 'recipe' &&
    props.resource.revision !== item.grant.resource.revision
  )
}
</script>
<template>
  <ul class="share-grant-list">
    <li v-for="item in props.items" :key="item.grant.id">
      <div class="share-recipient">
        <NcUserBubble
          v-if="item.grant.recipient.kind !== 'household'"
          :user="item.grant.recipient.user"
          :size="32"
        />
        <strong v-else>{{ item.recipient_label || t('sharingUnavailable') }}</strong>
        <span class="muted"
          >{{
            item.grant.recipient.kind === 'household_invitation'
              ? t('sharingPending')
              : t('sharingActive')
          }}<template v-if="item.grant.resource.kind === 'recipe'">
            · {{ t('revision') }} {{ item.grant.resource.revision }}</template
          ></span
        >
      </div>
      <div class="share-grant-actions">
        <NcButton
          v-if="needsPublication(item)"
          :disabled="props.busy"
          @click="emit('publish', item.grant.id)"
          >{{ t('sharingPublishRevision') }}</NcButton
        >
        <NcButton
          variant="tertiary"
          :disabled="props.busy"
          :aria-label="`${t('sharingCopyLink')} · ${item.recipient_label}`"
          @click="emit('copy', item.grant.id)"
          ><template #icon><Icon :path="mdiContentCopy" /></template
        ></NcButton>
        <NcButton
          variant="tertiary"
          :disabled="props.busy"
          :aria-label="`${t('sharingRevoke')} · ${item.recipient_label}`"
          @click="emit('revoke', item.grant.id)"
          ><template #icon><Icon :path="mdiTrashCanOutline" /></template
        ></NcButton>
      </div>
    </li>
  </ul>
</template>
