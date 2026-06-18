pub mod ui;

use std::str::FromStr;
use xray_tui_config::AppConfig;
use xray_tui_core::protocol::Protocol;
use xray_tui_core::{resolve_core, CoreType};
use xray_tui_db::models::{Group, Profile, ProfileExtension, ServerStat};
use xray_tui_db::Database;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Profiles,
    Settings,
    Routing,
    Dns,
    Logs,
    Statistics,
}

impl Tab {
    pub const ALL: &[Tab] = &[
        Tab::Profiles,
        Tab::Settings,
        Tab::Routing,
        Tab::Dns,
        Tab::Logs,
        Tab::Statistics,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    ConfigType,
    Remarks,
    Address,
    Port,
    Delay,
    Speed,
    Traffic,
    Core,
}

pub struct ProfileRow {
    pub profile: Profile,
    pub extension: Option<ProfileExtension>,
    pub stats: Option<ServerStat>,
}

pub struct LogLine {
    pub level: String,
    pub message: String,
}

pub struct AppState {
    pub db: Database,
    pub config: AppConfig,
    pub current_tab: Tab,
    pub profiles: Vec<ProfileRow>,
    pub groups: Vec<Group>,
    pub selected_group_id: Option<String>,
    pub selected_index: usize,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub search_query: String,
    pub search_focused: bool,
    pub log_buffer: Vec<LogLine>,
    pub connected_core: Option<CoreType>,
    pub should_quit: bool,
}

impl AppState {
    pub fn new(db: Database, config: AppConfig) -> Self {
        let mut state = Self {
            db,
            config,
            current_tab: Tab::Profiles,
            profiles: Vec::new(),
            groups: Vec::new(),
            selected_group_id: None,
            selected_index: 0,
            sort_column: SortColumn::Remarks,
            sort_ascending: true,
            search_query: String::new(),
            search_focused: false,
            log_buffer: Vec::new(),
            connected_core: None,
            should_quit: false,
        };
        state.reload_profiles();
        state.reload_groups();
        state
    }

    pub fn reload_profiles(&mut self) {
        match self.db.get_all_profiles_with_details() {
            Ok(rows) => {
                self.profiles = rows
                    .into_iter()
                    .map(|(profile, extension, stats)| ProfileRow {
                        profile,
                        extension,
                        stats,
                    })
                    .collect();
            }
            Err(e) => {
                self.add_log("error", &format!("Failed to load profiles: {e}"));
                self.profiles.clear();
            }
        }
    }

    pub fn reload_groups(&mut self) {
        match self.db.get_all_groups() {
            Ok(groups) => self.groups = groups,
            Err(e) => {
                self.add_log("error", &format!("Failed to load groups: {e}"));
                self.groups.clear();
            }
        }
    }

    pub fn filtered_profiles(&self) -> Vec<&ProfileRow> {
        let mut filtered: Vec<&ProfileRow> = self.profiles.iter().filter(|row| {
            // Group filter
            if let Some(group_id) = &self.selected_group_id
                && row.profile.group_id.as_deref() != Some(group_id.as_str())
            {
                return false;
            }
            if !self.search_query.is_empty() {
                let q = self.search_query.to_lowercase();
                let remarks = row.profile.remarks.as_deref().unwrap_or("");
                let address = row.profile.address.as_deref().unwrap_or("");
                let port = row.profile.port.map(|p| p.to_string()).unwrap_or_default();
                if !remarks.to_lowercase().contains(&q)
                    && !address.to_lowercase().contains(&q)
                    && !port.contains(&q)
                {
                    return false;
                }
            }
            true
        }).collect();

        let asc = self.sort_ascending;
        filtered.sort_by(|a, b| {
            let cmp = match self.sort_column {
                SortColumn::ConfigType => a.profile.config_type.cmp(&b.profile.config_type),
                SortColumn::Remarks => {
                    a.profile.remarks.as_deref().unwrap_or("")
                        .cmp(b.profile.remarks.as_deref().unwrap_or(""))
                }
                SortColumn::Address => {
                    a.profile.address.as_deref().unwrap_or("")
                        .cmp(b.profile.address.as_deref().unwrap_or(""))
                }
                SortColumn::Port => {
                    let pa = a.profile.port.unwrap_or(0);
                    let pb = b.profile.port.unwrap_or(0);
                    pa.cmp(&pb)
                }
                SortColumn::Delay => {
                    let da = a.extension.as_ref().and_then(|e| e.delay).unwrap_or(-1);
                    let db = b.extension.as_ref().and_then(|e| e.delay).unwrap_or(-1);
                    da.cmp(&db)
                }
                SortColumn::Speed => {
                    let sa = a.extension.as_ref().and_then(|e| e.speed).unwrap_or(-1);
                    let sb = b.extension.as_ref().and_then(|e| e.speed).unwrap_or(-1);
                    sa.cmp(&sb)
                }
                SortColumn::Traffic => {
                    let ta = a.stats.as_ref().map(|s| s.total_down.unwrap_or(0) + s.total_up.unwrap_or(0)).unwrap_or(0);
                    let tb = b.stats.as_ref().map(|s| s.total_down.unwrap_or(0) + s.total_up.unwrap_or(0)).unwrap_or(0);
                    ta.cmp(&tb)
                }
                SortColumn::Core => {
                    let resolve = |row: &&ProfileRow| -> String {
                        let protocol = Protocol::try_from_i32(row.profile.config_type).unwrap_or(Protocol::Custom);
                        let core = resolve_core(protocol, Some(
                            CoreType::from_str(&row.profile.core_type).unwrap_or(CoreType::Auto),
                        ));
                        core.to_string()
                    };
                    resolve(a).cmp(&resolve(b))
                }
            };
            if asc { cmp } else { cmp.reverse() }
        });
        filtered
    }

    pub fn add_log(&mut self, level: &str, message: &str) {
        self.log_buffer.push(LogLine {
            level: level.to_owned(),
            message: message.to_owned(),
        });
        if self.log_buffer.len() > 1000 {
            self.log_buffer.remove(0);
        }
    }
}
