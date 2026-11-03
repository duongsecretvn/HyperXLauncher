# Hướng dẫn đóng góp (Contributing Guidelines)

Cảm ơn bạn đã quan tâm và muốn đóng góp cho dự án **HyperXLauncher**!

## 1. Môi trường phát triển (Development Setup)

- **Node.js**: `>= 22.16.0`
- **pnpm**: Khuyên dùng `pnpm` phiên bản mới nhất
- **IDE**: Visual Studio Code (khuyên dùng kèm extension Volar và ESLint)

### Bắt đầu:

```bash
# Clone dự án
git clone https://github.com/duongsecretvn/HyperXLauncher.git
cd HyperXLauncher

# Cài đặt dependencies
pnpm install

# Biên dịch các package
pnpm compile
```

## 2. Cấu trúc dự án (Architecture)

- `xmcl-electron-app`: Ứng dụng Electron chính (Main Process & Packaging).
- `xmcl-keystone-ui`: Giao diện người dùng (Renderer Process viết bằng Vue 3, Vuetify 3, Vite).
- `xmcl-runtime`: Logic backend nền tảng (Service layer, Quản lý game, Quản lý tiến trình).
- `xmcl-runtime-api`: Định nghĩa TypeScript Interfaces, State và IPC contracts giữa Main và Renderer.

## 3. Quy trình gửi đóng góp (Workflow)

1. **Fork** repository và tạo một nhánh tính năng mới từ nhánh `main`:
   ```bash
   git checkout -b feature/ten-tinh-nang
   ```
2. Thực hiện các chỉnh sửa code và đảm bảo code chạy không có lỗi:
   ```bash
   pnpm check
   ```
3. Commit thay đổi tuân theo quy chuẩn [Conventional Commits](https://www.conventionalcommits.org/):
   ```bash
   git commit -m "feat: mô tả tính năng mới"
   ```
4. Push nhánh của bạn lên GitHub và tạo một **Pull Request** về repo chính `duongsecretvn/HyperXLauncher`.
