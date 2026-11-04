<template>
  <SettingCard>
    <div class="pa-5">
      <!-- Top Brand Row -->
      <div class="d-flex flex-wrap align-center justify-space-between gap-4">
        <div class="d-flex align-center gap-4">
          <v-img
            :src="logo"
            alt="HyperX Logo"
            width="68"
            height="68"
            class="rounded-xl flex-grow-0 flex-shrink-0 elevation-2"
          />
          <div>
            <div class="d-flex align-center flex-wrap gap-2">
              <span class="text-h5 font-weight-bold select-none text-high-emphasis">
                HyperXLauncher
              </span>
              <v-chip
                size="small"
                color="primary"
                variant="tonal"
                label
                class="font-weight-bold"
              >
                v{{ displayVersion }}
              </v-chip>
              <v-chip
                v-if="hasNewUpdate"
                size="small"
                color="warning"
                label
                class="font-weight-bold"
              >
                CÓ BẢN MỚI {{ updateInfo?.name }}
              </v-chip>
              <v-chip
                v-else
                size="small"
                color="success"
                variant="flat"
                label
                class="font-weight-medium"
              >
                MỚI NHẤT
              </v-chip>
            </div>
            <div class="text-body-2 text-medium-emphasis mt-1 select-none">
              Trình khởi chạy Minecraft hiện đại, siêu tốc và tối ưu hóa hệ thống.
            </div>
          </div>
        </div>

        <!-- Action Buttons Row -->
        <div class="d-flex flex-wrap align-center gap-2">
          <!-- Download on GitHub Button -->
          <v-btn
            color="primary"
            variant="elevated"
            class="font-weight-bold"
            @click="openGitHubReleases"
          >
            <v-icon start>download</v-icon>
            Tải trên GitHub
          </v-btn>

          <!-- Check Update Button -->
          <v-btn
            data-testid="settings-check-update"
            color="primary"
            variant="tonal"
            :loading="checkingUpdate"
            @click="checkUpdate"
          >
            <v-icon start>refresh</v-icon>
            {{ t('setting.checkUpdate') }}
          </v-btn>

          <!-- View / Apply Update Button -->
          <v-btn
            v-if="updateStatus !== 'none'"
            :color="updateStatus === 'ready' ? 'success' : 'primary'"
            variant="flat"
            @click="showUpdateInfo()"
          >
            <v-icon start>{{ updateStatus === 'ready' ? 'restart_alt' : 'info' }}</v-icon>
            {{ updateStatus === 'ready' ? t('launcherUpdate.installAndQuit') : t('launcherUpdate.updateToThisVersion') }}
          </v-btn>
        </div>
      </div>

      <!-- Non-mandatory update alert banner if a new release is available -->
      <v-alert
        v-if="hasNewUpdate && !dismissedUpdate"
        color="primary"
        variant="tonal"
        class="mt-4 rounded-xl border border-primary/20"
        density="comfortable"
      >
        <div class="d-flex align-center justify-space-between flex-wrap gap-3">
          <div class="d-flex align-center gap-3">
            <v-avatar color="primary" variant="tonal" size="40">
              <v-icon color="primary">system_update</v-icon>
            </v-avatar>
            <div>
              <div class="font-weight-bold text-subtitle-1">
                Phát hiện bản cập nhật mới: {{ updateInfo?.name }}
              </div>
              <div class="text-caption text-medium-emphasis">
                Bản cập nhật không bắt buộc. Bạn có thể chọn cập nhật ngay hoặc tiếp tục chơi ở phiên bản hiện tại (v{{ displayVersion }}).
              </div>
            </div>
          </div>
          <div class="d-flex align-center gap-2">
            <v-btn
              color="primary"
              variant="elevated"
              class="font-weight-bold"
              @click="showUpdateInfo()"
            >
              <v-icon start>cloud_download</v-icon>
              Cập nhật ngay
            </v-btn>
            <v-btn
              variant="text"
              color="grey"
              @click="dismissedUpdate = true"
            >
              Để sau
            </v-btn>
          </div>
        </div>
      </v-alert>

      <v-divider class="my-4 opacity-20" />

      <!-- Details List: Current Version & GitHub Links -->
      <div class="surface-panel rounded-lg p-3 d-flex flex-column gap-2 mb-3">
        <!-- Current Version Info Line -->
        <div class="d-flex align-center justify-space-between flex-wrap gap-2 text-sm px-2">
          <div class="d-flex align-center gap-2 text-medium-emphasis">
            <v-icon size="18" color="primary">verified</v-icon>
            <span class="font-weight-medium">Phiên bản hiện tại:</span>
          </div>
          <div class="font-weight-bold font-mono">
            HyperXLauncher v{{ displayVersion }}
          </div>
        </div>

        <v-divider class="opacity-10" />

        <!-- GitHub Releases Download Line -->
        <div class="d-flex align-center justify-space-between flex-wrap gap-2 text-sm px-2">
          <div class="d-flex align-center gap-2 text-medium-emphasis">
            <v-icon size="18" color="primary">cloud_download</v-icon>
            <span class="font-weight-medium">Đường dẫn tải bản phát hành (Releases):</span>
          </div>
          <a
            href="https://github.com/duongsecretvn/HyperXLauncher/releases"
            target="browser"
            class="text-primary hover:underline font-weight-medium d-flex align-center gap-1"
            @click.prevent="openGitHubReleases"
          >
            <span>https://github.com/duongsecretvn/HyperXLauncher/releases</span>
            <v-icon size="14">open_in_new</v-icon>
          </a>
        </div>

        <v-divider class="opacity-10" />

        <!-- GitHub Repository Line -->
        <div class="d-flex align-center justify-space-between flex-wrap gap-2 text-sm px-2">
          <div class="d-flex align-center gap-2 text-medium-emphasis">
            <v-icon size="18" color="primary">code</v-icon>
            <span class="font-weight-medium">Kho lưu trữ mã nguồn (Repository):</span>
          </div>
          <a
            href="https://github.com/duongsecretvn/HyperXLauncher"
            target="browser"
            class="text-primary hover:underline font-weight-medium d-flex align-center gap-1"
            @click.prevent="openGitHubRepo"
          >
            <span>https://github.com/duongsecretvn/HyperXLauncher</span>
            <v-icon size="14">open_in_new</v-icon>
          </a>
        </div>
      </div>

      <div class="text-caption opacity-60 select-none">
        NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.
      </div>
    </div>
  </SettingCard>
</template>

<script lang="ts" setup>
import { computed, inject, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { injection } from '@/util/inject'
import { useDialog } from '../composables/dialog'
import { kUpdateSettings, useUpdateSettings } from '../composables/setting'
import SettingCard from '@/components/SettingCard.vue'
import logo from '../assets/logo.webp'

const dismissedUpdate = ref(false)

const { show: showUpdateInfo } = useDialog('update-info')
const {
  updateInfo,
  updateStatus,
  checkUpdate,
  checkingUpdate,
  version,
} = inject(kUpdateSettings, undefined) ?? useUpdateSettings()

const { t } = useI18n()

// Ensure version defaults to 1.0.0 if not yet loaded from env
const displayVersion = computed(() => {
  const ver = version.value
  return (ver && ver !== '0.0.0') ? ver : '1.0.0'
})

const hasNewUpdate = computed(() => updateStatus.value !== 'none' && !!updateInfo.value?.newUpdate)

function openGitHubReleases() {
  window.open('https://github.com/duongsecretvn/HyperXLauncher/releases', 'browser')
}

function openGitHubRepo() {
  window.open('https://github.com/duongsecretvn/HyperXLauncher', 'browser')
}
</script>

<style scoped>
.surface-panel {
  background: rgba(var(--v-theme-surface-variant), 0.35);
  border: 1px solid rgba(var(--v-theme-on-surface), 0.08);
}
</style>
