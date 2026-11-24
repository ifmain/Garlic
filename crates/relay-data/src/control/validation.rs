use super::{Authority, ErrorCode, ProxyError, StreamOpen};

/// Syntax accepted by the deployed exit. Address resolution and destination
/// policy are independent of this wire-level check.
pub fn valid_connect_authority(authority: &str) -> bool {
    if authority.is_empty() || authority.len() > 255 {
        return false;
    }
    let (host, port) = if let Some(rest) = authority.strip_prefix('[') {
        let Some(parts) = rest.split_once("]:") else {
            return false;
        };
        parts
    } else {
        let Some(parts) = authority.split_once(':') else {
            return false;
        };
        parts
    };
    !host.is_empty()
        && !host.chars().any(|ch| {
            ch.is_whitespace() || ch.is_control() || matches!(ch, '/' | '?' | '#' | '@' | '[' | ']')
        })
        && !port.is_empty()
        && port.bytes().all(|byte| byte.is_ascii_digit())
        && port.parse::<u16>().is_ok_and(|number| number != 0)
}

impl StreamOpen {
    /// Check the protocol contract in the same order as the deployed worker.
    pub fn validate(&self, owner: &Authority, now_unix_ms: u64) -> Result<(), ProxyError> {
        use ErrorCode::*;
        let code = if self.control_version != 1 {
            IncompatibleControlVersion
        } else if self.data_version != 1 {
            IncompatibleDataVersion
        } else if &self.authority != owner {
            StaleAuthority
        } else if self.node_id.is_empty()
            || self.node_id.len() > 255
            || self
                .node_id
                .chars()
                .any(|ch| ch.is_control() || ch.is_whitespace())
            || [self.channel_id, self.user_id, self.flow_id, self.stream_id]
                .iter()
                .any(|id| id.is_nil())
        {
            InvalidIdentifier
        } else if !valid_connect_authority(&self.connect_authority) {
            InvalidAuthority
        } else if self.deadline_unix_ms <= now_unix_ms {
            DeadlineExpired
        } else if self.capabilities.len() > 16 {
            UnsupportedCapability
        } else if !(1024..=1_048_576).contains(&self.window_size) {
            InvalidWindow
        } else {
            return Ok(());
        };
        Err(code.into())
    }
}

impl From<ErrorCode> for ProxyError {
    fn from(code: ErrorCode) -> Self {
        use ErrorCode::*;
        let message = match code {
            IncompatibleControlVersion => "unsupported control API version",
            IncompatibleDataVersion => "unsupported data plane version",
            UnsupportedCapability => "unsupported proxy capability",
            InvalidAuthority => "invalid destination authority",
            InvalidDestination => "destination must be host:port metadata",
            InvalidIdentifier => "missing or invalid stream identifier",
            InvalidWindow => "window size is outside the supported range",
            DeadlineExpired => "stream deadline has expired",
            StaleAuthority => "authority or epoch is stale",
            RouteUnavailable => "proxy route is temporarily unavailable",
            Unauthorized => "proxy stream is not authorized",
            SubscriptionExpired => "user subscription has expired",
            Forbidden => "proxy stream belongs to another user",
            CapacityExhausted => "proxy stream capacity is exhausted",
            InvalidRequest => "invalid proxy stream request",
        };
        Self {
            code,
            message: message.into(),
            retryable: matches!(code, StaleAuthority | RouteUnavailable | CapacityExhausted),
        }
    }
}
