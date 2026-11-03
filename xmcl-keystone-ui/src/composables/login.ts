import {
  AUTHORITY_DEV,
  AUTHORITY_MICROSOFT,
  AUTHORITY_MOJANG,
  AuthorityMetadata,
  YggdrasilApi,
} from '@xmcl/runtime-api'
import { Ref } from 'vue'
import { DialogKey } from './dialog'
import { injection } from '@/util/inject'
import { kUserContext } from './user'
import { kSettingsState } from './setting'
import { useLocalStorage } from '@vueuse/core'

export function useAccountSystemHistory() {
  const authority = useLocalStorage(
    'loginLastAuthAuthority',
    AUTHORITY_MICROSOFT as string,
    { writeDefaults: false },
  )
  const allHistoryRaw = useLocalStorage(
    'loginAuthorityHistory',
    () => [] as { name: string; authority: string }[],
  )
  const history = computed({
    get: () =>
      allHistoryRaw.value.filter((v) => v.authority === authority.value).map((v) => v.name),
    set: (v) => {
      allHistoryRaw.value = [
        ...allHistoryRaw.value.filter((v) => v.authority !== authority.value),
        ...v.map((x) => ({ name: x, authority: authority.value })),
      ]
    },
  })
  return {
    authority,
    history,
  }
}
export interface AuthorityItem {
  icon: string
  text: string
  value: string
}

const strictLocales = [
  // 欧洲 (Europe)
  'de-DE', // 德国 (Germany)
  'en-GB', // 英国 (United Kingdom)
  'fr-FR', // 法国 (France)
  'es-ES', // 西班牙 (Spain)
  'it-IT', // 意大利 (Italy)
  'pt-PT', // 葡萄牙 (Portugal)
  'nl-NL', // 荷兰 (Netherlands)
  'sv-SE', // 瑞典 (Sweden)
  'da-DK', // 丹麦 (Denmark)
  'fi-FI', // 芬兰 (Finland)
  'no-NO', // 挪威 (Norway)
  'el-GR', // 希腊 (Greece)
  'tr-TR', // 土耳其 (Turkey)
  'is-IS', // 冰岛 (Iceland)
  'en-IE', // 爱尔兰 (Ireland)
  'el-CY', // 塞浦路斯 (Cyprus)

  // 澳洲 (Australia)
  'en-AU', // 澳大利亚 (Australia)

  // 北美 (North America)
  'en-US', // 美国 (United States)
  'en-CA', // 加拿大 (Canada)

  'ja-JP', // 日本 (Japan)
  'ko-KR', // 韩国 (South Korea)
]

export function useAllowThirdparty() {
  const { state: setting } = injection(kSettingsState)
  const { users } = injection(kUserContext)
  const allowThirdParty = computed(() => {
    if (users.value.some((u) => u.authority === AUTHORITY_MICROSOFT)) return true
    if (setting.value?.developerMode) return true
    const locale = new Intl.NumberFormat().resolvedOptions().locale
    if (strictLocales.includes(locale)) return false
    return true
  })
  return allowThirdParty
}

export const crackIcon = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAYAAABzenr0AAABRUlEQVR4AezWParCUBCG4S+CP2DhFqwsrdyBnVaW2ulGxM3YuQwrU6cIZAEpUgQSklS5THVfTuEtLmYahQfm4GFO8k0IGa1Wq/6d9Xrd/0fYe7lc9jSS8+97AaOqqkThRNq2FYX/h2vutXo2m4kmk4noOwL/BDgPq/f7veh8Posul4voeDyKTqeTaLvdiqIoUhT98k8gfIqHXvsncDgcRIvFQhQm0jSNqOs6Ubifvaze7XYi/wTCKx567Z+AzYWu16vo+XyKOG+ry7IUxXEsYi+reZbV/gkMPfPwPP8EOD+rb7ebaLPZiF6vlyhNUxH3Ws1eVtsZ5J9AOJOh1/4J1HUtyvNc9Hg8ROPxWO/c73cRe1ldFIXIP4GhZx6e55+AfQcSnwer7X1PWZaJkiQRTadTkfWg+Xwu8k8gnMnQ648n8NcN/QAAAP//sB2D8gAAAAZJREFUAwDqdnLE3tzDDQAAAABJRU5ErkJggg=='

export function useAuthorityItems(authorities: Ref<AuthorityMetadata[] | undefined>) {
  const { t } = useI18n()
  const thirdParty = useAllowThirdparty()
  const items: Ref<AuthorityItem[]> = computed(() => {
    const result = [
      {
        value: AUTHORITY_MICROSOFT,
        text: t('userServices.microsoft.name'),
        icon: 'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyMSAyMSI+PHJlY3QgeD0iMSIgeT0iMSIgd2lkdGg9IjkiIGhlaWdodD0iOSIgZmlsbD0iI2YyNTAyMiIvPjxyZWN0IHg9IjExIiB5PSIxIiB3aWR0aD0iOSIgaGVpZ2h0PSI5IiBmaWxsPSIjN2ZiYTAwIi8+PHJlY3QgeD0iMSIgeT0iMTEiIHdpZHRoPSI5IiBoZWlnaHQ9IjkiIGZpbGw9IiMwMGE0ZWYiLz48cmVjdCB4PSIxMSIgeT0iMTEiIHdpZHRoPSI5IiBoZWlnaHQ9IjkiIGZpbGw9IiNmZmI5MDAiLz48L3N2Zz4=',
      },
      {
        value: AUTHORITY_DEV,
        text: t('userServices.offline.name') || 'Crack',
        icon: crackIcon,
      },
    ] as AuthorityItem[]

    if (!authorities.value) return result

    for (const v of authorities.value) {
      if (v.authority === AUTHORITY_MICROSOFT || v.authority === AUTHORITY_DEV) continue
      if (!thirdParty.value) continue
      // Use custom SVG icons or fallback to favicon
      let icon = v.favicon ?? ''
      if (v.authority.includes('ely.by') && !icon) {
        icon = 'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PHBhdGggZmlsbD0iIzFhOGM2NiIgZD0iTTIyIDEyYzAgNS41LTQuNSAxMC0xMCAxMFMyIDE3LjUgMiAxMiA2LjUgMiAxMiAyczEwIDQuNSAxMCAxMHpNOCA3djEwaDh2LTJoLTZ2LTJoNXYtMmgtNVY5aDZWN0g4eiIvPjwvc3ZnPg=='
      } else if (v.authority.includes('littleskin') && !icon) {
        icon = 'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PGNpcmNsZSBjeD0iMTIiIGN5PSIxMiIgcj0iMTIiIGZpbGw9IiMzYjgyZjYiLz48dGV4dCB4PSIxMiIgeT0iMTYiIGZvbnQtZmFtaWx5PSJBcmlhbCxHZW5ldmEsc2Fucy1zZXJpZiIgZm9udC1zaXplPSI5IiBmb250LXdlaWdodD0iYm9sZCIgZmlsbD0iI2ZmZmZmZiIgdGV4dC1hbmNob3I9Im1pZGRsZSI+TFM8L3RleHQ+PC9zdmc+'
      }

      result.push({
        value: v.authority,
        text: v.authlibInjector?.meta.serverName ?? new URL(v.authority).host,
        icon,
      })
    }
    return result
  })

  return items
}

export const LoginDialog: DialogKey<{ username?: string; service?: string; error?: string }> =
  'login'
