<template>
  <v-dialog
    v-model="isShown"
    width="800"
  >
    <v-card
      v-if="updateInfo"
      outlined
      class="visible-scroll max-h-90vh overflow-auto flex flex-col"
    >
      <v-alert
        v-if="isAppX"
        variant="tonal"
        type="warning"
      >
        {{ t('setting.appxUpdateHint') }}
      </v-alert>

      <v-card-item>
        <div class="d-flex align-center justify-space-between flex-wrap gap-2">
          <div>
            <div class="d-flex align-center gap-2">
              <v-icon color="primary">system_update</v-icon>
              <v-card-title class="text-h6 font-weight-bold pa-0">
                {{ updateInfo.name }}
              </v-card-title>
              <v-chip
                v-if="isTestUpdate"
                size="x-small"
                color="secondary"
                label
                class="font-weight-bold"
              >
                TEST MODE
              </v-chip>
            </div>
            <v-card-subtitle class="pa-0 mt-1">
              {{ getLocalDateString(updateInfo.date) }}
            </v-card-subtitle>
          </div>

          <v-chip
            :color="updateStatus === 'ready' ? 'success' : 'primary'"
            variant="tonal"
            size="small"
            label
          >
            {{ updateStatus === 'ready' ? 'Sẵn sàng cài đặt' : 'Bản cập nhật mới' }}
          </v-chip>
        </div>
      </v-card-item>

      <v-divider class="opacity-20" />

      <!-- Progress Section when downloading -->
      <div v-if="downloadingUpdate || (isTestUpdate && testProgress > 0)" class="pa-4 bg-surface-variant mx-4 mt-4 rounded-lg">
        <div class="d-flex justify-space-between align-center mb-2">
          <span class="text-subtitle-2 font-weight-bold d-flex align-center gap-1">
            <v-icon size="small" color="primary">downloading</v-icon>
            Đang tải bản cập nhật...
          </span>
          <span class="text-caption font-mono font-weight-bold primary--text">
            {{ isTestUpdate ? `${testProgress}%` : '' }}
          </span>
        </div>
        <v-progress-linear
          :model-value="isTestUpdate ? testProgress : 0"
          :indeterminate="!isTestUpdate && downloadingUpdate"
          color="primary"
          height="12"
          rounded
          striped
        />
        <div v-if="isTestUpdate" class="d-flex justify-space-between text-caption text-medium-emphasis mt-2 font-mono">
          <span>Đã tải: {{ testDownloadedSize }} / {{ testTotalSize }}</span>
          <span>Tốc độ: {{ testSpeed }}</span>
        </div>
      </div>

      <!-- Success Alert when Ready -->
      <v-alert
        v-if="updateStatus === 'ready'"
        variant="tonal"
        type="success"
        class="mx-4 mt-4 mb-0"
      >
        Bản cập nhật đã được tải về thành công và sẵn sàng để cài đặt vào HyperXLauncher!
      </v-alert>

      <v-card-text class="markdown-body flex flex-col overflow-auto pa-4">
        <div v-html="body" />
      </v-card-text>

      <v-divider class="opacity-20" />

      <v-card-actions class="pa-4">
        <v-btn
          @click="openGithub()"
          variant="text"
        >
          <v-icon start>signpost</v-icon>
          GitHub Releases
        </v-btn>

        <v-btn
          v-if="isTestUpdate"
          @click="onReset"
          variant="text"
          color="grey"
        >
          <v-icon start>replay</v-icon>
          Đặt lại thử nghiệm
        </v-btn>

        <v-spacer />

        <v-btn
          variant="text"
          color="grey"
          class="mr-2"
          @click="isShown = false"
        >
          Để sau
        </v-btn>

        <template v-if="hintRedownload">
          <v-btn
            color="primary"
            variant="elevated"
            class="font-weight-bold"
            @click="openGithub()"
          >
            <v-icon start>cloud_download</v-icon>
            Tải bản cập nhật mới
          </v-btn>
        </template>
        <template v-else>
          <v-btn
            v-if="updateStatus === 'pending'"
            color="primary"
            variant="elevated"
            :loading="downloadingUpdate"
            :disabled="downloadingUpdate"
            @click="downloadUpdate()"
          >
            <v-icon start>cloud_download</v-icon>
            {{ t('launcherUpdate.updateToThisVersion') }}
          </v-btn>
          <v-btn
            v-else-if="updateStatus === 'ready'"
            color="success"
            variant="elevated"
            :loading="installing"
            @click="quitAndInstall()"
          >
            <v-icon start>restart_alt</v-icon>
            {{ t('launcherUpdate.installAndQuit') }}
          </v-btn>
        </template>
      </v-card-actions>
    </v-card>

    <v-card
      v-else
      class="pa-8 text-center"
    >
      <v-avatar color="primary" variant="tonal" size="64" class="mb-4">
        <v-icon size="36" color="primary">check_circle</v-icon>
      </v-avatar>
      <h3 class="text-h6 font-weight-bold mb-2">
        {{ t('launcherUpdate.alreadyLatest') }}
      </h3>
      <p class="text-body-2 text-medium-emphasis mb-6">
        HyperXLauncher v{{ version }} hiện là phiên bản mới nhất.
      </p>
      <v-btn
        color="primary"
        variant="elevated"
        @click="isShown = false"
      >
        Đóng
      </v-btn>
    </v-card>
  </v-dialog>
</template>

<script lang="ts" setup>
import { computed, inject, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { kEnvironment, useEnvironment } from '@/composables/environment'
import { useMarkdown } from '@/composables/markdown'
import { kUpdateSettings, useUpdateSettings } from '@/composables/setting'
import { getLocalDateString } from '@/util/date'
import { injection } from '@/util/inject'
import { useDialog } from '../composables/dialog'

const { isShown } = useDialog('update-info')
const { t } = useI18n()
const { render } = useMarkdown()
const {
  installing,
  downloadingUpdate,
  updateInfo,
  updateStatus,
  downloadUpdate,
  quitAndInstall,
  version,
  isTestUpdate,
  testProgress,
  testSpeed,
  testDownloadedSize,
  testTotalSize,
  resetTestUpdate,
} = inject(kUpdateSettings, undefined) ?? useUpdateSettings()

function renderUpdate() {
  const body = updateInfo.value?.body ?? ''
  const transformed = body.replace(/## \[(.+)\]\(#.+\)/g, (str, v) => `## ${v}`)
  return render(transformed)
}
const body = computed(() => renderUpdate())
const env = inject(kEnvironment, undefined) ?? useEnvironment()
const isAppX = computed(() => env.value?.env === 'appx')
const hintRedownload = computed(() =>
  updateInfo.value?.operation === 'manual',
)

function onReset() {
  resetTestUpdate()
  isShown.value = false
}

const openGithub = () => {
  window.open('https://github.com/duongsecretvn/HyperXLauncher/releases', 'browser')
}
</script>

<style scoped>
</style>
