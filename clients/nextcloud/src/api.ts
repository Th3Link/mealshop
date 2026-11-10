import axios, { isAxiosError } from '@nextcloud/axios'
import { generateUrl } from '@nextcloud/router'
import { z } from 'zod'
import { t, knownExplanation } from './i18n'
export class ApiError extends Error {
  constructor(
    readonly code: string,
    readonly status: number,
  ) {
    super(code)
  }
}
export function errorMessage(error: unknown): string {
  if (!(error instanceof ApiError)) return t('error')
  if (error.status === 401) return t('unauthorized')
  if (error.status === 403) return t('forbidden')
  const localized = knownExplanation(error.code)
  if (localized) return localized
  if (error.status === 409) return t('conflict')
  if (error.status === 503 || error.status === 507) return t('storage')
  if (error.status === 400 || error.status === 422) return t('invalid')
  return t('error')
}
export async function api<T>(
  path: string,
  schema: z.ZodType<T>,
  options: {
    method?: 'GET' | 'POST' | 'PUT' | 'DELETE'
    data?: unknown
    signal?: AbortSignal
    contentType?: string
  } = {},
): Promise<T> {
  try {
    const response = await axios.request<unknown>({
      url: generateUrl(`/apps/app_api/proxy/mealshop${path}`),
      method: options.method ?? 'GET',
      headers: {
        'X-Mealshop-Request': '1',
        ...(options.contentType ? { 'Content-Type': options.contentType } : {}),
      },
      data: options.data,
      signal: options.signal,
    })
    return schema.parse(response.data)
  } catch (error) {
    if (isAxiosError(error)) {
      const parsed = z.object({ code: z.string() }).safeParse(error.response?.data)
      throw new ApiError(
        parsed.success ? parsed.data.code : 'request_failed',
        error.response?.status ?? 0,
      )
    }
    throw error
  }
}
export function domain(home: string, path: string): string {
  return `/api/households/${encodeURIComponent(home)}/domain${path}`
}

export function imageUrl(home: string, id: string): string {
  return generateUrl(
    `/apps/app_api/proxy/mealshop${domain(home, `/api/household/images/${encodeURIComponent(id)}`)}`,
  )
}
