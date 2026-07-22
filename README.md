<p align="center">
  <img src="./assets/app-logo.png" width="128" alt="Game Shift Logo" />
</p>

<h1 align="center">Game Shift</h1>

<p align="center">简洁、轻量的 Windows 本地游戏库与启动器。</p>

Game Shift 用于集中管理 Windows 电脑上的本地游戏启动程序。你可以手动添加单个 `.exe`，也可以扫描游戏目录批量导入，并在统一界面中完成搜索、收藏、启动和最近游玩管理。

> 当前版本：`v0.1.0 Beta`

## 主要功能

- 手动选择 `.exe` 添加本地游戏
- 扫描文件夹并批量导入候选程序
- 自动提取游戏图标和发现本地封面
- 列表与网格两种游戏库视图
- 按名称或启动程序文件名搜索
- 收藏游戏，并按最近收藏时间排序
- 记录最近游玩时间和启动次数
- 支持工作目录与自定义启动参数
- 编辑或移除游戏记录
- 使用 SQLite 在本地保存游戏库

## 系统要求

- Windows 10 / 11
- x64 处理器与操作系统
- Microsoft Edge WebView2 Runtime

大多数 Windows 10 / 11 设备已经安装 WebView2。如果系统缺少运行时，安装程序会按 Tauri 默认策略尝试安装。

## 下载与安装

前往 [GitHub Releases](https://github.com/mpasdi/game-shift/releases) 下载最新版本：

```text
Game Shift_0.1.0_x64-setup.exe
```

运行安装程序并按照提示完成安装。请只从本项目官方 GitHub Releases 下载，不要运行来源不明的二次打包文件。

### Windows SmartScreen 提示

Game Shift 是免费开源软件，当前 Beta 安装包暂未购买商业 Windows 代码签名证书。从浏览器下载后，Windows SmartScreen 可能显示“未知发布者”或“Windows 已保护你的电脑”。

安装前请确认：

1. 文件来自本项目官方 GitHub Releases。
2. 文件名和版本号与 Release 页面一致。
3. 本地计算出的 SHA-256 与 Release 提供的 `.sha256` 文件一致。

PowerShell 校验命令：

```powershell
Get-FileHash -LiteralPath '.\Game Shift_0.1.0_x64-setup.exe' -Algorithm SHA256
```

如果无法确认文件来源或校验值不一致，请不要继续安装。

## 基本使用

### 添加单个游戏

1. 点击右上角“添加游戏”。
2. 选择游戏的 `.exe` 启动程序。
3. 确认游戏名称和工作目录。
4. 根据需要填写启动参数并保存。

### 批量扫描目录

1. 点击右上角“扫描目录”。
2. 选择游戏所在的文件夹。
3. 在扫描结果中勾选需要导入的候选程序。
4. 确认导入。

### 数据与本地文件

- 游戏库数据保存在本地 SQLite 数据库中，不会上传到服务器。
- 设置页面会显示当前应用数据目录和数据库文件位置。
- 从 Game Shift 中“移除游戏”只删除游戏库记录，不会删除磁盘中的游戏文件。
- 卸载或手动清理应用数据前，如需保留游戏库，请先备份设置页面显示的数据库文件。

## 当前限制

- 当前仅提供 Windows x64 安装包。
- 安装包暂未进行商业 Windows 代码签名。
- 暂不跟踪游戏进程运行状态。
- 暂不统计单次或累计游玩时长。
- 暂不提供自动更新，请从 GitHub Releases 获取新版本。
- 分类与标签功能尚未开放。

## 技术栈

- Tauri v2
- Vue 3
- TypeScript
- Vite
- Pinia
- Rust
- SQLite

## 本地开发

### 环境要求

- Node.js
- pnpm
- Rust stable MSVC toolchain
- Microsoft C++ Build Tools
- WebView2 Runtime

检查本地环境：

```powershell
node -v
pnpm -v
cargo --version
rustup show
where.exe cargo
```

### 安装依赖

```powershell
pnpm install --frozen-lockfile
```

### 启动开发环境

只启动前端 Vite 服务：

```powershell
pnpm dev
```

启动 Tauri 桌面应用：

```powershell
pnpm tauri dev
```

### 代码检查

执行前端与 Rust 全量检查：

```powershell
pnpm verify
```

单独执行：

```powershell
pnpm type-check
pnpm lint
pnpm format:check
pnpm check:rust
```

### 构建 Windows 安装包

```powershell
pnpm tauri build --bundles nsis
```

构建产物位于：

```text
src-tauri/target/release/bundle/nsis/
```

## 提交规范

项目使用 Husky、lint-staged、commitlint 和 czg 维护提交规范。

推荐使用交互式提交：

```powershell
pnpm commit
```

提交前会执行暂存文件检查：

```text
pre-commit -> pnpm lint-staged
commit-msg -> commitlint
```

## 项目文档

- [需求文档](./docs/requirements.md)
- [UI 设计说明](./docs/ui-design.md)
- [开发模块与实现清单](./workLine.md)
- [Windows 发布 SOP](./docs/windows-release-sop.md)
- [v0.1.0 发布记录](./docs/releases/v0.1.0.md)

## 反馈问题

如果遇到问题，请前往 [GitHub Issues](https://github.com/mpasdi/game-shift/issues) 提交，并尽量提供：

- Game Shift 版本号
- Windows 版本与系统架构
- 问题复现步骤
- 错误提示或截图
- 涉及路径时说明路径是否包含中文或空格；请隐藏个人隐私信息

## License

[MIT License](./LICENSE)
