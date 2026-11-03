import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { computed, ref } from 'vue'
import { isValidEmail, useLoginValidation } from './user'

describe('isValidEmail', () => {
  it('accepts email addresses containing hyphens', () => {
    expect(isValidEmail('first-last@outlook-mail.com')).toBe(true)
  })

  it('rejects malformed email addresses', () => {
    expect(isValidEmail('first-last@outlook')).toBe(false)
  })
})

describe('useLoginValidation offline / crack username rules', () => {
  beforeEach(() => {
    vi.stubGlobal('computed', computed)
    vi.stubGlobal('useI18n', () => ({
      t: (key: string) => key,
    }))
  })
  afterEach(() => vi.unstubAllGlobals())

  it('rejects usernames with 3 or fewer characters', () => {
    const { usernameRules } = useLoginValidation(ref(false), ref(true))
    const rules = usernameRules.value

    expect(rules.map((rule) => rule('abc'))).toContain('loginError.requireOver3Characters')
    expect(rules.map((rule) => rule('ab'))).toContain('loginError.requireOver3Characters')
    expect(rules.map((rule) => rule('a'))).toContain('loginError.requireOver3Characters')
  })

  it('accepts usernames with more than 3 valid characters', () => {
    const { usernameRules } = useLoginValidation(ref(false), ref(true))
    const rules = usernameRules.value

    for (const rule of rules) {
      expect(rule('Steve_123')).toBe(true)
    }
  })

  it('rejects empty username', () => {
    const { usernameRules } = useLoginValidation(ref(false), ref(true))
    const rules = usernameRules.value

    expect(rules.map((rule) => rule(''))).toContain('loginError.requireUsername')
  })

  it('rejects Cyrillic characters', () => {
    const { usernameRules } = useLoginValidation(ref(false), ref(true))
    const rules = usernameRules.value

    expect(rules.map((rule) => rule('Стив_123'))).toContain('loginError.cyrillicNotAllowed')
  })

  it('rejects usernames longer than 16 characters', () => {
    const { usernameRules } = useLoginValidation(ref(false), ref(true))
    const rules = usernameRules.value

    expect(rules.map((rule) => rule('UsernameIsWayTooLong'))).toContain('loginError.illegalOfflineUsername')
  })
})
