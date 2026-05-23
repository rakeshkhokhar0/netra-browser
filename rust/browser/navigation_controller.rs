/// Owns URL normalization behavior for user navigation input.
///
/// Responsibilities:
/// - normalize raw input into final URLs
///
/// Non-responsibilities:
/// - tab creation, deletion, activation ownership
/// - back/forward state ownership (native WebView2 is authoritative)
/// - WebView or platform-layer interaction
/// - async execution or thread management
#[derive(Debug, Clone, Default)]
pub struct NavigationController;

impl NavigationController {
    /// Creates a URL-normalization controller.
    pub fn new() -> Self {
        Self
    }

    /// Normalizes raw user input and returns the final URL.
    ///
    /// Rules:
    /// - free-form queries are treated as search input
    /// - direct addresses without a scheme are prefixed with `https://`
    /// - input with a scheme is kept as-is
    pub fn navigate(&self, input: String) -> String {
        Self::normalize_input(input)
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
        let encoded = query.split_whitespace().collect::<Vec<&str>>().join("+");
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
        let host_part = input.split(['/', '?', '#']).next().unwrap_or(input);
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
    fn navigate_normalizes_search_queries() {
        let controller = NavigationController::new();
        let normalized = controller.navigate("open ai gpt".to_string());
        assert_eq!(normalized, "https://www.google.com/search?q=open+ai+gpt");
    }
}
