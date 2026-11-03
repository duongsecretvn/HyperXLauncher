import { useLocalStorage } from '@vueuse/core'
import { computed, watch } from 'vue'
import type { Router } from 'vue-router'

export interface FontItem {
  text: string
  value: string
  fontFamily: string
}

export const LAUNCHER_FONTS: FontItem[] = [
  { text: 'Mặc định (Default)', value: 'default', fontFamily: "'Roboto', sans-serif" },
  { text: 'Minecraft', value: 'Minecraft', fontFamily: "'Minecraft', sans-serif" },
  { text: 'Minecraft Ten', value: 'Minecraft Ten', fontFamily: "'Minecraft Ten', sans-serif" },
  { text: 'Minecrafter', value: 'Minecrafter', fontFamily: "'Minecrafter', sans-serif" },
  { text: 'Minecraft Evenings', value: 'Minecraft Evenings', fontFamily: "'Minecraft Evenings', sans-serif" },
  { text: 'Minecraft CHMC', value: 'Minecraft CHMC', fontFamily: "'Minecraft CHMC', sans-serif" },
  { text: 'Block Craft', value: 'Block Craft', fontFamily: "'Block Craft', sans-serif" },
  { text: 'Bloxat', value: 'Bloxat', fontFamily: "'Bloxat', sans-serif" },
  { text: 'Pixel Purl', value: 'Pixel Purl', fontFamily: "'Pixel Purl', sans-serif" },
  { text: 'ProFont Windows', value: 'ProFont Windows', fontFamily: "'ProFont Windows', monospace" },
  { text: 'Nineteen Ninety Three', value: 'Nineteen Ninety Three', fontFamily: "'Nineteen Ninety Three', sans-serif" },
  { text: 'Brighton Two Sans', value: 'Brighton Two Sans', fontFamily: "'Brighton Two Sans', sans-serif" },
  { text: 'Bond Story', value: 'Bond Story', fontFamily: "'Bond Story', sans-serif" },
  { text: 'Domino', value: 'Domino', fontFamily: "'Domino', sans-serif" },
  { text: 'Maus', value: 'Maus', fontFamily: "'Maus', sans-serif" },
  { text: 'Nillambari', value: 'Nillambari', fontFamily: "'Nillambari', sans-serif" },
]

// Shared singleton state so font changes reflect immediately across the whole application
export const selectedFont = useLocalStorage<string>('hyperx-launcher-font', 'default')

export const currentFontFamily = computed(() => {
  const found = LAUNCHER_FONTS.find(f => f.value === selectedFont.value)
  if (!found || found.value === 'default') {
    return "'Roboto', sans-serif"
  }
  return `${found.fontFamily}, 'Roboto', sans-serif`
})

export function applyGlobalFontOverride(fontFamily: string, isDefault: boolean) {
  if (typeof document === 'undefined') return
  let el = document.getElementById('hyperx-font-override') as HTMLStyleElement | null
  if (isDefault) {
    if (el) el.textContent = ''
    return
  }
  if (!el) {
    el = document.createElement('style')
    el.id = 'hyperx-font-override'
  }
  // Guarantee highest cascade precedence by staying as the last element in head
  document.head.appendChild(el)

  el.textContent = `
    :root {
      --hyperx-font-family: ${fontFamily};
    }

    html,
    body,
    #app,
    .v-application,
    .v-overlay-container,
    .v-overlay-container *,
    .v-application *,
    .v-application button,
    .v-application input,
    .v-application select,
    .v-application textarea,
    .v-application a,
    .v-application span,
    .v-application p,
    .v-application div,
    .v-application label,
    .v-application h1,
    .v-application h2,
    .v-application h3,
    .v-application h4,
    .v-application h5,
    .v-application h6,
    .v-tab,
    .v-tab *,
    .v-tab .v-btn__content,
    .v-btn,
    .v-btn *,
    .v-btn .v-btn__content,
    .v-card,
    .v-card *,
    .v-card-title,
    .v-card-subtitle,
    .v-card-text,
    .v-list,
    .v-list-item,
    .v-list-item *,
    .v-list-item-title,
    .v-list-item-subtitle,
    .v-toolbar,
    .v-toolbar *,
    .v-toolbar-title,
    .v-app-bar,
    .v-app-bar *,
    .v-system-bar,
    .v-system-bar *,
    .v-navigation-drawer,
    .v-navigation-drawer *,
    .v-main,
    .v-main *,
    .text-body-1,
    .text-body-2,
    .text-subtitle-1,
    .text-subtitle-2,
    .text-caption,
    .text-button,
    .text-overline,
    .text-h1,
    .text-h2,
    .text-h3,
    .text-h4,
    .text-h5,
    .text-h6,
    .setting-item,
    .setting-item *,
    .setting-item__title,
    .setting-item__subtitle,
    .sidebar-item,
    .sidebar-item *,
    .sidebar-item__content {
      font-family: ${fontFamily} !important;
    }

    .v-icon,
    .v-icon *,
    .v-icon::before,
    .v-icon::after,
    .material-icons,
    .material-symbols-outlined,
    [class*="material-"] {
      font-family: 'Material Symbols Outlined', 'Material Icons', 'Material Icons Outlined' !important;
    }

    code, code *, kbd, samp, pre, pre *, .font-mono, .code-view, .log-view {
      font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace !important;
    }
  `
}

export function useLauncherFont(router?: Router) {
  if (typeof window !== 'undefined') {
    watch(
      [selectedFont, currentFontFamily],
      ([fontVal, fam]) => {
        applyGlobalFontOverride(fam, fontVal === 'default')
      },
      { immediate: true },
    )

    if (router) {
      router.afterEach(() => {
        applyGlobalFontOverride(currentFontFamily.value, selectedFont.value === 'default')
      })
    }
  }

  return {
    selectedFont,
    fonts: LAUNCHER_FONTS,
    currentFontFamily,
    applyGlobalFontOverride,
  }
}
