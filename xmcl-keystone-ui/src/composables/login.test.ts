import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { computed, ref } from 'vue'
import { AUTHORITY_DEV, AUTHORITY_MICROSOFT } from '@xmcl/runtime-api'
import { crackIcon, useAuthorityItems } from './login'

vi.mock('@/util/inject', () => ({
  injection: () => ({
    state: ref({ developerMode: false }),
    users: ref([]),
  }),
}))

describe('login authority items', () => {
  beforeEach(() => {
    vi.stubGlobal('computed', computed)
    vi.stubGlobal('useI18n', () => ({
      t: (key: string) => {
        if (key === 'userServices.microsoft.name') return 'Microsoft'
        if (key === 'userServices.offline.name') return 'Crack'
        return key
      },
    }))
  })
  afterEach(() => vi.unstubAllGlobals())

  it('exports crackIcon as valid base64 png data URI', () => {
    expect(crackIcon).toMatch(/^data:image\/png;base64,/)
  })

  it('includes Microsoft and Crack in authority items by default', () => {
    const items = useAuthorityItems(ref(undefined))
    expect(items.value).toHaveLength(2)
    expect(items.value[0].value).toBe(AUTHORITY_MICROSOFT)
    expect(items.value[0].text).toBe('Microsoft')
    expect(items.value[1].value).toBe(AUTHORITY_DEV)
    expect(items.value[1].text).toBe('Crack')
    expect(items.value[1].icon).toBe(crackIcon)
  })
})
