<template>
  <div class="log-view-container flex flex-col h-full bg-[#0c0e14] text-zinc-200 rounded-xl overflow-hidden border border-white/10 shadow-inner">
    <!-- Modern Header Toolbar -->
    <div class="log-toolbar flex flex-wrap items-center justify-between gap-2 px-3 py-2 bg-[#12151f] border-b border-white/10 flex-shrink-0 select-none">
      <!-- Left: Search and Result Counter -->
      <div class="flex items-center gap-2 flex-grow max-w-md">
        <div class="relative w-full">
          <input
            v-model="searchText"
            type="text"
            :placeholder="t('logView.searchPlaceholder') || 'Tìm kiếm trong nhật ký...'"
            class="w-full bg-[#181c2a] text-xs text-zinc-100 placeholder-zinc-500 rounded-lg pl-8 pr-8 py-1.5 border border-white/10 focus:border-cyan-500/60 focus:outline-none transition-colors font-mono"
          />
          <v-icon size="16" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-zinc-400 pointer-events-none">
            search
          </v-icon>
          <button
            v-if="searchText"
            type="button"
            class="absolute right-2 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-white transition-colors"
            @click="searchText = ''"
          >
            <v-icon size="14">close</v-icon>
          </button>
        </div>

        <span class="text-[11px] font-mono text-zinc-400 whitespace-nowrap px-1">
          {{ displayLogs.length }} / {{ logs.length }}
        </span>
      </div>

      <!-- Center: Filter Chips (All, Info, Warn, Error) -->
      <div class="flex items-center gap-1.5">
        <button
          type="button"
          class="filter-chip text-[11px] px-2.5 py-1 rounded-md font-mono transition-all flex items-center gap-1.5"
          :class="activeLevel === 'all'
            ? 'bg-zinc-700 text-white font-semibold shadow-sm'
            : 'bg-zinc-800/60 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200'"
          @click="activeLevel = 'all'"
        >
          <span>Tất cả</span>
          <span class="text-[10px] opacity-75">({{ counts.all }})</span>
        </button>

        <button
          type="button"
          class="filter-chip text-[11px] px-2.5 py-1 rounded-md font-mono transition-all flex items-center gap-1.5"
          :class="activeLevel === 'info'
            ? 'bg-emerald-600/30 text-emerald-300 border border-emerald-500/50 font-semibold'
            : 'bg-zinc-800/60 text-emerald-400/80 hover:bg-zinc-800 hover:text-emerald-300'"
          @click="activeLevel = 'info'"
        >
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 inline-block" />
          <span>INFO</span>
          <span class="text-[10px] opacity-75">({{ counts.info }})</span>
        </button>

        <button
          type="button"
          class="filter-chip text-[11px] px-2.5 py-1 rounded-md font-mono transition-all flex items-center gap-1.5"
          :class="activeLevel === 'warn'
            ? 'bg-amber-600/30 text-amber-300 border border-amber-500/50 font-semibold'
            : 'bg-zinc-800/60 text-amber-400/80 hover:bg-zinc-800 hover:text-amber-300'"
          @click="activeLevel = 'warn'"
        >
          <span class="w-1.5 h-1.5 rounded-full bg-amber-400 inline-block" />
          <span>WARN</span>
          <span class="text-[10px] opacity-75">({{ counts.warn }})</span>
        </button>

        <button
          type="button"
          class="filter-chip text-[11px] px-2.5 py-1 rounded-md font-mono transition-all flex items-center gap-1.5"
          :class="activeLevel === 'error'
            ? 'bg-rose-600/30 text-rose-300 border border-rose-500/50 font-semibold'
            : 'bg-zinc-800/60 text-rose-400/80 hover:bg-zinc-800 hover:text-rose-300'"
          @click="activeLevel = 'error'"
        >
          <span class="w-1.5 h-1.5 rounded-full bg-rose-400 inline-block" />
          <span>ERROR</span>
          <span class="text-[10px] opacity-75">({{ counts.error }})</span>
        </button>
      </div>

      <!-- Right: Action Controls -->
      <div class="flex items-center gap-1">
        <!-- Toggle Wrap Lines -->
        <button
          type="button"
          class="action-btn text-xs px-2 py-1 rounded-md flex items-center gap-1 font-mono transition-colors"
          :class="wrapLines ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40' : 'text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200'"
          :title="wrapLines ? 'Tắt ngắt dòng (Chế độ cuộn ngang)' : 'Bật tự động ngắt dòng'"
          @click="wrapLines = !wrapLines"
        >
          <v-icon size="14">wrap_text</v-icon>
          <span class="hidden sm:inline">Wrap</span>
        </button>

        <!-- Toggle Auto Scroll Lock -->
        <button
          type="button"
          class="action-btn text-xs px-2 py-1 rounded-md flex items-center gap-1 font-mono transition-colors"
          :class="locked ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40' : 'text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200'"
          :title="locked ? 'Đang tự động bám theo nhật ký mới' : 'Tự động cuộn đang tắt'"
          @click="toggleLock"
        >
          <v-icon size="14">{{ locked ? 'vertical_align_bottom' : 'lock_open' }}</v-icon>
          <span class="hidden sm:inline">{{ locked ? 'Bám đuôi' : 'Tự do' }}</span>
        </button>

        <!-- Copy All Button -->
        <button
          type="button"
          class="action-btn text-xs px-2 py-1 rounded-md flex items-center gap-1 font-mono text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200 transition-colors"
          :class="{ 'text-emerald-400': copied }"
          :title="'Sao chép toàn bộ nhật ký đang hiển thị'"
          @click="copyContent"
        >
          <v-icon size="14">{{ copied ? 'check' : 'content_copy' }}</v-icon>
          <span class="hidden sm:inline">{{ copied ? 'Đã chép' : 'Sao chép' }}</span>
        </button>
      </div>
    </div>

    <!-- Virtualized Log Rows Container -->
    <div
      ref="scroller"
      class="visible-scroll flex-grow overflow-auto relative font-mono text-xs select-text"
      @wheel="onWheel"
    >
      <div
        v-if="displayLogs.length === 0"
        class="flex flex-col items-center justify-center h-full min-h-[300px] text-zinc-500 gap-3"
      >
        <v-icon size="36" class="opacity-40">find_in_page</v-icon>
        <div class="text-sm">Không tìm thấy dòng nhật ký nào phù hợp.</div>
        <button
          v-if="searchText || activeLevel !== 'all'"
          type="button"
          class="text-xs text-cyan-400 hover:underline flex items-center gap-1"
          @click="resetFilter"
        >
          <v-icon size="14">refresh</v-icon>
          <span>Đặt lại bộ lọc tìm kiếm</span>
        </button>
      </div>

      <div
        v-else
        ref="container"
        :style="{
          height: `${totalHeight}px`,
          position: 'relative',
          width: '100%',
          marginTop: `${-offsetTop}px`,
        }"
      >
        <div
          v-for="virtualRow in virtualRows"
          :key="displayLogs[virtualRow.index].raw + virtualRow.index"
          :ref="measureElement"
          :data-index="virtualRow.index"
          class="log-row-wrapper"
          :style="{
            position: 'absolute',
            top: 0,
            left: 0,
            width: '100%',
            transform: `translateY(${virtualRow.start}px)`
          }"
        >
          <div
            :class="[
              rowBorderClass(displayLogs[virtualRow.index].level),
              rowBgClass(displayLogs[virtualRow.index].level)
            ]"
            class="log-record flex items-start py-0.5 px-3 transition-colors hover:bg-white/[0.04] border-l-2"
          >
            <!-- Line Number -->
            <span class="log-line-num select-none flex-shrink-0 text-zinc-600 font-mono text-[11px] text-right pr-3 min-w-[44px]">
              {{ virtualRow.index + 1 }}
            </span>

            <!-- Content Area -->
            <div class="flex-grow min-w-0" :class="wrapLines ? 'break-words whitespace-pre-wrap' : 'whitespace-pre'">
              <!-- Metadata Badges -->
              <span class="inline-flex items-center gap-1.5 select-none mr-2">
                <!-- Timestamp -->
                <span v-if="displayLogs[virtualRow.index].date" class="text-zinc-500 text-[11px]">
                  {{ displayLogs[virtualRow.index].date }}
                </span>

                <!-- Level Badge -->
                <span
                  class="log-level-badge text-[10px] font-bold uppercase px-1.5 py-[1px] rounded tracking-wider"
                  :class="levelBadgeClasses(displayLogs[virtualRow.index].level)"
                >
                  {{ formatLevel(displayLogs[virtualRow.index].level) }}
                </span>

                <!-- Source / Thread -->
                <span v-if="displayLogs[virtualRow.index].source" class="text-sky-400/80 text-[11px] font-medium">
                  [{{ displayLogs[virtualRow.index].source }}]
                </span>

                <!-- Group Count Multiplier -->
                <span
                  v-if="displayLogs[virtualRow.index].groupCount && displayLogs[virtualRow.index].groupCount! > 1"
                  class="bg-purple-500/20 text-purple-300 border border-purple-500/30 text-[10px] font-bold px-1 rounded"
                >
                  ×{{ displayLogs[virtualRow.index].groupCount }}
                </span>
              </span>

              <!-- Log Body Text -->
              <span
                class="log-content text-[12px] leading-relaxed select-text"
                :class="contentColor(displayLogs[virtualRow.index])"
              >{{ displayLogs[virtualRow.index].content }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Floating Jump-to-Bottom Button -->
      <transition name="fade-transition">
        <button
          v-if="!locked && displayLogs.length > 0"
          type="button"
          class="absolute bottom-4 right-6 z-20 flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-cyan-600 hover:bg-cyan-500 text-white font-mono text-xs shadow-lg shadow-cyan-900/40 transition-transform active:scale-95"
          @click="scrollToBottom"
        >
          <v-icon size="14">arrow_downward</v-icon>
          <span>Cuộn xuống cuối</span>
        </button>
      </transition>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { LogRecord } from '../util/log'
import { useDebounceFn, useScroll } from '@vueuse/core'
import { VirtualItem, VirtualizerOptions, useVirtualizer } from '@tanstack/vue-virtual'
import { ref, computed, watch, nextTick } from 'vue'

interface DisplayLogRecord extends LogRecord {
  groupCount?: number
  _contentParts?: string[]
  _rawParts?: string[]
}

const props = defineProps<{
  logs: LogRecord[]
}>()

const { t } = useI18n()

const container = ref<HTMLElement>()
const offsetTop = ref(0)
const scroller = ref<HTMLElement>()
const searchText = ref('')
const activeLevel = ref<'all' | 'info' | 'warn' | 'error'>('all')
const wrapLines = ref(false)
const locked = ref(true)
const copied = ref(false)

// Level counts
const counts = computed(() => {
  let info = 0
  let warn = 0
  let error = 0
  for (const log of props.logs) {
    const lvl = (log.level || '').toLowerCase()
    if (lvl.includes('err') || lvl.includes('fatal')) {
      error++
    } else if (lvl.includes('warn')) {
      warn++
    } else {
      info++
    }
  }
  return { all: props.logs.length, info, warn, error }
})

// Debounced search text
const debouncedSearchText = ref('')
const updateDebouncedSearch = useDebounceFn((val: string) => {
  debouncedSearchText.value = val
}, 150)
watch(searchText, updateDebouncedSearch)

// Filter by level first
const levelFilteredLogs = computed(() => {
  if (activeLevel.value === 'all') return props.logs
  return props.logs.filter((log) => {
    const lvl = (log.level || '').toLowerCase()
    if (activeLevel.value === 'error') return lvl.includes('err') || lvl.includes('fatal')
    if (activeLevel.value === 'warn') return lvl.includes('warn')
    if (activeLevel.value === 'info') return !lvl.includes('err') && !lvl.includes('fatal') && !lvl.includes('warn')
    return true
  })
})

// Search filter
const searchedLogs = computed(() => {
  const base = levelFilteredLogs.value
  const query = debouncedSearchText.value?.trim().toLowerCase()
  if (!query) return base
  return base.filter((log) =>
    (log.content && log.content.toLowerCase().includes(query)) ||
    (log.source && log.source.toLowerCase().includes(query)) ||
    (log.level && log.level.toLowerCase().includes(query)) ||
    (log.raw && log.raw.toLowerCase().includes(query))
  )
})

// Group consecutive identical logs
const displayLogs = computed<DisplayLogRecord[]>(() => {
  const logs = searchedLogs.value
  if (logs.length === 0) return []

  const result: DisplayLogRecord[] = []
  let currentGroup: DisplayLogRecord | null = null

  for (const log of logs) {
    if (
      currentGroup &&
      currentGroup.level === log.level &&
      currentGroup.date === log.date &&
      currentGroup.source === log.source
    ) {
      currentGroup.groupCount = (currentGroup.groupCount || 1) + 1
      if (!currentGroup._contentParts) {
        currentGroup._contentParts = [currentGroup.content]
        currentGroup._rawParts = [currentGroup.raw]
      }
      currentGroup._contentParts.push(log.content)
      currentGroup._rawParts!.push(log.raw)
    } else {
      if (currentGroup && currentGroup._contentParts) {
        currentGroup.content = currentGroup._contentParts.join('\n')
        currentGroup.raw = currentGroup._rawParts!.join('\n')
      }
      currentGroup = { ...log, groupCount: 1 }
      result.push(currentGroup)
    }
  }

  if (currentGroup && currentGroup._contentParts) {
    currentGroup.content = currentGroup._contentParts.join('\n')
    currentGroup.raw = currentGroup._rawParts!.join('\n')
  }

  return result
})

watch(container, (c) => {
  if (c) {
    nextTick().then(() => { offsetTop.value = c.offsetTop })
  }
})

const virtualizerOptions = computed(() => ({
  count: displayLogs.value.length,
  getScrollElement: () => scroller.value || null,
  estimateSize: () => 36,
  overscan: 10,
  paddingStart: offsetTop.value,
} satisfies Partial<VirtualizerOptions<HTMLElement, HTMLElement>>))

const totalHeight = computed(() => virtualizer.value.getTotalSize())
const virtualRows = computed(() => virtualizer.value.getVirtualItems() as (Omit<VirtualItem, 'key'> & { key: number })[])

const measureElement = (el: any) => {
  if (!el) return
  virtualizer.value.measureElement(el)
}

const { arrivedState } = useScroll(scroller)
const virtualizer = useVirtualizer(virtualizerOptions)

watch(computed(() => arrivedState.bottom), (bottom) => {
  if (bottom) {
    locked.value = true
  }
})

watch(() => displayLogs.value.length, () => {
  if (locked.value) {
    nextTick(() => {
      scrollToBottom()
    })
  }
})

watch(totalHeight, () => {
  if (locked.value) {
    nextTick(() => {
      scrollToBottom()
    })
  }
})

function onWheel(e: WheelEvent) {
  if (e.deltaY < 0) {
    locked.value = false
  }
}

function scrollToBottom() {
  locked.value = true
  if (displayLogs.value.length > 0) {
    virtualizer.value.scrollToIndex(displayLogs.value.length - 1, { align: 'end' })
  }
}

function toggleLock() {
  locked.value = !locked.value
  if (locked.value) {
    scrollToBottom()
  }
}

function resetFilter() {
  searchText.value = ''
  debouncedSearchText.value = ''
  activeLevel.value = 'all'
}

function copyContent() {
  const text = displayLogs.value.map((l) => l.raw || l.content).join('\n')
  windowController.writeClipboard(text)
  copied.value = true
  setTimeout(() => {
    copied.value = false
  }, 2000)
}

function formatLevel(lvl: string) {
  if (!lvl) return 'LOG'
  const upper = lvl.toUpperCase()
  if (upper.includes('ERR')) return 'ERROR'
  if (upper.includes('WARN')) return 'WARN'
  if (upper.includes('INFO')) return 'INFO'
  return upper
}

function levelBadgeClasses(lvl: string) {
  const l = (lvl || '').toLowerCase()
  if (l.includes('err') || l.includes('fatal')) {
    return 'bg-rose-500/20 text-rose-300 border border-rose-500/40'
  }
  if (l.includes('warn')) {
    return 'bg-amber-500/20 text-amber-300 border border-amber-500/40'
  }
  if (l.includes('info')) {
    return 'bg-emerald-500/15 text-emerald-300 border border-emerald-500/30'
  }
  return 'bg-zinc-700/40 text-zinc-300 border border-zinc-600/40'
}

function rowBorderClass(lvl: string) {
  const l = (lvl || '').toLowerCase()
  if (l.includes('err') || l.includes('fatal')) return 'border-rose-500'
  if (l.includes('warn')) return 'border-amber-500'
  if (l.includes('info')) return 'border-emerald-500/60'
  return 'border-transparent'
}

function rowBgClass(lvl: string) {
  const l = (lvl || '').toLowerCase()
  if (l.includes('err') || l.includes('fatal')) return 'bg-rose-950/15'
  if (l.includes('warn')) return 'bg-amber-950/10'
  return ''
}

function contentColor(record: DisplayLogRecord) {
  const l = (record.level || '').toLowerCase()
  if (l.includes('err') || l.includes('fatal')) return 'text-rose-200'
  if (l.includes('warn')) return 'text-amber-200'
  if (record.content && (record.content.includes('\tat ') || record.content.startsWith('at '))) {
    return 'text-purple-300/80 italic'
  }
  return 'text-zinc-300'
}
</script>

<style scoped>
.log-view-container {
  min-height: 250px;
}
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