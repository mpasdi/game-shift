# Game Shift

Game Shift 是一个本地游戏启动器，目标是统一管理 Windows 本地游戏的启动入口，减少用户手动进入目录查找 `.exe` 的成本。

当前项目处于 MVP 收尾阶段，已经完成本地游戏库主流程、SQLite 本地存储、目录扫描、候选导入、手动添加、编辑、移除、收藏、最近游玩和一键启动能力。第一阶段剩余重点是 Tauri 打包验证和 MVP 手动验收。

## 技术栈

- Tauri v2
- Vue 3
- TypeScript
- Vite
- Pinia
- Rust
- SQLite

## 环境要求

开发 Windows 桌面端需要：

- Node.js
- pnpm
- Rust stable MSVC toolchain
- Microsoft C++ Build Tools
- WebView2 Runtime

常用检查命令：

```powershell
node -v
pnpm -v
cargo --version
rustup show
where.exe cargo
```

如果 PowerShell 或 IDE 终端找不到 `cargo`，确认用户 PATH 中包含：

```text
C:\Users\Administrator\.cargo\bin
```

## 安装依赖

```powershell
pnpm install
```

## 本地开发

只启动前端 Vite 服务：

```powershell
pnpm dev
```

启动 Tauri 桌面应用：

```powershell
pnpm tauri dev
```

## 构建

前端构建：

```powershell
pnpm build
```

Tauri 打包：

```powershell
pnpm tauri build
```

## 代码检查

```powershell
pnpm type-check
pnpm lint
pnpm format:check
pnpm check
```

自动格式化：

```powershell
pnpm format
```

## 提交规范

项目使用 Husky、lint-staged、commitlint 和 czg。

推荐使用交互式提交：

```powershell
pnpm commit
```

提交前会自动执行暂存文件检查：

```text
pre-commit -> pnpm lint-staged
commit-msg -> commitlint
```

当前提交类型由 `cz.config.js` 和 `commitlint.config.js` 维护。

## 项目文档

- 需求文档：`docs/requirements.md`
- 开发清单：`workLine.md`

## 当前状态

已完成：

- Tauri + Vue + TypeScript 项目初始化
- SQLite 本地数据库和 `games` 表
- 游戏列表、搜索、收藏、最近游玩
- 目录扫描和候选游戏导入
- 手动添加、编辑和移除游戏
- 游戏启动、启动次数和最近游玩时间记录
- 自动提取游戏图标和自动发现本地封面
- 设置页基础应用信息
- ESLint / Prettier / EditorConfig
- Husky / lint-staged / commitlint / czg
- 前端和 Rust 基础检查脚本

下一步计划：

- Tauri 打包验证
- MVP 手动验收记录
- 运行中状态和游玩时长统计
