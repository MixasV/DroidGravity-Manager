pub mod account;
pub mod quota;
pub mod config;
pub mod logger;
pub mod db;
pub mod process;
pub mod oauth;
pub mod oauth_server;
pub mod migration;
pub mod tray;
pub mod i18n;
pub mod proxy_db;
pub mod device;
pub mod update_checker;
pub mod integration;
pub mod account_service;
pub mod security_db;
pub mod cache;
pub mod log_bridge;
pub mod token_stats;
pub mod http_api;
pub mod cloudflared;
pub mod scheduler;

use crate::models;

// 重新导出常用函数到 modules 命名空间顶级，方便外部调用
pub use account::*;
#[allow(unused_imports)]
pub use quota::*;
pub use config::*;
#[allow(unused_imports)]
pub use logger::*;
// pub use device::*;

pub async fn fetch_quota(access_token: &str, email: &str) -> crate::error::AppResult<(models::QuotaData, Option<String>)> {
    quota::fetch_quota(access_token, email).await
}
