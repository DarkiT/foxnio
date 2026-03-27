//! 告警历史记录模块

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{Alert, AlertSendResult};

/// 告警历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertHistoryEntry {
    /// 记录 ID
    pub id: String,
    /// 告警信息
    pub alert: Alert,
    /// 发送结果列表
    pub results: Vec<AlertSendResult>,
    /// 触发规则 ID
    pub rule_id: Option<String>,
    /// 触发规则名称
    pub rule_name: Option<String>,
    /// 是否被静默
    pub silenced: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl AlertHistoryEntry {
    pub fn new(alert: Alert, rule_id: Option<String>, rule_name: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            alert,
            results: Vec::new(),
            rule_id,
            rule_name,
            silenced: false,
            created_at: Utc::now(),
        }
    }

    /// 添加发送结果
    pub fn add_result(&mut self, result: AlertSendResult) {
        self.results.push(result);
    }

    /// 检查是否所有通道都发送成功
    pub fn is_all_success(&self) -> bool {
        if self.results.is_empty() {
            return false;
        }
        self.results.iter().all(|r| r.success)
    }

    /// 获取成功的通道数
    pub fn success_count(&self) -> usize {
        self.results.iter().filter(|r| r.success).count()
    }

    /// 获取失败的通道数
    pub fn failure_count(&self) -> usize {
        self.results.iter().filter(|r| !r.success).count()
    }
}

/// 告警历史记录过滤器
#[derive(Debug, Clone, Default)]
pub struct AlertHistoryFilter {
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    /// 告警级别
    pub level: Option<super::AlertLevel>,
    /// 规则 ID
    pub rule_id: Option<String>,
    /// 规则名称（模糊匹配）
    pub rule_name: Option<String>,
    /// 来源（模糊匹配）
    pub source: Option<String>,
    /// 是否只看失败的
    pub failed_only: bool,
    /// 最大返回数量
    pub limit: Option<usize>,
}

/// 告警历史统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertHistoryStats {
    /// 总告警数
    pub total_count: u64,
    /// 按级别统计
    pub by_level: std::collections::HashMap<String, u64>,
    /// 按来源统计
    pub by_source: std::collections::HashMap<String, u64>,
    /// 按规则统计
    pub by_rule: std::collections::HashMap<String, u64>,
    /// 成功发送数
    pub success_count: u64,
    /// 失败发送数
    pub failure_count: u64,
    /// 静默数
    pub silenced_count: u64,
    /// 时间范围
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

/// 告警历史记录管理器
pub struct AlertHistory {
    /// 历史记录存储（内存）
    entries: Arc<RwLock<VecDeque<AlertHistoryEntry>>>,
    /// 最大存储数量
    max_entries: usize,
}

impl AlertHistory {
    /// 创建新的告警历史记录管理器
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(VecDeque::with_capacity(max_entries))),
            max_entries,
        }
    }

    /// 添加记录
    pub async fn add(&self, entry: AlertHistoryEntry) {
        let mut entries = self.entries.write().await;

        // 如果超过最大数量，移除最旧的
        if entries.len() >= self.max_entries {
            entries.pop_front();
        }

        entries.push_back(entry);
    }

    /// 获取记录
    pub async fn get(&self, id: &str) -> Option<AlertHistoryEntry> {
        let entries = self.entries.read().await;
        entries.iter().find(|e| e.id == id).cloned()
    }

    /// 查询记录
    pub async fn query(&self, filter: &AlertHistoryFilter) -> Vec<AlertHistoryEntry> {
        let entries = self.entries.read().await;
        let limit = filter.limit.unwrap_or(100);

        entries
            .iter()
            .rev() // 最新在前
            .filter(|e| self.matches_filter(e, filter))
            .take(limit)
            .cloned()
            .collect()
    }

    /// 检查是否匹配过滤器
    fn matches_filter(&self, entry: &AlertHistoryEntry, filter: &AlertHistoryFilter) -> bool {
        // 时间范围
        if let Some(start) = filter.start_time {
            if entry.created_at < start {
                return false;
            }
        }
        if let Some(end) = filter.end_time {
            if entry.created_at > end {
                return false;
            }
        }

        // 级别
        if let Some(level) = &filter.level {
            if entry.alert.level != *level {
                return false;
            }
        }

        // 规则 ID
        if let Some(rule_id) = &filter.rule_id {
            if entry.rule_id.as_ref() != Some(rule_id) {
                return false;
            }
        }

        // 规则名称（模糊匹配）
        if let Some(rule_name) = &filter.rule_name {
            if let Some(ref name) = entry.rule_name {
                if !name.to_lowercase().contains(&rule_name.to_lowercase()) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // 来源（模糊匹配）
        if let Some(source) = &filter.source {
            if !entry
                .alert
                .source
                .to_lowercase()
                .contains(&source.to_lowercase())
            {
                return false;
            }
        }

        // 只看失败的
        if filter.failed_only && entry.is_all_success() {
            return false;
        }

        true
    }

    /// 获取统计信息
    pub async fn stats(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> AlertHistoryStats {
        let entries = self.entries.read().await;

        let end = end_time.unwrap_or_else(Utc::now);
        let start = start_time.unwrap_or(end - chrono::Duration::days(7));

        let mut stats = AlertHistoryStats {
            total_count: 0,
            by_level: std::collections::HashMap::new(),
            by_source: std::collections::HashMap::new(),
            by_rule: std::collections::HashMap::new(),
            success_count: 0,
            failure_count: 0,
            silenced_count: 0,
            start_time: start,
            end_time: end,
        };

        for entry in entries.iter() {
            if entry.created_at < start || entry.created_at > end {
                continue;
            }

            stats.total_count += 1;

            // 按级别统计
            let level_key = entry.alert.level.as_str().to_string();
            *stats.by_level.entry(level_key).or_insert(0) += 1;

            // 按来源统计
            *stats
                .by_source
                .entry(entry.alert.source.clone())
                .or_insert(0) += 1;

            // 按规则统计
            if let Some(ref rule_name) = entry.rule_name {
                *stats.by_rule.entry(rule_name.clone()).or_insert(0) += 1;
            }

            // 发送统计
            stats.success_count += entry.success_count() as u64;
            stats.failure_count += entry.failure_count() as u64;

            if entry.silenced {
                stats.silenced_count += 1;
            }
        }

        stats
    }

    /// 清理旧记录
    pub async fn cleanup(&self, before: DateTime<Utc>) -> usize {
        let mut entries = self.entries.write().await;
        let initial_len = entries.len();

        entries.retain(|e| e.created_at >= before);

        initial_len - entries.len()
    }

    /// 获取记录数量
    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    /// 检查是否为空
    pub async fn is_empty(&self) -> bool {
        self.entries.read().await.is_empty()
    }
}

impl Default for AlertHistory {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alert::{AlertChannelType, AlertLevel};

    fn create_test_entry(level: AlertLevel, source: &str) -> AlertHistoryEntry {
        let alert = Alert::new(level, "测试告警", "测试消息").with_source(source);

        let mut entry = AlertHistoryEntry::new(
            alert,
            Some("rule-1".to_string()),
            Some("测试规则".to_string()),
        );

        entry.add_result(AlertSendResult::success(AlertChannelType::Email));
        entry.add_result(AlertSendResult::failure(
            AlertChannelType::Slack,
            "Connection error",
        ));

        entry
    }

    #[tokio::test]
    async fn test_add_and_get() {
        let history = AlertHistory::new(100);
        let entry = create_test_entry(AlertLevel::Warning, "test");

        let id = entry.id.clone();
        history.add(entry).await;

        let retrieved = history.get(&id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().alert.level, AlertLevel::Warning);
    }

    #[tokio::test]
    async fn test_query_filter() {
        let history = AlertHistory::new(100);

        history
            .add(create_test_entry(AlertLevel::Warning, "source-a"))
            .await;
        history
            .add(create_test_entry(AlertLevel::Error, "source-b"))
            .await;
        history
            .add(create_test_entry(AlertLevel::Critical, "source-a"))
            .await;

        // 按级别过滤
        let filter = AlertHistoryFilter {
            level: Some(AlertLevel::Error),
            ..Default::default()
        };
        let results = history.query(&filter).await;
        assert_eq!(results.len(), 1);

        // 按来源过滤
        let filter = AlertHistoryFilter {
            source: Some("source-a".to_string()),
            ..Default::default()
        };
        let results = history.query(&filter).await;
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_stats() {
        let history = AlertHistory::new(100);

        history
            .add(create_test_entry(AlertLevel::Warning, "api"))
            .await;
        history
            .add(create_test_entry(AlertLevel::Error, "api"))
            .await;
        history
            .add(create_test_entry(AlertLevel::Critical, "db"))
            .await;

        let stats = history.stats(None, None).await;

        assert_eq!(stats.total_count, 3);
        assert_eq!(*stats.by_level.get("warning").unwrap_or(&0), 1);
        assert_eq!(*stats.by_level.get("error").unwrap_or(&0), 1);
        assert_eq!(*stats.by_level.get("critical").unwrap_or(&0), 1);
    }

    #[tokio::test]
    async fn test_max_entries() {
        let history = AlertHistory::new(3);

        history.add(create_test_entry(AlertLevel::Info, "1")).await;
        history.add(create_test_entry(AlertLevel::Info, "2")).await;
        history.add(create_test_entry(AlertLevel::Info, "3")).await;
        history.add(create_test_entry(AlertLevel::Info, "4")).await;

        assert_eq!(history.len().await, 3);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let history = AlertHistory::new(100);

        history
            .add(create_test_entry(AlertLevel::Info, "test"))
            .await;
        assert_eq!(history.len().await, 1);

        // 清理所有记录
        let removed = history
            .cleanup(Utc::now() + chrono::Duration::hours(1))
            .await;
        assert_eq!(removed, 1);
        assert_eq!(history.len().await, 0);
    }

    #[test]
    fn test_entry_results() {
        let mut entry = create_test_entry(AlertLevel::Warning, "test");

        assert!(!entry.is_all_success());
        assert_eq!(entry.success_count(), 1);
        assert_eq!(entry.failure_count(), 1);

        entry.add_result(AlertSendResult::success(AlertChannelType::DingTalk));
        assert_eq!(entry.success_count(), 2);
    }
}
