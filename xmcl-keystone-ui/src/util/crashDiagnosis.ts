export interface CrashDiagnosis {
  type: 'oom' | 'java-version' | 'mod-conflict' | 'duplicate-mod' | 'gpu-driver'
  title: string
  description: string
  solution: string
  highlight?: string
}

const CLASS_VERSION_MAP: Record<number, string> = {
  67: 'Java 23',
  66: 'Java 22',
  65: 'Java 21',
  64: 'Java 20',
  63: 'Java 19',
  62: 'Java 18',
  61: 'Java 17',
  60: 'Java 16',
  59: 'Java 15',
  58: 'Java 14',
  57: 'Java 13',
  56: 'Java 12',
  55: 'Java 11',
  52: 'Java 8',
}

/**
 * Smart Offline Crash Decision Tree inspired by PCL2 (#06) & HMCL (#05).
 * Parses crash logs & stacktraces locally without requiring network or AI services.
 */
export function diagnoseCrash(log: string): CrashDiagnosis | undefined {
  if (!log || log.trim().length === 0) return undefined

  // 1. Out of Memory (OOM)
  if (
    /java\.lang\.OutOfMemoryError|GC overhead limit exceeded|There is insufficient memory for the Java Runtime Environment/i.test(log)
  ) {
    return {
      type: 'oom',
      title: 'Thiếu bộ nhớ RAM (Out of Memory)',
      description: 'Minecraft đã sử dụng vượt quá lượng RAM tối đa được cấp phát, khiến máy ảo Java (JVM) bị tràn bộ nhớ.',
      solution: 'Vào Cài đặt phiên bản -> Tab Java -> Phần "Bộ nhớ RAM" và tăng mức RAM tối đa lên (khuyến nghị từ 4096 MB đến 6144 MB cho modpack).',
    }
  }

  // 2. Java Version Mismatch
  const javaVerMatch =
    log.match(/has been compiled by a more recent version of the Java Runtime \(class file version (\d+)(?:\.0)?\)/i) ||
    log.match(/class file version (\d+)(?:\.0)?.*only recognizes class file versions up to (\d+)/i)

  if (javaVerMatch) {
    const requiredClassVer = parseInt(javaVerMatch[1], 10)
    const requiredJava = CLASS_VERSION_MAP[requiredClassVer] || `Java phiên bản mới hơn (${requiredClassVer})`
    return {
      type: 'java-version',
      title: 'Không tương thích phiên bản Java',
      description: `Một số mod hoặc phiên bản Minecraft này được biên dịch bằng ${requiredJava}, nhưng bạn đang khởi chạy bằng phiên bản Java cũ hơn.`,
      solution: `Vào Cài đặt phiên bản -> Tab Java -> Chọn đúng ${requiredJava} tương thích để khắc phục lỗi.`,
      highlight: requiredJava,
    }
  }

  // 3. Duplicate Mods Found
  const dupMatch = log.match(
    /(?:DuplicateModsFoundException|Duplicate mods found|Found duplicate mod|Duplicate mod):?\s*([a-zA-Z0-9_.-]+)/i
  )
  if (dupMatch) {
    const modName = dupMatch[1]
    return {
      type: 'duplicate-mod',
      title: 'Trùng lặp Mod trong thư mục',
      description: `Phát hiện có 2 file mod cùng loại hoặc trùng phiên bản đang cùng được nạp: "${modName}".`,
      solution: `Mở thư mục Mods của Instance và xóa bớt một trong hai file mod bị trùng.`,
      highlight: modName,
    }
  }

  // 4. Mod Conflict / Mixin Error
  const mixinMatch =
    log.match(/(?:Mixin apply for mod|caused by mod|from mod|Failed to load mod:?|Incompatible mods found!.*(?:Mod|mod) ')\s*([a-zA-Z0-9_.-]+)/i) ||
    log.match(/net\.minecraftforge\.fml\.common\.LoaderExceptionModCrash: Caught exception from ([a-zA-Z0-9_.-]+)/i) ||
    log.match(/org\.spongepowered\.asm\.mixin\.transformer\.throwables\.MixinTransformerError:.*caused by (?:plugin|mod) ([a-zA-Z0-9_.-]+)/i) ||
    log.match(/Critical injection failure:.*in ([a-zA-Z0-9_.-]+)\.mixins\.json/i)

  if (mixinMatch) {
    const offendingMod = mixinMatch[1]
    return {
      type: 'mod-conflict',
      title: `Xung đột Mod: ${offendingMod}`,
      description: `Lỗi khởi động do xung đột thư viện (Mixin/Loader) bắt nguồn từ mod "${offendingMod}".`,
      solution: `Hãy thử tắt hoặc cập nhật mod "${offendingMod}" sang phiên bản mới tương thích hơn với modloader hiện tại.`,
      highlight: offendingMod,
    }
  }

  // 5. GPU Driver / OpenGL Error
  if (
    /GLFW error 65542|Pixel format not accelerated|WGL: The driver does not appear to support OpenGL|org\.lwjgl\.opengl\.OpenGLException/i.test(log)
  ) {
    return {
      type: 'gpu-driver',
      title: 'Lỗi Card màn hình / Trình điều khiển OpenGL',
      description: 'Máy tính của bạn không khởi tạo được ngữ cảnh đồ họa OpenGL do thiếu hoặc lỗi Driver Card màn hình (GPU).',
      solution: 'Hãy cập nhật Driver Card màn hình mới nhất từ trang chủ NVIDIA, AMD hoặc Intel; hoặc cài đặt OpenGL Compatibility Pack.',
    }
  }

  return undefined
}
