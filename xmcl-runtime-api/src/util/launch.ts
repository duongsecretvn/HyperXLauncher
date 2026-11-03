import type { Instance } from '@xmcl/instance'
import { UserProfile } from '../entities/user.schema'
import { LaunchException, LaunchOptions } from '../services/LaunchService'
import { AUTHORITY_DEV } from './authority'

function tryParseUrl(url: string) {
  try {
    return new URL(url)
  } catch {
    return undefined
  }
}

export interface GenerateLaunchOptions {
  token: string
  operationId: string
  side?: 'client' | 'server'
  overrides?: Partial<LaunchOptions>
  dry?: boolean
  javaPath?: string
  globalEnv: Record<string, string>
  globalVmOptions: string[]
  globalMcOptions: string[]
  globalPrependCommand: string
  globalAssignMemory: boolean | 'auto'
  globalMinMemory: number
  globalMaxMemory: number
  globalHideLauncher?: boolean
  globalShowLog?: boolean
  globalFastLaunch?: boolean
  globalDisableAuthlibInjector?: boolean
  globalDisableElyByAuthlib?: boolean
  globalSyncGameLanguage?: boolean
  globalPreExecuteCommand?: string
  globalResolution?: { width?: number; height?: number; fullscreen?: boolean; monitor?: string }
  modCount: number
  getOrInstallAuthlibInjector: () => Promise<string>
  track: <T, S = string>(
    token: string,
    operation: () => Promise<T>,
    name: S,
    id: string,
  ) => Promise<T>
}

export async function generateLaunchOptionsWithGlobal(
  inst: Instance,
  userProfile: UserProfile,
  version: string | undefined,
  {
    token,
    operationId,
    side = 'client',
    overrides,
    dry,
    javaPath: _javaPath,
    globalEnv,
    globalVmOptions,
    globalMcOptions,
    globalPrependCommand,
    globalAssignMemory,
    globalMinMemory,
    globalMaxMemory,
    globalHideLauncher,
    globalShowLog,
    globalFastLaunch,
    globalDisableAuthlibInjector,
    globalDisableElyByAuthlib,
    globalSyncGameLanguage,
    globalPreExecuteCommand,
    globalResolution,
    modCount,
    getOrInstallAuthlibInjector,
    track,
  }: GenerateLaunchOptions
) {
  const ver = version

  if (!ver) {
    throw new LaunchException({ type: 'launchNoVersionInstalled' })
  }

  const javaPath = overrides?.java ?? _javaPath
  if (!javaPath) {
    throw new LaunchException({ type: 'launchNoProperJava', javaPath: '' })
  }

  let yggdrasilAgent: LaunchOptions['yggdrasilAgent']

  const authority = tryParseUrl(userProfile.authority)

  const disableAuthlibInjector = inst.disableAuthlibInjector ?? globalDisableAuthlibInjector
  if (
    !disableAuthlibInjector
    && authority
    && (authority.protocol === 'http:' || authority?.protocol === 'https:' || userProfile.authority === AUTHORITY_DEV)
    && !dry
  ) {
    try {
      yggdrasilAgent = {
        jar: await track(token, getOrInstallAuthlibInjector, 'preparing-authlib', operationId),
        server: userProfile.authority,
      }
    } catch {
      // TODO: notify user
    }
  }

  const assignMemory = inst.assignMemory ?? globalAssignMemory
  const hideLauncher = inst.hideLauncher ?? globalHideLauncher
  const preExecuteCommand = inst.preExecuteCommand ?? globalPreExecuteCommand
  const env = {
    ...globalEnv,
    ...inst.env,
  }
  const showLog = inst.showLog ?? globalShowLog
  const fastLaunch = inst.fastLaunch ?? globalFastLaunch ?? true
  const disableElyByAuthlib = inst.disableElybyAuthlib ?? globalDisableElyByAuthlib

  let minMemory: number | undefined = inst.minMemory ?? globalMinMemory
  let maxMemory: number | undefined = inst.maxMemory ?? globalMaxMemory
  if (assignMemory === true && minMemory > 0) {
    // noop
  } else if (assignMemory === 'auto') {
    // TLauncher #40 Smart Tier Allocation based on mod count & workload
    if (modCount === 0) {
      minMemory = 1024
      maxMemory = 3072
    } else if (modCount <= 30) {
      minMemory = 2048
      maxMemory = 4096
    } else if (modCount <= 80) {
      minMemory = 3072
      maxMemory = 6144
    } else {
      minMemory = 4096
      maxMemory = 8192
    }
  } else {
    minMemory = undefined
  }
  maxMemory = assignMemory === true && maxMemory > 0 ? maxMemory : (assignMemory === 'auto' ? maxMemory : undefined)

  const vmOptions = inst.vmOptions ?? globalVmOptions.filter(v => !!v)
  const mcOptions = inst.mcOptions ?? globalMcOptions.filter(v => !!v)
  const prependCommand = inst.prependCommand ?? globalPrependCommand

  const options: LaunchOptions = {
    operationId,
    version: ver,
    gameDirectory: inst.path,
    user: userProfile,
    java: javaPath,
    hideLauncher,
    env,
    showLog,
    minMemory,
    maxMemory,
    skipAssetsCheck: fastLaunch,
    vmOptions,
    mcOptions,
    yggdrasilAgent,
    disableElyByAuthlib,
    preExecuteCommand,
    prependCommand,
    side,
    resolution: inst.resolution ?? globalResolution,
    server: inst.server ?? undefined,
    ...overrides,
  }
  return options
}