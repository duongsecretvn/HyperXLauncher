import { computed, ref, type Ref } from 'vue'

const pendingAgentKind = ref<'common' | 'css'>('common')

export function usePendingAgentKind() {
  return pendingAgentKind
}

export function useAgentChatStatus() {
  return {
    shown: computed({
      get: () => false,
      set: () => {},
    }),
    running: ref(false),
    confirmationPending: ref(false),
  }
}

/** Open the agent chat - no-op since AI is disabled */
export function useAgentChatOpen() {
  return {
    open: (_options?: { prompt?: string; kind?: 'common' | 'css' }) => {},
  }
}

export function useAgentChatEntry() {
  return {
    configured: computed(() => false),
    open: () => {},
  }
}

/** Bind Ctrl/Cmd+Shift+A to open the agent chat - no-op */
export function useAgentChatHotkey(_enabled: Ref<boolean> = ref(true)) {}

