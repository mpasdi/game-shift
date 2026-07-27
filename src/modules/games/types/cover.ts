// 这些类型镜像 Rust `games/covers/models.rs` 的 command 数据契约。
// 修改字段或 CoverSelection 分支时，两端必须同步更新。

/** 第三方数据源中与搜索词匹配的游戏，不等同于本地游戏记录。 */
export interface GameMatch {
  /** 数据源标识，例如 `steamgriddb`。 */
  provider: string
  /** 游戏在第三方数据源中的 ID。 */
  providerGameId: string
  name: string
  /** 用于区分重名游戏。 */
  releaseYear?: number | null
}

/** 当前匹配游戏下可供用户选择的一张网络封面。 */
export interface CoverCandidate {
  provider: string
  /** 保存时传回后端的可信资源 ID。 */
  assetId: string
  providerGameId: string
  /** 只用于候选预览，不作为保存时的下载地址。 */
  previewUrl: string
  width?: number | null
  height?: number | null
}

/** 一次搜索包含的最佳匹配、纠错选项和封面候选。 */
export interface CoverSearchResult {
  matchedGame?: GameMatch | null
  alternativeGames: GameMatch[]
  candidates: CoverCandidate[]
}

/** 用户保存游戏时对封面的明确处理意图。 */
export type CoverSelection =
  // 未操作封面，保留原值。
  | { type: 'unchanged' }
  // 使用本机选择的图片。
  | { type: 'local'; path: string }
  // 使用网络候选；不允许前端提供任意下载 URL。
  | {
      type: 'remote'
      provider: string
      providerGameId: string
      assetId: string
    }
