<template>
  <div
    class="log-file-item rounded-xl p-3 mb-2 bg-[#141824]/80 hover:bg-[#1a2133] border border-white/5 hover:border-cyan-500/30 transition-all flex items-center justify-between cursor-pointer group select-none"
    :class="{ 'opacity-50 pointer-events-none': disabled }"
    @click="openFile(source.name)"
  >
    <div class="flex items-center gap-3 min-w-0">
      <!-- Icon based on file type -->
      <div
        class="w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0 transition-transform group-hover:scale-105"
        :class="iconBgClass"
      >
        <v-icon size="20" :color="iconColor">{{ fileIcon }}</v-icon>
      </div>

      <div class="flex flex-col min-w-0">
        <div class="flex items-center gap-2">
          <span class="font-mono text-xs font-semibold text-zinc-200 group-hover:text-cyan-300 transition-colors truncate">
            {{ source.name }}
          </span>
          <span
            v-if="isLatest"
            class="px-1.5 py-0.5 rounded text-[10px] font-bold font-mono bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 uppercase"
          >
            Mới nhất
          </span>
          <span
            v-else-if="isCrash"
            class="px-1.5 py-0.5 rounded text-[10px] font-bold font-mono bg-rose-500/20 text-rose-300 border border-rose-500/40 uppercase"
          >
            Sự cố
          </span>
        </div>
        <span class="text-[11px] text-zinc-500 font-mono">Nhấn để xem chi tiết nội dung</span>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex items-center gap-1 opacity-70 group-hover:opacity-100 transition-opacity">
      <v-btn
        v-shared-tooltip="() => t('logsCrashes.showFile')"
        icon
        variant="text"
        size="small"
        class="text-zinc-400 hover:text-white"
        @click.prevent.stop="showFile(source.name)"
      >
        <v-icon size="18">folder_open</v-icon>
      </v-btn>
      <v-btn
        v-shared-tooltip="() => t('shared.delete')"
        icon
        color="error"
        variant="text"
        size="small"
        class="text-zinc-500 hover:text-rose-400"
        @click.prevent.stop="removeFile(source.name)"
      >
        <v-icon size="18">delete</v-icon>
      </v-btn>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { vSharedTooltip } from '@/directives/sharedTooltip'
import { computed } from 'vue'

const { t } = useI18n()

const props = defineProps<{
  source: { name: string; id: string }
  disabled: boolean
  removeFile(file: string): Promise<void>
  showFile(file: string): void
  openFile(file: string): void
}>()

const isLatest = computed(() => props.source.name === 'latest.log')
const isCrash = computed(() => props.source.name.startsWith('crash-') || props.source.name.includes('crash'))
const isGzip = computed(() => props.source.name.endsWith('.gz'))

const fileIcon = computed(() => {
  if (isLatest.value) return 'terminal'
  if (isCrash.value) return 'bug_report'
  if (isGzip.value) return 'archive'
  return 'description'
})

const iconBgClass = computed(() => {
  if (isLatest.value) return 'bg-emerald-500/15 border border-emerald-500/30'
  if (isCrash.value) return 'bg-rose-500/15 border border-rose-500/30'
  if (isGzip.value) return 'bg-amber-500/15 border border-amber-500/30'
  return 'bg-zinc-800 border border-white/5'
})

const iconColor = computed(() => {
  if (isLatest.value) return 'emerald-accent-3'
  if (isCrash.value) return 'error'
  if (isGzip.value) return 'amber'
  return undefined
})
</script>

<style scoped>
</style>
