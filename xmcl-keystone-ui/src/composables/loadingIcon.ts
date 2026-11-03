import { useLocalStorage } from '@vueuse/core'
import { computed } from 'vue'
import { injection } from '@/util/inject'
import { kTheme } from '@/composables/theme'

export type LoadingIconType = 'progressRing' | 'spinner' | 'progressBar'

export function useLoadingIcon() {
  const loadingIconType = useLocalStorage<LoadingIconType>('loading_icon_type', 'spinner')
  const customColor = useLocalStorage<string>('loading_icon_custom_color', '')

  const effectiveColor = computed(() => {
    if (customColor.value && customColor.value.trim() !== '') {
      return customColor.value
    }
    try {
      const theme = injection(kTheme)
      if (theme && theme.primaryColor && theme.primaryColor.value) {
        return theme.primaryColor.value
      }
    } catch {
      // kTheme might not be injected yet
    }
    return '#ffffff'
  })

  const hasCustomColor = computed(() => !!customColor.value && customColor.value.trim() !== '')

  function resetColor() {
    customColor.value = ''
  }

  return {
    loadingIconType,
    customColor,
    effectiveColor,
    hasCustomColor,
    resetColor,
  }
}
