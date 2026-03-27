use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::core::entities::tab::Tab;
use crate::core::error::NetraError;

const BACKGROUND_LIMIT: usize = 5;
const SUSPEND_TIMEOUT_MS: u64 = 5 * 60 * 1000; // 5 minutes

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Debug, Clone, PartialEq)]
pub enum TabStatus {
    /// Currently visible tab. Only one at a time.
    Active,
    /// Alive in memory but not visible. Max 5 at a time.
    Background,
    /// Tab struct exists but WebView frame is destroyed to free memory.
    Suspended,
}

struct TabEntry {
    tab: Tab,
    status: TabStatus,
    /// Unix timestamp milliseconds of last user interaction.
    last_active_ms: u64,
}

pub struct TabManager {
    tabs: HashMap<String, TabEntry>,
    active_tab_id: Option<String>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            active_tab_id: None,
        }
    }

    /// Creates a new tab. Adds it as Background.
    /// If background limit would be exceeded, suspends the LRU background tab first.
    pub fn create_tab(&mut self, tab_id: String) -> Result<Tab, NetraError> {
        if self.tabs.contains_key(&tab_id) {
            return Err(NetraError::InvalidInput(format!(
                "tab `{tab_id}` already exists"
            )));
        }
        self.enforce_background_limit();
        let tab = Tab::new(tab_id.clone());
        self.tabs.insert(
            tab_id,
            TabEntry {
                tab: tab.clone(),
                status: TabStatus::Background,
                last_active_ms: now_ms(),
            },
        );
        Ok(tab)
    }

    /// Removes a tab entirely. If active, active_tab_id becomes None.
    pub fn close_tab(&mut self, tab_id: &str) -> Result<(), NetraError> {
        if self.tabs.remove(tab_id).is_none() {
            return Err(NetraError::NotFound(format!("tab `{tab_id}` not found")));
        }
        if self.active_tab_id.as_deref() == Some(tab_id) {
            self.active_tab_id = None;
        }
        Ok(())
    }

    /// Sets the given tab as active. Previous active tab becomes Background.
    pub fn set_active_tab(&mut self, tab_id: &str) -> Result<(), NetraError> {
        if !self.tabs.contains_key(tab_id) {
            return Err(NetraError::NotFound(format!("tab `{tab_id}` not found")));
        }

        // No-op if already active.
        if self.active_tab_id.as_deref() == Some(tab_id) {
            return Ok(());
        }

        // Demote current active to background.
        if let Some(prev_id) = self.active_tab_id.clone() {
            if let Some(entry) = self.tabs.get_mut(&prev_id) {
                entry.status = TabStatus::Background;
                entry.last_active_ms = now_ms();
                entry.tab.set_active(false);
            }
            self.enforce_background_limit();
        }

        // Promote new tab to active.
        let entry = self.tabs.get_mut(tab_id).unwrap();
        entry.status = TabStatus::Active;
        entry.last_active_ms = now_ms();
        entry.tab.set_active(true);
        self.active_tab_id = Some(tab_id.to_string());
        Ok(())
    }

    /// Returns clones of all Tab structs.
    pub fn get_all_tabs(&self) -> Vec<Tab> {
        self.tabs.values().map(|e| e.tab.clone()).collect()
    }

    /// Returns a single tab by ID.
    pub fn get_tab(&self, tab_id: &str) -> Result<Tab, NetraError> {
        self.tabs
            .get(tab_id)
            .map(|e| e.tab.clone())
            .ok_or_else(|| NetraError::NotFound(format!("tab `{tab_id}` not found")))
    }

    /// Returns the active tab ID if any.
    pub fn active_tab_id(&self) -> Option<&str> {
        self.active_tab_id.as_deref()
    }

    /// Manually suspends a background tab. Cannot suspend the active tab.
    pub fn suspend_tab(&mut self, tab_id: &str) -> Result<(), NetraError> {
        let entry = self
            .tabs
            .get_mut(tab_id)
            .ok_or_else(|| NetraError::NotFound(format!("tab `{tab_id}` not found")))?;
        if entry.status == TabStatus::Active {
            return Err(NetraError::InvalidInput(
                "cannot suspend the active tab".to_string(),
            ));
        }
        entry.status = TabStatus::Suspended;
        Ok(())
    }

    /// Marks a suspended tab as Background (resuming).
    pub fn resume_tab(&mut self, tab_id: &str) -> Result<(), NetraError> {
        let entry = self
            .tabs
            .get_mut(tab_id)
            .ok_or_else(|| NetraError::NotFound(format!("tab `{tab_id}` not found")))?;
        if entry.status != TabStatus::Suspended {
            return Err(NetraError::InvalidInput(format!(
                "tab `{tab_id}` is not suspended"
            )));
        }
        entry.status = TabStatus::Background;
        entry.last_active_ms = now_ms();
        Ok(())
    }

    /// Updates last_active_ms for a tab to now.
    pub fn touch_tab(&mut self, tab_id: &str) -> Result<(), NetraError> {
        let entry = self
            .tabs
            .get_mut(tab_id)
            .ok_or_else(|| NetraError::NotFound(format!("tab `{tab_id}` not found")))?;
        entry.last_active_ms = now_ms();
        Ok(())
    }

    /// Suspends background tabs idle longer than SUSPEND_TIMEOUT_MS.
    pub fn suspend_idle_tabs(&mut self) -> Vec<String> {
        let cutoff = now_ms().saturating_sub(SUSPEND_TIMEOUT_MS);
        let to_suspend: Vec<String> = self
            .tabs
            .iter()
            .filter(|(_, e)| e.status == TabStatus::Background && e.last_active_ms < cutoff)
            .map(|(id, _)| id.clone())
            .collect();
        for id in &to_suspend {
            if let Some(entry) = self.tabs.get_mut(id) {
                entry.status = TabStatus::Suspended;
            }
        }
        to_suspend
    }

    /// Count of Active + Background tabs.
    pub fn live_tab_count(&self) -> usize {
        self.tabs
            .values()
            .filter(|e| e.status != TabStatus::Suspended)
            .count()
    }

    /// Count of Background tabs only.
    pub fn background_tab_count(&self) -> usize {
        self.tabs
            .values()
            .filter(|e| e.status == TabStatus::Background)
            .count()
    }

    // --- private helpers ---

    /// Suspends the LRU background tab if background count exceeds the limit.
    fn enforce_background_limit(&mut self) {
        if self.background_tab_count() <= BACKGROUND_LIMIT {
            return;
        }
        if let Some(lru_id) = self.find_lru_background() {
            if let Some(entry) = self.tabs.get_mut(&lru_id) {
                entry.status = TabStatus::Suspended;
            }
        }
    }

    /// Returns the ID of the least recently used background tab.
    fn find_lru_background(&self) -> Option<String> {
        self.tabs
            .iter()
            .filter(|(_, e)| e.status == TabStatus::Background)
            .min_by_key(|(_, e)| e.last_active_ms)
            .map(|(id, _)| id.clone())
    }
}

pub fn get_tab_manager() -> &'static Mutex<TabManager> {
    static TAB_MANAGER: OnceLock<Mutex<TabManager>> = OnceLock::new();
    TAB_MANAGER.get_or_init(|| Mutex::new(TabManager::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_close_tab() {
        let mut mgr = TabManager::new();
        mgr.create_tab("t1".to_string()).unwrap();
        assert!(mgr.get_tab("t1").is_ok());
        mgr.close_tab("t1").unwrap();
        assert!(mgr.get_tab("t1").is_err());
    }

    #[test]
    fn test_active_tab_switches_correctly() {
        let mut mgr = TabManager::new();
        mgr.create_tab("t1".to_string()).unwrap();
        mgr.create_tab("t2".to_string()).unwrap();
        mgr.set_active_tab("t1").unwrap();
        mgr.set_active_tab("t2").unwrap();
        assert_eq!(mgr.active_tab_id(), Some("t2"));
        let e1 = mgr.tabs.get("t1").unwrap();
        assert_eq!(e1.status, TabStatus::Background);
    }

    #[test]
    fn test_background_limit_suspends_lru() {
        let mut mgr = TabManager::new();
        // Create 7 tabs and make each active in turn so the previous becomes background.
        for i in 1..=7u32 {
            let id = format!("t{i}");
            mgr.create_tab(id.clone()).unwrap();
            mgr.set_active_tab(&id).unwrap();
            // small sleep to ensure distinct timestamps
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        // Active tab is t7. Background tabs must not exceed the limit.
        let bg = mgr.background_tab_count();
        assert!(
            bg <= BACKGROUND_LIMIT,
            "background count {bg} exceeded limit {BACKGROUND_LIMIT}"
        );
        // At least one tab must be suspended.
        let suspended = mgr
            .tabs
            .values()
            .filter(|e| e.status == TabStatus::Suspended)
            .count();
        assert!(suspended >= 1, "expected at least 1 suspended tab, got {suspended}");
    }

    #[test]
    fn test_suspend_and_resume() {
        let mut mgr = TabManager::new();
        mgr.create_tab("t1".to_string()).unwrap();
        // t1 is Background by default after create
        mgr.suspend_tab("t1").unwrap();
        assert_eq!(mgr.tabs.get("t1").unwrap().status, TabStatus::Suspended);
        mgr.resume_tab("t1").unwrap();
        assert_eq!(mgr.tabs.get("t1").unwrap().status, TabStatus::Background);
    }

    #[test]
    fn test_cannot_suspend_active_tab() {
        let mut mgr = TabManager::new();
        mgr.create_tab("t1".to_string()).unwrap();
        mgr.set_active_tab("t1").unwrap();
        let err = mgr.suspend_tab("t1").unwrap_err();
        assert!(matches!(err, NetraError::InvalidInput(_)));
    }
}
