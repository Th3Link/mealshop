import { onMounted, onBeforeUnmount } from 'vue'
import { onBeforeRouteLeave, onBeforeRouteUpdate } from 'vue-router'
import { t } from './i18n'

export function useUnsavedChanges(changed: () => boolean) {
  const discard = () => !changed() || window.confirm(t('unsaved'))
  const beforeUnload = (event: BeforeUnloadEvent) => {
    if (changed()) {
      event.preventDefault()
      event.returnValue = ''
    }
  }
  onBeforeRouteLeave(discard)
  onBeforeRouteUpdate(discard)
  onMounted(() => window.addEventListener('beforeunload', beforeUnload))
  onBeforeUnmount(() => window.removeEventListener('beforeunload', beforeUnload))
  return discard
}
