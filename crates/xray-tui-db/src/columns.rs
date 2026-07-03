/// Column indices for Profile JOIN query (gp + pc, 19 cols).
#[repr(usize)]
pub(crate) enum ProfileCol {
    Id = 0,
    SubUid,
    GroupId,
    Remarks,
    IsSub,
    SubId,
    SortOrder,
    IsActive,
    UpdatedAt,
    ConfigType,
    CoreType,
    Address,
    Port,
    UserId,
    Security,
    Network,
    StreamSettings,
    ProtocolSettings,
    CreatedAt,
}

/// Column indices for groups table (SELECT * order).
#[repr(usize)]
pub(crate) enum GroupCol {
    Id = 0,
    Name,
    SubscriptionUrl,
    SubscriptionEnabled,
    UserAgent,
    ConvertTarget,
    CoreType,
    SortOrder,
    IsSystem,
}

/// Column indices for subscriptions table (SELECT * order).
#[repr(usize)]
pub(crate) enum SubscriptionCol {
    Id = 0,
    GroupId,
    Url,
    LastUpdated,
    UpdateInterval,
    UserAgent,
    Status,
    ErrorMessage,
}

/// Column indices for `profile_extensions` table (SELECT * order).
#[repr(usize)]
pub(crate) enum ProfileExtCol {
    ProfileId = 0,
    Delay,
    Speed,
    SortOrder,
    IpInfo,
}

/// Column indices for `server_stats` table (SELECT * order).
#[repr(usize)]
pub(crate) enum ServerStatCol {
    ProfileId = 0,
    TodayUp,
    TodayDown,
    TotalUp,
    TotalDown,
    LastUpdated,
}

/// Column indices for `ping_sessions` table (SELECT * order).
#[repr(usize)]
pub(crate) enum PingSessionCol {
    Id = 0,
    BatchId,
    ProfileId,
    ConfigType,
    CoreType,
    Address,
    Port,
    TripletRank,
    PingType,
    Status,
    LatencyMs,
    SpeedBps,
    IpInfo,
    Error,
    CreatedAt,
    UpdatedAt,
}

/// Column indices for `routing_rules` table (SELECT * order).
#[repr(usize)]
pub(crate) enum RoutingRuleCol {
    Id = 0,
    GroupId,
    Type,
    DomainMatcher,
    Domains,
    Ips,
    InboundTags,
    Port,
    SourcePorts,
    Network,
    Protocols,
    DomainStrategy,
    OutboundTag,
    BalancerTag,
    RuleSetFile,
    RuleSetUrl,
    SortOrder,
}

/// Column indices for `dns_settings` table (SELECT * order).
#[repr(usize)]
pub(crate) enum DnsSettingCol {
    Id = 0,
    Name,
    Servers,
    Hosts,
    QueryStrategy,
    DisableCache,
    DisableFallback,
    ClientIp,
}

/// Column indices for the 3-table LEFT JOIN query (30 cols).
/// Profile columns 0-18, extension columns 19-23, stats columns 24-29.
#[repr(usize)]
#[allow(dead_code)]
pub(crate) enum ProfileDetailsCol {
    // Profile columns (0-18) — same order as ProfileCol
    Id = 0,
    SubUid,
    GroupId,
    Remarks,
    IsSub,
    SubId,
    SortOrder,
    IsActive,
    UpdatedAt,
    ConfigType,
    CoreType,
    Address,
    Port,
    UserId,
    Security,
    Network,
    StreamSettings,
    ProtocolSettings,
    CreatedAt,
    // Extension columns (19-23)
    ExtProfileId,
    Delay,
    Speed,
    ExtSortOrder,
    IpInfo,
    // Stats columns (24-29)
    StatsProfileId,
    TodayUp,
    TodayDown,
    TotalUp,
    TotalDown,
    LastUpdated,
}

/// Column indices for the real-ping JOIN query (35 cols).
/// Profile fields 0-18 (`ProfileCol` order), `PingSession` fields 19-34 (`PingSessionCol` order).
#[repr(usize)]
#[allow(dead_code, clippy::enum_variant_names)]
pub(crate) enum PingSessionJoinCol {
    SessionId = 19,
    SessionBatchId,
    SessionProfileId,
    SessionConfigType,
    SessionCoreType,
    SessionAddress,
    SessionPort,
    SessionTripletRank,
    SessionPingType,
    SessionStatus,
    SessionLatencyMs,
    SessionSpeedBps,
    SessionIpInfo,
    SessionError,
    SessionCreatedAt,
    SessionUpdatedAt,
}
