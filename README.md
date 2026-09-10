# Xedis-TUI Landing Page

纯 HTML + CSS + JavaScript 产品主页，内容参考 `alpha/v0.1.0` 分支。无需依赖、框架或构建步骤。

## 本地预览

在此目录运行 `uv run --no-project python -m http.server 4173 --bind 127.0.0.1`，访问 http://127.0.0.1:4173 。也可以直接打开 `index.html`；复制按钮仅在浏览器支持安全剪贴板环境时显示。

## GitHub Pages

将本分支提交并推送后，在仓库 Settings → Pages 中选择 **Deploy from a branch**，选择 **pages/v0.1.0** 分支与 **/(root)** 目录，保存。
默认站点地址为 https://mikewang0731.github.io/Xedis-TUI/ （以 GitHub 实际给出的地址为准）。
`.nojekyll` 用于直接提供静态文件，资源均使用相对路径，无需额外 GitHub Actions 工作流。

## 修改内容

- `index.html`：页面文案、GitHub 链接和截图占位。
- `styles.css`：响应式布局以及深浅两套颜色变量。
- `theme.js`：首次加载时读取主题，避免闪烁。
- `script.js`：主题切换、偏好记忆、命令复制。存储不可用时仍可切换；未设置偏好时跟随系统。
- `assets/`：放置产品截图，替换步骤见该目录 README。

页面不加载外部字体、脚本或统计服务；禁用 JavaScript 时仍可阅读内容、访问仓库与下载页面。
