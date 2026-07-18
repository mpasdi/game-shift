//! 第三方封面数据源的统一接口。

use std::future::Future;
use std::pin::Pin;

use super::models::{CoverCandidate, GameMatch};

/// Provider 异步操作的统一返回类型。
///
/// 这里使用装箱 Future，是为了让不同 Provider 可以放进 `Box<dyn CoverProvider>`，
/// 运行时按配置选择具体实现，同时不提前引入额外的异步 trait 依赖。
pub(crate) type ProviderFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, String>> + Send + 'a>>;

/// 所有联网封面数据源都必须实现的最小能力集合。
///
/// SteamGridDB 是首个实现；以后增加其他数据源时也实现此接口，上层 command
/// 只依赖 `CoverProvider`，不依赖任何第三方专有字段。
pub(crate) trait CoverProvider: Send + Sync {
    /// 返回稳定的数据源标识，用于候选和最终选择，例如 `steamgriddb`。
    fn provider_id(&self) -> &'static str;

    /// 根据用户输入的名称模糊搜索游戏。
    fn search_games<'a>(&'a self, query: &'a str) -> ProviderFuture<'a, Vec<GameMatch>>;

    /// 根据第三方游戏 ID 查询该游戏的封面候选。
    fn search_covers<'a>(
        &'a self,
        provider_game_id: &'a str,
    ) -> ProviderFuture<'a, Vec<CoverCandidate>>;
}

#[cfg(test)]
mod tests {
    use super::{CoverProvider, ProviderFuture};
    use crate::games::covers::models::{CoverCandidate, GameMatch};

    struct FakeProvider;

    impl CoverProvider for FakeProvider {
        fn provider_id(&self) -> &'static str {
            "fake"
        }

        fn search_games<'a>(&'a self, query: &'a str) -> ProviderFuture<'a, Vec<GameMatch>> {
            Box::pin(async move {
                Ok(vec![GameMatch {
                    provider: "fake".to_string(),
                    provider_game_id: "game-1".to_string(),
                    name: query.to_string(),
                    release_year: None,
                }])
            })
        }

        fn search_covers<'a>(
            &'a self,
            provider_game_id: &'a str,
        ) -> ProviderFuture<'a, Vec<CoverCandidate>> {
            Box::pin(async move {
                Ok(vec![CoverCandidate {
                    provider: "fake".to_string(),
                    asset_id: "cover-1".to_string(),
                    provider_game_id: provider_game_id.to_string(),
                    preview_url: "https://example.invalid/cover-1.jpg".to_string(),
                    width: 600,
                    height: 900,
                }])
            })
        }
    }

    #[test]
    fn provider_contract_is_object_safe() {
        // 能装入 trait object，才能在运行时用统一类型持有不同 Provider。
        let provider: Box<dyn CoverProvider> = Box::new(FakeProvider);
        assert_eq!(provider.provider_id(), "fake");

        // 本测试只验证接口可以调用；真正执行异步请求会在 Provider 实现测试中覆盖。
        drop(provider.search_games("Example Game"));
        drop(provider.search_covers("game-1"));
    }
}
