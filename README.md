# Lumine Desktop

Ứng dụng desktop nhỏ gọn (Rust + Tauri 2) bọc 2 website của Lumine vào **một cửa sổ có tab**:

- **Portal** — https://portal.luminecloud.com/
- **Control Panel** — https://cp.luminecloud.com/

Mỗi site chạy trong một webview riêng, nên:

- Đăng nhập / cookie của Portal và Panel **độc lập** với nhau.
- Chuyển tab **không tải lại trang**, không mất trạng thái đang làm dở.
- Dữ liệu phiên được giữ lại giữa các lần mở app (profile WebView2 riêng của app).

Ngoài ra app còn có:

- **Nhớ cửa sổ và tab**: vị trí/kích thước cửa sổ và tab đang mở ở phiên trước được khôi phục khi mở lại.
- **Giữ phiên đăng nhập**: session cookie của Portal/Panel được tự "làm bền" mỗi phút, nên tắt app mở lại vẫn còn đăng nhập (trong giới hạn thời gian phiên mà server cho phép).
- **Icon khay hệ thống (tray)**: click trái hiện cửa sổ, chuột phải có menu "Hiện cửa sổ Lumine" / "Thoát".
- **Hiệu ứng fade in** khi chuyển giữa hai tab.
- Nút tải lại, mở tab hiện tại bằng trình duyệt và bộ nút thu nhỏ / phóng to / đóng.

## Yêu cầu máy

- Windows 10/11 có **WebView2 Runtime** (máy đã cập nhật Edge/Windows là có sẵn; nếu app mở ra trắng trơn thì cài "Microsoft Edge WebView2 Runtime").
- **Node.js 18+** và npm.
- **Rust** (rustup, toolchain MSVC) + Visual Studio Build Tools với "Desktop development with C++" **và Windows SDK** (thiếu SDK sẽ báo lỗi `kernel32.lib` khi build).

## Chạy thử (dev)

```bash
npm install
npm run tauri dev
```

Lần đầu chạy sẽ biên dịch phần Rust nên mất vài phút; các lần sau mở nhanh hơn nhiều.

> Trên Windows, các lệnh `npm run tauri ...` và `npm run check` tự nạp môi trường Visual Studio qua `scripts/msvc-env.cmd` (dò bằng `vswhere`), nên chạy được cả trong Git Bash lẫn PowerShell mà không cần mở "Developer Command Prompt".

Kiểm tra nhanh phần Rust mà không mở app:

```bash
npm run check
```

## Build bản cài đặt

```bash
npm run tauri build
```

File cài nằm trong `src-tauri/target/release/bundle/` (NSIS `.exe` và `.msi` trên Windows).

## Cấu trúc chính

```
index.html                 Giao diện thanh tab + nút cửa sổ (chạy local)
src/main.ts                Bắt sự kiện tab, nút reload / mở trình duyệt / điều khiển cửa sổ
src/styles.css             Giao diện thanh tab (nền đen, accent vàng theo portal)
public/logo.webp           Logo cloud hiển thị trên thanh tab (thay file này để đổi logo)
src-tauri/src/lib.rs       Tạo 2 webview, lệnh chuyển tab, xử lý resize
src-tauri/tauri.conf.json  Cấu hình cửa sổ và đóng gói
src-tauri/capabilities/    Quyền mà giao diện shell được phép gọi
scripts/msvc-env.cmd       Tự nạp môi trường Visual Studio cho lệnh build
scripts/generate-icon.mjs  Sinh icon nguồn 1024px (chạy `npm run icon`)
```

## Đổi URL hoặc thêm tab

1. Sửa danh sách `SITES` trong `src-tauri/src/lib.rs`.
2. Thêm một nút `<button class="tab" data-tab="...">` vào `<nav class="tabs">` trong `index.html`.
3. Mở rộng `type TabId` trong `src/main.ts` cho khớp id mới.

Lưu ý: chiều cao thanh tab là **48px**, khai báo ở hai nơi — `TITLEBAR_HEIGHT` (Rust) và `--titlebar-height` (CSS). Nếu đổi kích thước thanh tab phải sửa cả hai.

## Ghi chú kỹ thuật

- App dùng **nhiều webview con trong cùng một cửa sổ** (Tauri feature `unstable`). Đây là cách duy nhất để giữ 2 site sống song song mà không cần 2 cửa sổ.
- App **không nhúng hay sửa nội dung** website: site tự chạy mã và chính sách bảo mật của chính nó. Lớp vỏ chỉ hiển thị và điều hướng tab.
- OAuth/captcha của site (nếu có) chạy trong webview Edge Chromium bình thường, không bị chặn như iframe.
- Icon app được sinh tự động từ `public/logo.webp`: chạy `npm run icon` mỗi khi thay logo (dùng thư viện sharp để chuyển webp → PNG 1024px rồi tạo bộ icon).

## Lỗi thường gặp

- **`LINK : fatal error LNK1181: cannot open input file 'kernel32.lib'`** hoặc **`link: extra operand`**: máy chưa có Windows SDK hoặc terminal đang gọi nhầm linker. Cài "Windows SDK" qua Visual Studio Installer → Individual components, sau đó chạy lại bằng các lệnh `npm run ...` (các lệnh này tự nạp môi trường qua `scripts/msvc-env.cmd`).
- **App mở lên trắng trơn**: thiếu WebView2 Runtime → cài "Microsoft Edge WebView2 Runtime".
- **Cổng 1420 đang bị chiếm**: đóng tiến trình Vite cũ trước khi chạy lại `npm run tauri dev`.

## Bảo mật

- Hai website chạy trong webview **không có quyền IPC nào**: không thể gọi lệnh của app, không điều khiển được cửa sổ (đã kiểm chứng bằng ACL của Tauri — capability chỉ áp dụng cho URL local của cửa sổ chính).
- Lớp vỏ không nhúng/chỉnh sửa nội dung website; phiên đăng nhập (cookie) lưu cục bộ trong profile WebView2 của app trên máy bạn, không có gì gửi đi nơi khác.

## Giấy phép

MIT — xem [LICENSE](LICENSE).
