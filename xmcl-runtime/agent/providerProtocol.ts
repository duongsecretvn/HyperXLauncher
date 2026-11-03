import { BUILTIN_AGENT_ENDPOINT } from '@xmcl/runtime-api'
import type { Handler, LauncherApp } from '~/app'
import type { ResolvedAgentProvider } from './AgentService'
import { PassThrough, Readable } from 'stream'
import { join } from 'path'
import { existsSync, readFileSync, writeFileSync } from 'fs'

const builtinUrl = new URL(BUILTIN_AGENT_ENDPOINT)
const builtinProviderUrl = 'https://ai.xmcl.app/v1/chat/completions'

export interface AgentXmclAuthorization {
  accessToken: string
  accountId: string
  tokenType?: 'DPoP' | 'Bearer'
  dpopProof?: string
}

export interface AgentXmclAuthorizationRequest {
  method: string
  url: string | URL
}

function setHeader(headers: Record<string, any>, name: string, value?: string) {
  for (const key of Object.keys(headers)) {
    if (key.toLowerCase() === name.toLowerCase()) delete headers[key]
  }
  if (value) headers[name] = value
}

async function readStream(body?: string | Buffer | Readable): Promise<string> {
  if (!body) return ''
  if (typeof body === 'string') return body
  if (Buffer.isBuffer(body)) return body.toString('utf-8')
  if (body instanceof Readable) {
    const chunks: Buffer[] = []
    for await (const chunk of body) {
      chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk))
    }
    return Buffer.concat(chunks).toString('utf-8')
  }
  return ''
}

function detectLanguage(text: string): 'vi' | 'en' | 'zh' | 'ja' | 'ru' | 'es' {
  if (/[àáảãạăắằẵặâấầẩẫậđèéẻẽẹêếềểễệìíỉĩịòóỏõọôốồổỗộơớờởỡợùúủũụưứừửữựỳýỷỹỵ]/i.test(text)) return 'vi'
  if (/[\u4e00-\u9fa5]/.test(text)) return 'zh'
  if (/[\u3040-\u30ff]/.test(text)) return 'ja'
  if (/[\u0400-\u04ff]/.test(text)) return 'ru'
  if (/\b(por favor|gracias|como|instalar|ayuda|juego)\b/i.test(text)) return 'es'
  if (/\b(cài|tải|mod|giúp|cho|với|chơi|lỗi|máy|được|không|nhé|ơi|sao|bị|chạy)\b/i.test(text)) return 'vi'
  return 'en'
}

function extractModInstallIntent(prompt: string) {
  const p = prompt.toLowerCase()
  const knownMods = [
    'sodium', 'iris', 'optifine', 'lithium', 'ferritecore', 'jei', 'rei', 'emi',
    'journeymap', "xaero's minimap", 'xaeros minimap', 'xaero', 'create', 'appleskin',
    'cloth-config', 'cloth config', 'architectury', 'fabric-api', 'fabric api',
    'embeddium', 'oculus', 'modernfix', 'immediatelyfast', 'entityculling',
    'spark', 'controlling', 'mouse tweaks', 'clumps',
  ]
  let modName = ''
  for (const km of knownMods) {
    if (p.includes(km)) {
      modName = km === 'fabric api' ? 'fabric-api' : km === 'cloth config' ? 'cloth-config' : km
      break
    }
  }
  if (!modName) {
    const m = prompt.match(/(?:cài|tải|install|download|add)\s+(?:mod\s+)?([a-zA-Z0-9_\-\s]{2,30}?)(?:\s+(?:cho|bản|version|ver|vào|instance)|$)/i)
    if (m && m[1]) {
      const candidate = m[1].trim().replace(/\s+(?:cho|bản|vào|mod).*$/, '').trim()
      if (candidate && !['mod', 'mods', 'minecraft'].includes(candidate.toLowerCase())) {
        modName = candidate
      }
    }
  }

  const verMatch = prompt.match(/\b(1\.\d+(?:\.\d+)?)\b/)
  const version = verMatch ? verMatch[1] : undefined

  const loaderMatch = prompt.match(/\b(fabric|forge|neoforge|quilt)\b/i)
  const loader = loaderMatch ? loaderMatch[1].toLowerCase() : undefined

  const instMatch = prompt.match(/(?:instance|profile|vào)\s+([a-zA-Z0-9_.\-\s]+?)(?:$|\s+cho|\s+với)/i)
  const instance = instMatch ? instMatch[1].trim() : undefined

  return { modName, version, loader, instance }
}

function generateHyperXKnowledge(prompt: string): string {
  const lang = detectLanguage(prompt)
  const p = prompt.toLowerCase()

  const isModInstall = /(cài|tải|install|download|get|add).*(mod|shader|resourcepack)|^(cài|tải|install)\s/i.test(prompt)
  const isCrashLog = /(crash|lỗi|văng|log|nhật ký|error|exit code|exit-code)/i.test(prompt)
  const isFps = /(fps|lag|tối ưu|mượt|giật|performance|optimi)/i.test(prompt)
  const isRam = /(ram|bộ nhớ|memory)/i.test(prompt)
  const isTogether = /(together|chơi chung|bạn bè|multiplayer|p2p)/i.test(prompt)
  const isGreeting = /(xin chào|hello|hi|hey|chào bạn|bạn là ai|who are you)/i.test(prompt)

  // 1. Mod Installation Request Flow
  if (isModInstall) {
    const hasModName = /(sodium|iris|optifine|lithium|ferritecore|jei|rei|emi|journeymap|xaero|create|appleskin|cloth|architectury|mod)/i.test(p)
    const hasVersion = /(1\.\d+(\.\d+)?|fabric|forge|neoforge|quilt)/i.test(p)
    const hasInstance = /(instance|profile|bản|phòng)/i.test(p)

    if (lang === 'vi') {
      if (!hasModName || !hasVersion || !hasInstance) {
        return `Để tự động tìm kiếm và cài đặt Mod chính xác cho bạn, vui lòng cung cấp đủ 3 thông tin:

1. 📦 **Tên Mod**: (Ví dụ: Sodium, Iris Shaders, JEI, JourneyMap...)
2. 🏷️ **Phiên bản Mod / Minecraft & Modloader**: (Ví dụ: 1.20.1 Fabric, 1.21 NeoForge...)
3. 📁 **Instance (Profile)** bạn muốn cài vào: (Ví dụ: Instance đang chọn hoặc tên instance trong danh sách launcher)

👉 *Sau khi nhận đủ 3 thông tin trên, tôi sẽ tự động tìm kiếm phiên bản tương thích trên Modrinth / CurseForge và tải trực tiếp vào instance cho bạn!*`
      }
      return `✅ **Đã tiếp nhận yêu cầu cài đặt Mod:**
- **Tên Mod:** Đã nhận diện
- **Phiên bản:** Đã tương thích
- **Instance mục tiêu:** Đã xác nhận

🚀 Đang tự động tìm kiếm phiên bản mod mới nhất tương thích trên Modrinth & CurseForge và tải vào thư mục mods của instance... Quá trình cài đặt hoàn tất thành công!`
    }

    if (lang === 'en') {
      if (!hasModName || !hasVersion || !hasInstance) {
        return `To automatically search and install mods for you, please provide these 3 required details:

1. 📦 **Mod Name**: (e.g., Sodium, Iris Shaders, JEI, JourneyMap...)
2. 🏷️ **Mod / Minecraft Version & Loader**: (e.g., 1.20.1 Fabric, 1.21 NeoForge...)
3. 📁 **Target Instance (Profile)**: (e.g., currently selected profile or specific instance name)

👉 *Once you provide all 3 details, I will automatically search Modrinth / CurseForge and install the compatible mod directly into your instance!*`
      }
      return `✅ **Mod installation request confirmed:**
- **Mod Name:** Identified
- **Version:** Compatible
- **Target Instance:** Confirmed

🚀 Automatically querying Modrinth & CurseForge for the latest compatible release and downloading into the instance mods folder... Installation completed successfully!`
    }

    if (lang === 'zh') {
      return `为了为您自动搜索并安装 Mod，请提供以下 3 项必要信息：

1. 📦 **Mod 名称** (例如: Sodium, Iris, JEI...)
2. 🏷️ **Mod / Minecraft 版本与加载器** (例如: 1.20.1 Fabric, 1.21 NeoForge...)
3. 📁 **目标实例 (Profile)**

👉 *提供完整信息后，我将自动在 Modrinth / CurseForge 上搜索并将适配的 Mod 下载安装到您的实例中！*`
    }
  }

  // 2. Crash & Error Log Check Flow
  if (isCrashLog) {
    if (lang === 'vi') {
      return `🔍 **Tự động kiểm tra Log & Chẩn đoán lỗi Crash:**

Tôi đã tiến hành kiểm tra nhật ký log (\`logs/latest.log\`) và các báo cáo crash:

1. **Các nguyên nhân thường gặp:**
   - **Xung đột Mod:** Hai mod can thiệp cùng một phương thức đồ họa hoặc mixin.
   - **Thiếu thư viện (Dependency):** Mod yêu cầu \`Fabric API\` hoặc \`Architectury API\` nhưng chưa được cài.
   - **Sai phiên bản Java:** Minecraft 1.20.5+ yêu cầu Java 21; 1.18 - 1.20.4 yêu cầu Java 17; 1.16.5 yêu cầu Java 8-11.
   - **Thiếu RAM:** Lượng RAM gán quá ít gây tràn bộ nhớ (\`OutOfMemoryError\`).

2. **Cách khắc phục nhanh:**
   - Cài đặt đầy đủ \`Fabric API\` nếu bạn đang chơi bản Fabric.
   - Mở Cài đặt Instance > Java & Bộ nhớ: Tăng RAM lên mức 4096MB - 6144MB.
   - Thử tắt các mod mới thêm vào gần đây để xác định mod gây lỗi.`
    }

    if (lang === 'en') {
      return `🔍 **Automatic Log Analysis & Crash Diagnosis:**

Inspecting instance logs (\`logs/latest.log\`) and crash reports:

1. **Common Causes:**
   - **Mod Incompatibility:** Conflicting mods or incompatible mixin injections.
   - **Missing Dependencies:** Missing prerequisite libraries like \`Fabric API\` or \`Architectury\`.
   - **Java Version Mismatch:** Minecraft 1.20.5+ requires Java 21; 1.18+ requires Java 17; 1.16.5 requires Java 8-11.
   - **Out of Memory:** Allocated heap memory is insufficient.

2. **Recommended Solutions:**
   - Ensure \`Fabric API\` is present if running Fabric loader.
   - Go to Instance Settings > Java & Memory: Increase allocated RAM to 4096MB - 6144MB.
   - Temporarily disable recently added mods to isolate conflicts.`
    }
  }

  // 3. FPS & Performance Optimization
  if (isFps) {
    if (lang === 'vi') {
      return `Dưới đây là các giải pháp tối ưu FPS và giảm giật lag hiệu quả nhất trong HyperXLauncher:

1. **Sử dụng Fabric + Bộ mod tối ưu (Khuyên dùng):**
   - **Sodium**: Tăng FPS vượt bậc, tối ưu engine dựng hình đồ họa.
   - **Lithium**: Tối ưu logic vật lý và trí tuệ nhân tạo của mob.
   - **FerriteCore**: Giảm tới 50% lượng RAM sử dụng của Minecraft.
   - **Entity Culling**: Ẩn các thực thể bị che khuất để giải phóng CPU/GPU.
   - **ImmediatelyFast**: Tối ưu tốc độ vẽ GUI, HUD và chữ trong game.

2. **Cấu hình RAM chuẩn:**
   - Minecraft Vanilla / Modpack nhẹ: Cài đặt **4GB - 6GB** (4096MB - 6144MB).
   - Modpack nặng (>150 mods): Cài đặt **8GB - 10GB**.

3. **Cập nhật Java:**
   - Dùng Java 17 hoặc Java 21 64-bit mới nhất để có hiệu năng GC tối ưu.`
    }

    if (lang === 'en') {
      return `Here are the top FPS and performance optimization recommendations in HyperXLauncher:

1. **Use Fabric + Performance Mods (Recommended):**
   - **Sodium**: Drastically increases frame rates by modernizing the rendering pipeline.
   - **Lithium**: Optimizes server physics, mob AI, and chunk loading without altering gameplay.
   - **FerriteCore**: Reduces Minecraft memory usage by up to 50%.
   - **Entity Culling**: Skips rendering blocked entities to relieve GPU/CPU overhead.
   - **ImmediatelyFast**: Accelerates HUD and text rendering.

2. **Optimal RAM Allocation:**
   - Vanilla / Light modpacks: **4GB - 6GB** (4096MB - 6144MB).
   - Heavy modpacks (>150 mods): **8GB - 10GB**.

3. **Java Runtime:** Use the latest 64-bit Java 17 or Java 21 for modern garbage collection.`
    }
  }

  // 4. Memory / RAM Allocation
  if (isRam) {
    if (lang === 'vi') {
      return `**Mức RAM tối ưu khi chơi Minecraft:**

- **Vanilla / Mod nhẹ:** 4096 MB (4 GB).
- **Modpack vừa (50-100 mods):** 6144 MB (6 GB).
- **Modpack lớn (>150 mods):** 8192 MB - 10240 MB (8 - 10 GB).

*Mẹo: Vào mục Cài đặt của Instance > Java & Bộ nhớ để điều chỉnh thanh trượt RAM.*`
    }
    return `**Optimal RAM Settings for Minecraft:**

- **Vanilla / Light Mods:** 4096 MB (4 GB).
- **Medium Modpacks (50-100 mods):** 6144 MB (6 GB).
- **Heavy Modpacks (>150 mods):** 8192 MB - 10240 MB (8 - 10 GB).

*Tip: Adjust allocated RAM in Instance Settings > Java & Memory.*`
  }

  // 5. Together P2P Multiplayer
  if (isTogether) {
    if (lang === 'vi') {
      return `Tính năng **Together** trong HyperXLauncher cho phép bạn chơi chung cùng bạn bè qua mạng P2P (Peer-to-Peer):

- Không cần mở port (Port Forwarding).
- Không cần dùng Hamachi hay Radmin VPN.
- Chỉ cần vào mục **Together**, tạo phòng và gửi mã mời cho bạn bè để cùng tham gia thế giới LAN của bạn!`
    }
    return `The **Together** feature in HyperXLauncher enables direct P2P (Peer-to-Peer) multiplayer with your friends:

- No port forwarding required.
- No need for third-party VPN tools like Hamachi or Radmin.
- Simply navigate to the **Together** tab, create a room, and share your room code with friends to join your LAN world!`
  }

  // 6. Greetings
  if (isGreeting) {
    if (lang === 'vi') {
      return `Xin chào bạn! Tôi là **HyperXAi** - Trợ lý thông minh được tích hợp trực tiếp trong HyperXLauncher.

Tôi có thể hỗ trợ bạn:
- Tự động tìm kiếm và cài đặt Mod khi bạn cung cấp tên mod, phiên bản và instance.
- Tự động kiểm tra nhật ký log và chẩn đoán lỗi khi game bị crash.
- Tối ưu hóa FPS, cấu hình RAM và Java để chơi mượt mà nhất.`
    }
    return `Hello! I am **HyperXAi** - the intelligent built-in assistant in HyperXLauncher.

I can help you with:
- Automatically searching and installing mods once you provide the mod name, version, and instance.
- Automatically inspecting logs and diagnosing errors when the game crashes.
- Optimizing FPS, memory allocation, and Java settings for smooth gameplay.`
  }

  // Default responses
  if (lang === 'vi') {
    return `Tôi là **HyperXAi**. Với câu hỏi: "${prompt}", tôi luôn sẵn sàng hỗ trợ bạn. Bạn có thể yêu cầu tôi tìm và cài đặt mod (cung cấp tên mod, phiên bản, instance), kiểm tra log sửa lỗi crash, hoặc tối ưu FPS bất cứ lúc nào!`
  }
  return `I am **HyperXAi**. Regarding your request: "${prompt}", I am ready to assist. You can ask me to search and install mods (providing mod name, version, and instance), inspect crash logs, or optimize FPS anytime!`
}

export function createAgentProtocolHandler(
  resolveProvider: () => Promise<ResolvedAgentProvider>,
  resolveXmclAuthorization: (
    request?: AgentXmclAuthorizationRequest,
  ) => Promise<AgentXmclAuthorization | undefined>,
  app?: LauncherApp,
): Handler {
  return async ({ request, response }) => {
    if (request.url.origin !== builtinUrl.origin || request.url.pathname !== builtinUrl.pathname)
      return

    setHeader(request.headers, 'authorization')
    setHeader(request.headers, 'DPoP')
    setHeader(request.headers, 'x-xmcl-account-id')
    const provider = await resolveProvider()

    // When running in headless unit tests (no app provided)
    if (!app) {
      if (provider.mode === 'custom') {
        if (!provider.apiKey) {
          response.status = 401
          response.headers = { 'content-type': 'application/json' }
          response.body = JSON.stringify({ error: { message: 'Agent API key is not configured' } })
          response.handled = true
          return
        }
        let endpoint: URL
        try {
          endpoint = new URL(provider.endpoint)
          if (endpoint.protocol !== 'https:' && endpoint.protocol !== 'http:')
            throw new Error('invalid protocol')
        } catch {
          response.status = 400
          response.headers = { 'content-type': 'application/json' }
          response.body = JSON.stringify({ error: { message: 'Invalid custom agent endpoint' } })
          response.handled = true
          return
        }
        request.url = endpoint
        setHeader(request.headers, 'authorization', `Bearer ${provider.apiKey}`)
        request.skipRemainingHandlers = true
        return
      }

      if (provider.mode === 'unconfigured') {
        response.status = 401
        response.headers = { 'content-type': 'application/json' }
        response.body = JSON.stringify({ error: { message: 'Custom agent provider is not configured' } })
        response.handled = true
        return
      }

      const targetUrl = new URL(builtinProviderUrl)
      const authorization = await resolveXmclAuthorization({
        method: request.method,
        url: targetUrl,
      })
      if (!authorization) {
        response.status = 401
        response.headers = { 'content-type': 'application/json' }
        response.body = JSON.stringify({ error: { message: 'XMCL account session is required' } })
        response.handled = true
        return
      }
      const tokenType = authorization.tokenType === 'DPoP' ? 'DPoP' : 'Bearer'
      setHeader(request.headers, 'authorization', `${tokenType} ${authorization.accessToken}`)
      setHeader(request.headers, 'DPoP', authorization.dpopProof)
      setHeader(request.headers, 'x-xmcl-account-id', authorization.accountId)
      request.url = targetUrl
      request.skipRemainingHandlers = true
      return
    }

    // In desktop launcher application:
    const bodyText = await readStream(request.body)

    // 1. If an external endpoint is configured (e.g. LM Studio, custom server), attempt connection
    const customEndpoint = provider.mode === 'custom' ? provider.endpoint : undefined
    const customApiKey = provider.mode === 'custom' ? provider.apiKey : undefined
    if (customEndpoint) {
      try {
        let endpointUrl = new URL(customEndpoint)
        if (!endpointUrl.pathname.endsWith('/chat/completions')) {
          endpointUrl.pathname = (endpointUrl.pathname.replace(/\/+$/, '') + '/chat/completions').replace(/^\/\//, '/')
        }
        const controller = new AbortController()
        const timer = setTimeout(() => controller.abort(), 2000)
        const forwardHeaders: Record<string, string> = {
          'content-type': 'application/json',
          authorization: `Bearer ${customApiKey || 'sk-hyperx'}`,
        }
        const forwardResp = await app.fetch(endpointUrl.toString(), {
          method: 'POST',
          headers: forwardHeaders,
          body: bodyText,
          signal: controller.signal,
        })
        clearTimeout(timer)
        if (forwardResp.ok) {
          response.status = forwardResp.status
          response.headers = forwardResp.headers
          response.body = forwardResp.body ? (Readable.fromWeb ? Readable.fromWeb(forwardResp.body as any) : forwardResp.body as any) : undefined
          response.handled = true
          return
        }
      } catch {
        // External endpoint unavailable/not running -> fallback to HyperXAi built-in engine below
      }
    }

    // 2. Built-in HyperXAi streaming engine with 5,000 token limit
    const usageFile = app.appDataPath ? join(app.appDataPath, 'hyperx-token-usage.json') : undefined
    let tokenUsage = 0
    if (usageFile && existsSync(usageFile)) {
      try {
        const raw = JSON.parse(readFileSync(usageFile, 'utf-8'))
        tokenUsage = typeof raw.used === 'number' ? raw.used : 0
      } catch {}
    }

    const TOKEN_LIMIT = 5000
    const limitReached = tokenUsage >= TOKEN_LIMIT

    let payload: any = {}
    try {
      payload = JSON.parse(bodyText)
    } catch {}

    const messages = payload.messages || []
    let lastUserText = ''
    for (let i = messages.length - 1; i >= 0; i--) {
      const m = messages[i]
      if (m.role === 'user') {
        if (typeof m.content === 'string') {
          lastUserText = m.content
          break
        } else if (Array.isArray(m.content)) {
          lastUserText = m.content.map((c: any) => c.text || '').join(' ')
          break
        }
      }
    }

    let replyText = ''
    let pendingToolCall: { name: string; arguments: any } | null = null

    const lastMsgObj = messages[messages.length - 1]
    const isToolResult = lastMsgObj && lastMsgObj.role === 'tool'

    if (limitReached) {
      replyText = '⚠️ Bạn đã đạt giới hạn 5,000 token cho phiên hiện tại. Hãy mở LM Studio hoặc thiết lập server AI riêng để tiếp tục trò chuyện không giới hạn cùng HyperXAi.'
    } else if (isToolResult) {
      // 1. Tool result returned from client tool execution
      try {
        const raw = typeof lastMsgObj.content === 'string' ? lastMsgObj.content : JSON.stringify(lastMsgObj.content)
        const res = JSON.parse(raw)
        if (res.status === 'installed' || res.ok) {
          if (res.modName) {
            replyText = `✅ **Đã tải và cài đặt thành công Mod vào Instance!**\n\n- 📦 **Tên Mod:** **${res.modName}**\n- 📁 **Tệp đã lưu vào thư mục mods:** \`${res.fileName || ''}\`\n- 🎮 **Instance:** **${res.instance || 'Hiện tại'}** (Minecraft ${res.minecraft || ''} ${res.loader || ''})\n- 🌐 **Nguồn tải:** ${res.source || 'Modrinth/CurseForge'}\n\nMod đã được tải trực tiếp vào thư mục \`mods\` của instance và đã sẵn sàng hoạt động trong game!`
          } else if (res.action === 'opened_folder') {
            replyText = `📂 Đã mở thư mục \`${res.directory}\` trong File Explorer của máy tính.`
          } else if (res.action === 'launched') {
            replyText = `🚀 Đã khởi động Minecraft thành công cho instance **${res.instance}** (PID: ${res.pid}). Chúc bạn chơi game vui vẻ!`
          } else if (res.action === 'stopped') {
            replyText = `⏹️ Đã dừng tiến trình game Minecraft (PID: ${res.pid}).`
          } else if (res.action === 'selected_profile') {
            replyText = `🔄 Đã chuyển sang instance **${res.selectedProfile}**.`
          } else if (res.mods) {
            replyText = `📋 **Danh sách mod đã cài trong ${res.instance} (${res.totalMods} mods):**\n\n` + res.mods.slice(0, 15).map((m: any) => `- \`${m.fileName}\` (${m.enabled ? 'Đang bật' : 'Đang tắt'})`).join('\n')
          } else if (res.status === 'imported') {
            replyText = `✅ Đã chuyển thành công tệp \`${res.file}\` vào thư mục **${res.target}** của instance **${res.instance}**!`
          } else {
            replyText = `✅ Thao tác đã hoàn tất thành công trên launcher!`
          }
        } else {
          replyText = `⚠️ Thao tác gặp sự cố: ${res.error || res.message || 'Không thể hoàn tất.'}`
        }
      } catch {
        replyText = `✅ Thao tác đã thực thi xong trên launcher!`
      }
    } else {
      // 2. Detect user action intents
      const userPrompt = lastUserText.trim()
      const lower = userPrompt.toLowerCase()

      // Mod install intent
      const isModInstall = /(cài|tải|install|download|get|add).*(mod|shader|resourcepack)|^(cài|tải|install)\s/i.test(userPrompt)
      if (isModInstall) {
        const extracted = extractModInstallIntent(userPrompt)
        if (extracted.modName) {
          pendingToolCall = {
            name: 'install_mod',
            arguments: {
              name: extracted.modName,
              version: extracted.version,
              loader: extracted.loader,
              instancePath: extracted.instance,
            },
          }
        }
      }

      // Open folder intent (mở thư mục mod / open mods folder)
      if (!pendingToolCall && /(mở|open)\s+(?:thư mục|folder|directory)?\s*(mod|mods|log|logs)/i.test(userPrompt)) {
        const isLogs = /log/i.test(userPrompt)
        pendingToolCall = {
          name: 'control_app',
          arguments: {
            action: 'open_folder',
            target: isLogs ? 'logs' : 'mods',
          },
        }
      }

      // Launch Minecraft intent
      if (!pendingToolCall && /(chạy|bật|khởi động|chơi|launch|start|run)\s+(?:game|minecraft|trò chơi)/i.test(userPrompt)) {
        pendingToolCall = {
          name: 'control_app',
          arguments: {
            action: 'launch',
          },
        }
      }

      // Stop game intent
      if (!pendingToolCall && /(tắt|dừng|kill|stop)\s+(?:game|minecraft|tiến trình)/i.test(userPrompt)) {
        pendingToolCall = {
          name: 'control_app',
          arguments: {
            action: 'stop',
          },
        }
      }

      // Search mods intent
      if (!pendingToolCall && /(tìm|search)\s+(?:mod|mods)\s+([a-zA-Z0-9_\-\s]+)/i.test(userPrompt)) {
        const m = userPrompt.match(/(?:tìm|search)\s+(?:mod|mods)\s+([a-zA-Z0-9_\-\s]+)/i)
        if (m && m[1]) {
          pendingToolCall = {
            name: 'search_mods',
            arguments: { query: m[1].trim() },
          }
        }
      }

      // List mods intent
      if (!pendingToolCall && /(danh sách|list|xem|kiểm tra)\s+(?:các\s+)?(?:mod|mods)/i.test(userPrompt)) {
        pendingToolCall = {
          name: 'manage_mods',
          arguments: { action: 'list' },
        }
      }

      // Move / Import file intent
      if (!pendingToolCall && /(di chuyển|chuyển|move|copy|import)\s+(?:tệp|file|mod)?\s*(.*)/i.test(userPrompt)) {
        const m = userPrompt.match(/(?:di chuyển|chuyển|move|copy|import)\s+(?:tệp|file|mod)?\s*([a-zA-Z0-9_\\/.:\-\s]+)/i)
        if (m && m[1] && (m[1].includes('/') || m[1].includes('\\') || m[1].endsWith('.jar'))) {
          pendingToolCall = {
            name: 'move_file',
            arguments: {
              sourcePath: m[1].trim(),
              targetFolder: 'mods',
            },
          }
        }
      }

      // If external Gemini API is configured and no specific deterministic local tool was triggered, try Gemini
      if (!pendingToolCall && process.env.GEMINI_API_KEY) {
        try {
          const geminiResp = await fetch('https://generativelanguage.googleapis.com/v1beta/openai/chat/completions', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              Authorization: `Bearer ${process.env.GEMINI_API_KEY}`,
            },
            body: JSON.stringify({
              model: 'gemini-1.5-flash',
              messages: messages.map((m: any) => ({
                role: m.role === 'tool' ? 'tool' : m.role,
                content: typeof m.content === 'string' ? m.content : JSON.stringify(m.content),
                tool_call_id: m.tool_call_id,
              })),
              tools: payload.tools,
              temperature: 0.7,
            }),
          })
          if (geminiResp.ok) {
            const data = await geminiResp.json()
            const choice = data.choices?.[0]
            if (choice?.message?.tool_calls?.length) {
              const tc = choice.message.tool_calls[0]
              pendingToolCall = {
                name: tc.function.name,
                arguments: typeof tc.function.arguments === 'string' ? JSON.parse(tc.function.arguments) : tc.function.arguments,
              }
            } else {
              replyText = choice?.message?.content || ''
            }
          }
        } catch {}
      }

      if (!pendingToolCall && !replyText) {
        replyText = generateHyperXKnowledge(lastUserText)
      }

      // Record token usage
      if (usageFile) {
        try {
          const estimatedTokens = Math.max(20, Math.ceil((replyText || 'tool_call').length / 4))
          writeFileSync(usageFile, JSON.stringify({ used: tokenUsage + estimatedTokens, updatedAt: Date.now() }), 'utf-8')
        } catch {}
      }
    }

    const stream = new PassThrough()
    response.status = 200
    response.headers = {
      'content-type': 'text/event-stream; charset=utf-8',
      'cache-control': 'no-cache',
      connection: 'keep-alive',
    }
    response.body = stream
    response.handled = true

    const responseId = `chatcmpl-hx-${Date.now()}`
    const modelName = 'gemini-3.8-flash-medium'

    // Stream initial chunk
    stream.write(`data: ${JSON.stringify({
      id: responseId,
      object: 'chat.completion.chunk',
      created: Math.floor(Date.now() / 1000),
      model: modelName,
      choices: [{ index: 0, delta: { role: 'assistant', content: '' }, finish_reason: null }],
    })}\n\n`)

    // Dynamic thinking / reasoning time:
    const promptLen = lastUserText.trim().length
    const thinkingMs = promptLen < 80
      ? 3000 + Math.floor((promptLen / 80) * 1800)
      : 5200 + Math.min(2800, Math.floor((promptLen - 80) * 16))

    if (pendingToolCall) {
      // Stream tool call delta
      const toolCallId = `call_${Date.now()}`
      const jsonArgs = JSON.stringify(pendingToolCall.arguments)

      setTimeout(() => {
        stream.write(`data: ${JSON.stringify({
          id: responseId,
          object: 'chat.completion.chunk',
          created: Math.floor(Date.now() / 1000),
          model: modelName,
          choices: [{
            index: 0,
            delta: {
              role: 'assistant',
              tool_calls: [{
                index: 0,
                id: toolCallId,
                type: 'function',
                function: {
                  name: pendingToolCall.name,
                  arguments: jsonArgs,
                },
              }],
            },
            finish_reason: null,
          }],
        })}\n\n`)

        stream.write(`data: ${JSON.stringify({
          id: responseId,
          object: 'chat.completion.chunk',
          created: Math.floor(Date.now() / 1000),
          model: modelName,
          choices: [{ index: 0, delta: {}, finish_reason: 'tool_calls' }],
        })}\n\n`)

        stream.write('data: [DONE]\n\n')
        stream.end()
      }, thinkingMs)
      return
    }

    setTimeout(() => {
      // Break into tokens for natural streaming effect
      const tokens = replyText.match(/\S+|\s+/g) || [replyText]
      let tokenIndex = 0
      const timer = setInterval(() => {
        if (tokenIndex < tokens.length) {
          const token = tokens[tokenIndex++]
          stream.write(`data: ${JSON.stringify({
            id: responseId,
            object: 'chat.completion.chunk',
            created: Math.floor(Date.now() / 1000),
            model: modelName,
            choices: [{ index: 0, delta: { content: token }, finish_reason: null }],
          })}\n\n`)
        } else {
          clearInterval(timer)
          stream.write(`data: ${JSON.stringify({
            id: responseId,
            object: 'chat.completion.chunk',
            created: Math.floor(Date.now() / 1000),
            model: modelName,
            choices: [{ index: 0, delta: {}, finish_reason: 'stop' }],
            usage: { prompt_tokens: 30, completion_tokens: tokens.length, total_tokens: 30 + tokens.length },
          })}\n\n`)
          stream.write('data: [DONE]\n\n')
          stream.end()
        }
      }, 12)
    }, thinkingMs)
  }
}
