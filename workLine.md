# Game Shift 开发文档

## 1. 文档用途

本文档用于跟踪 Game Shift 的开发拆分和完成状态。

- 需求来源：`docs/requirements.md`
- 当前阶段：MVP
- 技术路线：Tauri v2 + Vue 3 + TypeScript + Vite + Pinia + Rust + SQLite

## 2. 当前完成状态

- [x] 创建正式需求文档 `docs/requirements.md`
- [x] 补充项目 README
- [x] 初始化 Tauri v2 项目骨架
- [x] 初始化 Vue 3 + TypeScript + Vite 前端工程
- [x] 接入 Pinia
- [x] 配置 `package.json` 基础脚本
- [x] 配置 Tauri 应用名称、窗口尺寸、应用标识
- [x] 创建 Game Shift 主界面空状态
- [x] 创建游戏模块基础类型 `Game`
- [x] 创建游戏模块基础 store
- [x] 集成 EditorConfig
- [x] 配置 .gitattributes 统一 LF 换行
- [x] 集成 Prettier
- [x] 集成 ESLint
- [x] 接入 `@vueuse/core`
- [x] 接入 `@lucide/vue` 图标
- [x] 前端生产构建验证通过
- [x] Rust / Tauri `cargo check` 验证通过
- [x] 替换 Game Shift 应用图标和侧边栏品牌图标
- [x] 区分游戏 `icon` 与 `cover` 视觉资产
- [x] 完成本地游戏库 MVP 主流程
- [x] 完成设置页基础应用信息

## 2.1 UI 技术策略

- [x] 不引入大型 UI 组件库
- [x] 使用自定义组件体系维护桌面工具风格
- [x] 使用 `@lucide/vue` 提供图标
- [x] 使用 `@vueuse/core` 提供组合式工具能力
- [x] 自建 Button 基础组件
- [x] 自建 IconButton 基础组件
- [x] 自建 Modal 基础组件
- [x] 自建 Toast 基础组件
- [ ] 自建 ConfirmDialog 基础组件
- [x] 自建 EmptyState 基础组件
- [x] 自建 TextField 基础组件
- [x] 确定全局主题色

## 3. 开发阶段拆分

### 3.1 阶段一：项目基础设施

- [x] 初始化前端工程
- [x] 初始化 Tauri 工程
- [x] 配置 TypeScript
- [x] 配置 Vite
- [x] 配置 Pinia
- [x] 配置基础目录结构
- [x] 配置 `.gitignore`
- [x] 生成 `pnpm-lock.yaml`
- [x] 生成 `Cargo.lock`
- [x] 配置 EditorConfig
- [x] 配置 Prettier
- [x] 配置 ESLint
- [x] 清理默认 Tauri 图标并替换为 Game Shift 图标
- [x] 增加基础错误提示组件
- [ ] 增加基础确认弹窗组件
- [ ] 增加统一 loading 状态组件

### 3.2 阶段二：本地数据库模块

模块目标：提供游戏数据的本地持久化能力。

- [x] 选择 Rust SQLite 依赖方案
- [x] 配置 SQLite 依赖
- [x] 获取应用数据目录
- [x] 初始化数据库文件
- [x] 创建 `games` 表
- [x] 创建 `settings` 表
- [x] 实现数据库迁移入口
- [x] 实现数据库连接管理
- [x] 实现游戏新增方法
- [x] 实现游戏列表查询方法
- [x] 实现游戏详情查询方法
- [x] 实现游戏更新方法
- [x] 实现游戏删除方法
- [x] 实现重复路径校验
- [x] 为数据库方法补充基础错误处理

### 3.3 阶段三：Tauri Command 接口模块

模块目标：让前端通过 Tauri command 调用 Rust 后端能力。

- [x] 创建基础 Tauri command：`app_info`
- [x] 创建游戏列表查询 command
- [x] 创建游戏新增 command
- [x] 创建游戏编辑 command
- [x] 创建游戏删除 command
- [x] 创建目录扫描 command
- [x] 创建游戏启动 command
- [ ] 创建打开所在目录 command
- [ ] 创建设置读取 command
- [ ] 创建设置保存 command
- [ ] 统一 command 返回结构
- [ ] 统一 command 错误结构

### 3.4 阶段四：游戏扫描模块

模块目标：扫描本地目录中的 `.exe`，生成候选游戏列表。

- [x] 实现文件夹选择入口
- [ ] 实现多目录扫描参数
- [x] 实现 Rust 递归扫描目录
- [x] 只识别 `.exe` 文件
- [x] 实现默认排除规则
- [x] 实现文件名大小写不敏感匹配
- [ ] 跳过不可访问目录并返回错误摘要
- [x] 生成候选游戏名称
- [x] 返回 `ScanCandidate` 列表
- [x] 标记已存在的候选项
- [x] 扫描过程中避免阻塞 UI
- [x] 扫描完成后展示结果弹窗

### 3.5 阶段五：游戏导入模块

模块目标：扫描结果不直接入库，由用户确认导入。

- [x] 创建扫描结果弹窗页面结构
- [x] 展示候选游戏名称
- [x] 展示 `.exe` 文件名
- [x] 展示完整路径
- [x] 展示是否已存在
- [x] 支持勾选 / 取消勾选候选项
- [x] 默认勾选未重复候选项
- [x] 禁止重复项再次导入
- [x] 实现确认导入
- [x] 导入成功后刷新游戏列表
- [x] 导入失败时展示错误原因

### 3.6 阶段六：游戏库模块

模块目标：展示和管理已保存的游戏。

- [x] 创建游戏基础类型 `Game`
- [x] 创建游戏 store 基础结构
- [x] 创建主界面空状态
- [x] 从数据库加载游戏列表
- [x] 实现游戏列表刷新
- [x] 实现游戏卡片组件
- [x] 实现游戏列表视图
- [x] 实现游戏网格视图
- [ ] 实现游戏详情页
- [x] 展示游戏名称
- [x] 展示默认图标占位
- [x] 展示自动提取的游戏图标
- [x] 展示自动发现的游戏封面
- [x] 展示 `.exe` 路径摘要
- [x] 展示最近游玩时间
- [x] 展示启动次数
- [x] 实现默认排序：最近游玩按游玩时间倒序，普通列表按创建时间倒序
- [x] 空列表时展示扫描和手动添加入口

### 3.7 阶段七：搜索与筛选模块

模块目标：帮助用户快速定位游戏。

- [x] 创建搜索输入框 UI
- [x] 创建筛选导航 UI
- [x] 创建全部 / 收藏 / 最近筛选状态
- [x] 实现前端本地搜索逻辑
- [x] 实现基础筛选计数逻辑
- [x] 接入真实数据库列表
- [x] 搜索游戏名称
- [x] 搜索 `.exe` 文件名
- [ ] 支持路径搜索，并在搜索结果中提供命中路径的解释或展示策略
- [x] 收藏筛选只展示收藏游戏
- [x] 最近筛选只展示有启动记录的游戏
- [x] 无搜索结果时展示空结果状态

### 3.8 阶段八：手动添加游戏模块

模块目标：用户可以手动创建游戏记录。

- [x] 创建手动添加按钮交互
- [x] 创建添加游戏弹窗
- [x] 创建游戏名称输入框
- [x] 创建 `.exe` 路径选择器
- [x] 接入 Tauri dialog 文件选择器
- [x] 启动程序字段只读，只能通过文件选择器修改
- [x] 工作目录字段只读，只能通过文件夹选择器修改
- [x] 创建工作目录输入框
- [x] 创建启动参数输入框
- [x] 选择 `.exe` 后自动填充游戏名称
- [x] 选择 `.exe` 后自动填充工作目录
- [x] 校验游戏名称不能为空
- [x] 校验 `.exe` 路径不能为空
- [x] 校验 `.exe` 路径必须存在
- [x] 校验 `.exe` 路径不能重复
- [x] 保存成功后刷新游戏列表
- [x] 保存失败时展示错误原因

### 3.9 阶段九：编辑游戏模块

模块目标：用户可以维护已导入游戏的信息。

- [x] 创建编辑入口
- [x] 复用添加游戏弹窗作为编辑弹窗
- [x] 回填游戏名称
- [x] 回填 `.exe` 路径
- [x] 回填工作目录
- [x] 回填启动参数
- [x] 支持修改收藏状态
- [x] 保存编辑结果
- [x] 编辑成功后刷新游戏列表
- [x] 编辑失败时展示错误原因
- [ ] 避免扫描结果覆盖用户手动编辑字段

### 3.10 阶段十：移除游戏模块

模块目标：从游戏库移除记录，但不删除磁盘文件。

- [x] 创建移除入口
- [x] 创建二次确认弹窗
- [x] 确认文案明确说明不会删除本地文件
- [x] 调用删除 command 删除数据库记录
- [x] 删除成功后刷新游戏列表
- [x] 删除失败时展示错误原因

### 3.11 阶段十一：游戏启动模块

模块目标：点击后启动本地游戏程序。

- [x] 创建启动按钮交互
- [x] Rust 侧实现启动 `.exe`
- [x] 支持启动参数 `args`
- [x] 支持 working directory
- [x] 工作目录为空时默认使用 `.exe` 所在目录
- [x] 启动前校验 `.exe` 是否存在
- [x] 启动前校验 working directory 是否存在
- [x] 启动失败时展示错误原因
- [x] 启动成功后更新 `last_play_time`
- [x] 启动成功后更新 `play_count`
- [x] 启动成功后刷新游戏列表
- [x] 启动按钮短时间防重复点击
- [ ] 基于游戏进程运行状态禁用启动按钮
- [ ] 展示游戏运行中状态
- [ ] 记录游戏进程启动时间和结束时间
- [ ] 统计单次游玩时长和累计游玩时长
- [ ] 关闭游戏时优先尝试温和关闭进程
- [ ] 强制结束游戏进程前进行二次确认

### 3.12 阶段十二：收藏与最近游玩模块

模块目标：提供常用游戏入口和游玩记录。

- [x] 创建收藏按钮组件
- [x] 收藏状态持久化
- [x] 支持取消收藏
- [x] 收藏游戏优先展示
- [x] 收藏筛选接入真实数据
- [x] 最近游玩筛选接入真实数据
- [x] 最近游玩按 `last_play_time` 倒序展示
- [x] 没有最近游玩记录时展示空状态

### 3.13 阶段十三：游戏视觉资产模块

模块目标：区分列表识别用的游戏图标和卡片展示用的游戏封面，降低默认占位感。

- [x] 明确 `icon` 用于列表 / 小尺寸识别
- [x] 明确 `cover` 用于卡片 / 大尺寸展示
- [x] 为 games 表新增 `cover` 字段迁移
- [x] 添加 / 编辑游戏时自动提取 `.exe` 图标作为 `icon`
- [x] 添加 / 编辑游戏时从游戏目录自动发现本地封面作为 `cover`
- [x] 将视觉资产缓存到应用数据目录
- [x] 列表模式优先展示 `icon`
- [x] 卡片模式优先展示 `cover`
- [x] 无 `cover` 时不强行拉伸 `icon` 作为封面
- [ ] 支持用户手动更换封面
- [ ] 支持用户手动更换图标
- [ ] 支持联网匹配封面候选
- [ ] 为本地封面缓存增加文件大小限制，避免异常大图导致缓存膨胀或 WebView 解码卡顿

### 3.14 阶段十四：设置模块

模块目标：展示基础配置，为后续扩展预留入口。

- [x] 创建设置入口
- [x] 创建设置页面
- [x] 展示应用名称、版本和应用标识
- [x] 展示应用数据目录
- [x] 展示数据库文件位置
- [ ] 展示已保存扫描目录
- [ ] 展示默认扫描排除规则
- [ ] 支持重新扫描已保存目录
- [ ] 预留自定义排除规则能力
- [ ] 预留数据导入 / 导出能力

### 3.15 阶段十五：应用体验与异常处理

模块目标：完善使用体验，避免操作失败时无反馈。

- [x] 创建 toast / message 组件
- [ ] 创建全局错误处理工具
- [x] 主要 command 失败时展示错误信息
- [x] 扫描过程中展示 loading
- [x] 导入过程中展示 loading
- [x] 保存过程中展示 loading
- [x] 启动游戏过程中展示状态反馈
- [x] 路径过长时 UI 正确省略
- [x] 按钮在处理中禁用，避免重复提交
- [x] 空状态文案统一

### 3.16 阶段十六：构建、测试与发布准备

模块目标：确保 MVP 可以稳定构建和手动验收。

- [x] 前端 `npm run build` 通过
- [x] Rust `cargo check` 通过
- [x] Tauri `pnpm tauri dev` 通过
- [x] Tauri `pnpm tauri build` 通过
- [x] 手动测试首次启动流程
- [x] 手动测试目录扫描流程
- [x] 手动测试候选导入流程
- [x] 手动测试手动添加流程
- [x] 手动测试编辑流程
- [x] 手动测试移除流程
- [x] 手动测试启动游戏流程
- [x] 手动测试带引号和 Windows 完整路径的启动参数
- [x] 手动测试关闭后数据仍存在
- [x] 编写 MVP 验收记录

### 3.17 阶段十七：代码规范与工程约束

模块目标：统一代码风格和基础质量检查。

- [x] 集成 ESLint
- [x] 集成 Prettier
- [x] 集成 EditorConfig
- [x] 配置 .gitattributes 统一 LF 换行
- [x] 配置 Vue / TypeScript lint 规则
- [x] 配置 Prettier 格式化规则
- [x] 按项目约定更新 Prettier 配置
- [x] 配置 `lint` 脚本
- [x] 配置 `lint:fix` 脚本
- [x] 配置 `format` 脚本
- [x] 配置 `format:check` 脚本
- [x] 配置 `check` 汇总检查脚本
- [x] 配置 check:rust Rust 检查脚本
- [x] 配置 `verify` 全量验证脚本
- [x] 配置 Git 文本换行规范
- [x] 接入 Git hooks
- [x] 接入 Husky
- [x] 接入 lint-staged
- [x] 接入 commitlint
- [x] 同步 commitlint 提交类型枚举
- [x] 接入 czg
- [x] 按项目约定配置 czg 提交类型
- [x] 修正 czg 配置加载文件为 `cz.config.js`
- [x] 配置 `commit` 交互式提交脚本
- [x] 接入提交信息规范

### 3.18 阶段十八：UI 体验重构

模块目标：基于 `docs/ui-design.md` 和设计稿方案 1，将界面调整为左侧导航、顶部搜索、紫色暗色主题、收藏优先的本地游戏启动器界面；最近游玩依赖启动记录，后置到启动功能完成后接入。

- [x] 整理并确认 UI 设计说明 `docs/ui-design.md`
- [x] 重构全局紫色暗色主题变量
- [x] 保留并重构左侧导航结构
- [x] 重构顶部搜索和操作区
- [x] 最近游玩首页区块
- [x] 增加收藏游戏首页区块
- [x] 重构全部游戏区域
- [x] 重构游戏卡片视觉和操作层级
- [x] 重构新增 / 编辑游戏弹窗视觉
- [x] 重构删除确认弹窗视觉
- [x] 增加弹窗打开动效
- [x] 增加卡片 hover 动效
- [x] 增加收藏切换动效
- [x] 保持查 / 增 / 改 / 删 / 收藏功能不回退
- [x] UI 重构后 `pnpm verify` 通过

## 4. 页面清单

### 4.1 主界面 / 游戏库页面

状态：MVP 已完成。

- [x] 应用主布局
- [x] 左侧筛选导航
- [x] 顶部搜索栏
- [x] 扫描目录按钮
- [x] 手动添加按钮占位
- [x] 空状态 UI
- [x] 真实游戏列表
- [x] 游戏卡片操作
- [x] 游戏卡片支持快速收藏 / 取消收藏
- [x] 卡片展示自动发现的封面
- [x] 列表展示自动提取的图标
- [ ] 游戏详情展示
- [x] 列表 / 网格真实切换

### 4.2 扫描结果弹窗

状态：已完成。

- [x] 扫描结果统计
- [x] 候选列表
- [x] 勾选导入
- [x] 重复项提示
- [x] 确认导入
- [x] 取消导入

### 4.3 添加游戏弹窗

状态：已完成。

- [x] 游戏名称字段
- [x] `.exe` 路径字段
- [x] 文件选择按钮
- [x] 工作目录字段
- [x] 启动参数字段
- [x] 保存按钮
- [x] 取消按钮
- [x] 表单校验提示

### 4.4 编辑游戏弹窗

状态：已完成。

- [x] 复用添加游戏表单
- [x] 字段回填
- [x] 保存编辑
- [x] 取消编辑
- [x] 错误提示

### 4.5 删除确认弹窗

状态：已完成。

- [x] 展示游戏名称
- [x] 明确说明不会删除本地文件
- [x] 确认删除按钮
- [x] 取消按钮

### 4.6 设置页面 / 设置弹窗

状态：基础信息已完成。

- [ ] 扫描目录列表
- [ ] 默认排除规则展示
- [x] 数据库路径展示
- [ ] 后续设置项占位

## 5. Rust 后端模块清单

### 5.1 `commands` 模块

- [x] 基础 `app_info` command
- [x] 游戏 CRUD commands
- [x] 扫描 commands
- [x] 启动 commands
- [ ] 设置 commands

### 5.2 `db` 模块

- [x] 数据库路径解析
- [x] SQLite 连接初始化
- [x] 数据库迁移
- [x] games repository
- [ ] settings repository

### 5.3 `scanner` 模块

- [x] 递归目录扫描
- [x] `.exe` 文件识别
- [x] 默认排除规则
- [x] 候选名称推断
- [x] 重复项标记

### 5.4 `launcher` 模块

- [x] 路径校验
- [x] 参数解析
- [x] 工作目录处理
- [x] 启动进程
- [x] 启动结果返回

### 5.5 `settings` 模块

- [ ] 扫描目录持久化
- [ ] 默认排除规则读取
- [ ] 数据库位置读取

## 6. 前端模块清单

### 6.1 `modules/games`

- [x] `Game` 类型
- [x] `games` store
- [x] 游戏 API 封装
- [x] 前端封装游戏新增 API
- [x] 游戏列表组件
- [x] 游戏卡片组件
- [x] 游戏表单组件
- [x] 扫描结果组件
- [x] 删除确认组件

### 6.2 `modules/settings`

- [x] 设置类型
- [ ] 设置 store
- [x] 设置 API 封装
- [x] 设置页面组件

### 6.3 `shared/components`

- [x] Button 基础组件
- [x] IconButton 基础组件
- [x] Modal 基础组件
- [x] Toast 基础组件
- [x] EmptyState 基础组件
- [x] TextField 基础组件
- [ ] Loading 基础组件
- [ ] ConfirmDialog 基础组件

### 6.4 `shared/utils`

- [ ] 时间格式化工具
- [ ] 路径显示工具
- [ ] command 错误格式化工具
- [ ] ESLint 规则维护
- [ ] Prettier 格式维护
- [ ] 防抖搜索工具

## 7. 推荐开发顺序

1. [x] SQLite 初始化和 games 表
2. [x] Rust 游戏 CRUD commands
3. [x] 前端加载真实游戏列表
4. [x] 手动添加游戏
5. [x] 编辑游戏
6. [x] 移除游戏
7. [x] 游戏启动
8. [x] 收藏和最近游玩
9. [x] 目录扫描
10. [x] 扫描候选导入
11. [x] 游戏图标提取和封面自动发现
12. [x] 设置页基础信息
13. [ ] Tauri 打包和 MVP 验收

## 8. 当前下一步

建议下一步开发：

- [x] 手动添加游戏
- [x] 编辑游戏
- [x] 移除游戏
- [x] 游戏启动
- [x] 游戏图标提取和封面自动发现
- [x] 设置页基础信息
- [ ] Tauri 打包和 MVP 验收

## 9. 发布阶段建议

### 9.1 第一阶段：MVP / 首个开源可用版

模块目标：优先保证本地游戏库主流程可用，不被非核心管理能力阻塞。

- [x] 完成设置页基础信息
- [ ] 完成 Tauri 打包和 MVP 验收
- [ ] 验证扫描 / 导入 / 手动添加 / 编辑 / 移除 / 启动主流程
- [x] 首版暂不开放分类入口

### 9.2 第二阶段：运行中状态

模块目标：优先补齐启动后的状态感知，避免重复启动，并为后续时长统计和关闭游戏打基础。

- [ ] 后端跟踪由 Game Shift 启动的游戏进程
- [ ] 维护前端运行中游戏状态
- [ ] 游戏运行中时禁用重复启动
- [ ] 卡片 / 列表展示运行中状态
- [ ] 应用重启后处理已失效的运行中状态

### 9.3 第三阶段：游玩时长统计

模块目标：在运行状态可靠后，补充本次游玩、最近一次游玩和累计游玩时长。

- [ ] 记录游戏进程启动时间和结束时间
- [ ] 统计单次游玩时长
- [ ] 汇总累计游玩时长
- [ ] 最近游玩列表展示最近一次游玩时长
- [ ] 游戏详情或列表中展示累计游玩时长

### 9.4 第四阶段：游戏详情页

模块目标：在运行状态和游玩时长数据具备后，提供独立游戏档案页承载描述、统计、时间线和管理入口。

- [ ] 创建游戏详情页路由 `/games/:id`
- [ ] 展示封面 / 图标 / 游戏名称 / 收藏状态
- [ ] 展示启动入口、编辑入口、删除入口
- [ ] 展示 `.exe` 路径、工作目录和启动参数
- [ ] 展示启动次数、最近游玩时间和累计游玩时长
- [ ] 展示游玩时间线或最近游玩会话列表
- [ ] 支持维护游戏描述或备注
- [ ] 保持首页 / 列表 / 收藏的一键启动能力，不让详情页成为启动必经路径

### 9.5 第五阶段：关闭游戏能力

模块目标：在进程跟踪可靠后，谨慎提供关闭游戏能力，避免误杀非目标进程。

- [ ] 支持关闭由 Game Shift 启动的游戏进程
- [ ] 关闭游戏时优先尝试温和关闭进程
- [ ] 强制结束游戏进程前进行二次确认
- [ ] 明确区分普通关闭和强制结束
- [ ] 启动器 / 平台拉起的子进程不可靠时不强行关闭

### 9.6 第六阶段：分类与标签管理

模块目标：在游戏库主流程稳定后，补充面向中大型游戏库的组织能力。

- [ ] 将分类能力按多标签系统设计，而不是单一分类
- [ ] 游戏编辑时支持选择 / 新增标签
- [ ] 分类页按标签聚合游戏
- [ ] 全部游戏列表支持按标签筛选
- [ ] 支持标签重命名和删除
- [ ] 评估批量添加 / 移除标签能力

### 9.7 第七阶段：后台任务通知中心

模块目标：为耗时任务和非即时反馈提供右下角 notification，与顶部即时 message 保持职责分离。

- [ ] 区分即时 message 和后台 notification 的使用边界
- [ ] 大目录扫描完成后支持右下角通知
- [ ] 批量导入完成后支持结果摘要通知
- [ ] 后台任务失败时支持可保留错误通知
- [ ] 支持通知手动关闭
- [ ] 评估是否需要通知历史记录

### 9.8 第八阶段：启动体验优化

模块目标：在应用初始化流程变复杂后，避免窗口先显示空白页面，再进入主界面造成割裂感。

- [ ] 第一阶段先通过窗口背景色和 `index.html` 首屏背景降低白屏突兀感
- [ ] 后续评估将主窗口配置为启动时隐藏
- [ ] 前端路由、数据库和基础状态初始化完成后再调用窗口 `show`
- [ ] 初始化失败时展示可恢复的错误页面或错误提示，避免隐藏窗口后无反馈
- [ ] 评估是否需要独立 splash / loading 页面承载启动过程

## 10. Windows 版本发布 SOP

本章节用于指导 Game Shift 从 `dev` 合并到 `master`、创建版本 Tag、构建 Windows NSIS 安装包、生成 SHA-256 校验值并发布到 GitHub Releases。

### 10.1 发布原则

- 日常开发、发布文档和版本号修改都先在 `dev` 完成。
- `master` 只保留已经通过检查、可以打包发布的代码。
- 每个公开版本对应一个固定 Tag，例如 `v0.1.0`。
- Tag 一旦推送到远端，不要移动、覆盖或复用；修复问题应发布新版本，例如 `v0.1.1`。
- 安装包必须从 `master` 的目标发布提交构建，不能从存在未提交修改的工作区构建。
- GitHub Release 只上传由项目维护者从目标 Tag 对应提交生成的文件。
- 首版未购买 Windows 代码签名证书时，可以发布未签名安装包，但必须提供 SHA-256 并明确 SmartScreen 提示。
- `identifier` 已固定为 `com.gameshift.desktop`，首次公开发布后不要随意修改。

### 10.2 首次发布前只需准备一次

- [ ] 确认 GitHub 仓库是唯一官方发布地址。
- [ ] 确认本机使用 Windows、Rust stable MSVC、Node.js、pnpm 和 WebView2。
- [ ] 确认 `pnpm verify` 可以完整通过。
- [ ] 确认 `pnpm tauri build --bundles nsis` 可以生成安装包。
- [ ] 确认应用图标、产品名称和应用标识正确。
- [ ] 确认 LICENSE 中的版权信息正确。
- [ ] 确认是否暂时采用未签名发布；当前 `v0.1.0` 可按未签名 Beta 发布。
- [ ] 确认 GitHub Release 页面会同时提供安装包和 SHA-256 文件。

### 10.3 确定版本号

Game Shift 使用语义化版本号：

```text
主版本.次版本.修订号
MAJOR.MINOR.PATCH
```

- 不兼容的大改动：增加 `MAJOR`。
- 向后兼容的新功能：增加 `MINOR`。
- 向后兼容的问题修复：增加 `PATCH`。
- 首个公开测试版本：`0.1.0`，对应 Tag `v0.1.0`。

每次发布前检查以下三个文件的版本号完全一致：

- `package.json > version`
- `src-tauri/tauri.conf.json > version`
- `src-tauri/Cargo.toml > package.version`

可以使用以下命令快速检查：

```powershell
Select-String -Path package.json,src-tauri\tauri.conf.json,src-tauri\Cargo.toml -Pattern 'version'
```

### 10.4 在 `dev` 完成发布准备

#### 10.4.1 更新本地分支

```powershell
git checkout dev
git fetch origin
git pull --ff-only origin dev
git status --short
```

要求：

- 当前分支必须是 `dev`。
- 开始修改前工作区应干净。
- 如果存在未提交修改，先确认并提交，不要直接切换或覆盖。

#### 10.4.2 更新发布相关内容

- [x] 确认三个位置的版本号均为 `0.1.0`。
- [x] 更新 README 的当前状态、安装方式和版本信息。
- [x] README 增加 Windows 系统要求与 GitHub Releases 下载说明。
- [x] README 说明本地数据存储和卸载行为。
- [x] README 记录当前已知限制。
- [x] 整理本版本新增功能、修复内容和已知问题，供 GitHub Release 使用。
- [x] 确认文档不再包含已经完成的“待打包”“待发布”等过时描述。

建议提交：

```powershell
git add README.md workLine.md package.json src-tauri\tauri.conf.json src-tauri\Cargo.toml
git commit -m "docs(release): prepare v0.1.0 release"
```

如果实际只有部分文件发生变化，只暂存真实改动的文件。

### 10.5 在 `dev` 执行发布前检查

#### 10.5.1 安装锁定版本依赖

```powershell
pnpm install --frozen-lockfile
```

- [x] `pnpm install --frozen-lockfile` 执行通过。

#### 10.5.2 执行全量静态检查

```powershell
pnpm verify
```

- [x] `pnpm verify` 在 `dev` 执行通过。

该命令应完成：

- TypeScript 类型检查
- ESLint
- Prettier 格式检查
- Vite 生产构建
- Rust `cargo check`

检查失败时不要继续合并或打 Tag，应回到 `dev` 修复并重新执行。

#### 10.5.3 MVP 手动验收清单

- [x] 首次启动可以正常创建数据库并进入首页。
- [x] 空游戏库状态显示正常。
- [x] 手动选择 `.exe` 可以添加游戏。
- [x] 扫描目录可以展示候选程序并导入。
- [x] 取消文件选择和取消扫描不会报错。
- [x] 重复 `.exe` 路径无法重复导入。
- [x] 中文、空格和较长路径可以正常处理。
- [x] 编辑游戏名称、路径、工作目录和启动参数正常。
- [x] 带双引号、反斜杠和完整 Windows 路径的启动参数正常传递。
- [x] 移除游戏只删除数据库记录，不删除本地文件。
- [x] 收藏、取消收藏、重新收藏及收藏时间排序正常。
- [x] 搜索在首页、全部游戏、收藏和最近游玩页面范围正确。
- [x] 点击启动可以运行游戏。
- [x] 启动后最近游玩时间和启动次数正确更新。
- [x] 关闭并重新打开 Game Shift 后数据仍然存在。
- [x] 设置页显示的版本号、标识和数据目录正确。
- [x] 大屏、小屏和最小窗口尺寸下主要页面无明显错位或滚动异常。

### 10.6 合并 `dev` 到 `master`

当前项目优先使用 fast-forward，避免不必要的合并提交。

```powershell
git status --short
git checkout master
git fetch origin
git pull --ff-only origin master
git merge --ff-only dev
```

如果 `git merge --ff-only dev` 失败：

1. 不要使用 `--force`。
2. 检查 `master` 是否存在 `dev` 没有的提交。
3. 回到 `dev` 合并或变基需要保留的 `master` 改动。
4. 重新执行检查后再尝试 fast-forward。

合并后确认：

```powershell
git branch --show-current
git status --short
git log -5 --oneline
pnpm verify
```

要求：

- 当前分支为 `master`。
- 工作区干净。
- 最新提交是准备发布的提交。
- `pnpm verify` 在 `master` 再次通过。

### 10.7 从 `master` 构建 Windows EXE 安装包

Game Shift 首版只发布 Windows x64 NSIS 安装包：

```powershell
pnpm tauri build --bundles nsis
```

默认输出目录：

```text
src-tauri/target/release/bundle/nsis/
```

`v0.1.0` 预期文件名：

```text
Game Shift_0.1.0_x64-setup.exe
```

注意：

- 不要直接发布 `src-tauri/target/release/game-shift.exe`，优先发布 NSIS `*-setup.exe`。
- 构建过程中如果失败，不要创建或推送 Tag；回到 `dev` 修复。
- 每次代码或版本号变化后都必须重新构建，不能复用旧安装包。

### 10.8 验证安装包

#### 10.8.1 检查文件信息

```powershell
$installer = Resolve-Path '.\src-tauri\target\release\bundle\nsis\Game Shift_0.1.0_x64-setup.exe'
Get-Item $installer | Select-Object Name, Length, LastWriteTime
```

#### 10.8.2 检查数字签名状态

```powershell
Get-AuthenticodeSignature -LiteralPath $installer
```

首版未签名时预期：

```text
Status: NotSigned
```

这是已知发布策略，不代表构建失败。Release 说明必须明确安装包当前未签名。

#### 10.8.3 安装包手动验收

- [ ] 双击安装包可以完成安装。
- [ ] 安装过程展示正确的 Game Shift 名称和图标。
- [ ] 安装后可以从开始菜单启动。
- [ ] 安装后的版本功能与开发环境一致。
- [ ] 安装版本可以正常读写应用数据目录。
- [ ] 卸载程序可以正常运行。
- [ ] 卸载不会误删用户本地游戏文件。
- [ ] 如果保留数据库，重新安装后数据行为符合预期。
- [ ] 最好在另一台 Windows 10/11 x64 电脑或 Windows Sandbox 中测试一次。

### 10.9 生成 SHA-256 校验文件

在仓库根目录执行：

```powershell
$installer = Resolve-Path '.\src-tauri\target\release\bundle\nsis\Game Shift_0.1.0_x64-setup.exe'
$hash = (Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash.ToLowerInvariant()
$checksumFile = "$installer.sha256"
"$hash  $([System.IO.Path]::GetFileName($installer))" | Set-Content -LiteralPath $checksumFile -Encoding ascii
Get-Content -LiteralPath $checksumFile
```

预期同时得到：

```text
Game Shift_0.1.0_x64-setup.exe
Game Shift_0.1.0_x64-setup.exe.sha256
```

校验方法：

```powershell
Get-FileHash -LiteralPath '.\Game Shift_0.1.0_x64-setup.exe' -Algorithm SHA256
Get-Content -LiteralPath '.\Game Shift_0.1.0_x64-setup.exe.sha256'
```

两个哈希值必须完全一致。

### 10.10 创建并推送 Tag

仅在代码检查、构建、安装和主流程验收全部通过后执行：

```powershell
git status --short
git tag -a v0.1.0 -m "Game Shift v0.1.0"
git show v0.1.0 --no-patch
```

确认 Tag 指向当前 `master` 发布提交后推送：

```powershell
git push origin master
git push origin v0.1.0
```

如果 Tag 尚未推送且发现问题，可以删除本地 Tag：

```powershell
git tag -d v0.1.0
```

如果 Tag 已经公开推送，不要覆盖它；修复后发布 `v0.1.1`。

### 10.11 创建 GitHub Release

在 GitHub 仓库页面执行：

1. 打开 `Releases`。
2. 点击 `Draft a new release`。
3. 选择 Tag：`v0.1.0`。
4. Release title 填写：`Game Shift v0.1.0`。
5. 上传以下两个文件：
   - `Game Shift_0.1.0_x64-setup.exe`
   - `Game Shift_0.1.0_x64-setup.exe.sha256`
6. 填写版本说明并保存为草稿。
7. 最后复核文件、版本号和说明，确认无误后点击发布。

建议 Release 内容结构：

```markdown
## Game Shift v0.1.0

Game Shift 是一个用于管理和启动 Windows 本地游戏的桌面应用。

### 主要功能

- 手动添加本地游戏
- 扫描目录并批量导入
- 游戏列表与网格视图
- 收藏和最近游玩
- 一键启动游戏
- 本地 SQLite 数据存储

### 系统要求

- Windows 10/11 x64
- WebView2 Runtime

### 下载

请下载 `Game Shift_0.1.0_x64-setup.exe`。

### 安全说明

当前版本未购买商业 Windows 代码签名证书，安装时可能出现 SmartScreen 提示。
请只从本项目官方 GitHub Releases 下载，并使用随附的 SHA-256 文件校验完整性。

### 已知限制

- 暂不统计游戏运行状态和游玩时长
- 暂不提供自动更新
- 当前仅提供 Windows x64 安装包
```

### 10.12 发布后验证

- [ ] 从 GitHub Release 页面重新下载安装包，不使用本地原文件。
- [ ] 对下载文件重新计算 SHA-256，并与 Release 中的文件比较。
- [ ] 在至少一台 Windows 10/11 x64 环境安装并启动。
- [ ] 确认 Release 页面只包含正确版本文件。
- [ ] 确认 Tag 指向 `master` 的目标发布提交。
- [ ] 确认 README 中的下载链接或说明有效。
- [ ] 在 Issues 或 Release 说明中记录首版已知问题。

发布完成后切回开发分支：

```powershell
git checkout dev
git status --short
```

如果发布后直接在 `master` 修复了文档或紧急问题，必须将对应提交同步回 `dev`，避免分支长期分叉。

### 10.13 后续版本快速清单

每次发布都按以下顺序执行：

1. [ ] 在 `dev` 完成功能、版本号、README 和 Release Notes。
2. [ ] 在 `dev` 执行 `pnpm verify` 和手动验收。
3. [ ] fast-forward 合并 `dev` 到 `master`。
4. [ ] 在 `master` 再次执行 `pnpm verify`。
5. [ ] 在 `master` 构建 NSIS 安装包。
6. [ ] 安装并手动测试构建产物。
7. [ ] 生成并核对 SHA-256 文件。
8. [ ] 创建并检查 annotated Tag。
9. [ ] 推送 `master` 和 Tag。
10. [ ] 创建 GitHub Release 并上传 EXE 与 SHA-256。
11. [ ] 从 GitHub 重新下载并完成发布后验证。
12. [ ] 切回 `dev` 继续开发。
