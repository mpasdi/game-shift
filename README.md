# Game Shift

Game Shift 是一个本地游戏启动器，目标是统一管理 Windows 本地游戏的启动入口，减少用户手动进入目录查找 `.exe` 的成本。

当前项目处于 MVP 初始化阶段，已经完成桌面应用工程骨架、基础 UI、代码规范和提交规范集成。

## 技术栈

- Tauri v2
- Vue 3
- TypeScript
- Vite
- Pinia
- Rust
- SQLite（待接入）

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
- Game Shift 主界面空状态
- Pinia 基础 store
- ESLint / Prettier / EditorConfig
- Husky / lint-staged / commitlint / czg
- 基础构建检查脚本

下一步计划：

- SQLite 初始化
- `games` 表创建
- Rust 游戏列表查询 command
- 前端主界面接入真实本地数据
