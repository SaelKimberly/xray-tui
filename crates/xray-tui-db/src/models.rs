use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub config_type: i32,
    pub core_type: String,
    pub remarks: Option<String>,
    pub address: Option<String>,
    pub port: Option<i32>,
    pub user_id: Option<String>,
    pub security: Option<String>,
    pub network: Option<String>,
    pub stream_settings: Option<String>,
    pub protocol_settings: Option<String>,
    pub is_sub: Option<i32>,
    pub sub_id: Option<String>,
    pub group_id: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<i32>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: Option<String>,
    pub subscription_url: Option<String>,
    pub subscription_enabled: Option<i32>,
    pub user_agent: Option<String>,
    pub convert_target: Option<i32>,
    pub core_type: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub group_id: Option<String>,
    pub url: String,
    pub last_updated: Option<String>,
    pub update_interval: Option<i32>,
    pub user_agent: Option<String>,
    pub status: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub id: String,
    pub group_id: Option<String>,
    pub r#type: i32,
    pub domain_matcher: Option<String>,
    pub domains: Option<String>,
    pub ips: Option<String>,
    pub inbound_tags: Option<String>,
    pub port: Option<String>,
    pub source_ports: Option<String>,
    pub network: Option<String>,
    pub protocols: Option<String>,
    pub domain_strategy: Option<String>,
    pub outbound_tag: Option<String>,
    pub balancer_tag: Option<String>,
    pub rule_set_file: Option<String>,
    pub rule_set_url: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsSetting {
    pub id: String,
    pub name: Option<String>,
    pub servers: Option<String>,
    pub hosts: Option<String>,
    pub query_strategy: Option<String>,
    pub disable_cache: Option<i32>,
    pub disable_fallback: Option<i32>,
    pub client_ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileExtension {
    pub profile_id: String,
    pub delay: Option<i32>,
    pub speed: Option<i32>,
    pub sort_order: Option<i32>,
    pub ip_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStat {
    pub profile_id: String,
    pub today_up: Option<i32>,
    pub today_down: Option<i32>,
    pub total_up: Option<i32>,
    pub total_down: Option<i32>,
    pub last_updated: Option<String>,
}
