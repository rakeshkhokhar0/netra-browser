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

/// Navigation lifecycle events emitted by [NavigationController].
///
/// These events are intentionally transport-agnostic and are prepared for
/// future event-bus wiring in the Rust core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationEvent {
    /// Emitted when a normalized navigation target has been resolved.
    NavigationRequested(TabId, String),
    /// Emitted when navigation is considered complete.
    NavigationCompleted(TabId, String),
}

/// Emits a navigation event.
///
/// This is a temporary stub for future integration with the Rust event bus.
fn emit_event(_event: NavigationEvent) {}

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
#[derive(Debug, Default)]
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
    /// - input containing spaces is treated as a search query
    /// - input without a scheme is prefixed with `https://`
    /// - input with a scheme is kept as-is
    ///
    /// History behavior:
    /// - forward history is truncated before pushing a new entry
    /// - pushed URL becomes the active index
    ///
    /// Emits:
    /// - [NavigationEvent::NavigationRequested]
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

        emit_event(NavigationEvent::NavigationRequested(
            tab_id,
            final_url.clone(),
        ));

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
        let url = history.entries.get(history.current_index).cloned()?;
        emit_event(NavigationEvent::NavigationRequested(
            tab_id,
            url.clone(),
        ));
        Some(url)
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
        let url = history.entries.get(history.current_index).cloned()?;
        emit_event(NavigationEvent::NavigationRequested(
            tab_id,
            url.clone(),
        ));
        Some(url)
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

    /// Normalizes raw navigation input into a final URL.
    ///
    /// Normalization pipeline:
    /// - search-query conversion when spaces are present
    /// - scheme auto-prefixing when missing
    fn normalize_input(input: String) -> String {
        if input.contains(' ') {
            return Self::to_search_url(input);
        }

        if !input.contains("://") {
            return format!("https://{input}");
        }

        input
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
}
