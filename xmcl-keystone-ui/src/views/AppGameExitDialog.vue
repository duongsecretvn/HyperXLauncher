<template>
  <v-dialog
    v-model="data.isShown"
    :width="960"
  >
    <v-card class="bg-[#12151f]/95 backdrop-blur-xl border border-rose-500/30 rounded-2xl overflow-hidden shadow-2xl flex flex-col max-h-[85vh]">
      <!-- Header Toolbar -->
      <div class="flex items-center justify-between px-5 py-3 border-b border-rose-500/20 bg-gradient-to-r from-rose-950 via-red-950 to-zinc-950 select-none">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl bg-rose-500/20 border border-rose-500/40 flex items-center justify-center text-rose-400">
            <v-icon size="20">error</v-icon>
          </div>
          <div>
            <div class="font-bold text-base text-rose-100 flex items-center gap-2">
              <span>{{ data.isCrash ? t('launchFailed.crash') : t('launchFailed.title') }}</span>
              <span class="text-[10px] px-2 py-0.5 rounded-full bg-rose-500/20 text-rose-300 font-mono font-bold uppercase">
                {{ data.isCrash ? 'Game Crash' : 'Launch Exit' }}
              </span>
            </div>
            <div class="text-xs text-zinc-400">Trò chơi đã dừng lại không mong muốn</div>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <!-- Open Log / Crash Folder -->
          <v-btn
            v-if="!data.launcherError"
            variant="tonal"
            size="small"
            class="text-xs font-mono"
            prepend-icon="folder_open"
            @click="openFolder"
          >
            {{ data.isCrash ? t('instance.openCrashReportFolder') : t('instance.openLogFolder') }}
          </v-btn>

          <!-- Copy All Logs -->
          <v-btn
            variant="tonal"
            size="small"
            class="text-xs font-mono"
            :color="copied ? 'success' : undefined"
            :prepend-icon="copied ? 'check' : 'content_copy'"
            @click="copyAllLogs"
          >
            {{ copied ? 'Đã chép' : 'Sao chép nhật ký' }}
          </v-btn>

          <!-- Close -->
          <v-btn
            icon
            variant="text"
            size="small"
            @click="data.isShown = false"
          >
            <v-icon size="18">close</v-icon>
          </v-btn>
        </div>
      </div>

      <!-- Dialog Body -->
      <div class="p-5 flex flex-col gap-4 overflow-auto visible-scroll">
        <!-- Ely.by Warning -->
        <div
          v-if="data.elyByWarning"
          class="surface-rounded-item bg-amber-950/40 p-4 border border-amber-500/40 rounded-xl"
        >
          <div class="flex items-center gap-2 mb-1.5">
            <v-icon color="warning" size="20">warning</v-icon>
            <span class="font-bold text-sm text-amber-300">{{ t('launchFailed.elyByAuthlib.title') }}</span>
          </div>
          <div class="text-xs text-zinc-300 leading-relaxed">
            {{ t('launchFailed.elyByAuthlib.description', { version: data.elyByMinecraftVersion }) }}
          </div>
          <div class="text-xs mt-2 text-amber-400 font-medium">
            {{ t('launchFailed.elyByAuthlib.suggestion') }}
          </div>
        </div>

        <!-- Smart Offline Crash Diagnosis Banner (Zero AI) -->
        <div
          v-if="diagnosis"
          class="surface-rounded-item p-4 bg-rose-950/40 border border-rose-500/50 rounded-xl flex flex-col gap-2"
        >
          <div class="flex items-center gap-2">
            <v-icon color="error" size="22">troubleshoot</v-icon>
            <span class="font-bold text-base text-rose-200">{{ diagnosis.title }}</span>
            <span
              v-if="diagnosis.highlight"
              class="px-2 py-0.5 rounded text-[11px] font-mono font-bold bg-rose-500/20 text-rose-200 border border-rose-500/40"
            >
              {{ diagnosis.highlight }}
            </span>
          </div>
          <div class="text-xs text-zinc-300 leading-relaxed">
            {{ diagnosis.description }}
          </div>
          <div class="text-xs pt-2.5 mt-1 border-t border-rose-500/20 text-amber-300 flex items-start gap-1.5 font-medium">
            <v-icon size="16" class="mt-0.5 text-amber-400">lightbulb</v-icon>
            <span><strong>Cách khắc phục:</strong> {{ diagnosis.solution }}</span>
          </div>
        </div>

        <!-- Error Log Section -->
        <div v-if="data.errorLog" class="flex flex-col gap-2">
          <div class="flex items-center justify-between text-xs font-mono text-zinc-400">
            <span class="font-bold text-rose-300 flex items-center gap-1.5">
              <v-icon size="14" color="error">pest_control</v-icon>
              <span>{{ data.launcherError ? t('launchFailed.failedToLaunch') : data.isCrash ? 'Chi tiết sự cố (Error Stacktrace)' : t('launchFailed.description') }}</span>
            </span>
          </div>
          <pre class="bg-[#0c0e14] border border-rose-500/30 text-rose-200/90 rounded-xl p-4 font-mono text-xs leading-relaxed overflow-auto max-h-[300px] visible-scroll whitespace-pre-wrap select-text">{{ data.errorLog }}</pre>
        </div>

        <!-- Latest Game Log Section -->
        <div v-if="data.log" class="flex flex-col gap-2">
          <div class="flex items-center justify-between text-xs font-mono text-zinc-400">
            <span class="font-bold text-zinc-300 flex items-center gap-1.5">
              <v-icon size="14">terminal</v-icon>
              <span>{{ data.isCrash ? 'Báo cáo sự cố (Crash Report)' : t('launchFailed.latestLog') }}</span>
            </span>
          </div>
          <pre class="bg-[#0c0e14] border border-white/10 text-zinc-300 rounded-xl p-4 font-mono text-xs leading-relaxed overflow-auto max-h-[320px] visible-scroll whitespace-pre-wrap select-text">{{ data.log }}</pre>
        </div>
      </div>
    </v-card>
  </v-dialog>
</template>

<script lang="ts" setup>
import { useService } from '@/composables'
import { kInstance } from '@/composables/instance'
import { kInstanceLaunch } from '@/composables/instanceLaunch'
import { diagnoseCrash } from '@/util/crashDiagnosis'
import { injection } from '@/util/inject'
import { BaseServiceKey, InstanceLogServiceKey, LaunchServiceKey } from '@xmcl/runtime-api'
import { reactive, computed, watch, ref } from 'vue'

const data = reactive({
  isShown: false,
  log: '',
  isCrash: false,
  isServer: false,
  launcherError: false,
  launchId: '',
  crashReportLocation: '',
  errorLog: '',
  elyByWarning: false,
  elyByMinecraftVersion: '',
})

const copied = ref(false)

const diagnosis = computed(() => {
  const combinedLog = `${data.errorLog}\n${data.log}`
  return diagnoseCrash(combinedLog)
})

watch(() => data.isShown, (isShown) => {
  if (!isShown) {
    data.log = ''
    data.isCrash = false
    data.isServer = false
    data.launcherError = false
    data.launchId = ''
    data.crashReportLocation = ''
    data.errorLog = ''
    data.elyByWarning = false
    data.elyByMinecraftVersion = ''
  }
})

const { t } = useI18n()
const { path } = injection(kInstance)
const { getLogContent, getCrashReportContent, getServerLogContent, showLog, showServerLog } = useService(InstanceLogServiceKey)
const { on } = useService(LaunchServiceKey)
const { showItemInDirectory } = useService(BaseServiceKey)
const { error } = injection(kInstanceLaunch)

watch(error, (e) => {
  if (!e) return
  data.launcherError = true
  data.errorLog = JSON.stringify(e, null, 2)
})

async function displayLog() {
  const log = data.isServer
    ? await getServerLogContent(path.value, 'latest.log')
    : await getLogContent(path.value, 'latest.log')
  data.log = log
  data.isShown = true
}

async function displayCrash(crashReport: string | undefined) {
  const log = await getCrashReportContent(path.value, data.crashReportLocation) || crashReport || ''
  data.log = log
  data.isShown = true
}

on('minecraft-exit', ({ code, signal, launchId, crashReport, crashReportLocation, errorLog, stdLog, side, elyByAuthlibReplaced, elyByMinecraftVersion }) => {
  if (!code && signal === 'SIGTERM') {
    return
  }
  if (code !== 0) {
    data.isServer = side === 'server'
    data.launchId = launchId
    data.errorLog = `${errorLog || ''}\n${stdLog || ''}`.trim()
    
    // Check if Ely.by authlib was used
    if (elyByAuthlibReplaced) {
      data.elyByWarning = true
      data.elyByMinecraftVersion = elyByMinecraftVersion || ''
    }
    
    if (crashReportLocation) {
      data.crashReportLocation = crashReportLocation
      data.isCrash = true
      displayCrash(crashReport)
    } else {
      displayLog()
    }
  }
})

function openFolder() {
  if (data.isCrash) {
    showItemInDirectory(data.crashReportLocation)
  } else if (data.isServer) {
    showServerLog(path.value, 'latest.log')
  } else {
    showLog(path.value, 'latest.log')
  }
}

function copyAllLogs() {
  const text = `=== HYPERX CRASH / ERROR LOG ===\n${data.errorLog}\n\n=== GAME LOG ===\n${data.log}`.trim()
  windowController.writeClipboard(text)
  copied.value = true
  setTimeout(() => {
    copied.value = false
  }, 2000)
}
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
