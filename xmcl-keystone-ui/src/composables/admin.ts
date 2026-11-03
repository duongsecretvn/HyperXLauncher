import { useLocalStorage } from '@vueuse/core'
import { computed } from 'vue'

const adminState = useLocalStorage('hyperx_admin_activated', false)

export function useAdmin() {
  const isAdmin = computed({
    get: () => adminState.value,
    set: (v: boolean) => { adminState.value = v },
  })

  function activateAdmin() {
    adminState.value = true
  }

  function deactivateAdmin() {
    adminState.value = false
  }

  return {
    isAdmin,
    activateAdmin,
    deactivateAdmin,
  }
}
