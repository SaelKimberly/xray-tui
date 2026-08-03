use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;

use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinHandle;
use tracing::warn;

use xray_tui_config::import_export::{encode_profile_spec, profile_config, Profile};
use xray_tui_config::{AppConfig, ValidationSettings, ValidationSummary};
use xray_tui_core::grpc_client;
use xray_tui_core::log_heed::HeedLogStorage;
use xray_tui_core::protocol::Protocol;
use xray_tui_core::speed_test::TestType;
use xray_tui_core::{
    ping::PingError, BackendUpdateStatus, BuildParams, CLASH_API_PORT, ConfigBuilder,
    RealCoreManager, CoreManager, CoreType, find_binary, resolve_core,
};
use xray_tui_db::models::{
    DnsSetting, EndpointGroup, Group, PingResultUpdate, PingSession, ProfileExtension, ProtocolRow,
    RoutingRule, ServerStat,
};
use xray_tui_db::Database;
use xray_tui_proto::proto_spec::ProtoSpec;

use crate::types::*;
use crate::ui::settings::PROTOCOL_CORE_DEFS;
use crate::{common_field_defaults, get_field, try_send_or_warn, ClashTraffic};
