<p align="center">
  <img alt="HyperXLauncher Logo" width="128" height="128" src="https://raw.githubusercontent.com/duongsecretvn/HyperXLauncher/main/xmcl-electron-app/icons/dark%40256x256.png">
</p>

<h1 align="center">HyperXLauncher</h1>

<p align="center">
  <strong>Trình khởi chạy Minecraft hiện đại, tối ưu hóa tốc độ và giao diện tùy biến cao.</strong>
</p>

<p align="center">
  <a href="https://github.com/duongsecretvn/HyperXLauncher/releases">
    <img src="https://img.shields.io/github/v/release/duongsecretvn/HyperXLauncher?color=blue&label=Phi%C3%AAn%20b%E1%BA%A3n" alt="Release">
  </a>
  <a href="https://github.com/duongsecretvn/HyperXLauncher/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/Gi%E1%BA%A5y%20ph%C3%A9p-MIT-green.svg" alt="License">
  </a>
  <a href="https://nodejs.org">
    <img src="https://img.shields.io/badge/Node.js-%3E%3D22.16.0-brightgreen" alt="Node.js">
  </a>
  <a href="https://pnpm.io">
    <img src="https://img.shields.io/badge/Package%20Manager-pnpm-orange" alt="pnpm">
  </a>
</p>

---

## 🌟 Tính năng nổi bật (Features)

- ⚡ **Khởi động siêu tốc & Tối ưu hóa JVM**: Tích hợp bộ cờ G1GC tối tân, kích hoạt ép buộc GPU rời hiệu năng cao (NVIDIA / AMD High-Performance Dedicated GPU), nâng mức ưu tiên CPU và cơ chế chống sập (anti-crash heap protection).
- 🎨 **Giao diện hiện đại & Độc quyền**: Thiết kế tối giản, hỗ trợ bộ chọn font chữ linh hoạt (kèm font pixel Minecraft đặc trưng), hiệu ứng chuyển cảnh mượt mà và giao diện tối (Dark theme).
- 🔄 **Cập nhật tự động (In-App Updates)**: Dễ dàng kiểm tra bản cập nhật mới nhất và tự động tải phiên bản mới trực tiếp bên trong launcher.
- 🧩 **Hỗ trợ Mod đa dạng**: Tương thích hoàn toàn với Modrinth, CurseForge, Fabric, Forge, NeoForge, Quilt và OptiFine.
- 🔐 **Quản lý tài khoản thông minh**: Đăng nhập nhanh tài khoản Microsoft và tài khoản Ngoại tuyến (Offline), chuyển đổi tài khoản chỉ với 1 cú click.
- 💻 **Đa nền tảng (Cross-Platform)**: Hoạt động mượt mà trên hệ điều hành Windows, Linux và macOS.

---

## 🚀 Cài đặt & Tải về (Download)

Tải phiên bản mới nhất tại mục [Releases](https://github.com/duongsecretvn/HyperXLauncher/releases):

- **Windows**: `HyperXLauncher-Setup-x64.exe` (hoặc bản Portable `.zip`)
- **Linux**: `HyperXLauncher-x86_64.AppImage` / `.deb`
- **macOS**: `HyperXLauncher-universal.dmg`

---

## 🛠️ Hướng dẫn Build từ Source (Build from Source)

### 1. Yêu cầu hệ thống
- **Node.js**: Phiên bản `>= 22.16.0`
- **pnpm**: Khuyên dùng `pnpm@11` (`corepack enable && corepack use pnpm@latest`)
- **Git**

### 2. Cài đặt môi trường
```bash
# Clone source code
git clone https://github.com/duongsecretvn/HyperXLauncher.git
cd HyperXLauncher

# Cài đặt toàn bộ dependencies
pnpm install

# Biên dịch các package lõi
pnpm compile
```

### 3. Chạy ở chế độ phát triển (Development)
```bash
# Terminal 1: Chạy Vite Renderer UI
pnpm dev:renderer

# Terminal 2: Chạy Electron Main Process
pnpm dev:main
```

### 4. Đóng gói ứng dụng (Build App)
```bash
# Đóng gói bộ cài đặt đầy đủ cho nền tảng hiện tại
pnpm build:all
```
File cài đặt hoàn thiện sẽ được xuất tại thư mục: `xmcl-electron-app/dist/`.

---

## 📄 Bản quyền (License)

Dự án được phân phối dưới giấy phép [MIT License](LICENSE).
Minecraft là thương hiệu đã đăng ký của Mojang Synergies AB / Microsoft. Dự án này không phải là sản phẩm chính thức và không được liên kết với Mojang hoặc Microsoft.
