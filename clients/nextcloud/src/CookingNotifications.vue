<script setup lang="ts">
import { ref } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import type { CookingDetail, CookingCommand, DevicePermission } from './cooking-api'
import { localDate } from './cooking-format'
import { locale, t, explanation } from './i18n'
const props = defineProps<{
  detail: Pick<CookingDetail, 'notification_transport' | 'notices' | 'scheduler'> & {
    run: Pick<CookingDetail['run'], 'id' | 'timezone' | 'status' | 'steps' | 'subscription'>
  }
  open?: boolean
  disabled: boolean
  writable: boolean
  now: number
}>()
const emit = defineEmits<{ command: [value: CookingCommand] }>()
const permission = ref<DevicePermission>(
  typeof Notification === 'undefined' ? 'unsupported' : Notification.permission,
)
function subscribe(enabled: boolean, reported = permission.value) {
  emit('command', {
    operation: 'subscribe',
    run_id: props.detail.run.id,
    enabled,
    language: locale.value,
    permission: reported,
  })
}
async function requestPermission() {
  if (typeof Notification !== 'undefined') {
    const reported = await Notification.requestPermission()
    permission.value = reported
    subscribe(props.detail.run.subscription?.enabled ?? false, reported)
  }
}
function retry(id: string, unknown: boolean) {
  if (!unknown || window.confirm(t('cookRetryRisk')))
    emit('command', { operation: 'retry', notice_id: id, duplicate_risk_accepted: unknown })
}
const eventKeys = {
  prepare: 'cookEventPrepare',
  start: 'cookEventStart',
  active_ended: 'cookEventActive',
  timer_ended: 'cookEventEnd',
} as const
const deliveryKeys = {
  queued: 'cookDeliveryQueued',
  sending: 'cookDeliverySending',
  accepted: 'cookDeliveryAccepted',
  failed: 'cookDeliveryFailed',
  unknown: 'cookDeliveryUnknown',
  missed: 'cookDeliveryMissed',
  cancelled: 'cookCancelled',
} as const
</script>
<template>
  <details class="cooking-notifications" :open="open">
    <summary>{{ t('cookNotifications') }}</summary>
    <p>{{ t('cookNotificationHelp') }}</p>
    <NcNoteCard v-if="!detail.notification_transport" type="info">{{
      t('cookNoTransport')
    }}</NcNoteCard
    ><NcNoteCard v-if="permission === 'denied' || permission === 'unsupported'" type="warning">{{
      t(permission === 'denied' ? 'cookPermissionDenied' : 'cookPermissionUnsupported')
    }}</NcNoteCard>
    <p>
      {{ t('cookBrowserPermission') }}:
      {{
        t(
          (
            {
              default: 'cookPermissionDefault',
              granted: 'cookPermissionGranted',
              denied: 'cookPermissionDeniedShort',
              unsupported: 'cookPermissionUnsupportedShort',
            } as const
          )[permission],
        )
      }}
    </p>
    <NcButton
      v-if="writable && permission === 'default'"
      :disabled="disabled"
      @click="requestPermission"
      >{{ t('cookRequestPermission') }}</NcButton
    >
    <NcCheckboxRadioSwitch
      v-if="detail.run.status === 'active'"
      :model-value="detail.run.subscription?.enabled ?? false"
      :disabled="disabled || !writable || !detail.notification_transport"
      @update:model-value="subscribe(!detail.run.subscription?.enabled)"
      >{{ t('cookSubscribe') }}</NcCheckboxRadioSwitch
    >
    <NcNoteCard v-if="detail.scheduler.error" type="error">{{
      explanation(detail.scheduler.error)
    }}</NcNoteCard>
    <p
      v-else-if="
        detail.notification_transport &&
        (!detail.scheduler.last_check_at || now - detail.scheduler.last_check_at > 45)
      "
      role="status"
    >
      {{ t('cookSchedulerWaiting') }}
    </p>
    <p v-else-if="detail.scheduler.last_check_at" class="muted">
      {{ t('cookSchedulerChecked') }}:
      {{ localDate(detail.scheduler.last_check_at, detail.run.timezone) }}
    </p>
    <h3>{{ t('cookDeliveryHistory') }}</h3>
    <p v-if="!detail.notices.length">{{ t('cookNoNotices') }}</p>
    <ul class="plain-list">
      <li v-for="notice in detail.notices" :key="notice.id">
        <strong
          >{{ t(eventKeys[notice.event]) }} ·
          {{ detail.run.steps.find((s) => s.stage.id === notice.step_id)?.stage.name }}</strong
        >
        <p>
          {{ t(deliveryKeys[notice.delivery.state]) }} ·
          {{ localDate(notice.due, detail.run.timezone) }}
        </p>
        <p v-if="notice.delivery.state === 'failed'">{{ explanation(notice.delivery.code) }}</p>
        <p v-if="notice.acknowledged_at" class="muted">
          {{ t('cookAcknowledged') }} · {{ localDate(notice.acknowledged_at, detail.run.timezone) }}
        </p>
        <div v-else-if="writable" class="actions">
          <NcButton
            :disabled="disabled"
            @click="emit('command', { operation: 'acknowledge', notice_id: notice.id })"
            >{{ t('cookAcknowledge') }}</NcButton
          ><NcButton
            v-if="
              (notice.delivery.state === 'failed' || notice.delivery.state === 'unknown') &&
              notice.attempts < 3 &&
              detail.run.status === 'active'
            "
            :disabled="disabled"
            @click="retry(notice.id, notice.delivery.state === 'unknown')"
            >{{ t('retry') }}</NcButton
          >
        </div>
      </li>
    </ul>
  </details>
</template>
