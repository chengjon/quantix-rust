#![allow(clippy::collapsible_if)]

use crate::core::{QuantixError, Result};
use crate::watchlist::models::{
    WatchlistAction, WatchlistEntry, WatchlistHistoryEvent, WatchlistListItem, WatchlistStore,
};
use chrono::{DateTime, Utc};

/// 自选股列表的领域服务，封装分组的增删改查、标签管理与历史事件流水。
#[derive(Debug, Clone)]
pub struct WatchlistService {
    history_limit: usize,
}

impl WatchlistService {
    /// 用指定历史容量上限构造服务；超过上限时丢弃最旧的事件。
    pub fn new(history_limit: usize) -> Self {
        Self { history_limit }
    }

    /// 把 code 加入指定分组（缺省取 default_group）；已存在或不合法时返回错误，并写 `Add` 历史事件。
    pub fn add(
        &self,
        store: &mut WatchlistStore,
        code: &str,
        group: Option<&str>,
        now: DateTime<Utc>,
    ) -> Result<()> {
        validate_code(code)?;
        let group_name = group.unwrap_or(&store.default_group).to_string();
        let codes = store
            .groups
            .get_mut(&group_name)
            .ok_or_else(|| QuantixError::Other(format!("分组不存在: {}", group_name)))?;

        if codes.iter().any(|item| item == code) {
            return Err(QuantixError::Other(format!(
                "股票 {} 已存在于分组 {}",
                code, group_name
            )));
        }

        codes.push(code.to_string());
        store
            .entries
            .entry(code.to_string())
            .or_insert(WatchlistEntry {
                tags: Vec::new(),
                added_at: now,
                updated_at: now,
            });
        self.touch(store, now);
        self.push_history(
            store,
            WatchlistHistoryEvent {
                ts: now,
                action: WatchlistAction::Add,
                code: Some(code.to_string()),
                group: Some(group_name),
                tag: None,
            },
        );
        Ok(())
    }

    /// 从所有分组中移除 code 及其 entry；任何分组都没找到时返回错误，成功时写 `Remove` 历史事件。
    pub fn remove(&self, store: &mut WatchlistStore, code: &str, now: DateTime<Utc>) -> Result<()> {
        validate_code(code)?;
        let mut removed = false;

        for codes in store.groups.values_mut() {
            let before = codes.len();
            codes.retain(|item| item != code);
            removed |= before != codes.len();
        }

        if !removed {
            return Err(QuantixError::Other(format!("股票不存在: {}", code)));
        }

        store.entries.remove(code);
        self.touch(store, now);
        self.push_history(
            store,
            WatchlistHistoryEvent {
                ts: now,
                action: WatchlistAction::Remove,
                code: Some(code.to_string()),
                group: None,
                tag: None,
            },
        );
        Ok(())
    }

    /// 新建命名分组；分组名为空或已存在时返回错误，成功时写 `GroupCreate` 历史事件。
    pub fn create_group(
        &self,
        store: &mut WatchlistStore,
        name: &str,
        now: DateTime<Utc>,
    ) -> Result<()> {
        if name.trim().is_empty() {
            return Err(QuantixError::Other("分组名不能为空".to_string()));
        }

        if store.groups.contains_key(name) {
            return Err(QuantixError::Other(format!("分组已存在: {}", name)));
        }

        store.groups.insert(name.to_string(), Vec::new());
        self.touch(store, now);
        self.push_history(
            store,
            WatchlistHistoryEvent {
                ts: now,
                action: WatchlistAction::GroupCreate,
                code: None,
                group: Some(name.to_string()),
                tag: None,
            },
        );
        Ok(())
    }

    /// 将 code 从所有其他分组移除并加入 target_group；目标分组不存在或源处未持有 code 时返回错误。
    pub fn move_code(
        &self,
        store: &mut WatchlistStore,
        code: &str,
        target_group: &str,
        now: DateTime<Utc>,
    ) -> Result<()> {
        validate_code(code)?;
        if !store.groups.contains_key(target_group) {
            return Err(QuantixError::Other(format!("分组不存在: {}", target_group)));
        }

        let mut found = false;
        for codes in store.groups.values_mut() {
            let before = codes.len();
            codes.retain(|item| item != code);
            found |= before != codes.len();
        }

        if !found {
            return Err(QuantixError::Other(format!("股票不存在: {}", code)));
        }

        let target_codes = store
            .groups
            .get_mut(target_group)
            .ok_or_else(|| QuantixError::Other(format!("分组不存在: {}", target_group)))?;
        if !target_codes.iter().any(|item| item == code) {
            target_codes.push(code.to_string());
        }

        if let Some(entry) = store.entries.get_mut(code) {
            entry.updated_at = now;
        }

        self.touch(store, now);
        self.push_history(
            store,
            WatchlistHistoryEvent {
                ts: now,
                action: WatchlistAction::Move,
                code: Some(code.to_string()),
                group: Some(target_group.to_string()),
                tag: None,
            },
        );
        Ok(())
    }

    /// 给 code 追加一个标签；股票不存在或标签已存在时返回错误，成功时写 `TagAdd` 历史事件。
    pub fn add_tag(
        &self,
        store: &mut WatchlistStore,
        code: &str,
        tag: &str,
        now: DateTime<Utc>,
    ) -> Result<()> {
        validate_code(code)?;
        let entry = store
            .entries
            .get_mut(code)
            .ok_or_else(|| QuantixError::Other(format!("股票不存在: {}", code)))?;

        if entry.tags.iter().any(|item| item == tag) {
            return Err(QuantixError::Other(format!("标签已存在: {}", tag)));
        }

        entry.tags.push(tag.to_string());
        entry.updated_at = now;
        self.touch(store, now);
        self.push_history(
            store,
            WatchlistHistoryEvent {
                ts: now,
                action: WatchlistAction::TagAdd,
                code: Some(code.to_string()),
                group: self.find_group_for_code(store, code),
                tag: Some(tag.to_string()),
            },
        );
        Ok(())
    }

    /// 从 code 上移除一个标签；股票或标签不存在时返回错误，成功时写 `TagRemove` 历史事件。
    pub fn remove_tag(
        &self,
        store: &mut WatchlistStore,
        code: &str,
        tag: &str,
        now: DateTime<Utc>,
    ) -> Result<()> {
        validate_code(code)?;
        let entry = store
            .entries
            .get_mut(code)
            .ok_or_else(|| QuantixError::Other(format!("股票不存在: {}", code)))?;

        let before = entry.tags.len();
        entry.tags.retain(|item| item != tag);
        if before == entry.tags.len() {
            return Err(QuantixError::Other(format!("标签不存在: {}", tag)));
        }

        entry.updated_at = now;
        self.touch(store, now);
        self.push_history(
            store,
            WatchlistHistoryEvent {
                ts: now,
                action: WatchlistAction::TagRemove,
                code: Some(code.to_string()),
                group: self.find_group_for_code(store, code),
                tag: Some(tag.to_string()),
            },
        );
        Ok(())
    }

    /// 列出自选股条目，可按 group 和 tag 过滤；结果按 (group, code) 排序后返回。
    pub fn list(
        &self,
        store: &WatchlistStore,
        group: Option<&str>,
        tag: Option<&str>,
    ) -> Vec<WatchlistListItem> {
        let mut items = Vec::new();

        for (group_name, codes) in &store.groups {
            if group.is_some() && group != Some(group_name.as_str()) {
                continue;
            }

            for code in codes {
                let tags = store
                    .entries
                    .get(code)
                    .map(|entry| entry.tags.clone())
                    .unwrap_or_default();

                if let Some(filter_tag) = tag {
                    if !tags.iter().any(|item| item == filter_tag) {
                        continue;
                    }
                }

                items.push(WatchlistListItem {
                    code: code.clone(),
                    group: group_name.clone(),
                    tags,
                });
            }
        }

        items.sort_by(|left, right| {
            left.group
                .cmp(&right.group)
                .then_with(|| left.code.cmp(&right.code))
        });
        items
    }

    /// 返回历史事件，倒序（最新在前）；可按 code 过滤并截断到 `limit` 条。
    pub fn history(
        &self,
        store: &WatchlistStore,
        code: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<WatchlistHistoryEvent> {
        let mut events: Vec<WatchlistHistoryEvent> = store
            .history
            .iter()
            .filter(|event| match code {
                Some(target) => event.code.as_deref() == Some(target),
                None => true,
            })
            .cloned()
            .collect();

        events.reverse();
        if let Some(limit) = limit {
            events.truncate(limit);
        }
        events
    }

    fn push_history(&self, store: &mut WatchlistStore, event: WatchlistHistoryEvent) {
        store.history.push(event);
        if store.history.len() > self.history_limit {
            let overflow = store.history.len() - self.history_limit;
            store.history.drain(0..overflow);
        }
    }

    fn touch(&self, store: &mut WatchlistStore, now: DateTime<Utc>) {
        store.updated_at = now;
    }

    fn find_group_for_code(&self, store: &WatchlistStore, code: &str) -> Option<String> {
        store.groups.iter().find_map(|(group, codes)| {
            codes
                .iter()
                .any(|candidate| candidate == code)
                .then(|| group.clone())
        })
    }
}

impl Default for WatchlistService {
    fn default() -> Self {
        Self::new(500)
    }
}

fn validate_code(code: &str) -> Result<()> {
    let is_valid = code.len() == 6 && code.chars().all(|ch| ch.is_ascii_digit());
    if is_valid {
        Ok(())
    } else {
        Err(QuantixError::Other(format!("股票代码格式不合法: {}", code)))
    }
}
