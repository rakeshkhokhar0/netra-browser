use std::collections::HashMap;

/// Stable identifier type used to address tab-scoped navigation history.
pub type TabId = String;

/// In-memory navigation history for a single tab.
///
/// `entries` stores normalized URLs in visit order, while `current_index`
/// points at the currently active entry inside that list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History {
    /// Ordered list of normalized URLs visited by the tab.
    pub entries: Vec<String>,
    /// Zero-based cursor to the active entry in [History::entries].
    pub current_index: usize,
}

/// Owns tab-scoped navigation history and URL normalization behavior.
///
/// Responsibilities:
/// - normalize raw input into final URLs
/// - manage back/forward history stacks per tab
/// - expose navigation state flags (`can_go_back`, `can_go_forward`)
///
/// Non-responsibilities:
/// - tab creation, deletion, activation ownership
/// - WebView or platform-layer interaction
/// - async execution or thread management
#[derive(Debug, Clone, Default)]
pub struct NavigationController {
    /// Per-tab navigation history storage.
    pub histories: HashMap<TabId, History>,
}

impl NavigationController {
    /// Creates an empty navigation controller with no tab histories.
    pub fn new() -> Self {
        Self {
            histories: HashMap::new(),
        }
    }

    /// Normalizes raw user input, pushes it into tab history, and returns the
    /// final URL.
    ///
    /// Rules:
    /// - free-form queries are treated as search input
    /// - direct addresses without a scheme are prefixed with `https://`
    /// - input with a scheme is kept as-is
    ///
    /// History behavior:
    /// - forward history is truncated before pushing a new entry
    /// - pushed URL becomes the active index
    ///
    pub fn navigate(&mut self, tab_id: TabId, input: String) -> String {
        let final_url = Self::normalize_input(input);
        let history = self.histories.entry(tab_id.clone()).or_insert(History {
            entries: Vec::new(),
            current_index: 0,
        });

        if !history.entries.is_empty() {
            history.entries.truncate(history.current_index + 1);
        }

        history.entries.push(final_url.clone());
        history.current_index = history.entries.len() - 1;

        final_url
    }

    /// Moves the history cursor one step backward for the provided tab.
    ///
    /// Returns the resolved URL when backward navigation is possible, otherwise
    /// returns `None`.
    pub fn go_back(&mut self, tab_id: TabId) -> Option<String> {
        let history = self.histories.get_mut(&tab_id)?;
        if history.current_index == 0 {
            return None;
        }

        history.current_index -= 1;
        history.entries.get(history.current_index).cloned()
    }

    /// Moves the history cursor one step forward for the provided tab.
    ///
    /// Returns the resolved URL when forward navigation is possible, otherwise
    /// returns `None`.
    pub fn go_forward(&mut self, tab_id: TabId) -> Option<String> {
        let history = self.histories.get_mut(&tab_id)?;
        if history.current_index + 1 >= history.entries.len() {
            return None;
        }

        history.current_index += 1;
        history.entries.get(history.current_index).cloned()
    }

    /// Indicates whether backward navigation is currently possible for a tab.
    pub fn can_go_back(&self, tab_id: TabId) -> bool {
        self.histories
            .get(&tab_id)
            .map(|history| history.current_index > 0)
            .unwrap_or(false)
    }

    /// Indicates whether forward navigation is currently possible for a tab.
    pub fn can_go_forward(&self, tab_id: TabId) -> bool {
        self.histories
            .get(&tab_id)
            .map(|history| history.current_index + 1 < history.entries.len())
            .unwrap_or(false)
    }

    /// Removes all navigation history owned by the specified tab.
    ///
    /// This is used when a tab is permanently closed so stale per-tab history
    /// entries do not remain in Rust state after the tab lifecycle ends.
    pub fn remove_tab_history(&mut self, tab_id: &str) {
        self.histories.remove(tab_id);
    }

    /// Normalizes raw navigation input into a final URL.
    ///
    /// Normalization pipeline:
    /// - search-query conversion for non-address input
    /// - scheme auto-prefixing for direct addresses without a scheme
    fn normalize_input(input: String) -> String {
        let trimmed_input = input.trim().to_string();
        if trimmed_input.is_empty() {
            return trimmed_input;
        }

        if trimmed_input.contains(' ') {
            return Self::to_search_url(trimmed_input);
        }

        if trimmed_input.contains("://") {
            return trimmed_input;
        }

        if Self::looks_like_direct_address(&trimmed_input) {
            return format!("https://{trimmed_input}");
        }

        Self::to_search_url(trimmed_input)
    }

    /// Builds a Google search URL from raw query text.
    ///
    /// Query words are joined with `+` to produce a URL-safe search string.
    fn to_search_url(query: String) -> String {
        let encoded = query
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join("+");
        format!("https://www.google.com/search?q={encoded}")
    }

    /// Returns whether the input resembles a direct host/address entry.
    fn looks_like_direct_address(input: &str) -> bool {
        if input.starts_with("localhost") {
            return true;
        }

        if Self::looks_like_ipv4_address(input) {
            return true;
        }

        input.contains('.')
    }

    /// Returns whether the input starts with an IPv4 address, optionally
    /// followed by a port or path.
    fn looks_like_ipv4_address(input: &str) -> bool {
        let host_part = input
            .split(['/', '?', '#'])
            .next()
            .unwrap_or(input);
        let address_part = host_part.split(':').next().unwrap_or(host_part);
        let octets: Vec<&str> = address_part.split('.').collect();
        if octets.len() != 4 {
            return false;
        }

        octets.into_iter().all(|octet| octet.parse::<u8>().is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_tab_history_clears_only_target_tab_history() {
        let mut controller = NavigationController::new();
        let first_tab_id = "first-tab".to_string();
        let second_tab_id = "second-tab".to_string();

        controller.navigate(first_tab_id.clone(), "example.com".to_string());
        controller.navigate(second_tab_id.clone(), "openai.com".to_string());

        controller.remove_tab_history(&first_tab_id);

        assert!(!controller.histories.contains_key(&first_tab_id));
        assert!(controller.histories.contains_key(&second_tab_id));
    }
}
