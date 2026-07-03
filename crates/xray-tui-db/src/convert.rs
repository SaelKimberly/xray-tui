use crate::columns::*;
use crate::models::*;

impl Profile {
    pub(crate) fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            id: row.get::<String>(ProfileCol::Id as usize)?,
            config_type: row.get::<i32>(ProfileCol::ConfigType as usize)?,
            core_type: row.get::<String>(ProfileCol::CoreType as usize)?,
            remarks: row.get::<Option<String>>(ProfileCol::Remarks as usize)?,
            address: row.get::<Option<String>>(ProfileCol::Address as usize)?,
            port: row.get::<Option<i32>>(ProfileCol::Port as usize)?,
            user_id: row.get::<Option<String>>(ProfileCol::UserId as usize)?,
            security: row.get::<Option<String>>(ProfileCol::Security as usize)?,
            network: row.get::<Option<String>>(ProfileCol::Network as usize)?,
            stream_settings: row.get::<Option<String>>(ProfileCol::StreamSettings as usize)?,
            protocol_settings: row.get::<Option<String>>(ProfileCol::ProtocolSettings as usize)?,
            is_sub: row.get::<Option<i32>>(ProfileCol::IsSub as usize)?,
            sub_id: row.get::<Option<String>>(ProfileCol::SubId as usize)?,
            group_id: row.get::<Option<String>>(ProfileCol::GroupId as usize)?,
            sort_order: row.get::<Option<i32>>(ProfileCol::SortOrder as usize)?,
            is_active: row.get::<Option<i32>>(ProfileCol::IsActive as usize)?,
            created_at: row.get::<Option<String>>(ProfileCol::CreatedAt as usize)?,
            updated_at: row.get::<Option<String>>(ProfileCol::UpdatedAt as usize)?,
            sub_uid: row.get::<Option<i64>>(ProfileCol::SubUid as usize)?,
        })
    }
}

impl Subscription {
    pub(crate) fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            id: row.get::<String>(SubscriptionCol::Id as usize)?,
            group_id: row.get::<Option<String>>(SubscriptionCol::GroupId as usize)?,
            url: row.get::<String>(SubscriptionCol::Url as usize)?,
            last_updated: row.get::<Option<String>>(SubscriptionCol::LastUpdated as usize)?,
            update_interval: row.get::<Option<i32>>(SubscriptionCol::UpdateInterval as usize)?,
            user_agent: row.get::<Option<String>>(SubscriptionCol::UserAgent as usize)?,
            status: row.get::<Option<String>>(SubscriptionCol::Status as usize)?,
            error_message: row.get::<Option<String>>(SubscriptionCol::ErrorMessage as usize)?,
        })
    }
}

impl Group {
    pub(crate) fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            id: row.get::<String>(GroupCol::Id as usize)?,
            name: row.get::<Option<String>>(GroupCol::Name as usize)?,
            subscription_url: row.get::<Option<String>>(GroupCol::SubscriptionUrl as usize)?,
            subscription_enabled: row.get::<Option<i32>>(GroupCol::SubscriptionEnabled as usize)?,
            user_agent: row.get::<Option<String>>(GroupCol::UserAgent as usize)?,
            convert_target: row.get::<Option<i32>>(GroupCol::ConvertTarget as usize)?,
            core_type: row.get::<Option<String>>(GroupCol::CoreType as usize)?,
            sort_order: row.get::<Option<i32>>(GroupCol::SortOrder as usize)?,
            is_system: row.get::<Option<i32>>(GroupCol::IsSystem as usize)?,
        })
    }
}

impl ProfileExtension {
    pub(crate) fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            profile_id: row.get::<String>(ProfileExtCol::ProfileId as usize)?,
            delay: row.get::<Option<i32>>(ProfileExtCol::Delay as usize)?,
            speed: row.get::<Option<i32>>(ProfileExtCol::Speed as usize)?,
            sort_order: row.get::<Option<i32>>(ProfileExtCol::SortOrder as usize)?,
            ip_info: row.get::<Option<String>>(ProfileExtCol::IpInfo as usize)?,
        })
    }
}

impl ServerStat {
    pub(crate) fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            profile_id: row.get::<String>(ServerStatCol::ProfileId as usize)?,
            today_up: row.get::<Option<i64>>(ServerStatCol::TodayUp as usize)?,
            today_down: row.get::<Option<i64>>(ServerStatCol::TodayDown as usize)?,
            total_up: row.get::<Option<i64>>(ServerStatCol::TotalUp as usize)?,
            total_down: row.get::<Option<i64>>(ServerStatCol::TotalDown as usize)?,
            last_updated: row.get::<Option<String>>(ServerStatCol::LastUpdated as usize)?,
        })
    }
}

impl PingSession {
    pub(crate) fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            id: row.get::<String>(PingSessionCol::Id as usize)?,
            batch_id: row.get::<String>(PingSessionCol::BatchId as usize)?,
            profile_id: row.get::<String>(PingSessionCol::ProfileId as usize)?,
            config_type: row.get::<i32>(PingSessionCol::ConfigType as usize)?,
            core_type: row.get::<String>(PingSessionCol::CoreType as usize)?,
            address: row.get::<Option<String>>(PingSessionCol::Address as usize)?,
            port: row.get::<Option<i32>>(PingSessionCol::Port as usize)?,
            triplet_rank: row.get::<i32>(PingSessionCol::TripletRank as usize)?,
            ping_type: row.get::<String>(PingSessionCol::PingType as usize)?,
            status: row.get::<String>(PingSessionCol::Status as usize)?,
            latency_ms: row.get::<Option<i32>>(PingSessionCol::LatencyMs as usize)?,
            speed_bps: row.get::<Option<i32>>(PingSessionCol::SpeedBps as usize)?,
            ip_info: row.get::<Option<String>>(PingSessionCol::IpInfo as usize)?,
            error: row.get::<Option<String>>(PingSessionCol::Error as usize)?,
            created_at: row.get::<Option<String>>(PingSessionCol::CreatedAt as usize)?,
            updated_at: row.get::<Option<String>>(PingSessionCol::UpdatedAt as usize)?,
        })
    }
}

impl RoutingRule {
    pub(crate) fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            id: row.get::<String>(RoutingRuleCol::Id as usize)?,
            group_id: row.get::<Option<String>>(RoutingRuleCol::GroupId as usize)?,
            r#type: row.get::<i32>(RoutingRuleCol::Type as usize)?,
            domain_matcher: row.get::<Option<String>>(RoutingRuleCol::DomainMatcher as usize)?,
            domains: row.get::<Option<String>>(RoutingRuleCol::Domains as usize)?,
            ips: row.get::<Option<String>>(RoutingRuleCol::Ips as usize)?,
            inbound_tags: row.get::<Option<String>>(RoutingRuleCol::InboundTags as usize)?,
            port: row.get::<Option<String>>(RoutingRuleCol::Port as usize)?,
            source_ports: row.get::<Option<String>>(RoutingRuleCol::SourcePorts as usize)?,
            network: row.get::<Option<String>>(RoutingRuleCol::Network as usize)?,
            protocols: row.get::<Option<String>>(RoutingRuleCol::Protocols as usize)?,
            domain_strategy: row.get::<Option<String>>(RoutingRuleCol::DomainStrategy as usize)?,
            outbound_tag: row.get::<Option<String>>(RoutingRuleCol::OutboundTag as usize)?,
            balancer_tag: row.get::<Option<String>>(RoutingRuleCol::BalancerTag as usize)?,
            rule_set_file: row.get::<Option<String>>(RoutingRuleCol::RuleSetFile as usize)?,
            rule_set_url: row.get::<Option<String>>(RoutingRuleCol::RuleSetUrl as usize)?,
            sort_order: row.get::<Option<i32>>(RoutingRuleCol::SortOrder as usize)?,
        })
    }
}

impl DnsSetting {
    pub(crate) fn from_row(row: &turso::Row) -> turso::Result<Self> {
        Ok(Self {
            id: row.get::<String>(DnsSettingCol::Id as usize)?,
            name: row.get::<Option<String>>(DnsSettingCol::Name as usize)?,
            servers: row.get::<Option<String>>(DnsSettingCol::Servers as usize)?,
            hosts: row.get::<Option<String>>(DnsSettingCol::Hosts as usize)?,
            query_strategy: row.get::<Option<String>>(DnsSettingCol::QueryStrategy as usize)?,
            disable_cache: row.get::<Option<i32>>(DnsSettingCol::DisableCache as usize)?,
            disable_fallback: row.get::<Option<i32>>(DnsSettingCol::DisableFallback as usize)?,
            client_ip: row.get::<Option<String>>(DnsSettingCol::ClientIp as usize)?,
        })
    }
}
