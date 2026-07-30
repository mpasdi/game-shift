# Game Shift Windows 发布 SOP

本文档用于指导 Game Shift 从 `dev` 合并到 `master`、构建并签名 Windows NSIS 更新安装包、生成 SHA-256 与 Updater 元数据、创建版本 Tag 并发布到 GitHub Releases。

## 1. 发布原则

- 日常开发、版本号和发布文档修改先在 `dev` 完成。
- `master` 只保留已经通过检查、可以公开发布的代码。
- 每个公开版本对应一个固定 Tag，例如 `v0.2.0`。
- 已公开的 Tag 不移动、不覆盖、不复用；修复问题时发布新版本。
- 安装包必须从 `master` 的目标发布提交构建。
- 不从存在未提交修改的工作区构建发布包。
- GitHub Release 只上传由项目维护者从目标版本提交生成的文件。
- 未进行 Windows Authenticode 代码签名的安装包必须同时提供 SHA-256，并在发布说明中提示 SmartScreen 风险。
- Updater 安装包、`.sig` 和 `latest.json` 必须来自同一次正式构建，不得混用或覆盖。
- Updater 私钥和密码不得提交到仓库；公钥一经随正式版本发布，不得随意更换。
- 应用标识 `com.gameshift.desktop` 不随普通版本变更。

## 2. 发布前准备

### 2.1 确定版本号

项目使用语义化版本号 `MAJOR.MINOR.PATCH`。

发布前确认以下文件版本号一致：

- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`

```powershell
Select-String -Path package.json,src-tauri\tauri.conf.json,src-tauri\Cargo.toml -Pattern 'version'
```

### 2.2 更新发布内容

- 更新 README 中的当前版本、功能和限制。
- 在 `docs/releases/<version>.md` 创建或更新本版本发布记录。
- 确认发布记录与准备填写的 GitHub Release 文案一致。
- 确认安装包名称、版本号和文档描述一致。

### 2.3 准备 Updater 签名

- 确认 `src-tauri/tauri.conf.json` 已启用 `createUpdaterArtifacts`，并配置正式公钥和 HTTPS 更新端点。
- 确认私钥文件已安全备份且不在仓库目录中。
- 在执行构建的 PowerShell 或 IDE 运行配置中设置 `TAURI_SIGNING_PRIVATE_KEY` 和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。
- 不在终端日志、截图、提交记录或发布附件中暴露私钥及密码。

### 2.4 提交并推送开发分支

先将版本号、README 和发布记录作为独立的发布准备提交落在 `dev`，再执行：

```powershell
git switch dev
git status --short
git log -1 --oneline
pnpm install --frozen-lockfile
pnpm verify
git push origin dev
```

工作区必须干净，最新提交必须是本次发布准备提交，静态检查、前端构建和 Rust 检查必须全部通过。推送后确认 `dev` 与 `origin/dev` 指向同一个提交，避免只在本地合并尚未备份到远端的改动。

## 3. 手动验收

每个版本至少验证：

- 首次启动和数据库初始化。
- 游戏列表读取和持久化。
- 手动添加、目录扫描和候选导入。
- 编辑、移除、收藏和搜索。
- 游戏启动、启动参数和工作目录。
- 本版本新增或修改的功能。
- 中文、空格和较长 Windows 路径。
- 最小窗口尺寸和主要页面滚动。
- 错误场景不会导致应用崩溃或丢失已有数据。

验收失败时回到 `dev` 修复，不继续合并、构建或创建 Tag。

## 4. 合并并推送到 master

项目优先使用 fast-forward，避免不必要的合并提交。

```powershell
git switch master
git pull --ff-only origin master
git merge --ff-only dev
git status --short
pnpm verify
git push origin master
```

确认当前分支是 `master`、工作区干净、最新提交包含本版本的代码与文档，并且 `pnpm verify` 再次通过。验证通过后立即推送 `master`，确保后续安装包和 Tag 都基于远端可追溯的发布提交。fast-forward 失败时不使用强制参数，先检查并整理分支差异。

## 5. 构建 Windows 安装包

确认当前分支为 `master`、工作区干净，并且签名环境变量已传入当前构建进程，然后执行：

```powershell
pnpm tauri build --bundles nsis
```

默认输出目录：

```text
src-tauri/target/release/bundle/nsis/
├── Game Shift_<version>_x64-setup.exe
└── Game Shift_<version>_x64-setup.exe.sig
```

构建成功时必须同时存在安装包和同名 `.sig`。只发布 NSIS `*-setup.exe` 安装包，不直接发布 `target/release/game-shift.exe`。代码、版本号、私钥或公钥发生变化后必须重新构建，不复用旧安装包或签名。

## 6. 验证安装包

```powershell
Get-Item -LiteralPath '<installer>' | Select-Object Name, Length, LastWriteTime
Get-Item -LiteralPath '<installer>.sig' | Select-Object Name, Length, LastWriteTime
Get-AuthenticodeSignature -LiteralPath '<installer>'
```

- `.sig` 文件存在且内容非空。
- 安装、启动和卸载流程正常。
- 安装版本的名称、版本、图标和功能正确。
- 应用可以正常读写本地数据目录。
- 卸载不会删除用户的本地游戏文件。
- 条件允许时，在另一台 Windows 10/11 x64 设备或 Windows Sandbox 中测试。

## 7. 生成 SHA-256

```powershell
pnpm release:checksum
```

此命令只能在项目仓库中用于生成发布校验文件，不用于验证从 GitHub 下载的安装包。确认输出目录同时存在安装包和同名 `.sha256` 文件，并重新核对哈希值。

## 8. 生成 Updater 元数据

`latest.json` 中的下载 URL 可以在 Release 发布前按 Tag 和安装包名称预先确定。发布前该地址返回 404 属于正常现象，Release 发布并上传对应文件后才会生效。

在项目根目录执行以下 PowerShell，并填写本版本的简要更新说明：

```powershell
$package = Get-Content -LiteralPath 'package.json' -Raw | ConvertFrom-Json
$tauriConfig = Get-Content -LiteralPath 'src-tauri\tauri.conf.json' -Raw | ConvertFrom-Json
$version = $package.version
$bundleDir = 'src-tauri\target\release\bundle\nsis'
$installerName = "$($tauriConfig.productName)_$($version)_x64-setup.exe"
$installer = Get-Item -LiteralPath (Join-Path $bundleDir $installerName)
$signaturePath = "$($installer.FullName).sig"
$signature = (Get-Content -LiteralPath $signaturePath -Raw).Trim()
$assetName = [Uri]::EscapeDataString($installer.Name)
$downloadUrl = "https://github.com/mpasdi/game-shift/releases/download/v$version/$assetName"

$latest = [ordered]@{
  version = $version
  notes = '填写本版本的简要更新说明'
  pub_date = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
  platforms = [ordered]@{
    'windows-x86_64' = [ordered]@{
      signature = $signature
      url = $downloadUrl
    }
  }
}

$outputPath = Join-Path $bundleDir 'latest.json'
$json = $latest | ConvertTo-Json -Depth 5
[System.IO.File]::WriteAllText($outputPath, $json, [System.Text.UTF8Encoding]::new($false))
Write-Host "Created: $outputPath"
```

生成后确认：

- `version` 与三个项目版本号和安装包一致。
- `platforms.windows-x86_64.signature` 等于 `.sig` 文件的完整内容，不是文件路径或 URL。
- `platforms.windows-x86_64.url` 指向同一 Tag 下准备上传的 NSIS 安装包。
- `pub_date` 是 RFC 3339 格式的 UTC 时间。

## 9. 创建并推送 Tag

```powershell
git tag -a v<version> -m 'Game Shift v<version>'
git show v<version> --no-patch
git push origin v<version>
```

仅在 `master` 已推送，并且检查、构建和安装验收全部通过后创建 Tag。Tag 推送前发现问题可以删除本地 Tag；Tag 已公开后不得覆盖。

## 10. 创建 GitHub Release

1. 选择对应的版本 Tag。
2. Release 标题使用 `Game Shift v<version>`。
3. 发布说明与 `docs/releases/<version>.md` 保持一致。
4. 上传 NSIS 安装包、同名 `.sig`、同名 `.sha256` 和 `latest.json`。
5. 复核 `latest.json` 中的版本、下载地址和签名与本次上传文件完全对应。
6. 复核文件名、版本号、系统要求、安全说明和已知限制。
7. 所有文件上传完成后再发布 Release。

## 11. 发布后验证

- 从 GitHub Release 重新下载安装包，不使用本地原文件。
- 同时下载对应的 `.sha256` 文件。
- 在只包含这两个下载文件的目录中打开 PowerShell，执行：

```powershell
$downloadedInstallers = @(Get-ChildItem -LiteralPath . -Filter '*_x64-setup.exe' -File)
if ($downloadedInstallers.Count -ne 1) {
  throw "Expected exactly one downloaded installer, found $($downloadedInstallers.Count)."
}

$downloadedInstaller = $downloadedInstallers[0]
$downloadedChecksum = "$($downloadedInstaller.FullName).sha256"
$actualHash = (Get-FileHash -LiteralPath $downloadedInstaller.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
$expectedHash = ((Get-Content -LiteralPath $downloadedChecksum -Raw).Trim() -split '\s+')[0].ToLowerInvariant()

if ($actualHash -ne $expectedHash) {
  throw "SHA-256 verification failed. Actual: $actualHash Expected: $expectedHash"
}

Write-Host "SHA-256 verified: $actualHash"
```

- 看到 `SHA-256 verified` 表示下载的安装包与发布时生成校验文件的安装包完全一致。
- GitHub 可能把安装包文件名中的空格显示或下载为点号；上述命令按实际下载文件名查找，不受此差异影响。
- 在 Windows 10/11 x64 环境安装并启动。
- 确认 Release 页面只包含正确版本的文件。
- 打开 `https://github.com/mpasdi/game-shift/releases/latest/download/latest.json`，确认可以访问且内容与 Release 附件一致。
- 使用包含相同正式公钥、但版本低于本次 Release 的内部验收构建执行一次应用内检查、下载、签名验证、安装和重启测试。
- 首个 Updater 版本的低版本验收构建只用于测试，不得覆盖或重新上传到既有公开版本。
- 确认 Tag 指向 `master` 的目标发布提交。
- 确认 README 下载说明和版本信息有效。
- 将发现的问题记录到 Issues 或版本发布记录。
- 切回 `dev` 继续开发。

## 12. 快速清单

1. [ ] 在 `dev` 完成功能、版本号、README 和版本发布记录，并提交发布准备改动。
2. [ ] 在 `dev` 执行 `pnpm verify` 和手动验收。
3. [ ] 推送 `dev`，确认与 `origin/dev` 同步。
4. [ ] fast-forward 合并 `dev` 到 `master`。
5. [ ] 在 `master` 再次执行 `pnpm verify`，然后推送 `master`。
6. [ ] 使用正式私钥构建并安装测试 NSIS 安装包，确认同名 `.sig` 存在。
7. [ ] 生成并核对 SHA-256 和 `latest.json`。
8. [ ] 创建、检查并推送 annotated Tag。
9. [ ] 创建 GitHub Release，上传安装包、`.sig`、`.sha256` 和 `latest.json`。
10. [ ] 从 GitHub 重新下载，完成校验和应用内更新验收。
11. [ ] 切回 `dev`。
