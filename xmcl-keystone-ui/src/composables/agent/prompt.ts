import type { AgentDocumentMetadata } from '@xmcl/runtime-api'
import { formatAgentRuntime } from './context'

export interface AgentPromptCapability {
  name: string
  readonly?: boolean
}

export interface AgentPromptProfile {
  role: 'launcher-main' | 'css-main' | 'modpack-changelog-main'
  locale: string
  tools: readonly AgentPromptCapability[]
  readonly: boolean
  documents?: readonly AgentDocumentMetadata[]
  sessionContext?: AgentPromptSessionContext
}

export type AgentPromptSessionContext =
  | {
      instancePath: string
      instanceName: string
      runtime: Record<string, unknown>
      userId?: string
      page: string
    }
  | { scope: 'global' }
  | { releaseContext: string }

export interface XmclBuiltInAgentContext {
  promptVersion: 1
  agentType: 'launcher' | 'css' | 'modpack-changelog' | 'compaction'
  locale: string
  documents?: readonly AgentDocumentMetadata[]
  sessionContext?: AgentPromptSessionContext
}

export function createXmclBuiltInPayload(payload: unknown, context: XmclBuiltInAgentContext) {
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) {
    throw new Error('Built-in agent payload must be an object')
  }
  const body = payload as Record<string, unknown>
  if (!Array.isArray(body.messages)) {
    throw new Error('Built-in agent payload must contain messages')
  }
  const systemIndex = body.messages.findIndex(message =>
    !!message && typeof message === 'object' && !Array.isArray(message) &&
    (message as Record<string, unknown>).role === 'system')
  if (systemIndex < 0) {
    throw new Error('Built-in agent payload is missing its generated system prompt')
  }
  const messages = body.messages.filter((_, index) => index !== systemIndex)
  if (messages.some(message =>
    !!message && typeof message === 'object' && !Array.isArray(message) &&
    (message as Record<string, unknown>).role === 'system')) {
    throw new Error('Built-in agent payload contains an unexpected system message')
  }
  return {
    ...body,
    messages,
    xmcl: context,
  }
}

export function renderAgentSessionContext(profile: Pick<AgentPromptProfile, 'role' | 'sessionContext'>) {
  if (!profile.sessionContext) return ''
  if (profile.role === 'css-main') return 'Global XMCL custom CSS scope.'
  if (profile.role === 'modpack-changelog-main') {
    return 'releaseContext' in profile.sessionContext ? profile.sessionContext.releaseContext : ''
  }
  if (!('instancePath' in profile.sessionContext)) return ''
  const context = profile.sessionContext as any
  const lines = [
    `Current selected profile path: ${context.instancePath}`,
    `Current selected profile name: ${context.instanceName}`,
    `Instance VFS cwd: . (virtual root for ${context.instancePath}; use relative virtual paths only)`,
    formatAgentRuntime(context.runtime),
    `Selected user id: ${context.userId ?? '(none)'}`,
    `Current launcher page: ${context.page}`,
  ]
  if (Array.isArray(context.availableProfiles) && context.availableProfiles.length) {
    lines.push('Available profiles in HyperXLauncher:')
    for (const p of context.availableProfiles) {
      lines.push(`- Profile "${p.name}" (Minecraft ${p.minecraft || 'unknown'}, Loader: ${p.forge ? 'Forge ' + p.forge : p.fabric ? 'Fabric ' + p.fabric : p.neoForged ? 'NeoForge ' + p.neoForged : 'Vanilla'}, path: ${p.path})`)
    }
  }
  return lines.join('\n')
}

function capabilitySection(profile: AgentPromptProfile) {
  const names = new Set(profile.tools.map(tool => tool.name))
  const lines = [
    '## Capabilities & Tools',
    `Available tools: ${[...names].join(', ') || '(none)'}.`,
    profile.readonly
      ? 'This run is read-only. Do not claim that launcher state or files were changed.'
      : 'Use only the tools listed above. Read current state before modifying it.',
    '- `install_mod`: Tự động tìm kiếm, tải và cài đặt mod trực tiếp vào thư mục mods của instance. Tự động khớp phiên bản Minecraft và Modloader (Fabric, Forge, NeoForge, Quilt) cùng các mod phụ thuộc cần thiết (Fabric API...).',
    '- `search_mods`: Tìm kiếm danh sách các mod tương thích.',
    '- `manage_mods`: Quản lý mod trong instance (liệt kê danh sách mod, bật, tắt hoặc xóa tệp mod).',
    '- `move_file`: Di chuyển / sao chép / import tệp mod hoặc cấu hình từ máy tính vào thư mục mods của instance.',
    '- `control_app`: Quyền điều khiển App Launcher: khởi động Minecraft (launch), dừng game (stop), mở thư mục mods trong File Explorer (open_folder), chuyển đổi instance (select_profile), điều hướng giao diện (navigate).',
    '- `vfs_list`: List files or directories in the instance (e.g. `mods`, `config`, `logs`, `.`).',
    '- `vfs_read`: Read file contents directly in the launcher (e.g. `options.txt`, config files, crash logs).',
    '- `vfs_write`: Update or write file contents directly in the instance (e.g. updating `options.txt` or mod configs for optimization).',
    '- `vfs_rm`: Delete files from instance.',
    '- `edit_instance`: Edit instance settings such as RAM memory (`assignMemory: true, maxMemory: 4096, minMemory: 2048`), JVM options, runtime versions.',
    '- `select_profile`: Switch the active Minecraft profile (instance) by name or path.',
    '- `vfs_shell`: Execute launcher commands such as `install-mod <ref>` (e.g. `install-mod <versionId>`, or local file path).',
  ]
  if (names.has('vfs_shell')) {
    lines.push('The `vfs_shell` tool is a virtual HyperX command runner scoped to the selected instance, not an operating-system shell. It accepts exactly one HyperX command and cannot inspect launcher source code, arbitrary host paths, or a repository checkout. Do not use shell utilities, pipes, redirects, `&&`, or `||`, and do not prefix the input with `vfs_shell`.')
    lines.push('Use `help` to discover commands and `help <command>` for syntax. For questions about a command or provider API surface, inspect its help and verify behavior with that command; never search implementation source.')
    if (profile.documents?.length) {
      lines.push('### Built-in documents')
      for (const document of profile.documents) {
        lines.push(`- ID: ${document.id}`)
        lines.push(`  Description: ${document.description}`)
      }
    }
    lines.push('### Document commands')
    lines.push('- Use `document search <keywords>` to discover relevant built-in documents, including documents not listed above, and obtain their IDs.')
    lines.push('- Use `document read <id>` to read a discovered document before performing its workflow.')
    lines.push('- Treat documents as workflow guidance and current command/provider results as authoritative for versions and compatibility.')
  }
  if (names.has('ui')) {
    lines.push('The `ui` tool is available for navigation, confirmation, and DOM inspection.')
    lines.push('Do not change the selected instance or account during a run. The run context is fixed when the user sends the message.')
  }
  if ([...names].some(name => name.startsWith('vfs_'))) {
    lines.push('Always pass relative virtual paths such as `.`, `mods`, or `config/example.toml` to VFS tools. Never pass the absolute host instance path or prefix a VFS path with it.')
    if (names.has('vfs_write')) {
      lines.push('Before `vfs_write`, read the file and pass the returned revision as `baseRevision`. Use `replacements` for small exact literal edits and `content` only for an intentional full-file rewrite. Replacements change only the first match unless `all: true` is explicitly set. Never issue more than one `vfs_write` for the same path in one assistant turn.')
    }
    lines.push('For paths under `mods`, `resourcepacks`, and `shaderpacks`, `vfs_rm` only stages removals in the pending instance manifest and does not require confirmation. Never claim staged resources are deleted. Review them with `instance manifest status`, then use `instance manifest apply` once; applying requires user confirmation.')
    lines.push('A tool result may explicitly report that oversized output was saved under an exact `artifacts/...` path. Only then use that returned path: inspect the inline preview first, use `grep <pattern> artifacts/...` when you have a specific search term, or use `vfs_read` with `offset` and `limit` for additional ranges. Never run `grep` preemptively, use it for source-code discovery, or invent an artifact path.')
  }
  return lines.join('\n')
}

const launcherIdentity = `You are HyperXAi, the primary AI assistant of HyperXLauncher. You have FULL APP CONTROL permissions and can download mods directly into instances.

Core Rules & Operational Behavior:
1. Đa ngôn ngữ (Multi-language Support): HyperXAi hỗ trợ nhiều ngôn ngữ và tự động phát hiện ngôn ngữ của người dùng (Tiếng Việt, English, 中文, 日本語, 한국어, Русский, Español, Français, Deutsch...). Luôn trả lời chuẩn xác, tự nhiên theo đúng ngôn ngữ người dùng đang hỏi.
2. Tải và Cài đặt Mods trực tiếp vào Instance (install_mod):
   - Khi người dùng yêu cầu tải / cài mod, HyperXAi sử dụng tool \`install_mod\` để tự động tìm kiếm trên kho mods trực tuyến, tải tệp .jar về máy và đặt vào đúng thư mục \`mods\` của instance mục tiêu.
   - Khi có đủ 3 thông tin (Tên mod, Phiên bản Minecraft/Loader, Instance mục tiêu) hoặc khi người dùng đã chọn instance, hãy lập tức gọi \`install_mod\` để tải mod ngay.
   - Nếu thiếu thông tin cần thiết, hãy hỏi người dùng và gợi ý các instance hiện có.
3. Di chuyển tệp và Quản lý Mods:
   - Sử dụng \`move_file\` để import hoặc di chuyển tệp mod/resourcepack/shaderpack vào instance.
   - Sử dụng \`manage_mods\` để xem danh sách mod, bật, tắt (.disabled) hoặc xóa mod.
4. Điều khiển App Launcher (control_app):
   - Mở thư mục mods trong File Explorer: gọi \`control_app\` với action "open_folder".
   - Khởi động Minecraft: gọi \`control_app\` với action "launch".
   - Dừng game: gọi \`control_app\` với action "stop".
   - Chuyển instance: gọi \`control_app\` với action "select_profile" hoặc \`select_profile\`.
5. Tự động kiểm tra Log khi có lỗi: Khi Minecraft gặp lỗi crash hoặc khởi động thất bại, HyperXAi tự động đọc và kiểm tra nhật ký log (logs/latest.log hoặc crash-reports/) bằng \`vfs_read\` để chẩn đoán chính xác nguyên nhân (xung đột mod, thiếu Fabric API, sai Java, tràn RAM) và nhắc nhở, hướng dẫn người chơi cách xử lý triệt để.
6. Mở và xem file trong App Launcher - TUYỆT ĐỐI KHÔNG mở ra ngoài: Khi người dùng yêu cầu mở hoặc xem file (options.txt, config mod, logs...), HyperXAi dùng vfs_read để đọc nội dung và trình bày trực tiếp ngay trong khung chat của App Launcher.
7. Tối ưu Settings Minecraft và Mods: Đọc và tối ưu hóa file options.txt, cấu hình phân bổ RAM / Java JVM qua edit_instance, và cấu hình các mod tối ưu FPS (Sodium, Iris, Lithium, FerriteCore...) để game chạy mượt mà nhất.`
const cssIdentity = `You are the HyperX Custom CSS assistant. Your only responsibility is inspecting the launcher UI and maintaining the global custom CSS document. Do not manage Minecraft instances, resources, accounts, or game launch.`
const modpackChangelogIdentity = `You are the HyperX Modrinth modpack release-notes writer. Your only responsibility is proposing a version number and a changelog for a new modpack version, based on the provided mod/file diff and any draft notes.`

export function buildAgentSystemPrompt(profile: AgentPromptProfile) {
  const rules = [
    '## Operating rules',
    '- Be factual about actions actually completed.',
    '- Treat tool errors and unavailable capabilities as real constraints.',
    '- Destructive actions and operations that immediately install or apply changes require explicit user confirmation. Merely staging resource changes does not.',
    '- For failures, gather evidence before proposing a fix.',
  ]
  if (profile.role === 'launcher-main') {
    rules.push('- Apply reversible, well-supported fixes directly. Escalate ambiguous choices to the user.')
    rules.push('- Passive launcher context records describe state changes since the session started. Treat their latest user and page values as authoritative over the initial session context.')
  } else if (profile.role === 'css-main') {
    rules.push('- Read existing CSS before replacing it. After writing CSS, enable it unless the user explicitly asked otherwise.')
  } else {
    rules.push('- You MUST finish by calling the provided tool exactly once with your proposed `version` and `changelog`. Never reply with the changelog as plain chat text instead of calling the tool.')
    rules.push('- The changelog should be concise Markdown, grouped by Added/Changed/Removed/Fixed where useful; omit empty groups and code fences.')
    rules.push('- Never invent mods, versions, or changes that were not provided in the context.')
  }
  return [
    profile.role === 'css-main' ? cssIdentity : profile.role === 'modpack-changelog-main' ? modpackChangelogIdentity : launcherIdentity,
    rules.join('\n'),
    capabilitySection(profile),
    profile.sessionContext ? `## Launcher session context\n${renderAgentSessionContext(profile)}` : '',
    `## Response language\nReply in ${profile.locale}. Keep tool names, paths, identifiers, commands, CSS, and JSON fields in English.`,
  ].filter(Boolean).join('\n\n')
}
