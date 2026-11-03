<template>
  <div class="min-h-[460px] flex flex-col">
    <Transition name="fade-transition" mode="out-in">
      <!-- File List View -->
      <div v-if="content === '' && files.length !== 0" :key="0" class="flex flex-col h-full">
        <v-virtual-scroll
          class="visible-scroll h-full max-h-[70vh] overflow-auto select-none pr-1"
          :bench="10"
          :items="files.map((name, index) => ({ name, id: `${name}-${index}` }))"
          :item-height="68"
        >
          <template #default="{ item }">
            <HomeLogDialogTabItem
              :source="item"
              :open-file="openFile"
              :remove-file="removeFile"
              :show-file="showFile"
              :disabled="pending"
            />
          </template>
        </v-virtual-scroll>
      </div>

      <!-- Empty State -->
      <div v-else-if="content === '' && files.length === 0" :key="2" class="h-[420px] flex items-center justify-center">
        <div class="flex flex-col items-center justify-center gap-3 text-zinc-500 select-none">
          <v-progress-circular v-if="pending" :size="48" color="cyan" indeterminate />
          <template v-else>
            <div class="w-16 h-16 rounded-2xl bg-white/5 border border-white/10 flex items-center justify-center text-zinc-400">
              <v-icon size="32">{{ log ? 'subtitles' : 'bug_report' }}</v-icon>
            </div>
            <div class="text-sm font-medium text-zinc-400">
              {{ t('logsCrashes.placeholder') || 'Chưa có bản ghi nào trong mục này.' }}
            </div>
            <div class="text-xs text-zinc-500 font-mono">Các tệp nhật ký mới sẽ xuất hiện ở đây sau khi khởi động game.</div>
          </template>
        </div>
      </div>

      <!-- File Content Viewer -->
      <div
        v-else
        :key="1"
        class="log-viewer flex flex-col overflow-hidden h-full flex-grow"
      >
        <!-- Header Bar -->
        <div class="log-viewer__header flex items-center justify-between gap-3 px-4 py-2.5 rounded-xl bg-[#141824] border border-white/10 mb-3 select-none">
          <div class="flex items-center gap-2 min-w-0">
            <v-btn
              v-shared-tooltip="() => t('shared.back')"
              icon="arrow_back"
              variant="text"
              size="small"
              class="text-zinc-400 hover:text-white"
              @click="goBack"
            />
            <div
              class="w-7 h-7 rounded-lg flex items-center justify-center text-xs"
              :class="log ? 'bg-cyan-500/20 text-cyan-300' : 'bg-rose-500/20 text-rose-300'"
            >
              <v-icon size="16">{{ log ? 'description' : 'bug_report' }}</v-icon>
            </div>
            <span class="font-mono text-xs font-bold text-zinc-100 truncate">{{ showedFile }}</span>
            <span class="text-[11px] font-mono text-zinc-500 hidden sm:inline">
              ({{ lineCount.toLocaleString() }} dòng)
            </span>
          </div>

          <div class="flex items-center gap-1.5">
            <!-- Open File in Explorer -->
            <v-btn
              v-shared-tooltip="() => 'Mở trong thư mục'"
              icon="folder_open"
              variant="text"
              size="small"
              class="text-zinc-400 hover:text-white"
              @click="showFile(showedFile)"
            />

            <!-- Copy Content Button -->
            <v-btn
              v-shared-tooltip="() => copied ? t('shared.copied') : t('shared.copy')"
              :icon="copied ? 'check' : 'content_copy'"
              variant="tonal"
              size="small"
              :color="copied ? 'success' : undefined"
              class="text-xs font-mono"
              @click="copyContent"
            >
              <template #prepend>
                <v-icon size="14">{{ copied ? 'check' : 'content_copy' }}</v-icon>
              </template>
              <span>{{ copied ? 'Đã sao chép' : 'Sao chép' }}</span>
            </v-btn>
          </div>
        </div>

        <!-- Normal Log View -->
        <LogView v-if="log" class="h-[62vh] overflow-hidden" :logs="logs" />

        <!-- Crash Report View (Modern Inspector with Offline Smart Diagnosis, ZERO AI) -->
        <div v-else class="flex flex-col gap-3 h-[62vh] overflow-hidden">
          <!-- Offline Smart Diagnosis Card (if detected) -->
          <div
            v-if="diagnosis"
            class="surface-rounded-item p-3.5 bg-rose-950/40 border border-rose-500/40 rounded-xl flex flex-col gap-2 flex-shrink-0"
          >
            <div class="flex items-center gap-2">
              <v-icon color="error" size="20">troubleshoot</v-icon>
              <span class="font-bold text-sm text-rose-300">{{ diagnosis.title }}</span>
              <span
                v-if="diagnosis.highlight"
                class="px-1.5 py-0.5 rounded text-[10px] font-mono font-bold bg-rose-500/20 text-rose-200 border border-rose-500/40"
              >
                {{ diagnosis.highlight }}
              </span>
            </div>
            <div class="text-xs text-zinc-300 leading-relaxed">
              {{ diagnosis.description }}
            </div>
            <div class="text-xs pt-2 border-t border-rose-500/20 text-amber-300 flex items-start gap-1.5 font-medium">
              <v-icon size="16" class="mt-0.5 text-amber-400">lightbulb</v-icon>
              <span><strong>Cách khắc phục:</strong> {{ diagnosis.solution }}</span>
            </div>
          </div>

          <!-- Crash Report Text Viewer -->
          <div class="flex-grow min-h-0 bg-[#0c0e14] border border-white/10 rounded-xl overflow-auto p-3 font-mono text-xs leading-relaxed visible-scroll select-text">
            <div
              v-for="(line, idx) in crashLines"
              :key="idx"
              class="flex items-start py-0.5 hover:bg-white/[0.04] transition-colors"
            >
              <span class="select-none text-zinc-600 font-mono text-[11px] text-right pr-3 min-w-[48px] flex-shrink-0">
                {{ idx + 1 }}
              </span>
              <span :class="crashLineClass(line)" class="break-words whitespace-pre-wrap select-text">
                {{ line }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script lang="ts" setup>
import { parseLog } from '@/util/log'
import LogView from '@/components/LogView.vue'
import HomeLogDialogTabItem from './HomeLogDialogTabItem.vue'
import { diagnoseCrash } from '@/util/crashDiagnosis'
import { vSharedTooltip } from '@/directives/sharedTooltip'
import { ref, computed, watch } from 'vue'

const props = defineProps<{
  files: string[]
  log?: boolean
  getFileContent(file: string): Promise<string>
  getLaunchId?(file: string): Promise<string | undefined>
  removeFile(file: string): Promise<void>
  showFile(file: string): void
  visible: boolean
  refreshing: boolean
}>()

const { t } = useI18n()
const content = ref('')
const loading = ref(false)
const showedFile = ref('')
const launchId = ref('')

const goBack = () => {
  content.value = ''
  showedFile.value = ''
  launchId.value = ''
}

const openFile = async (name: string) => {
  loading.value = true
  showedFile.value = name
  const [nextContent, nextLaunchId] = await Promise.all([
    props.getFileContent(name),
    props.getLaunchId?.(name).catch(() => undefined) ?? Promise.resolve(undefined),
  ]).finally(() => {
    loading.value = false
  })
  content.value = nextContent
  launchId.value = nextLaunchId ?? ''
}

const pending = computed(() => props.refreshing || loading.value)

const lineCount = computed(() => {
  if (!content.value) return 0
  return content.value.split('\n').length
})

const logs = computed(() => {
  const lines = content.value.split(/[\r?\n]/g)
  const logLines = [] as string[]
  for (const line of lines) {
    if (line.startsWith('[')) {
      logLines.push(line)
    } else {
      if (logLines.length > 0) {
        logLines[logLines.length - 1] += '\n' + line
      } else {
        logLines.push(line)
      }
    }
  }
  return logLines.map(parseLog)
})

const crashLines = computed(() => {
  if (!content.value) return []
  return content.value.split(/\r?\n/)
})

const diagnosis = computed(() => {
  if (props.log || !content.value) return undefined
  return diagnoseCrash(content.value)
})

function crashLineClass(line: string) {
  const trimmed = line.trim()
  if (trimmed.startsWith('at ')) return 'text-purple-300/80 italic'
  if (trimmed.startsWith('Caused by:')) return 'text-rose-400 font-bold'
  if (trimmed.includes('Exception:') || trimmed.includes('Error:')) return 'text-rose-300 font-semibold'
  if (trimmed.startsWith('//') || trimmed.startsWith('--')) return 'text-zinc-500 font-bold'
  if (trimmed.startsWith('Time:') || trimmed.startsWith('Description:')) return 'text-amber-300 font-medium'
  return 'text-zinc-300'
}

const copied = ref(false)
function copyContent() {
  windowController.writeClipboard(content.value)
  copied.value = true
  setTimeout(() => {
    copied.value = false
  }, 2000)
}

watch(
  () => props.visible,
  (v) => {
    if (!v) {
      goBack()
    }
  },
)
</script>

<style scoped>
.visible-scroll::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}
.visible-scroll::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.2);
}
.visible-scroll::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 4px;
}
.visible-scroll::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.25);
}
</style>
