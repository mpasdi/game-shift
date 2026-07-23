//! 联网封面模块。
//!
//! `models` 定义 Game Shift 内部统一的数据格式，`provider` 定义第三方封面服务
//! 必须实现的能力。以后接入或更换数据源时，游戏保存和前端展示无需跟着改写。

pub(crate) mod models;
pub(crate) mod provider;
pub(crate) mod steamgriddb;
