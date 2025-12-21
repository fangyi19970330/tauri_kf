# Tauri 应用图标占位符

由于图标是二进制文件，你需要自己准备应用图标。

## 所需图标

在 `src-tauri/icons/` 目录下需要以下图标文件：

- 32x32.png
- 128x128.png
- 128x128@2x.png
- icon.icns (macOS)
- icon.ico (Windows)

## 生成图标

你可以使用以下工具生成图标：

1. 在线工具: https://tauri.app/v1/guides/features/icons
2. 或者暂时移除 tauri.conf.json 中的 icon 配置来跳过图标要求
