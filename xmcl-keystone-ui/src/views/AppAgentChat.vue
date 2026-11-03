<template>
  <div data-testid="agent-dialog" class="agent-surface flex flex-col">
    <!-- HyperXAi Header matching Launcher Acrylic Theme -->
    <div class="hyperxai-header flex items-center justify-between px-4 py-3">
      <div class="flex items-center gap-3">
        <!-- Avatar Robot mini with thinking pulse animation -->
        <AppAgentIcon :size="24" :thinking="running" />
        <div>
          <div class="flex items-center gap-2">
            <span class="hyperxai-title">HyperXAi</span>
          </div>
          <div class="flex items-center gap-1.5 font-medium mt-0.5">
            <span class="w-2 h-2 rounded-full bg-[#22c55e] inline-block shadow-[0_0_8px_#22c55e]" :class="{ 'animate-pulse': running }" />
            <span class="hyperxai-status">{{ running ? 'Đang suy luận...' : 'Đang trực tuyến' }}</span>
          </div>
        </div>
      </div>
      <div class="flex items-center gap-1">
        <v-btn
          icon="add_comment"
          size="small"
          variant="text"
          :disabled="running || !available"
          title="Đoạn chat mới"
          @click="startNewConversation"
        />
        <v-btn
          icon="close"
          size="small"
          variant="text"
          :aria-label="t('shared.close')"
          title="Đóng"
          @click="closeChat"
        />
      </div>
    </div>

    <!-- Loading / connecting state -->
    <div
      v-if="!available"
      class="agent-disabled-card flex-1 flex flex-col items-center justify-center p-6 text-center"
    >
      <v-avatar size="64" color="primary" variant="tonal" class="mb-3">
        <v-icon size="36">smart_toy</v-icon>
      </v-avatar>
      <div class="text-base font-semibold mb-1 text-white">
        Đang khởi động HyperXAi
      </div>
      <div class="text-xs text-medium-emphasis max-w-xs mb-4">
        Đang đồng bộ kết nối dịch vụ HyperXAi. Vui lòng đợi trong giây lát...
      </div>
    </div>

    <!-- Transcript Area -->
    <div
      v-else
      ref="scrollEl"
      data-testid="agent-transcript"
      class="agent-transcript flex-1 overflow-y-auto"
      @scroll.passive="onTranscriptScroll"
    >
      <!-- Empty state: HyperXAi welcome greeting bubble matching launcher theme -->
      <div
        v-if="transcriptItems.length === 0 && !displayError"
        class="hyperxai-empty-wrap pa-4 flex flex-col items-center text-center"
      >
        <div class="hyperxai-welcome-card w-full text-left">
          <div class="hyperxai-welcome-title">
            Xin chào! Tôi là <strong class="hyperxai-welcome-name">HyperXAi</strong>.
          </div>
          <div class="hyperxai-welcome-sub">
            Tôi có thể tối ưu gì cho bạn?
          </div>
        </div>
      </div>

      <!-- Messages list -->
      <template v-for="item in transcriptItems" :key="item.key">
        <!-- User message bubble -->
        <div v-if="item.kind === 'message' && item.message.role === 'user'" class="flex justify-end my-1">
          <div class="bubble bubble-user">
            {{ messageText(item.message.content) }}
          </div>
        </div>

        <!-- HyperXAi message bubble -->
        <div
          v-else-if="item.kind === 'message' && item.message.role === 'assistant' && messageText(item.message.content)"
          class="flex justify-start my-1"
        >
          <div
            class="bubble bubble-assistant md-content"
            v-html="renderAssistant(item.message.content)"
          />
        </div>

        <!-- Tool call item -->
        <div v-else-if="item.kind === 'tool'" class="flex justify-start my-1 w-full">
          <AgentToolCall :item="item" />
        </div>
      </template>

      <!-- User confirmation -->
      <div v-if="confirmationShown" class="flex justify-start my-2">
        <div data-testid="agent-confirm-dialog" class="agent-confirm">
          <div class="agent-confirm__header">
            <div
              class="agent-confirm__icon"
              :class="{ 'agent-confirm__icon--destructive': confirmationRequest?.destructive }"
            >
              <v-icon size="18">{{ confirmationRequest?.destructive ? 'delete_outline' : 'priority_high' }}</v-icon>
            </div>
            <div class="min-w-0">
              <div class="agent-confirm__title font-bold text-sm">
                {{ confirmationRequest?.title || 'Xác nhận thực hiện' }}
              </div>
              <div class="agent-confirm__message text-xs text-medium-emphasis">
                {{ confirmationRequest?.message }}
              </div>
            </div>
          </div>
          <div v-if="confirmationRequest?.details?.length" class="agent-confirm__details">
            <div v-for="detail in confirmationRequest.details" :key="detail" class="agent-confirm__detail">
              <v-icon size="15" class="flex-shrink-0">subdirectory_arrow_right</v-icon>
              <code>{{ detail }}</code>
            </div>
          </div>
          <div v-if="confirmationRequest?.presentations?.length" class="agent-confirm__presentations">
            <AgentMarketList
              v-for="presentation in confirmationRequest.presentations"
              :key="presentation.source"
              :presentation="presentation"
              readonly
            />
          </div>
          <div class="agent-confirm__actions flex items-center justify-end gap-2 p-2">
            <v-btn size="small" variant="text" @click="declineConfirmation">
              Hủy
            </v-btn>
            <v-btn
              v-if="confirmationRequest?.allowAll"
              size="small"
              variant="text"
              @click="allowAllConfirmations"
            >
              Cho phép tất cả
            </v-btn>
            <v-btn
              size="small"
              :color="confirmationRequest?.destructive ? 'error' : 'primary'"
              variant="flat"
              :prepend-icon="confirmationRequest?.destructive ? 'delete_outline' : 'check'"
              @click="acceptConfirmation"
            >
              {{ confirmationRequest?.confirmLabel || 'Xác nhận' }}
            </v-btn>
          </div>
        </div>
      </div>

      <!-- Live progress status -->
      <div v-if="running" class="flex justify-start my-2">
        <div data-testid="agent-live-status" class="agent-live-card flex items-center gap-2">
          <v-progress-circular indeterminate size="14" width="2" color="primary" />
          <span class="text-xs text-zinc-300">{{ liveStatus }}</span>
        </div>
      </div>

      <!-- Error display in transcript -->
      <div v-if="displayError && !running" data-testid="agent-error" class="flex justify-start my-2">
        <div class="agent-error">
          <v-icon size="small" color="error" class="agent-error__icon">error_outline</v-icon>
          <div class="min-w-0 flex-1">
            <div class="agent-error__title">
              Thông báo
            </div>
            <div class="agent-error__body">
              {{ displayError }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Khung Input nhập câu hỏi phía dưới đáy theo đúng phong cách launcher -->
    <div class="hyperxai-input-wrap">
      <div class="hyperxai-input-box">
        <textarea
          ref="textareaRef"
          v-model="input"
          :placeholder="available ? 'Nhập câu hỏi cho HyperXAi...' : 'Đang kết nối HyperXAi...'"
          :disabled="!available"
          rows="1"
          class="hyperxai-input-field"
          @compositionstart="composing = true"
          @compositionend="composing = false"
          @keydown="onKeydown"
        />
        <button
          v-if="!running"
          type="button"
          class="hyperxai-send-btn"
          :disabled="!available || !input.trim()"
          title="Gửi"
          aria-label="Gửi"
          @click="onSend"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M4 2L9 7L4 12" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
        <button
          v-else
          type="button"
          class="hyperxai-send-btn hyperxai-stop-btn"
          title="Dừng phản hồi"
          aria-label="Dừng phản hồi"
          @click="abort"
        >
          <v-icon size="16" color="white">stop</v-icon>
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import AppAgentIcon from '@/components/AppAgentIcon.vue'
import AgentMarketList from '@/components/AgentMarketList.vue'
import AgentToolCall from '@/components/AgentToolCall.vue'
import { kAgent } from '@/composables/agent'
import { useAgentConfirmation } from '@/composables/agent/confirm'
import { projectAgentTranscript } from '@/composables/agent/projection'
import { useAgentRouteReturn } from '@/composables/agent/routeReturn'
import { useAgentSettings } from '@/composables/agent/settings'
import { useAgentChatStatus } from '@/composables/agentChat'
import { useMarkdown } from '@/composables/markdown'
import { useOmniDialog } from '@/composables/omniDialog'
import { kSettingsState } from '@/composables/setting'
import { useAdmin } from '@/composables/admin'
import { injection } from '@/util/inject'
import { computed, nextTick, ref, watch } from 'vue'
import type { AgentContentPart as ContentPart } from '@xmcl/runtime-api'

const { t } = useI18n()
const { isAdmin } = useAdmin()
const commonAgent = injection(kAgent)
const agentSettings = useAgentSettings()
const { state: settingsState } = injection(kSettingsState)
useAgentRouteReturn()

const available = computed(() => commonAgent.available.value)
const running = computed(() => commonAgent.running.value)
const runError = computed(() => commonAgent.runError.value)
const messages = computed(() => commonAgent.messages.value)
const events = computed(() => commonAgent.events.value)
const contextUsage = computed(() => commonAgent.contextUsage.value)
const developerMode = computed(() => settingsState.value?.developerMode ?? false)

const {
  request: confirmationRequest,
  shown: confirmationShown,
  accept: acceptConfirmation,
  allowAll: allowAllConfirmations,
  decline: declineConfirmation,
} = useAgentConfirmation()

const numberFormat = new Intl.NumberFormat('en-US', { notation: 'compact', maximumFractionDigits: 1 })
const contextUsageLabel = computed(() => {
  const { usedTokens, contextWindow } = contextUsage.value
  const used = usedTokens ? numberFormat.format(usedTokens) : '—'
  const percent = usedTokens ? ` (${Math.round((usedTokens / contextWindow) * 100)}%)` : ''
  return `${used} / ${numberFormat.format(contextWindow)}${percent}`
})
const contextUsageTitle = computed(() => {
  const { usedTokens, contextWindow } = contextUsage.value
  return `${usedTokens.toLocaleString('en-US')} / ${contextWindow.toLocaleString('en-US')} tokens`
})

const { render: renderMd } = useMarkdown()
const chatStatus = useAgentChatStatus()
const isShown = chatStatus.shown
const { close: closeChat, agentInput: input } = useOmniDialog()
const lastError = ref('')
const scrollEl = ref<HTMLElement | null>(null)
const followTranscript = ref(true)
const composing = ref(false)
const textareaRef = ref<HTMLTextAreaElement | null>(null)

watch(isShown, async (visible, wasVisible) => {
  if (!visible || wasVisible) return
  await agentSettings.refreshStatus().catch((error) => {
    lastError.value = error instanceof Error ? error.message : String(error)
  })
  await commonAgent.loadConversationForCurrentInstance()
  followTranscript.value = true
  await nextTick()
  scrollTranscriptToBottom()
  textareaRef.value?.focus()
})

const transcriptItems = computed(() => projectAgentTranscript(messages.value).filter(item => item.kind === 'message' || item.kind === 'tool'))

const liveStatus = computed(() => 'HyperXAi đang đọc & suy luận...')
const displayError = computed(() => lastError.value || runError.value)

watch(running, value => { chatStatus.running.value = value }, { immediate: true })

function messageText(content: string | ContentPart[] | null | undefined): string {
  if (!content) return ''
  if (typeof content === 'string') return content
  return content.map((p) => (p.type === 'text' ? p.text : '[image]')).join('')
}

function renderAssistant(content: string | ContentPart[] | null | undefined): string {
  return renderMd(messageText(content))
}

async function onSend() {
  const text = input.value.trim()
  if (!text || running.value || !available.value) return
  input.value = ''
  lastError.value = ''
  followTranscript.value = true
  try {
    await commonAgent.send(text)
  } catch (err) {
    lastError.value = err instanceof Error ? err.message : String(err)
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.preventDefault()
    closeChat()
    return
  }
  if (event.key === 'Enter' && !event.shiftKey && !composing.value) {
    event.preventDefault()
    void onSend()
  }
}

function startNewConversation() {
  commonAgent.reset()
  input.value = ''
  lastError.value = ''
}

function abort() {
  commonAgent.abort()
}

defineExpose({
  available,
  running,
  send: onSend,
  abort,
})

const { push } = useRouter()
function openSettings() {
  isShown.value = false
  push({ path: '/setting', query: { target: 'agent' } })
}

function onTranscriptScroll() {
  const el = scrollEl.value
  if (el) followTranscript.value = el.scrollHeight - el.scrollTop - el.clientHeight <= 32
}

function scrollTranscriptToBottom() {
  const el = scrollEl.value
  if (!el) return
  el.scrollTop = el.scrollHeight
  followTranscript.value = true
}

watch([transcriptItems, events, confirmationShown, displayError], async () => {
  if (!followTranscript.value) return
  await nextTick()
  if (followTranscript.value) scrollTranscriptToBottom()
}, { deep: true })
</script>

<style scoped>
.bubble {
  max-width: 90%;
  min-width: 0;
  padding: 10px 13px;
  border-radius: 10px;
  font-size: 12.5px;
  line-height: 1.45;
  white-space: pre-wrap;
  word-break: break-word;
  overflow-wrap: anywhere;
  box-shadow: 0 2px 6px rgb(0 0 0 / 0.05);
}
.bubble-user {
  background: #212529;
  color: #FFFFFF;
  margin-left: auto;
}
:global(.dark) .bubble-user,
:global(.theme--dark) .bubble-user {
  background: #2b303c;
  color: #FFFFFF;
  border: 1px solid rgba(255, 255, 255, 0.15);
}
.bubble-assistant {
  background: #F8F9FA;
  border: 1.2px solid #DEE2E6;
  color: #343A40;
  margin-right: auto;
}
:global(.dark) .bubble-assistant,
:global(.theme--dark) .bubble-assistant {
  background: #252830;
  border: 1.2px solid rgba(255, 255, 255, 0.12);
  color: #ECEFF4;
}
.md-content {
  white-space: normal;
}
.md-content :deep(> *:first-child) {
  margin-top: 0;
}
.md-content :deep(> *:last-child) {
  margin-bottom: 0;
}
.md-content :deep(p) {
  margin: 0 0 8px;
}
.agent-surface {
  display: flex;
  width: 370px;
  max-width: 95vw;
  height: 540px;
  max-height: 85vh;
  min-height: 480px;
  flex-direction: column;
  box-sizing: border-box;
  overflow: hidden;
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(20, 23, 31, 0.95);
  backdrop-filter: blur(24px) saturate(180%);
  box-shadow: 0 24px 60px rgba(0, 0, 0, 0.65), 0 0 0 1px rgba(255, 255, 255, 0.05);
}

.hyperxai-header {
  height: 58px;
  flex-shrink: 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.03);
}

.hyperxai-title {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 14px;
  font-weight: 700;
  color: #ffffff;
  letter-spacing: -0.01em;
}

.hyperxai-status {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 11px;
  font-weight: 500;
  color: #a1a1aa;
}

.agent-transcript {
  display: flex;
  width: 100%;
  flex: 1 1 auto;
  flex-direction: column;
  gap: 10px;
  box-sizing: border-box;
  overflow-x: hidden;
  overflow-y: auto;
  padding: 14px 16px;
  scrollbar-gutter: stable;
}

.agent-transcript::-webkit-scrollbar {
  width: 4px;
}
.agent-transcript::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 4px;
}
.agent-transcript::-webkit-scrollbar-track {
  background: transparent;
}

.hyperxai-empty-wrap {
  display: flex;
  min-height: 100%;
  align-items: center;
  justify-content: center;
  padding: 12px;
  text-align: center;
}

.hyperxai-welcome-card {
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.04);
  backdrop-filter: blur(12px);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.25);
  padding: 14px 16px;
  width: 100%;
}
.hyperxai-welcome-title {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 13px;
  font-weight: 400;
  color: #e4e4e7;
}
.hyperxai-welcome-name {
  font-weight: 700;
  color: #ffffff;
}
.hyperxai-welcome-sub {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 12.5px;
  font-weight: 400;
  color: #a1a1aa;
  margin-top: 4px;
}

.bubble {
  max-width: 90%;
  min-width: 0;
  padding: 10px 14px;
  border-radius: 12px;
  font-size: 12.5px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
  overflow-wrap: anywhere;
}
.bubble-user {
  background: rgba(255, 255, 255, 0.14);
  border: 1px solid rgba(255, 255, 255, 0.18);
  color: #ffffff;
  margin-left: auto;
}
.bubble-assistant {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.09);
  color: #f4f4f5;
  margin-right: auto;
}

/* Bottom Input Area */
.hyperxai-input-wrap {
  padding: 10px 14px 14px;
  flex-shrink: 0;
  background: rgba(16, 18, 24, 0.7);
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.hyperxai-input-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px 7px 14px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  background: rgba(0, 0, 0, 0.35);
  transition: all 0.2s ease;
}
.hyperxai-input-box:focus-within {
  border-color: rgba(255, 255, 255, 0.35);
  box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.06);
}

.hyperxai-input-field {
  flex: 1 1 auto;
  min-width: 0;
  max-height: 80px;
  border: none;
  outline: none;
  background: transparent;
  resize: none;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 12px;
  line-height: 1.45;
  color: #ffffff;
}
.hyperxai-input-field::placeholder {
  color: rgba(255, 255, 255, 0.38);
  font-size: 11.5px;
}

.hyperxai-send-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  min-width: 32px;
  border-radius: 8px;
  background: #ffffff;
  color: #181b22;
  cursor: pointer;
  border: none;
  transition: all 0.15s ease;
}
.hyperxai-send-btn:hover:not(:disabled) {
  opacity: 0.92;
  transform: scale(1.05);
}
.hyperxai-send-btn:disabled {
  opacity: 0.25;
  cursor: not-allowed;
}
.hyperxai-stop-btn {
  background: #ef4444 !important;
  color: #ffffff !important;
}
.agent-live-card {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 11px;
  color: rgba(var(--v-theme-on-surface), 0.7);
  font-size: 12px;
}
.agent-error {
  display: flex;
  max-width: 85%;
  min-width: 0;
  align-items: flex-start;
  gap: 9px;
  padding: 9px 13px;
  border-radius: 12px;
  border: 1px solid rgba(var(--v-theme-error), 0.5);
  background: rgba(var(--v-theme-error), 0.12);
}
.agent-error__icon {
  margin-top: 1px;
  flex-shrink: 0;
}
.agent-error__title {
  color: rgb(var(--v-theme-error));
  font-size: 13px;
  font-weight: 600;
  line-height: 1.45;
}
.agent-error__body {
  margin-top: 2px;
  color: rgba(var(--v-theme-on-surface), 0.8);
  font-size: 12.5px;
  line-height: 1.45;
  white-space: pre-wrap;
  word-break: break-word;
  overflow-wrap: anywhere;
}
.agent-confirm {
  display: flex;
  width: 90%;
  max-width: 640px;
  max-height: calc(100vh - 16px);
  min-width: 0;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid rgba(var(--v-theme-on-surface), 0.14);
  border-radius: 12px;
  background: rgb(var(--v-theme-surface));
  box-shadow: 0 4px 14px rgb(0 0 0 / 0.22);
}
.agent-confirm__header {
  display: grid;
  grid-template-columns: 32px minmax(0, 1fr);
  align-items: start;
  gap: 9px;
  flex-shrink: 0;
  padding: 12px 13px 9px;
}
.agent-confirm__icon {
  display: flex;
  width: 32px;
  height: 32px;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  color: rgb(var(--v-theme-warning));
  background: rgba(var(--v-theme-warning), 0.13);
}
.agent-confirm__icon--destructive {
  color: rgb(var(--v-theme-error));
  background: rgba(var(--v-theme-error), 0.13);
}
.agent-confirm__title {
  font-size: 13px;
  font-weight: 600;
  line-height: 1.4;
}
.agent-confirm__message {
  margin-top: 2px;
  color: rgba(var(--v-theme-on-surface), 0.7);
  font-size: 12.5px;
  line-height: 1.45;
}
.agent-confirm__details {
  display: flex;
  max-height: 160px;
  min-height: 0;
  flex-direction: column;
  gap: 4px;
  margin: 0 13px 9px 54px;
  overflow: auto;
}
.agent-confirm__detail {
  display: flex;
  min-height: 32px;
  align-items: flex-start;
  gap: 7px;
  padding: 7px 9px;
  border-radius: 6px;
  color: rgba(var(--v-theme-on-surface), 0.72);
  background: rgba(var(--v-theme-on-surface), 0.055);
}
.agent-confirm__detail code {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  line-height: 1.45;
  overflow-wrap: anywhere;
}
.agent-confirm__presentations {
  display: flex;
  max-height: 280px;
  min-height: 0;
  flex-direction: column;
  gap: 8px;
  margin: 0 13px 9px;
  overflow: auto;
}
.agent-confirm__actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
  flex-shrink: 0;
  padding: 8px 10px;
  border-top: 1px solid rgba(var(--v-theme-on-surface), 0.1);
  background: rgba(var(--v-theme-on-surface), 0.025);
}
@media (max-width: 700px) {
  .agent-mode-controls {
    gap: 4px;
  }
  .agent-composer-status {
    display: none;
  }
  .agent-confirm {
    width: 95%;
  }
  .agent-confirm__details {
    margin-left: 13px;
  }
  .agent-surface {
    width: 100%;
    max-width: 100%;
  }
}
</style>
