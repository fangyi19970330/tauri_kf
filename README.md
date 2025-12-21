# 百度桌面应用

这是一个使用 Tauri 创建的百度桌面应用。

## 前置要求

1. 安装 Node.js (推荐 v16 或更高版本)
2. 安装 Rust (https://rustup.rs/)
3. 安装 Tauri 依赖

### macOS 依赖
```bash
xcode-select --install
```

## 安装

```bash
npm install
```

## 运行开发环境

```bash
npm run dev
```

## 构建生产版本

```bash
npm run build
```

构建完成后，应用程序将在 `src-tauri/target/release/bundle` 目录中。

## 注意事项

- 应用图标需要自行准备，放在 `src-tauri/icons/` 目录下
- 需要的图标格式：32x32.png, 128x128.png, 128x128@2x.png, icon.icns (macOS), icon.ico (Windows)
- 你可以使用在线工具生成这些图标，或暂时从 tauri.conf.json 中移除 icon 配置

## 功能

该应用将百度网站嵌入到桌面应用中，提供：
- 原生窗口体验
- 可调整大小的窗口
- 最小化、最大化等窗口控制
