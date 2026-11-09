import { ref, shallowRef } from 'vue'
import { api, errorMessage } from './api'
import { homesSchema, catalogueSchema } from './contracts'
import type { Home } from './contracts'
export const homes = ref<Home[]>([])
export async function loadHomes() {
  const data = await api('/api/households', homesSchema)
  homes.value = data.households
  return data
}
export function useTask() {
  let generation = 0
  const busy = ref(false)
  const error = shallowRef<unknown>(null)
  async function run<T>(action: () => Promise<T>): Promise<T | undefined> {
    const current = ++generation
    busy.value = true
    error.value = null
    try {
      return await action()
    } catch (e) {
      if (current === generation) error.value = e
      return undefined
    } finally {
      if (current === generation) busy.value = false
    }
  }
  return { busy, error, run, errorMessage }
}

// The ExApp catalogue is immutable for this application build; never cache household data here.
let catalogueRequest: Promise<import('./contracts').Catalogue> | undefined
export function loadCatalogue() {
  catalogueRequest ??= api('/api/ingredients', catalogueSchema).catch((error) => {
    catalogueRequest = undefined
    throw error
  })
  return catalogueRequest
}
