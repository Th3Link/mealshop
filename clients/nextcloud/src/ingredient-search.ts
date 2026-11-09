/** Match compound spellings without substituting ingredient identities. */
export function foldSearch(value: string): string {
  return value
    .normalize('NFD')
    .replace(/\p{M}/gu, '')
    .toLocaleLowerCase()
    .replace(/[\s-]+/g, '')
}
export function matchesSearch(text: string, query: string): boolean {
  const normalized = foldSearch(text)
  return query
    .trim()
    .split(/\s+/)
    .every((term) => normalized.includes(foldSearch(term)))
}
