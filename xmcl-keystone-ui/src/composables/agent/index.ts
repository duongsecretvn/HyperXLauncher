import type { InjectionKey, Ref } from 'vue'
import { computed, ref } from 'vue'
import type { AgentMessage, AgentRunEvent } from '@xmcl/runtime-api'
import type { AgentContextUsage } from './local'

export * from './local'
export * from './ui'
export * from './cssAgent'

export interface AgentSession {
  readonly available: Readonly<Ref<boolean>>
  readonly running: Ref<boolean>
  readonly runError: Ref<string>
  readonly messages: Ref<AgentMessage[]>
  readonly events: Ref<AgentRunEvent[]>
  readonly contextUsage: Readonly<Ref<AgentContextUsage>>
  loadConversationForCurrentInstance(): Promise<void>
  replaceMessages(messages: AgentMessage[]): Promise<void>
  send(userInput: string): Promise<void>
  reset(): Promise<void>
  abort(): void
}

export function useAgent(): AgentSession {
  return {
    available: computed(() => false),
    running: ref(false),
    runError: ref(''),
    messages: ref([]),
    events: ref([]),
    contextUsage: computed<AgentContextUsage>(() => ({ usedTokens: 0, contextWindow: 0 })),
    loadConversationForCurrentInstance: async () => {},
    replaceMessages: async () => {},
    send: async () => {},
    reset: async () => {},
    abort: () => {},
  }
}

export const kAgent: InjectionKey<AgentSession> = Symbol('Agent')

export function installAgentDevLauncher(_session: AgentSession, _enabled: Ref<boolean>) {
  // no-op: AI HyperX completely disabled
}

