import { getKoffi } from './gameWindow/koffi'

/**
 * Safe working set and GC memory optimization for background and minimized states.
 */
export function trimMemory() {
  if (global.gc) {
    try {
      global.gc()
    } catch {}
  }
  if (process.platform === 'win32') {
    try {
      const koffi = getKoffi()
      if (koffi) {
        const kernel32 = koffi.load('kernel32.dll')
        const GetCurrentProcess = kernel32.func('void* GetCurrentProcess()')
        const SetProcessWorkingSetSize = kernel32.func(
          'int SetProcessWorkingSetSize(void* hProcess, size_t dwMinimumWorkingSetSize, size_t dwMaximumWorkingSetSize)',
        )
        SetProcessWorkingSetSize(GetCurrentProcess(), -1, -1)
      }
    } catch {}
  }
}
