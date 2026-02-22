use regex::Regex;

/// Domain route rule
#[derive(Debug, Clone)]
pub struct DomainRoute {
    pub id: i64,
    pub priority: i32,
    pub pattern: Regex,
    pub target_addr: String,
    pub target_port: u16,
    pub pattern_str: String, // Original pattern string for display
    pub backend_id: Option<i64>, // Associated backend server ID
}

impl DomainRoute {
    /// Create from wildcard pattern (e.g. *.example.com -> (.+)\.example\.com)
    pub fn new(pattern: &str, target_addr: &str, target_port: u16, backend_id: Option<i64>) -> Result<Self, regex::Error> {
        Self::new_with_priority(pattern, target_addr, target_port, 0, backend_id)
    }

    /// Create from wildcard pattern with priority
    pub fn new_with_priority(pattern: &str, target_addr: &str, target_port: u16, priority: i32, backend_id: Option<i64>) -> Result<Self, regex::Error> {
        // Support three patterns:
        // 1. Starts with ^ or ends with $ -> native regex, use directly
        // 2. Contains * but not .* -> wildcard pattern, escape then replace * with (.+)
        // 3. Others (plain domain) -> escape special characters (like .) and add anchors
        let regex_str = if pattern.starts_with('^') || pattern.ends_with('$') {
            pattern.to_string()
        } else if pattern.contains('*') {
            format!("^{}$", regex::escape(pattern).replace(r"\*", "(.+)"))
        } else {
            format!("^{}$", regex::escape(pattern))
        };
        let regex = Regex::new(&regex_str)?;
        Ok(Self {
            id: 0, // Set when loading from database
            priority,
            pattern: regex,
            target_addr: target_addr.to_string(),
            target_port,
            pattern_str: pattern.to_string(),
            backend_id,
        })
    }

    /// Convert from database DomainRoute
    pub fn from_db(route: &crate::db::domain_routes::DomainRoute) -> Result<Self, regex::Error> {
        let mut r = Self::new_with_priority(&route.pattern, &route.target_addr, route.target_port, route.priority, route.backend_id)?;
        r.id = route.id;
        Ok(r)
    }

    pub fn matches(&self, address: &str) -> bool {
        self.pattern.is_match(address)
    }
}

/// Extract pure domain from server_address (remove FML marker and port)
pub fn extract_domain(server_address: &str) -> &str {
    // 1. Remove FML/Forge marker (separated by \0)
    let after_fml = server_address.split('\0').next().unwrap_or(server_address);
    // 2. Remove port number (e.g. play.example.com:25565 -> play.example.com)
    //    IPv6 address format is [::1]:25565, no special handling needed (rsplit(':').last() doesn't work for left segment)
    //    Use rsplit_once(':') is safer
    if let Some((host, _port)) = after_fml.rsplit_once(':') {
        // Ensure it's a port number (not IPv6 plain address): host cannot be empty
        if !host.is_empty() {
            return host;
        }
    }
    after_fml
}
