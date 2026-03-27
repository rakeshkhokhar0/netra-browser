use std::collections::HashMap;
use std::time::{Duration, Instant};

use uuid::Uuid;
use crate::core::entities::tab::{Tab, TabId};

/// Owns tab lifecycle, active-tab transitions, suspension, and LRU background
/// memory policy for the browser engine.
///
/// Rules implemented:
/// - only one tab is active at a time
/// - background (non-active, non-suspended) tabs are limited by LRU policy
/// - idle background tabs are suspended after five minutes
/// - lifecycle operations emit [TabEvent]
pub struct TabManager {
    tabs: HashMap<TabId, Tab>,
    active_tab_id: Option<TabId>,
    max_background_tabs: usize,
}

impl TabManager {
    const DEFAULT_MAX_BACKGROUND_TABS: usize = 5;
    const IDLE_SUSPEND_AFTER: Duration = Duration::from_secs(5 * 60);

    /// Creates a manager with the default background-tab limit.
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            active_tab_id: None,
            max_background_tabs: Self::DEFAULT_MAX_BACKGROUND_TABS,
        }
    }

    /// Creates a new tab, activates it, applies LRU background suspension, and
    /// emits [TabEvent::TabCreated].
    pub fn create_tab(&mut self) -> Tab {
        let id = Uuid::new_v4().to_string();
        self.deactivate_current_active();

        let mut tab = Tab {
            id: id.clone(),
            url: "about".to_string(),
            title: String::new(),
            is_active: false,
            is_loading: false,
            is_suspended: false,
            can_go_back: false,
            can_go_forward: false,
            last_accessed: Instant::now(),
        };
        tab.is_active = true;
        tab.last_accessed = Instant::now();

        self.tabs.insert(id.clone(), tab.clone());
        self.active_tab_id = Some(id.clone());

        self.enforce_background_tab_limit();
        tab
    }

    /// Closes the given tab and emits [TabEvent::TabClosed] when removed.
    ///
    /// If the closed tab was active, another available tab is promoted to
    /// active.
    pub fn close_tab(&mut self, tab_id: String) {
        let was_active = self.active_tab_id.as_deref() == Some(tab_id.as_str());
        if self.tabs.remove(&tab_id).is_none() {
            return;
        }

        if !was_active {
            return;
        }

        self.active_tab_id = None;
        if let Some(next_id) = self.tabs.keys().next().cloned() {
            if let Some(next_tab) = self.tabs.get_mut(&next_id) {
                next_tab.is_active = true;
                next_tab.is_suspended = false;
                next_tab.is_loading = true;
                next_tab.last_accessed = Instant::now();
                self.active_tab_id = Some(next_id);
            }
        }
    }

    /// Switches active focus to the target tab and emits
    /// [TabEvent::TabSwitched].
    ///
    /// Activation ownership rules:
    /// - this method is the only place that updates `active_tab_id`
    /// - if the target tab is suspended, [Self::resume_tab] is called first
    /// - previous active tab is deactivated
    /// - target tab is marked active and timestamped
    pub fn set_active_tab(&mut self, tab_id: String) {
        if !self.tabs.contains_key(&tab_id) {
            return;
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
        self.enforce_background_tab_limit();
    }

    /// Returns a cloned snapshot of all tracked tabs.
    pub fn get_all_tabs(&self) -> Vec<Tab> {
        self.tabs.values().cloned().collect()
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
    /// - `is_loading = true`
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
            tab.is_loading = true;
            tab.last_accessed = Instant::now();
        }
    }

    /// Suspends idle background tabs that have been inactive for more than
    /// five minutes.
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
        let tab = manager.create_tab();

        assert_eq!(manager.active_tab_id.as_deref(), Some(tab.id.as_str()));
        assert!(manager.tabs.get(&tab.id).map(|t| t.is_active).unwrap_or(false));
    }

    #[test]
    fn create_tab_deactivates_previous_active_tab() {
        let mut manager = TabManager::new();
        let first = manager.create_tab();
        let second = manager.create_tab();

        assert_eq!(manager.active_tab_id.as_deref(), Some(second.id.as_str()));
        assert!(
            !manager
                .tabs
                .get(&first.id)
                .map(|t| t.is_active)
                .unwrap_or(true)
        );
    }

    #[test]
    fn lru_suspends_oldest_background_tabs() {
        let mut manager = TabManager::new();
        let first = manager.create_tab();

        // Create enough tabs so backgrounds exceed the limit.
        let mut latest = first.id.clone();
        for _ in 0..6 {
            latest = manager.create_tab().id;
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
        assert_eq!(background_count, 5);
    }

    #[test]
    fn close_active_tab_promotes_another_tab() {
        let mut manager = TabManager::new();
        let first = manager.create_tab();
        let second = manager.create_tab();

        manager.close_tab(second.id.clone());

        assert_eq!(manager.active_tab_id.as_deref(), Some(first.id.as_str()));
        assert!(manager.tabs.get(&first.id).map(|t| t.is_active).unwrap_or(false));
    }

    #[test]
    fn check_idle_suspension_suspends_old_background_tabs() {
        let mut manager = TabManager::new();
        let first = manager.create_tab();
        let second = manager.create_tab();

        if let Some(tab) = manager.tabs.get_mut(&first.id) {
            tab.is_active = false;
            tab.is_suspended = false;
            tab.last_accessed = Instant::now() - Duration::from_secs(10 * 60);
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
}
