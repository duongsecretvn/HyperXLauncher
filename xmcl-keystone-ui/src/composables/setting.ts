import { useService } from '@/composables'
import { injection } from '@/util/inject'
import { clientCurseforgeV1 } from '@/util/clients'
import { BaseServiceKey, Environment, Settings, ReleaseInfo } from '@xmcl/runtime-api'
import { InjectionKey, Ref } from 'vue'
import { useLocalStorage } from '@vueuse/core'
import { kEnvironment } from './environment'
import { useState } from './syncableState'

export const kUpdateSettings: InjectionKey<ReturnType<typeof useUpdateSettings>> = Symbol('UpdateSettings')

export function useUpdateSettings() {
  const { checkUpdate, downloadUpdate, quitAndInstall } = useService(BaseServiceKey)
  const env: Ref<Environment | undefined> = injection(kEnvironment)
  const { state } = injection(kSettingsState)
  const updateStatus = computed(() => state.value?.updateStatus)
  const updateInfo = computed(() => state.value?.updateInfo)
  const version = computed(() => env.value?.version ?? '0.0.0')
  const installing = ref(false)

  // Test mode states
  const isTestUpdate = ref(false)
  const testProgress = ref(0)
  const testSpeed = ref('15.4 MB/s')
  const testDownloadedSize = ref('0.0 MB')
  const testTotalSize = ref('93.4 MB')
  const testUpdateInfo = ref<ReleaseInfo | null>(null)
  const testUpdateStatus = ref<'none' | 'pending' | 'ready'>('none')
  let testTimer: any = null

  function startTestUpdate() {
    isTestUpdate.value = true
    testProgress.value = 0
    testDownloadedSize.value = '0.0 MB'
    testUpdateStatus.value = 'pending'
    testUpdateInfo.value = {
      name: 'v1.0.1 (Bản thử nghiệm)',
      date: new Date().toISOString(),
      body: `## 🚀 HyperXLauncher v1.0.1 - Bản Cập Nhật Thử Nghiệm

Chào mừng bạn đến với phiên bản thử nghiệm tính năng cập nhật của **HyperXLauncher**!

### ✨ Các tính năng & Cải tiến mới
- ⚡ **Tăng tốc khởi động Launcher**: Giảm 40% thời gian tải tài nguyên và tối ưu hóa bộ nhớ RAM.
- 🎨 **Bộ sưu tập Phông chữ Minecraft Retro**: Hỗ trợ 15+ font chữ tùy chỉnh toàn diện với chế độ xem trước trực quan khi rê chuột.
- 🔄 **Hệ thống Cập nhật Launcher Tự động**: Tự động phát hiện và cài đặt các phiên bản mới mượt mà, an toàn.
- 🛡️ **Tối ưu hóa Giao diện Cài đặt**: Tinh gọn các mục điều hướng, tăng tính tương thích và loại bỏ các thành phần không cần thiết.
- 🛠️ **Sửa lỗi & Ổn định**: Khắc phục lỗi crash tiềm ẩn, sửa lỗi hiển thị skin và biểu tượng người chơi.

> [!TIP]
> Bạn có thể nhấn nút **"Tải bản cập nhật"** bên dưới để kiểm tra quy trình tải xuống mô phỏng tốc độ cao.`,
      files: [{ name: 'HyperXLauncher-Setup-1.0.1.exe', url: '#' }],
      newUpdate: true,
      operation: 'autoupdater' as any,
    }
  }

  function downloadTestUpdate() {
    if (downloadingUpdate.value || testProgress.value > 0) return
    testProgress.value = 5
    testDownloadedSize.value = '4.6 MB'

    if (testTimer) clearInterval(testTimer)
    testTimer = setInterval(() => {
      testProgress.value += Math.floor(Math.random() * 12) + 8
      if (testProgress.value >= 100) {
        testProgress.value = 100
        testDownloadedSize.value = '93.4 MB'
        testUpdateStatus.value = 'ready'
        clearInterval(testTimer)
        testTimer = null
      } else {
        const currentMB = ((testProgress.value / 100) * 93.4).toFixed(1)
        testDownloadedSize.value = `${currentMB} MB`
      }
    }, 250)
  }

  function installTestUpdate() {
    installing.value = true
    setTimeout(() => {
      installing.value = false
      alert('Đã hoàn thành kiểm thử quy trình cập nhật launcher! Hệ thống sẽ khởi động lại HyperXLauncher.')
      location.reload()
    }, 1200)
  }

  function resetTestUpdate() {
    if (testTimer) {
      clearInterval(testTimer)
      testTimer = null
    }
    isTestUpdate.value = false
    testProgress.value = 0
    testUpdateStatus.value = 'none'
    testUpdateInfo.value = null
  }

  async function install() {
    if (isTestUpdate.value) {
      installTestUpdate()
      return
    }
    installing.value = true
    try {
      await quitAndInstall()
    } finally {
      installing.value = false
    }
  }

  const downloadingUpdate = ref(false)
  async function download() {
    if (isTestUpdate.value) {
      downloadTestUpdate()
      return
    }
    downloadingUpdate.value = true
    try {
      await downloadUpdate()
    } finally {
      downloadingUpdate.value = false
    }
  }

  const checkingUpdate = ref(false)
  async function check() {
    resetTestUpdate()
    checkingUpdate.value = true
    try {
      await checkUpdate()
    } finally {
      checkingUpdate.value = false
    }
  }

  const effectiveUpdateStatus = computed(() => isTestUpdate.value ? testUpdateStatus.value : updateStatus.value)
  const effectiveUpdateInfo = computed(() => isTestUpdate.value ? testUpdateInfo.value : updateInfo.value)
  const effectiveDownloading = computed(() => isTestUpdate.value ? (testProgress.value > 0 && testProgress.value < 100) : downloadingUpdate.value)

  return {
    version,
    quitAndInstall: install,
    downloadUpdate: download,
    checkUpdate: check,
    installing,
    updateStatus: effectiveUpdateStatus,
    checkingUpdate,
    downloadingUpdate: effectiveDownloading,
    updateInfo: effectiveUpdateInfo,
    // Test update helpers
    isTestUpdate,
    testProgress,
    testSpeed,
    testDownloadedSize,
    testTotalSize,
    startTestUpdate,
    downloadTestUpdate,
    installTestUpdate,
    resetTestUpdate,
  }
}
export const kSettingsState: InjectionKey<ReturnType<typeof useSettingsState>> = Symbol('Settings')

export function useSettingsState() {
  const { getSettings } = useService(BaseServiceKey)
  return useState(getSettings, Settings)
}

export function useGlobalSettings({ state } = injection(kSettingsState)) {
  const globalAssignMemory = computed(() => state.value?.globalAssignMemory ?? false)
  const globalMinMemory = computed(() => state.value?.globalMinMemory ?? 0)
  const globalMaxMemory = computed(() => state.value?.globalMaxMemory ?? 0)
  const globalJava = computed(() => state.value?.globalJava ?? '')
  const globalVmOptions = computed(() => state.value?.globalVmOptions ?? [])
  const globalMcOptions = computed(() => state.value?.globalMcOptions ?? [])
  const globalFastLaunch = computed(() => state.value?.globalFastLaunch ?? false)
  const globalHideLauncher = computed(() => state.value?.globalHideLauncher ?? false)
  const globalShowLog = computed(() => state.value?.globalShowLog ?? false)
  const globalDisableAuthlibInjector = computed(() => state.value?.globalDisableAuthlibInjector ?? true)
  const globalDisableElyByAuthlib = computed(() => state.value?.globalDisableElyByAuthlib ?? false)
  const globalSyncGameLanguage = computed(() => state.value?.globalSyncGameLanguage ?? false)
  const globalPrependCommand = computed(() => state.value?.globalPrependCommand ?? '')
  const globalPreExecuteCommand = computed(() => state.value?.globalPreExecuteCommand ?? '')
  const globalEnv = computed(() => state.value?.globalEnv ?? {})
  const globalResolution = computed(() => state.value?.globalResolution)
  const setGlobalSettings = (setting: {
    globalMinMemory: number
    globalMaxMemory: number
    globalAssignMemory: boolean | 'auto'
    globalJava: string
    globalVmOptions: string[]
    globalMcOptions: string[]
    globalFastLaunch: boolean
    globalHideLauncher: boolean
    globalShowLog: boolean
    globalDisableAuthlibInjector: boolean
    globalDisableElyByAuthlib: boolean
    globalSyncGameLanguage?: boolean
    globalPrependCommand: string
    globalPreExecuteCommand: string
    globalEnv: Record<string, string>
    globalResolution: { width?: number; height?: number; fullscreen?: boolean; monitor?: string }
  }) => {
    state.value?.globalInstanceSetting(setting)
  }

  return {
    globalAssignMemory,
    globalMinMemory,
    globalMaxMemory,
    globalJava,
    globalVmOptions,
    globalMcOptions,
    globalFastLaunch,
    globalHideLauncher,
    globalShowLog,
    globalDisableAuthlibInjector,
    globalDisableElyByAuthlib,
    globalSyncGameLanguage,
    globalPrependCommand,
    globalPreExecuteCommand,
    globalEnv,
    globalResolution,
    setGlobalSettings,
  }
}
export function useGameDirectory() {
  const { getGameDataDirectory, migrate, openDirectory } = useService(BaseServiceKey)
  const root = ref('')
  onMounted(() => {
    getGameDataDirectory().then((r) => {
      root.value = r
    })
  })
  const setGameDirectory = async (value: string) => {
    root.value = value
    await migrate({ destination: value })
  }

  function showGameDirectory() {
    if (root.value) {
      openDirectory(root.value)
    }
  }
  return {
    root,
    setGameDirectory,
    showGameDirectory,
  }
}

export function parseProxy(proxy: string) {
  try {
    const url = new URL(proxy)
    return {
      host: url.hostname,
      port: url.port,
    }
  } catch (e) {
    return {
      host: '',
      port: '',
    }
  }
}

export function formatProxy(host: string, port: string) {
  const value = host.trim()
  const configuredPort = port.trim()
  if (!value && !configuredPort) return ''

  let protocol = 'http:'
  let hostname = value
  let embeddedPort = ''
  try {
    const url = new URL(value.includes('://') ? value : `http://${value}`)
    protocol = url.protocol
    hostname = url.hostname
    embeddedPort = url.port
  } catch (e) {}

  const effectivePort = configuredPort || embeddedPort
  return `${protocol}//${hostname}${effectivePort ? `:${effectivePort}` : ''}`
}

export function useSettings() {
  const streamerMode = inject('streamerMode', useLocalStorage('streamerMode', false, { writeDefaults: false }))
  const { state, error, isValidating } = injection(kSettingsState)

  const getProxy = () => {
    return parseProxy(state.value?.httpProxy || '')
  }

  const locales = computed(() => state.value?.locales || [])
  const selectedLocale = computed({
    get: () => locales.value.find(l => l.locale === state.value?.locale)?.locale || 'en',
    set: v => state.value?.localeSet(v),
  })
  const enableDedicatedGPUOptimization = computed({
    get: () => state.value?.enableDedicatedGPUOptimization ?? false,
    set: v => state.value?.enableDedicatedGPUOptimizationSet(v),
  })
  const apiSetsPreference = computed({
    get: () => state.value?.apiSetsPreference ?? '',
    set: v => state.value?.apiSetsPreferenceSet(v),
  })
  const replaceNative = computed({
    get: () => state.value?.replaceNatives ?? 'legacy-only' as string | false,
    set: v => state.value?.replaceNativesSet(v as any),
  })
  const maxSockets = ref(state.value?.maxSockets)
  const maxAPISockets = ref(state.value?.maxAPISockets)
  const proxy = ref(getProxy())
  const httpProxyEnabled = computed({
    get: () => state.value?.httpProxyEnabled ?? false,
    set: v => state.value?.httpProxyEnabledSet(v),
  })
  const developerMode = computed({
    get: () => state.value?.developerMode ?? false,
    set: v => state.value?.developerModeSet(v),
  })
  const disableTelemetry = computed({
    get: () => state.value?.disableTelemetry ?? false,
    set: v => state.value?.disableTelemetrySet(v),
  })
  const multiplayerTransport = computed({
    get: () => state.value?.multiplayerTransport ?? 'webrtc',
    set: v => state.value?.multiplayerTransportSet(v),
  })
  const enableDiscord = computed({
    get: () => state.value?.discordPresence ?? false,
    set: (v) => state.value?.discordPresenceSet(v),
  })
  const discordClientId = computed({
    get: () => state.value?.discordClientId ?? '',
    set: (v) => state.value?.discordClientIdSet(v),
  })
  const curseforgeApiKey = computed({
    get: () => state.value?.curseforgeApiKey ?? '',
    set: (v) => {
      state.value?.curseforgeApiKeySet(v)
      clientCurseforgeV1.setApiKey(v)
    },
  })
  watch(() => state.value?.curseforgeApiKey, (val) => {
    if (val !== undefined) {
      clientCurseforgeV1.setApiKey(val)
    }
  }, { immediate: true })
  const apiSets = computed(() => state.value?.apiSets || [])

  onMounted(() => {
    const p = getProxy()
    proxy.value.host = p.host
    proxy.value.port = p.port
  })

  watch(computed(() => state.value?.httpProxy), () => {
    const p = getProxy()
    proxy.value.host = p.host
    proxy.value.port = p.port
  })

  onUnmounted(() => {
    const p = proxy.value
    const newValue = formatProxy(p.host, p.port)
    if (newValue !== state.value?.httpProxy) {
      state.value?.httpProxySet(newValue)
    }
    if (state.value?.maxSockets !== maxSockets.value) {
      state.value?.maxSocketsSet(Number(maxSockets.value))
    }
    if (state.value?.maxAPISockets !== maxAPISockets.value) {
      state.value?.maxAPISocketsSet(Number(maxAPISockets.value))
    }
  })

  const quickActionShortcut = computed({
    get: () => state.value?.quickActionShortcut ?? '',
    set: (v) => state.value?.quickActionShortcutSet(v),
  })

  return {
    enableDedicatedGPUOptimization,
    streamerMode,
    developerMode,
    httpProxyEnabled,
    enableDiscord,
    discordClientId,
    curseforgeApiKey,
    quickActionShortcut,
    maxSockets,
    maxAPISockets,
    locales,
    proxy,
    replaceNative,
    selectedLocale,
    apiSetsPreference,
    apiSets,
    disableTelemetry,
    multiplayerTransport,
    error,
    isValidating,
  }
}
