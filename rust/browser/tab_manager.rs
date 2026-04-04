use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::core::entities::tab::{Tab, TabId};
use crate::core::error::NetraError;
use uuid::Uuid;

/// Owns tab lifecycle, active-tab transitions, suspension, and LRU background
/// memory policy for the browser engine.
///
/// Rules implemented:
/// - only one tab is active at a time
/// - background (non-active, non-suspended) tabs are limited by LRU policy
/// - idle background tabs are suspended after five minutes
/// - lifecycle operations emit [TabEvent]
#[derive(Clone)]
pub struct TabManager {
    tabs: HashMap<TabId, Tab>,
    tab_order: Vec<TabId>,
    active_tab_id: Option<TabId>,
    max_tabs: usize,
    max_background_tabs: usize,
}

impl TabManager {
    const DEFAULT_MAX_TABS: usize = 20;
    const DEFAULT_MAX_BACKGROUND_TABS: usize = 10;
    const IDLE_SUSPEND_AFTER: Duration = Duration::from_secs(2 * 60);

    /// Creates a manager with the default background-tab limit.
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            tab_order: Vec::new(),
            active_tab_id: None,
            max_tabs: Self::DEFAULT_MAX_TABS,
            max_background_tabs: Self::DEFAULT_MAX_BACKGROUND_TABS,
        }
    }

    /// Creates a new tab, activates it, applies LRU background suspension, and
    /// emits [TabEvent::TabCreated].
    pub fn create_tab(&mut self) -> Result<Tab, NetraError> {
        self.check_idle_suspension();
        if self.tabs.len() >= self.max_tabs {
            return Err(NetraError::OperationFailed(format!(
                "maximum tab limit of {} reached",
                self.max_tabs,
            )));
        }

        let id = Uuid::new_v4().to_string();
        self.deactivate_current_active();

        let mut tab = Tab {
            id: id.clone(),
            url: "about".to_string(),
            title: String::new(),
            favicon_url: None,
            is_active: false,
            is_loading: false,
            is_suspended: false,
            can_go_back: false,
            can_go_forward: false,
            blocked_count: 0,
            sequence_number: 0,
            last_accessed: Instant::now(),
        };
        tab.is_active = true;
        tab.last_accessed = Instant::now();

        self.tabs.insert(id.clone(), tab.clone());
        self.tab_order.push(id.clone());
        self.active_tab_id = Some(id.clone());

        self.apply_suspension_policies();
        Ok(tab)
    }

    /// Closes the given tab and emits [TabEvent::TabClosed] when removed.
    ///
    /// If the closed tab was active, another available tab is promoted to
    /// active.
    pub fn close_tab(&mut self, tab_id: String) -> bool {
        let was_active = self.active_tab_id.as_deref() == Some(tab_id.as_str());
        if self.tabs.remove(&tab_id).is_none() {
            return false;
        }
        self.tab_order.retain(|existing_id| existing_id != &tab_id);

        if !was_active {
            return true;
        }

        self.active_tab_id = None;
        if let Some(next_id) = self.tab_order.first().cloned() {
            if let Some(next_tab) = self.tabs.get_mut(&next_id) {
                next_tab.is_active = true;
                next_tab.is_suspended = false;
                next_tab.last_accessed = Instant::now();
                self.active_tab_id = Some(next_id);
            }
        }

        true
    }

    /// Switches active focus to the target tab and emits
    /// [TabEvent::TabSwitched].
    ///
    /// Activation ownership rules:
    /// - this method is the only place that updates `active_tab_id`
    /// - if the target tab is suspended, [Self::resume_tab] is called first
    /// - previous active tab is deactivated
    /// - target tab is marked active and timestamped
    pub fn set_active_tab(&mut self, tab_id: String) -> bool {
        self.check_idle_suspension();

        if !self.tabs.contains_key(&tab_id) {
            return false;
        }

        if self
            .tabs
            .get(&tab_id)
            .map(|tab| tab.is_suspended)
            .unwrap_or(false)
        {
            self.resume_tab(tab_id.clone());
        }

        self.deactivate_current_active();

        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            tab.is_active = true;
            tab.last_accessed = Instant::now();
            tab.is_suspended = false;
        }

        self.active_tab_id = Some(tab_id.clone());
        self.apply_suspension_policies();
        true
    }

    /// Returns a cloned snapshot of all tracked tabs.
    pub fn get_all_tabs(&self) -> Vec<Tab> {
        self.tab_order
            .iter()
            .filter_map(|tab_id| self.tabs.get(tab_id).cloned())
            .collect()
    }

    /// Returns whether the target tab exists in the manager.
    pub fn contains_tab(&self, tab_id: &str) -> bool {
        self.tabs.contains_key(tab_id)
    }

    /// Returns a cloned snapshot of the specified tab when it exists.
    pub fn get_tab(&self, tab_id: &str) -> Option<Tab> {
        self.tabs.get(tab_id).cloned()
    }

    /// Returns whether the specified tab is currently suspended.
    pub fn is_tab_suspended(&self, tab_id: &str) -> bool {
        self.tabs
            .get(tab_id)
            .map(|tab| tab.is_suspended)
            .unwrap_or(false)
    }

    /// Returns the number of tabs currently tracked by the manager.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Returns the identifier of the currently active tab, if any.
    ///
    /// This exposes the manager-owned active-tab pointer so coordinating
    /// modules can read active state without scanning all tab records.
    pub fn get_active_tab_id(&self) -> Option<TabId> {
        self.active_tab_id.clone()
    }

    /// Synchronizes navigation-derived state onto an existing tab.
    ///
    /// This keeps [TabManager] as the single source of truth for tab-visible
    /// state while allowing higher-level coordinators to mirror history and
    /// URL changes computed by other browser modules.
    pub fn update_navigation_state(
        &mut self,
        tab_id: &str,
        url: String,
        can_go_back: bool,
        can_go_forward: bool,
    ) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.url = url;
            tab.can_go_back = can_go_back;
            tab.can_go_forward = can_go_forward;
        }
    }

    /// Updates only the visible URL for a specific tab.
    pub fn update_url(&mut self, tab_id: &str, url: String) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.url = url;
        }
    }

    /// Updates only the visible title for a specific tab.
    pub fn update_title(&mut self, tab_id: &str, title: String) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.title = title;
        }
    }

    /// Updates only the favicon URL for a specific tab.
    pub fn update_favicon_url(&mut self, tab_id: &str, favicon_url: Option<String>) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.update_favicon_url(favicon_url);
        }
    }

    /// Updates the loading flag for a specific tab.
    pub fn set_loading_state(&mut self, tab_id: &str, is_loading: bool) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.is_loading = is_loading;
        }
    }

    /// Increments the blocked-request counter for a specific tab.
    pub fn increment_blocked_count(&mut self, tab_id: &str) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.increment_blocked_count();
        }
    }

    /// Suspends a tab when it is a non-active, non-suspended background tab
    /// and emits [TabEvent::TabSuspended].
    pub fn suspend_tab(&mut self, tab_id: String) {
        let Some(tab) = self.tabs.get_mut(&tab_id) else {
            return;
        };

        if tab.is_active || tab.is_suspended {
            return;
        }

        tab.is_suspended = true;
        tab.is_loading = false;
    }

    /// Resumes a suspended tab and emits [TabEvent::TabResumed].
    ///
    /// This method intentionally does not change active-tab ownership.
    /// It only applies resume-state fields:
    /// - `is_suspended = false`
    /// - `last_accessed = now`
    ///
    /// Any active-tab switch must be performed by [Self::set_active_tab].
    pub fn resume_tab(&mut self, tab_id: String) {
        let is_suspended = self
            .tabs
            .get(&tab_id)
            .map(|tab| tab.is_suspended)
            .unwrap_or(false);
        if !is_suspended {
            return;
        }

        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            tab.is_suspended = false;
            tab.last_accessed = Instant::now();
        }
    }

    /// Suspends idle background tabs that have been inactive for more than
    /// two minutes.
    pub fn check_idle_suspension(&mut self) {
        let now = Instant::now();
        let to_suspend: Vec<TabId> = self
            .tabs
            .values()
            .filter(|tab| {
                !tab.is_active
                    && !tab.is_suspended
                    && now.duration_since(tab.last_accessed) > Self::IDLE_SUSPEND_AFTER
            })
            .map(|tab| tab.id.clone())
            .collect();

        for tab_id in to_suspend {
            self.suspend_tab(tab_id);
        }
    }

    fn deactivate_current_active(&mut self) {
        if let Some(active_id) = self.active_tab_id.as_ref() {
            if let Some(active_tab) = self.tabs.get_mut(active_id) {
                active_tab.is_active = false;
            }
        }
    }

    fn apply_suspension_policies(&mut self) {
        self.check_idle_suspension();
        self.enforce_background_tab_limit();
    }

    /// Increments and returns the sequence number for the given tab.
    pub fn increment_sequence(&mut self, tab_id: &str) -> u32 {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.sequence_number = tab.sequence_number.wrapping_add(1);
            return tab.sequence_number;
        }
        0
    }

    /// Enforces LRU suspension for background tabs.
    ///
    /// Background tabs are those with:
    /// - `is_active == false`
    /// - `is_suspended == false`
    ///
    /// When the count exceeds `max_background_tabs`, the oldest by
    /// `last_accessed` are suspended until the limit is satisfied.
    fn enforce_background_tab_limit(&mut self) {
        let mut background_tabs: Vec<(TabId, Instant)> = self
            .tabs
            .values()
            .filter(|tab| !tab.is_active && !tab.is_suspended)
            .map(|tab| (tab.id.clone(), tab.last_accessed))
            .collect();

        if background_tabs.len() <= self.max_background_tabs {
            return;
        }

        background_tabs.sort_by_key(|(_, accessed_at)| *accessed_at);
        let to_suspend_count = background_tabs.len() - self.max_background_tabs;

        for (tab_id, _) in background_tabs.into_iter().take(to_suspend_count) {
            self.suspend_tab(tab_id);
        }
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_tab_sets_new_tab_active() {
        let mut manager = TabManager::new();
        let tab = manager.create_tab().expect("tab should be created");

        assert_eq!(manager.active_tab_id.as_deref(), Some(tab.id.as_str()));
        assert!(manager.tabs.get(&tab.id).map(|t| t.is_active).unwrap_or(false));
        assert_eq!(manager.tab_order, vec![tab.id]);
    }

    #[test]
    fn create_tab_deactivates_previous_active_tab() {
        let mut manager = TabManager::new();
        let first = manager.create_tab().expect("first tab should be created");
        let second = manager.create_tab().expect("second tab should be created");

        assert_eq!(manager.active_tab_id.as_deref(), Some(second.id.as_str()));
        assert_eq!(manager.tab_order, vec![first.id.clone(), second.id.clone()]);
        assert!(
            !manager
                .tabs
                .get(&first.id)
                .map(|t| t.is_active)
                .unwrap_or(true)
        );
    }

    #[test]
    fn lru_suspends_oldest_background_tabs_when_background_limit_exceeded() {
        let mut manager = TabManager::new();
        let first = manager.create_tab().expect("first tab should be created");

        // Create enough tabs so backgrounds exceed the limit.
        let mut latest = first.id.clone();
        for _ in 0..11 {
            latest = manager
                .create_tab()
                .expect("tab should be created within max limit")
                .id;
        }

        // Mark oldest background tab as very old to make it deterministic LRU.
        if let Some(tab) = manager.tabs.get_mut(&first.id) {
            tab.is_active = false;
            tab.last_accessed = Instant::now() - Duration::from_secs(60 * 60);
        }

        // Trigger enforcement after manual timestamp change.
        manager.set_active_tab(latest);

        let background_count = manager
            .tabs
            .values()
            .filter(|t| !t.is_active && !t.is_suspended)
            .count();
        assert_eq!(background_count, 10);
    }

    #[test]
    fn close_active_tab_promotes_another_tab() {
        let mut manager = TabManager::new();
        let first = manager.create_tab().expect("first tab should be created");
        let second = manager.create_tab().expect("second tab should be created");

        manager.close_tab(second.id.clone());

        assert_eq!(manager.active_tab_id.as_deref(), Some(first.id.as_str()));
        assert_eq!(manager.tab_order, vec![first.id.clone()]);
        assert!(manager.tabs.get(&first.id).map(|t| t.is_active).unwrap_or(false));
    }

    #[test]
    fn get_all_tabs_preserves_creation_order() {
        let mut manager = TabManager::new();
        let first = manager.create_tab().expect("first tab should be created");
        let second = manager.create_tab().expect("second tab should be created");
        let third = manager.create_tab().expect("third tab should be created");

        let ordered_ids: Vec<String> = manager
            .get_all_tabs()
            .into_iter()
            .map(|tab| tab.id)
            .collect();

        assert_eq!(ordered_ids, vec![first.id, second.id, third.id]);
    }

    #[test]
    fn close_middle_tab_preserves_remaining_order() {
        let mut manager = TabManager::new();
        let first = manager.create_tab().expect("first tab should be created");
        let second = manager.create_tab().expect("second tab should be created");
        let third = manager.create_tab().expect("third tab should be created");

        manager.close_tab(second.id);

        let ordered_ids: Vec<String> = manager
            .get_all_tabs()
            .into_iter()
            .map(|tab| tab.id)
            .collect();

        assert_eq!(ordered_ids, vec![first.id, third.id]);
    }

    #[test]
    fn check_idle_suspension_suspends_background_tabs_after_two_minutes() {
        let mut manager = TabManager::new();
        let first = manager.create_tab().expect("first tab should be created");
        let second = manager.create_tab().expect("second tab should be created");

        if let Some(tab) = manager.tabs.get_mut(&first.id) {
            tab.is_active = false;
            tab.is_suspended = false;
            tab.last_accessed = Instant::now() - Duration::from_secs(3 * 60);
        }
        manager.set_active_tab(second.id.clone());

        manager.check_idle_suspension();

        assert!(
            manager
                .tabs
                .get(&first.id)
                .map(|tab| tab.is_suspended)
                .unwrap_or(false)
        );
    }

    #[test]
    fn resuming_suspended_tab_does_not_mark_it_loading() {
        let mut manager = TabManager::new();
        let first = manager.create_tab().expect("first tab should be created");
        let second = manager.create_tab().expect("second tab should be created");

        manager.suspend_tab(first.id.clone());
        manager.set_active_tab(first.id.clone());

        let resumed_tab = manager.get_tab(&first.id).expect("tab should exist");

        assert_eq!(manager.active_tab_id.as_deref(), Some(first.id.as_str()));
        assert!(resumed_tab.is_active);
        assert!(!resumed_tab.is_suspended);
        assert!(!resumed_tab.is_loading);
        assert!(
            !manager
                .get_tab(&second.id)
                .map(|tab| tab.is_active)
                .unwrap_or(true)
        );
    }

    #[test]
    fn closing_active_tab_does_not_force_promoted_tab_into_loading() {
        let mut manager = TabManager::new();
        let first = manager.create_tab().expect("first tab should be created");
        let second = manager.create_tab().expect("second tab should be created");

        manager.close_tab(second.id);

        let promoted_tab = manager.get_tab(&first.id).expect("tab should exist");

        assert_eq!(manager.active_tab_id.as_deref(), Some(first.id.as_str()));
        assert!(promoted_tab.is_active);
        assert!(!promoted_tab.is_suspended);
        assert!(!promoted_tab.is_loading);
    }

    #[test]
    fn create_tab_rejects_creation_after_max_tab_limit() {
        let mut manager = TabManager::new();

        for _ in 0..TabManager::DEFAULT_MAX_TABS {
            manager
                .create_tab()
                .expect("tab should be created within max tab limit");
        }

        let error = manager
            .create_tab()
            .expect_err("tab creation should fail after max tab limit");

        assert_eq!(manager.tab_count(), TabManager::DEFAULT_MAX_TABS);
        assert_eq!(
            error.to_string(),
            format!(
                "operation failed: maximum tab limit of {} reached",
                TabManager::DEFAULT_MAX_TABS,
            ),
        );
    }
}
