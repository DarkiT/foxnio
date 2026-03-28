pub use sea_orm_migration::prelude::*;

mod m20240327_000001_create_users;
mod m20240327_000002_create_accounts;
mod m20240327_000003_create_api_keys;
mod m20240327_000004_create_usages;
mod m20240327_000005_create_refresh_tokens;
mod m20240327_000005_create_password_reset_tokens;
mod m20240327_000006_create_oauth_tokens;
mod m20240327_000007_create_audit_logs;
mod m20240327_000008_create_alert_rules;
mod m20240327_000009_create_alert_history;
mod m20240327_000010_create_alert_channels;
mod m20240328_000011_create_groups;
mod m20240328_000012_create_model_configs;
mod m20240328_000013_create_tls_fingerprint_profiles;
mod m20240328_000014_create_announcements;
mod m20240328_000015_create_promo_codes;
mod m20240328_000016_create_user_attributes;
mod m20240328_000017_create_error_passthrough_rules;
mod m20240328_000018_create_scheduled_test_plans;
mod m20240328_000019_create_proxies;
mod m20240328_000020_create_redeem_codes;
mod m20240328_000021_create_quota_usage_history;
mod m20240328_000022_create_subscriptions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240327_000001_create_users::Migration),
            Box::new(m20240327_000002_create_accounts::Migration),
            Box::new(m20240327_000003_create_api_keys::Migration),
            Box::new(m20240327_000004_create_usages::Migration),
            Box::new(m20240327_000005_create_refresh_tokens::Migration),
            Box::new(m20240327_000005_create_password_reset_tokens::Migration),
            Box::new(m20240327_000006_create_oauth_tokens::Migration),
            Box::new(m20240327_000007_create_audit_logs::Migration),
            Box::new(m20240327_000008_create_alert_rules::Migration),
            Box::new(m20240327_000009_create_alert_history::Migration),
            Box::new(m20240327_000010_create_alert_channels::Migration),
            Box::new(m20240328_000011_create_groups::Migration),
            Box::new(m20240328_000012_create_model_configs::Migration),
            Box::new(m20240328_000013_create_tls_fingerprint_profiles::Migration),
            Box::new(m20240328_000014_create_announcements::Migration),
            Box::new(m20240328_000015_create_promo_codes::Migration),
            Box::new(m20240328_000016_create_user_attributes::Migration),
            Box::new(m20240328_000017_create_error_passthrough_rules::Migration),
            Box::new(m20240328_000018_create_scheduled_test_plans::Migration),
            Box::new(m20240328_000019_create_proxies::Migration),
            Box::new(m20240328_000020_create_redeem_codes::Migration),
            Box::new(m20240328_000021_create_quota_usage_history::Migration),
            Box::new(m20240328_000022_create_subscriptions::Migration),
        ]
    }
}
