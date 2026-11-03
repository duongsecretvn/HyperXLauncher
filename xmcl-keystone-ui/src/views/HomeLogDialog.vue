<template>
  <v-dialog
    v-model="isShown"
    :width="1280"
    :content-props="{ 'aria-labelledby': 'log-dialog-title' }"
  >
    <v-card class="bg-[#12151f]/95 backdrop-blur-xl border border-white/10 rounded-2xl overflow-hidden shadow-2xl flex flex-col max-h-[85vh]">
      <!-- Modern Toolbar Header -->
      <div class="flex items-center justify-between px-5 py-3 border-b border-white/10 bg-gradient-to-r from-zinc-900 via-neutral-900 to-zinc-900 select-none">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center text-cyan-400">
            <v-icon size="20">terminal</v-icon>
          </div>
          <div>
            <div id="log-dialog-title" class="font-bold text-base text-zinc-100 flex items-center gap-2">
              <span>{{ t("logsCrashes.title") }}</span>
              <span class="text-xs px-2 py-0.5 rounded-full bg-white/10 text-zinc-300 font-mono font-normal">HyperX Console</span>
            </div>
            <div class="text-xs text-zinc-400">Theo dõi tiến trình game, lỗi crash và nhật ký hoạt động</div>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <!-- Open Logs Folder -->
          <v-btn
            variant="tonal"
            size="small"
            class="text-xs font-mono"
            prepend-icon="folder_open"
            @click="_showLog('')"
          >
            Thư mục Logs
          </v-btn>

          <!-- Refresh List -->
          <v-btn
            icon
            variant="text"
            size="small"
            :loading="data.loadingList"
            @click="refreshCurrentTab"
          >
            <v-icon size="18">refresh</v-icon>
          </v-btn>

          <!-- Close Button -->
          <v-btn
            icon
            variant="text"
            size="small"
            :aria-label="t('shared.close')"
            @click="hide"
          >
            <v-icon size="18">close</v-icon>
          </v-btn>
        </div>
      </div>

      <!-- Modern Tabs Bar -->
      <div class="px-5 pt-3 pb-2 bg-[#10131d] border-b border-white/5 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <button
            type="button"
            class="px-4 py-2 rounded-xl text-xs font-medium transition-all flex items-center gap-2"
            :class="data.tab === 0
              ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 shadow-sm'
              : 'text-zinc-400 hover:text-zinc-200 hover:bg-white/5'"
            :disabled="data.loadingList"
            @click="goLog"
          >
            <v-icon size="16">subtitles</v-icon>
            <span>{{ t("logsCrashes.logs") }}</span>
            <span class="text-[10px] px-1.5 py-0.5 rounded-full bg-black/40 text-cyan-300 font-mono">
              {{ data.logs.length }}
            </span>
          </button>

          <button
            type="button"
            class="px-4 py-2 rounded-xl text-xs font-medium transition-all flex items-center gap-2"
            :class="data.tab === 1
              ? 'bg-rose-500/20 text-rose-300 border border-rose-500/40 shadow-sm'
              : 'text-zinc-400 hover:text-zinc-200 hover:bg-white/5'"
            :disabled="data.loadingList"
            @click="goCrash"
          >
            <v-icon size="16">bug_report</v-icon>
            <span>{{ t("logsCrashes.crashes") }}</span>
            <span
              class="text-[10px] px-1.5 py-0.5 rounded-full font-mono"
              :class="data.crashes.length > 0 ? 'bg-rose-500/30 text-rose-300 font-bold' : 'bg-black/40 text-zinc-400'"
            >
              {{ data.crashes.length }}
            </span>
          </button>

          <button
            v-if="data.serverLogs.length > 0"
            type="button"
            class="px-4 py-2 rounded-xl text-xs font-medium transition-all flex items-center gap-2"
            :class="data.tab === 2
              ? 'bg-purple-500/20 text-purple-300 border border-purple-500/40 shadow-sm'
              : 'text-zinc-400 hover:text-zinc-200 hover:bg-white/5'"
            :disabled="data.loadingList"
            @click="goServerLogs"
          >
            <v-icon size="16">dns</v-icon>
            <span>{{ t("logsCrashes.serverLogs") }}</span>
            <span class="text-[10px] px-1.5 py-0.5 rounded-full bg-black/40 text-purple-300 font-mono">
              {{ data.serverLogs.length }}
            </span>
          </button>
        </div>
      </div>

      <!-- Tab Content Area -->
      <div class="flex-grow overflow-auto p-4 bg-[#0e111a]">
        <TabItem
          v-if="data.tab === 0"
          log
          :visible="data.tab === 0 && isShown"
          :files="data.logs"
          :refreshing="data.loadingList"
          :get-file-content="_getLogContent"
          :remove-file="removeLog"
          :show-file="_showLog"
        />

        <TabItem
          v-else-if="data.tab === 1"
          :visible="data.tab === 1 && isShown"
          :files="data.crashes"
          :refreshing="data.loadingList"
          :get-file-content="_getCrashReportContent"
          :get-launch-id="_getCrashReportLaunchId"
          :remove-file="removeCrashReport"
          :show-file="_showCrashReport"
        />

        <TabItem
          v-else-if="data.tab === 2"
          log
          :visible="data.tab === 2 && isShown"
          :files="data.serverLogs"
          :refreshing="data.loadingList"
          :get-file-content="_getServerLogContent"
          :remove-file="removeServerLog"
          :show-file="_showServerLog"
        />
      </div>
    </v-card>
  </v-dialog>
</template>

<script lang="ts" setup>
import { useService } from "@/composables";
import { kInstance } from "@/composables/instance";
import { injection } from "@/util/inject";
import { InstanceLogServiceKey } from "@xmcl/runtime-api";
import { useDialog } from "../composables/dialog";
import TabItem from "./HomeLogDialogTab.vue";

const {
  listLogs,
  listCrashReports,
  listServerLogs,
  removeLog: rmLog,
  removeCrashReport: rmCrash,
  removeServerLog: rmServerLog,
  getCrashReportContent,
  findLaunchByCrashReport,
  getLogContent,
  getServerLogContent,
  showLog,
  showCrash: showCrashReport,
  showServerLog: showServerLogFile,
} = useService(InstanceLogServiceKey);

const { isShown, hide } = useDialog("log");
const { t } = useI18n();

const { path } = injection(kInstance);

const data = reactive({
  tab: 0,
  loadingContent: false,
  loadingList: false,
  logs: [] as string[],
  crashes: [] as string[],
  serverLogs: [] as string[],
});

const _getLogContent = (name: string) => getLogContent(path.value, name);
const _getCrashReportContent = (name: string) =>
  getCrashReportContent(path.value, name);
const _getCrashReportLaunchId = (name: string) =>
  findLaunchByCrashReport(path.value, name).then((launch) => launch?.launchId);
const _getServerLogContent = (name: string) =>
  getServerLogContent(path.value, name);
const _showLog = (name: string) => showLog(path.value, name);
const _showCrashReport = (name: string) => showCrashReport(path.value, name);
const _showServerLog = (name: string) => showServerLogFile(path.value, name);

function loadLogs() {
  data.loadingList = true;
  listLogs(path.value)
    .then((l) => {
      data.logs = l;
    })
    .finally(() => {
      data.loadingList = false;
    });
}

function loadCrashes() {
  data.loadingList = true;
  listCrashReports(path.value)
    .then((l) => {
      data.crashes = l;
    })
    .finally(() => {
      data.loadingList = false;
    });
}

function loadServerLogs() {
  data.loadingList = true;
  listServerLogs(path.value)
    .then((l) => {
      data.serverLogs = l;
    })
    .finally(() => {
      data.loadingList = false;
    });
}

function refreshCurrentTab() {
  if (data.tab === 1) loadCrashes();
  else if (data.tab === 2) loadServerLogs();
  else loadLogs();
}

async function removeLog(name: string) {
  await rmLog(path.value, name);
  loadLogs();
}

async function removeCrashReport(name: string) {
  await rmCrash(path.value, name);
  loadCrashes();
}

async function removeServerLog(name: string) {
  await rmServerLog(path.value, name);
  loadServerLogs();
}

watch(isShown, (s) => {
  if (s) {
    data.tab = 0;
    loadLogs();
    loadServerLogs();
  } else {
    data.logs = [];
    data.crashes = [];
    data.serverLogs = [];
  }
});

function goLog() {
  data.tab = 0;
  loadLogs();
}

function goCrash() {
  data.tab = 1;
  loadCrashes();
}

function goServerLogs() {
  data.tab = 2;
  loadServerLogs();
}
</script>

<style scoped>
</style>
