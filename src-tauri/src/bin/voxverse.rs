#![allow(dead_code)]
#![allow(unused_variables)]

use keyring::Entry;
use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::Duration;

const CURRENT_APP_ID: &str = "com.voxverse.app";
const LEGACY_APP_ID: &str = "com.ai-speaking.desktop";
const LEGACY_APP_ID_2: &str = "com.ai-speaking.app";
const DB_FILE_NAME: &str = "voxverse.db";
const LEGACY_DB_FILE_NAME: &str = "speakmate.db";
const AUDIT_LOG_FILE_NAME: &str = "voxverse-audit.jsonl";
const LEGACY_AUDIT_LOG_FILE_NAME: &str = "speakmate-audit.jsonl";
const CLI_CHAT_COMPLETION_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone)]
enum Command {
    Help,
    Status,
    ProfilesList,
    ProfileSwitch {
        profile_id: String,
        yes: bool,
    },
    CharacterList {
        limit: usize,
    },
    DiagnosticsRun,
    SessionList {
        limit: usize,
    },
    SessionStart {
        character_id: String,
        title: Option<String>,
        mode_id: Option<String>,
        scenario_id: Option<String>,
        yes: bool,
    },
    SessionAppendMessage {
        session_id: String,
        role: String,
        content: String,
        yes: bool,
    },
    SessionSendMessage {
        session_id: String,
        content: String,
        yes: bool,
        allow_network: bool,
    },
    SessionRetryLastMessage {
        session_id: String,
        yes: bool,
        allow_network: bool,
    },
    SessionCoachReport {
        session_id: String,
        allow_network: bool,
    },
    SessionExport {
        session_id: String,
    },
    AuditList {
        limit: usize,
        filter: AuditFilter,
    },
    AuditExport {
        filter: AuditFilter,
    },
    AgentToolsList {
        allow_writes: bool,
        allow_network: bool,
        allowed_tools: Vec<String>,
    },
    AgentServe {
        read_only: bool,
        stdio: bool,
        allow_writes: bool,
        yes: bool,
        allow_network: bool,
        allowed_tools: Vec<String>,
    },
}

#[derive(Debug, Clone)]
struct CliOptions {
    command: Command,
    json: bool,
    db_override: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
struct DbCandidate {
    source: String,
    path: String,
    exists: bool,
}

#[derive(Debug, Clone, Serialize)]
struct DbInfo {
    path: Option<String>,
    source: Option<String>,
    exists: bool,
    candidates: Vec<DbCandidate>,
}

#[derive(Debug, Clone, Serialize)]
struct ProfileInfo {
    id: String,
    name: String,
    provider: String,
    base_url: String,
    model: String,
    is_default: bool,
    requires_api_key: bool,
    supports_stt: bool,
    keychain_status: String,
}

#[derive(Debug, Clone, Serialize)]
struct StatusReport {
    version: String,
    db: DbInfo,
    active_profile: Option<ProfileInfo>,
    profile_count: usize,
    character_count: Option<i64>,
    session_count: Option<i64>,
    message_count: Option<i64>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ProfileListReport {
    db: DbInfo,
    profiles: Vec<ProfileInfo>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ProfileSwitchReport {
    db: DbInfo,
    previous_profile: Option<ProfileInfo>,
    active_profile: ProfileInfo,
    changed: bool,
    audit_log: String,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CharacterInfo {
    id: String,
    name: String,
    language: String,
    style: String,
    voice_lang: Option<String>,
    voice_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CharacterListReport {
    db: DbInfo,
    characters: Vec<CharacterInfo>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SessionInfo {
    id: String,
    character_id: String,
    character_name: Option<String>,
    title: Option<String>,
    mode_id: Option<String>,
    mode_name: Option<String>,
    scenario_id: Option<String>,
    scenario_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode_prompt_suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scenario_description: Option<String>,
    created_at: String,
    updated_at: String,
    message_count: i64,
}

#[derive(Debug, Clone, Serialize)]
struct MessageInfo {
    id: String,
    role: String,
    content: String,
    timestamp: i64,
}

#[derive(Debug, Clone, Serialize)]
struct SessionListReport {
    db: DbInfo,
    sessions: Vec<SessionInfo>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SessionStartReport {
    db: DbInfo,
    session: SessionInfo,
    audit_log: String,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct MessageAppendReport {
    db: DbInfo,
    session: Option<SessionInfo>,
    message: MessageInfo,
    audit_log: String,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SendMessageReport {
    db: DbInfo,
    profile: ProfileInfo,
    session: Option<SessionInfo>,
    user_message: MessageInfo,
    assistant_message: MessageInfo,
    audit_log: String,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SessionExportReport {
    db: DbInfo,
    session: Option<SessionInfo>,
    messages: Vec<MessageInfo>,
    summary: Option<SessionExportSummary>,
    transcript_markdown: Option<String>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SessionCoachingReport {
    db: DbInfo,
    profile: ProfileInfo,
    session: SessionInfo,
    summary: SessionExportSummary,
    transcript_markdown: String,
    coaching_markdown: String,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SessionExportSummary {
    generated_at: String,
    title: Option<String>,
    character_id: String,
    character_name: Option<String>,
    mode_id: Option<String>,
    mode_name: Option<String>,
    scenario_id: Option<String>,
    scenario_name: Option<String>,
    message_count: usize,
    user_message_count: usize,
    assistant_message_count: usize,
    other_message_count: usize,
    first_message_at: Option<i64>,
    last_message_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
struct AuditLogReport {
    db: DbInfo,
    audit_log: Option<String>,
    #[serde(skip_serializing_if = "AuditFilter::is_empty")]
    filters: AuditFilter,
    events: Vec<AuditEvent>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DiagnosticsReport {
    ok: bool,
    version: String,
    generated_at: String,
    mode: String,
    db: DbDiagnostics,
    profiles: ProfileDiagnostics,
    runtime: RuntimeDiagnostics,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DbDiagnostics {
    path: Option<String>,
    source: Option<String>,
    exists: bool,
    provider_profiles_schema: bool,
    app_config_schema: bool,
    candidates: Vec<DbCandidate>,
}

#[derive(Debug, Clone, Serialize)]
struct ProfileDiagnostics {
    count: usize,
    active_profile_id: Option<String>,
    active_profile_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RuntimeDiagnostics {
    keychain_access: String,
    ollama_base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct AgentToolsReport {
    mode: String,
    protocol: String,
    network_enabled: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    write_allowlist: Vec<String>,
    tools: Vec<AgentToolInfo>,
}

#[derive(Debug, Clone, Serialize)]
struct AgentToolInfo {
    name: String,
    description: String,
    read_only: bool,
    arguments: Vec<AgentToolArgument>,
}

#[derive(Debug, Clone, Serialize)]
struct AgentToolArgument {
    name: String,
    kind: String,
    required: bool,
    description: String,
}

#[derive(Debug, Clone, Serialize)]
struct LlmChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<LlmChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionMessage,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionMessage {
    content: String,
}

#[derive(Debug, Clone)]
struct CharacterPromptInfo {
    system_prompt: String,
}

#[derive(Debug, Deserialize)]
struct AgentRequest {
    jsonrpc: Option<String>,
    id: Option<serde_json::Value>,
    method: Option<String>,
    tool: Option<String>,
    arguments: Option<serde_json::Value>,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct AgentResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum AgentWireResponse {
    Legacy(AgentResponse),
    JsonRpc(JsonRpcResponse),
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuditEvent {
    timestamp: String,
    operation: String,
    actor: String,
    db_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_character_id: Option<String>,
    previous_profile_id: Option<String>,
    result: String,
    details: String,
}

#[derive(Debug, Clone, Default, Serialize)]
struct AuditFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    operation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_character_id: Option<String>,
}

impl AuditFilter {
    fn is_empty(&self) -> bool {
        self.operation.is_none()
            && self.actor.is_none()
            && self.result.is_none()
            && self.target_profile_id.is_none()
            && self.target_session_id.is_none()
            && self.target_message_id.is_none()
            && self.target_character_id.is_none()
    }
}

fn main() {
    match run() {
        Ok(exit_code) => process::exit(exit_code),
        Err(error) => {
            eprintln!("Error: {error}");
            process::exit(1);
        }
    }
}

fn run() -> Result<i32, String> {
    let options = parse_args(env::args().skip(1).collect())?;

    match options.command {
        Command::Help => {
            print_help();
            Ok(0)
        }
        Command::Status => {
            let report = build_status_report(&options)?;
            if options.json {
                print_json(&report)?;
            } else {
                print_status_text(&report);
            }
            Ok(0)
        }
        Command::ProfilesList => {
            let report = build_profile_list_report(&options)?;
            if options.json {
                print_json(&report)?;
            } else {
                print_profiles_text(&report);
            }
            Ok(0)
        }
        Command::ProfileSwitch {
            ref profile_id,
            yes,
        } => {
            let report = build_profile_switch_report(&options, profile_id, yes)?;
            if options.json {
                print_json(&report)?;
            } else {
                print_profile_switch_text(&report);
            }
            Ok(0)
        }
        Command::CharacterList { limit } => {
            let report = build_character_list_report(&options, limit)?;
            if options.json {
                print_json(&report)?;
            } else {
                print_characters_text(&report);
            }
            Ok(0)
        }
        Command::DiagnosticsRun => {
            let report = build_diagnostics_report(&options)?;
            if options.json {
                print_json(&report)?;
            } else {
                print_diagnostics_text(&report);
            }
            Ok(0)
        }
        Command::SessionList { limit } => {
            let report = build_session_list_report(&options, limit)?;
            if options.json {
                print_json(&report)?;
            } else {
                print_sessions_text(&report);
            }
            Ok(0)
        }
        Command::SessionStart {
            ref character_id,
            ref title,
            ref mode_id,
            ref scenario_id,
            yes,
        } => {
            let report = build_session_start_report(
                &options,
                character_id,
                title.as_deref(),
                mode_id.as_deref(),
                scenario_id.as_deref(),
                yes,
            )?;
            if options.json {
                print_json(&report)?;
            } else {
                print_session_start_text(&report);
            }
            Ok(0)
        }
        Command::SessionAppendMessage {
            ref session_id,
            ref role,
            ref content,
            yes,
        } => {
            let report = build_message_append_report(&options, session_id, role, content, yes)?;
            if options.json {
                print_json(&report)?;
            } else {
                print_message_append_text(&report);
            }
            Ok(0)
        }
        Command::SessionSendMessage {
            ref session_id,
            ref content,
            yes,
            allow_network,
        } => {
            let report =
                build_send_message_report(&options, session_id, content, yes, allow_network)?;
            if options.json {
                print_json(&report)?;
            } else {
                print_send_message_text(&report);
            }
            Ok(0)
        }
        Command::SessionRetryLastMessage {
            ref session_id,
            yes,
            allow_network,
        } => {
            let report = build_retry_last_message_report(&options, session_id, yes, allow_network)?;
            if options.json {
                print_json(&report)?;
            } else {
                print_retry_last_message_text(&report);
            }
            Ok(0)
        }
        Command::SessionCoachReport {
            ref session_id,
            allow_network,
        } => {
            let report = build_session_coaching_report(&options, session_id, allow_network)?;
            if options.json {
                print_json(&report)?;
            } else {
                print_session_coaching_report_text(&report);
            }
            Ok(0)
        }
        Command::SessionExport { ref session_id } => {
            let report = build_session_export_report(&options, session_id)?;
            if options.json {
                print_json(&report)?;
            } else {
                print_session_export_text(&report);
            }
            Ok(0)
        }
        Command::AuditList { limit, ref filter } => {
            let report = build_audit_log_report(&options, Some(limit), filter)?;
            if options.json {
                print_json(&report)?;
            } else {
                print_audit_log_text(&report);
            }
            Ok(0)
        }
        Command::AuditExport { ref filter } => {
            let report = build_audit_log_report(&options, None, filter)?;
            if options.json {
                print_json(&report)?;
            } else {
                print_audit_log_text(&report);
            }
            Ok(0)
        }
        Command::AgentToolsList {
            allow_writes,
            allow_network,
            ref allowed_tools,
        } => {
            let report = build_agent_tools_report(allow_writes, allow_network, allowed_tools);
            if options.json {
                print_json(&report)?;
            } else {
                print_agent_tools_text(&report);
            }
            Ok(0)
        }
        Command::AgentServe {
            read_only,
            stdio,
            allow_writes,
            yes,
            allow_network,
            ref allowed_tools,
        } => {
            serve_agent(
                &options,
                read_only,
                stdio,
                allow_writes,
                yes,
                allow_network,
                allowed_tools,
            )?;
            Ok(0)
        }
    }
}

fn parse_args(args: Vec<String>) -> Result<CliOptions, String> {
    if args.is_empty() {
        return Ok(CliOptions {
            command: Command::Help,
            json: false,
            db_override: None,
        });
    }

    let mut json = false;
    let mut db_override = None;
    let mut limit = 50usize;
    let mut session_id = None;
    let mut profile_id = None;
    let mut character_id = None;
    let mut title = None;
    let mut mode_id = None;
    let mut scenario_id = None;
    let mut role = None;
    let mut content = None;
    let mut audit_operation = None;
    let mut audit_actor = None;
    let mut audit_result = None;
    let mut target_message_id = None;
    let mut raw_allowed_tools = Vec::new();
    let mut read_only = false;
    let mut stdio = false;
    let mut allow_writes = false;
    let mut allow_network = false;
    let mut yes = false;
    let mut command_parts = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "-h" | "--help" => {
                return Ok(CliOptions {
                    command: Command::Help,
                    json,
                    db_override,
                });
            }
            "--json" => json = true,
            "--read-only" => read_only = true,
            "--stdio" => stdio = true,
            "--allow-writes" => allow_writes = true,
            "--allow-network" => allow_network = true,
            "--yes" => yes = true,
            "--db" => {
                index += 1;
                let Some(path) = args.get(index) else {
                    return Err("--db requires a path value.".into());
                };
                db_override = Some(PathBuf::from(path));
            }
            _ if arg.starts_with("--db=") => {
                db_override = Some(PathBuf::from(arg.trim_start_matches("--db=")));
            }
            "--limit" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("--limit requires a number.".into());
                };
                limit = parse_limit(value)?;
            }
            _ if arg.starts_with("--limit=") => {
                limit = parse_limit(arg.trim_start_matches("--limit="))?;
            }
            "--session" | "--id" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(format!("{arg} requires a session id."));
                };
                session_id = Some(value.clone());
            }
            _ if arg.starts_with("--session=") => {
                session_id = Some(arg.trim_start_matches("--session=").into());
            }
            _ if arg.starts_with("--id=") => {
                session_id = Some(arg.trim_start_matches("--id=").into());
            }
            "--profile" | "--profile-id" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(format!("{arg} requires a profile id."));
                };
                profile_id = Some(value.clone());
            }
            _ if arg.starts_with("--profile=") => {
                profile_id = Some(arg.trim_start_matches("--profile=").into());
            }
            _ if arg.starts_with("--profile-id=") => {
                profile_id = Some(arg.trim_start_matches("--profile-id=").into());
            }
            "--character" | "--character-id" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(format!("{arg} requires a character id."));
                };
                character_id = Some(value.clone());
            }
            _ if arg.starts_with("--character=") => {
                character_id = Some(arg.trim_start_matches("--character=").into());
            }
            _ if arg.starts_with("--character-id=") => {
                character_id = Some(arg.trim_start_matches("--character-id=").into());
            }
            "--title" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("--title requires a value.".into());
                };
                title = Some(value.clone());
            }
            _ if arg.starts_with("--title=") => {
                title = Some(arg.trim_start_matches("--title=").into());
            }
            "--mode" | "--mode-id" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(format!("{arg} requires a practice mode id."));
                };
                mode_id = Some(value.clone());
            }
            _ if arg.starts_with("--mode=") => {
                mode_id = Some(arg.trim_start_matches("--mode=").into());
            }
            _ if arg.starts_with("--mode-id=") => {
                mode_id = Some(arg.trim_start_matches("--mode-id=").into());
            }
            "--scenario" | "--scenario-id" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(format!("{arg} requires a scenario id."));
                };
                scenario_id = Some(value.clone());
            }
            _ if arg.starts_with("--scenario=") => {
                scenario_id = Some(arg.trim_start_matches("--scenario=").into());
            }
            _ if arg.starts_with("--scenario-id=") => {
                scenario_id = Some(arg.trim_start_matches("--scenario-id=").into());
            }
            "--role" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("--role requires a value.".into());
                };
                role = Some(value.clone());
            }
            _ if arg.starts_with("--role=") => {
                role = Some(arg.trim_start_matches("--role=").into());
            }
            "--content" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("--content requires a value.".into());
                };
                content = Some(value.clone());
            }
            _ if arg.starts_with("--content=") => {
                content = Some(arg.trim_start_matches("--content=").into());
            }
            "--operation" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("--operation requires a value.".into());
                };
                audit_operation = Some(value.clone());
            }
            _ if arg.starts_with("--operation=") => {
                audit_operation = Some(arg.trim_start_matches("--operation=").into());
            }
            "--actor" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("--actor requires a value.".into());
                };
                audit_actor = Some(value.clone());
            }
            _ if arg.starts_with("--actor=") => {
                audit_actor = Some(arg.trim_start_matches("--actor=").into());
            }
            "--result" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("--result requires a value.".into());
                };
                audit_result = Some(value.clone());
            }
            _ if arg.starts_with("--result=") => {
                audit_result = Some(arg.trim_start_matches("--result=").into());
            }
            "--message" | "--message-id" | "--target-message" | "--target-message-id" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(format!("{arg} requires a message id."));
                };
                target_message_id = Some(value.clone());
            }
            _ if arg.starts_with("--message=") => {
                target_message_id = Some(arg.trim_start_matches("--message=").into());
            }
            _ if arg.starts_with("--message-id=") => {
                target_message_id = Some(arg.trim_start_matches("--message-id=").into());
            }
            _ if arg.starts_with("--target-message=") => {
                target_message_id = Some(arg.trim_start_matches("--target-message=").into());
            }
            _ if arg.starts_with("--target-message-id=") => {
                target_message_id = Some(arg.trim_start_matches("--target-message-id=").into());
            }
            "--allow-tool" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("--allow-tool requires a tool name.".into());
                };
                raw_allowed_tools.push(value.clone());
            }
            "--allow-tools" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("--allow-tools requires one or more tool names.".into());
                };
                raw_allowed_tools.push(value.clone());
            }
            _ if arg.starts_with("--allow-tool=") => {
                raw_allowed_tools.push(arg.trim_start_matches("--allow-tool=").into());
            }
            _ if arg.starts_with("--allow-tools=") => {
                raw_allowed_tools.push(arg.trim_start_matches("--allow-tools=").into());
            }
            _ if arg.starts_with('-') => {
                return Err(format!("Unknown option: {arg}"));
            }
            _ => command_parts.push(arg.clone()),
        }
        index += 1;
    }

    let allowed_tools = normalize_allowed_write_tools(raw_allowed_tools)?;
    let audit_filter = build_audit_filter(
        audit_operation,
        audit_actor,
        audit_result,
        profile_id.clone(),
        session_id.clone(),
        target_message_id,
        character_id.clone(),
    );

    let command = match command_parts.as_slice() {
        [] => Command::Help,
        [cmd] if cmd == "status" => Command::Status,
        [first, second, third] if first == "config" && second == "profiles" && third == "list" => {
            Command::ProfilesList
        }
        [first, second, third, profile_id]
            if first == "config" && second == "profiles" && third == "switch" =>
        {
            Command::ProfileSwitch {
                profile_id: profile_id.clone(),
                yes,
            }
        }
        [first, second, third]
            if first == "config" && second == "profiles" && third == "switch" =>
        {
            let Some(profile_id) = profile_id else {
                return Err(
                    "config profiles switch requires <profile-id> or --profile <id>.".into(),
                );
            };
            Command::ProfileSwitch { profile_id, yes }
        }
        [first, second] if (first == "character" || first == "characters") && second == "list" => {
            Command::CharacterList { limit }
        }
        [first, second] if first == "diagnostics" && second == "run" => Command::DiagnosticsRun,
        [first, second] if first == "session" && second == "list" => Command::SessionList { limit },
        [first, second] if first == "session" && second == "start" => {
            let Some(character_id) = character_id else {
                return Err("session start requires --character <id>.".into());
            };
            Command::SessionStart {
                character_id,
                title,
                mode_id,
                scenario_id,
                yes,
            }
        }
        [first, second] if first == "session" && second == "append-message" => {
            let Some(session_id) = session_id else {
                return Err("session append-message requires --session <id>.".into());
            };
            let Some(role) = role else {
                return Err("session append-message requires --role <user|assistant>.".into());
            };
            let Some(content) = content else {
                return Err("session append-message requires --content <text>.".into());
            };
            Command::SessionAppendMessage {
                session_id,
                role,
                content,
                yes,
            }
        }
        [first, second] if first == "session" && second == "send-message" => {
            let Some(session_id) = session_id else {
                return Err("session send-message requires --session <id>.".into());
            };
            let Some(content) = content else {
                return Err("session send-message requires --content <text>.".into());
            };
            Command::SessionSendMessage {
                session_id,
                content,
                yes,
                allow_network,
            }
        }
        [first, second] if first == "session" && second == "retry-last" => {
            let Some(session_id) = session_id else {
                return Err("session retry-last requires --session <id>.".into());
            };
            Command::SessionRetryLastMessage {
                session_id,
                yes,
                allow_network,
            }
        }
        [first, second]
            if first == "session" && (second == "coach-report" || second == "coaching-report") =>
        {
            let Some(session_id) = session_id else {
                return Err("session coach-report requires --session <id>.".into());
            };
            Command::SessionCoachReport {
                session_id,
                allow_network,
            }
        }
        [first, second] if first == "session" && second == "export" => {
            let Some(session_id) = session_id else {
                return Err("session export requires --session <id>.".into());
            };
            Command::SessionExport { session_id }
        }
        [first, second] if first == "audit" && second == "list" => Command::AuditList {
            limit,
            filter: audit_filter,
        },
        [first, second] if first == "audit" && second == "export" => Command::AuditExport {
            filter: audit_filter,
        },
        [first, second, third] if first == "agent" && second == "tools" && third == "list" => {
            if !allow_writes && !allowed_tools.is_empty() {
                return Err("agent tools list --allow-tool requires --allow-writes.".into());
            }
            Command::AgentToolsList {
                allow_writes,
                allow_network,
                allowed_tools,
            }
        }
        [first, second] if first == "agent" && second == "serve" => Command::AgentServe {
            read_only,
            stdio,
            allow_writes,
            yes,
            allow_network,
            allowed_tools,
        },
        _ => {
            return Err(format!("Unknown command: {}", command_parts.join(" ")));
        }
    };

    Ok(CliOptions {
        command,
        json,
        db_override,
    })
}

fn parse_limit(value: &str) -> Result<usize, String> {
    let limit = value
        .parse::<usize>()
        .map_err(|_| format!("Invalid --limit value: {value}"))?;

    if limit == 0 {
        return Err("--limit must be greater than 0.".into());
    }

    Ok(limit.min(500))
}

fn build_audit_filter(
    operation: Option<String>,
    actor: Option<String>,
    result: Option<String>,
    target_profile_id: Option<String>,
    target_session_id: Option<String>,
    target_message_id: Option<String>,
    target_character_id: Option<String>,
) -> AuditFilter {
    AuditFilter {
        operation: normalize_optional_filter_value(operation),
        actor: normalize_optional_filter_value(actor),
        result: normalize_optional_filter_value(result),
        target_profile_id: normalize_optional_filter_value(target_profile_id),
        target_session_id: normalize_optional_filter_value(target_session_id),
        target_message_id: normalize_optional_filter_value(target_message_id),
        target_character_id: normalize_optional_filter_value(target_character_id),
    }
}

fn normalize_optional_filter_value(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_allowed_write_tools(raw_tools: Vec<String>) -> Result<Vec<String>, String> {
    let mut allowed_tools = Vec::new();

    for raw_tool in raw_tools {
        for item in raw_tool.split(',') {
            let trimmed = item.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.eq_ignore_ascii_case("all") {
                return Ok(Vec::new());
            }

            let Some(tool_name) = canonical_write_tool_name(trimmed) else {
                return Err(format!(
                    "Unknown --allow-tool value '{trimmed}'. Supported values: start_practice_session, append_session_message, switch_provider_profile, send_message, retry_last_message, all."
                ));
            };

            if !allowed_tools.iter().any(|tool| tool == tool_name) {
                allowed_tools.push(tool_name.into());
            }
        }
    }

    Ok(allowed_tools)
}

fn canonical_write_tool_name(tool_name: &str) -> Option<&'static str> {
    match tool_name.trim().to_ascii_lowercase().as_str() {
        "start_practice_session" | "session_start" | "start_session" => {
            Some("start_practice_session")
        }
        "append_session_message" | "session_append_message" | "append_message" => {
            Some("append_session_message")
        }
        "send_message" | "session_send_message" | "send_session_message" => Some("send_message"),
        "retry_last_message" | "retry_last" | "session_retry_last" | "retry_last_user_message" => {
            Some("retry_last_message")
        }
        "switch_provider_profile" | "profile_switch" | "switch_profile" => {
            Some("switch_provider_profile")
        }
        _ => None,
    }
}

fn print_help() {
    println!(
        "SpeakMate CLI {version}

USAGE:
  speakmate status [--json] [--db <path>]
  speakmate config profiles list [--json] [--db <path>]
  speakmate config profiles switch <profile-id> --yes [--json] [--db <path>]
  speakmate character list [--json] [--limit <n>] [--db <path>]
  speakmate session list [--json] [--limit <n>] [--db <path>]
  speakmate session start --character <id> --yes [--title <title>] [--mode <id>] [--scenario <id>] [--json] [--db <path>]
  speakmate session append-message --session <id> --role <user|assistant> --content <text> --yes [--json] [--db <path>]
  speakmate session send-message --session <id> --content <text> --yes --allow-network [--json] [--db <path>]
  speakmate session retry-last --session <id> --yes --allow-network [--json] [--db <path>]
  speakmate session coach-report --session <id> --allow-network [--json] [--db <path>]
  speakmate session export --session <id> [--json] [--db <path>]
  speakmate audit list [--json] [--limit <n>] [--operation <op>] [--actor <actor>] [--result <result>] [--session <id>] [--db <path>]
  speakmate audit export [--json] [--operation <op>] [--actor <actor>] [--result <result>] [--db <path>]
  speakmate diagnostics run [--json] [--db <path>]
  speakmate agent tools list [--json] [--allow-writes] [--allow-network]
  speakmate agent serve --stdio --read-only [--allow-network] [--db <path>]
  speakmate agent serve --stdio --allow-writes --yes [--allow-network] [--allow-tool <tool>] [--db <path>]

ENV:
  SPEAKMATE_DB       Overrides automatic database discovery.

NOTES:
  Most commands are read-only. Profile switching, session start, message append, send-message, and retry-last require --yes and write audit logs.
  Network-backed send-message, retry-last, and coach-report additionally require --allow-network.
  Coach-report is generated in memory from a local session export and does not write the database or audit log.
  The CLI never reads or prints API keys.",
        version = env!("CARGO_PKG_VERSION")
    );
}

fn build_status_report(options: &CliOptions) -> Result<StatusReport, String> {
    let db = resolve_db(options.db_override.as_deref());
    let mut warnings = Vec::new();
    let mut profiles = Vec::new();
    let mut character_count = None;
    let mut session_count = None;
    let mut message_count = None;

    if let Some(path) = selected_existing_path(&db) {
        let conn = open_readonly(path)?;
        profiles = load_profiles(&conn)?;
        character_count = count_table_rows(&conn, "characters")?;
        session_count = count_table_rows(&conn, "chat_sessions")?;
        message_count = count_table_rows(&conn, "messages")?;
    } else {
        warnings.push("No SpeakMate database was found. Start the app once or pass --db.".into());
    }

    let active_profile = profiles
        .iter()
        .find(|profile| profile.is_default)
        .cloned()
        .or_else(|| profiles.first().cloned());

    Ok(StatusReport {
        version: env!("CARGO_PKG_VERSION").into(),
        db,
        active_profile,
        profile_count: profiles.len(),
        character_count,
        session_count,
        message_count,
        warnings,
    })
}

fn build_profile_list_report(options: &CliOptions) -> Result<ProfileListReport, String> {
    let db = resolve_db(options.db_override.as_deref());
    let mut warnings = Vec::new();
    let profiles = if let Some(path) = selected_existing_path(&db) {
        let conn = open_readonly(path)?;
        load_profiles(&conn)?
    } else {
        warnings.push("No SpeakMate database was found. Start the app once or pass --db.".into());
        Vec::new()
    };

    Ok(ProfileListReport {
        db,
        profiles,
        warnings,
    })
}

fn build_profile_switch_report(
    options: &CliOptions,
    profile_id: &str,
    yes: bool,
) -> Result<ProfileSwitchReport, String> {
    build_profile_switch_report_for_actor(options, profile_id, yes, "speakmate-cli")
}

fn build_character_list_report(
    options: &CliOptions,
    limit: usize,
) -> Result<CharacterListReport, String> {
    let db = resolve_db(options.db_override.as_deref());
    let mut warnings = Vec::new();
    let characters = if let Some(path) = selected_existing_path(&db) {
        let conn = open_readonly(path)?;
        load_characters(&conn, limit, &mut warnings)?
    } else {
        warnings.push("No SpeakMate database was found. Start the app once or pass --db.".into());
        Vec::new()
    };

    Ok(CharacterListReport {
        db,
        characters,
        warnings,
    })
}

fn build_profile_switch_report_for_actor(
    options: &CliOptions,
    profile_id: &str,
    yes: bool,
    actor: &str,
) -> Result<ProfileSwitchReport, String> {
    if !yes {
        return Err("config profiles switch is a write operation and requires --yes.".into());
    }

    let db = resolve_db(options.db_override.as_deref());
    let Some(path) = selected_existing_path(&db) else {
        return Err("No SpeakMate database was found. Start the app once or pass --db.".into());
    };

    let conn = open_readwrite(path)?;
    let mut warnings = Vec::new();

    let previous_profile = load_profiles(&conn)?
        .into_iter()
        .find(|profile| profile.is_default);

    let active_profile = if table_exists(&conn, "provider_profiles")? {
        switch_provider_profile(&conn, profile_id)?
    } else if profile_id == "default" && table_exists(&conn, "app_config")? {
        warnings.push(
            "provider_profiles table is missing; legacy default profile was selected as a no-op."
                .into(),
        );
        let profile = load_legacy_profile(&conn)?;
        save_app_config_value(&conn, "active_profile_id", "default")?;
        profile
    } else {
        return Err(
            "provider_profiles table is missing; only legacy profile 'default' can be selected."
                .into(),
        );
    };

    let changed = previous_profile
        .as_ref()
        .map(|profile| profile.id.as_str() != active_profile.id.as_str())
        .unwrap_or(true);

    let audit_event = AuditEvent {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "config.profiles.switch".into(),
        actor: actor.into(),
        db_path: db.path.clone(),
        target_profile_id: Some(active_profile.id.clone()),
        target_session_id: None,
        target_message_id: None,
        target_character_id: None,
        previous_profile_id: previous_profile.as_ref().map(|profile| profile.id.clone()),
        result: "success".into(),
        details: if changed {
            "Active provider profile changed.".into()
        } else {
            "Requested profile was already active.".into()
        },
    };
    let audit_log = append_audit_event(&db, &audit_event)?;

    Ok(ProfileSwitchReport {
        db,
        previous_profile,
        active_profile,
        changed,
        audit_log,
        warnings,
    })
}

fn build_session_list_report(
    options: &CliOptions,
    limit: usize,
) -> Result<SessionListReport, String> {
    let db = resolve_db(options.db_override.as_deref());
    let mut warnings = Vec::new();
    let sessions = if let Some(path) = selected_existing_path(&db) {
        let conn = open_readonly(path)?;
        load_sessions(&conn, limit, &mut warnings)?
    } else {
        warnings.push("No SpeakMate database was found. Start the app once or pass --db.".into());
        Vec::new()
    };

    Ok(SessionListReport {
        db,
        sessions,
        warnings,
    })
}

fn build_session_start_report(
    options: &CliOptions,
    character_id: &str,
    title: Option<&str>,
    mode_id: Option<&str>,
    scenario_id: Option<&str>,
    yes: bool,
) -> Result<SessionStartReport, String> {
    build_session_start_report_for_actor(
        options,
        character_id,
        title,
        mode_id,
        scenario_id,
        yes,
        "speakmate-cli",
    )
}

fn build_session_start_report_for_actor(
    options: &CliOptions,
    character_id: &str,
    title: Option<&str>,
    mode_id: Option<&str>,
    scenario_id: Option<&str>,
    yes: bool,
    actor: &str,
) -> Result<SessionStartReport, String> {
    if !yes {
        return Err("session start is a write operation and requires --yes.".into());
    }

    let db = resolve_db(options.db_override.as_deref());
    let Some(path) = selected_existing_path(&db) else {
        return Err("No SpeakMate database was found. Start the app once or pass --db.".into());
    };

    let conn = open_readwrite(path)?;
    let mut warnings = Vec::new();
    let session = create_practice_session(
        &conn,
        character_id,
        title,
        mode_id,
        scenario_id,
        &mut warnings,
    )?;

    let audit_event = AuditEvent {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "practice.session.start".into(),
        actor: actor.into(),
        db_path: db.path.clone(),
        target_profile_id: None,
        target_session_id: Some(session.id.clone()),
        target_message_id: None,
        target_character_id: Some(session.character_id.clone()),
        previous_profile_id: None,
        result: "success".into(),
        details: "Practice session created.".into(),
    };
    let audit_log = append_audit_event(&db, &audit_event)?;

    Ok(SessionStartReport {
        db,
        session,
        audit_log,
        warnings,
    })
}

fn build_message_append_report(
    options: &CliOptions,
    session_id: &str,
    role: &str,
    content: &str,
    yes: bool,
) -> Result<MessageAppendReport, String> {
    build_message_append_report_for_actor(options, session_id, role, content, yes, "speakmate-cli")
}

fn build_message_append_report_for_actor(
    options: &CliOptions,
    session_id: &str,
    role: &str,
    content: &str,
    yes: bool,
    actor: &str,
) -> Result<MessageAppendReport, String> {
    if !yes {
        return Err("session append-message is a write operation and requires --yes.".into());
    }

    let db = resolve_db(options.db_override.as_deref());
    let Some(path) = selected_existing_path(&db) else {
        return Err("No SpeakMate database was found. Start the app once or pass --db.".into());
    };

    let conn = open_readwrite(path)?;
    let mut warnings = Vec::new();
    let message = append_session_message(&conn, session_id, role, content, &mut warnings)?;
    let session = load_session(&conn, session_id, &mut warnings)?;

    let audit_event = AuditEvent {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "practice.message.append".into(),
        actor: actor.into(),
        db_path: db.path.clone(),
        target_profile_id: None,
        target_session_id: Some(session_id.into()),
        target_message_id: Some(message.id.clone()),
        target_character_id: session.as_ref().map(|session| session.character_id.clone()),
        previous_profile_id: None,
        result: "success".into(),
        details: format!("{} message appended to practice session.", message.role),
    };
    let audit_log = append_audit_event(&db, &audit_event)?;

    Ok(MessageAppendReport {
        db,
        session,
        message,
        audit_log,
        warnings,
    })
}

fn build_send_message_report(
    options: &CliOptions,
    session_id: &str,
    content: &str,
    yes: bool,
    allow_network: bool,
) -> Result<SendMessageReport, String> {
    build_send_message_report_for_actor(
        options,
        session_id,
        content,
        yes,
        allow_network,
        "speakmate-cli",
    )
}

fn build_send_message_report_for_actor(
    options: &CliOptions,
    session_id: &str,
    content: &str,
    yes: bool,
    allow_network: bool,
    actor: &str,
) -> Result<SendMessageReport, String> {
    if !yes {
        return Err(
            "session send-message is a write and network operation and requires --yes.".into(),
        );
    }

    if !allow_network {
        return Err("session send-message requires --allow-network.".into());
    }

    let db = resolve_db(options.db_override.as_deref());
    let Some(path) = selected_existing_path(&db) else {
        return Err("No SpeakMate database was found. Start the app once or pass --db.".into());
    };

    let conn = open_readwrite(path)?;
    let mut warnings = Vec::new();
    let session = load_session(&conn, session_id, &mut warnings)?
        .ok_or_else(|| format!("Session '{session_id}' was not found."))?;
    let character = load_character_prompt(&conn, &session.character_id)?
        .ok_or_else(|| format!("Character '{}' was not found.", session.character_id))?;
    let mut profile = load_active_profile(&conn)?
        .ok_or_else(|| "No active provider profile was found.".to_string())?;
    apply_send_message_env_overrides(&mut profile, &mut warnings);
    let existing_messages = load_messages(&conn, session_id, &mut warnings)?;

    let user_message = append_session_message(&conn, session_id, "user", content, &mut warnings)?;
    let assistant_response = match call_chat_completion_for_profile(
        &profile,
        &character,
        &session,
        &existing_messages,
        &user_message,
    ) {
        Ok(response) => response,
        Err(error) => {
            let audit_event = AuditEvent {
                timestamp: chrono::Utc::now().to_rfc3339(),
                operation: "practice.message.send".into(),
                actor: actor.into(),
                db_path: db.path.clone(),
                target_profile_id: Some(profile.id.clone()),
                target_session_id: Some(session_id.into()),
                target_message_id: Some(user_message.id.clone()),
                target_character_id: Some(session.character_id.clone()),
                previous_profile_id: None,
                result: "failure".into(),
                details: format!("Model call failed after user message append: {error}"),
            };
            let _ = append_audit_event(&db, &audit_event);
            return Err(error);
        }
    };

    let assistant_message = append_session_message(
        &conn,
        session_id,
        "assistant",
        &assistant_response,
        &mut warnings,
    )?;
    let session = load_session(&conn, session_id, &mut warnings)?;

    let audit_event = AuditEvent {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "practice.message.send".into(),
        actor: actor.into(),
        db_path: db.path.clone(),
        target_profile_id: Some(profile.id.clone()),
        target_session_id: Some(session_id.into()),
        target_message_id: Some(assistant_message.id.clone()),
        target_character_id: session.as_ref().map(|session| session.character_id.clone()),
        previous_profile_id: None,
        result: "success".into(),
        details: "User message sent to model and assistant response appended.".into(),
    };
    let audit_log = append_audit_event(&db, &audit_event)?;

    Ok(SendMessageReport {
        db,
        profile,
        session,
        user_message,
        assistant_message,
        audit_log,
        warnings,
    })
}

fn build_retry_last_message_report(
    options: &CliOptions,
    session_id: &str,
    yes: bool,
    allow_network: bool,
) -> Result<SendMessageReport, String> {
    build_retry_last_message_report_for_actor(
        options,
        session_id,
        yes,
        allow_network,
        "speakmate-cli",
    )
}

fn build_retry_last_message_report_for_actor(
    options: &CliOptions,
    session_id: &str,
    yes: bool,
    allow_network: bool,
    actor: &str,
) -> Result<SendMessageReport, String> {
    if !yes {
        return Err(
            "session retry-last is a write and network operation and requires --yes.".into(),
        );
    }

    if !allow_network {
        return Err("session retry-last requires --allow-network.".into());
    }

    let db = resolve_db(options.db_override.as_deref());
    let Some(path) = selected_existing_path(&db) else {
        return Err("No SpeakMate database was found. Start the app once or pass --db.".into());
    };

    let conn = open_readwrite(path)?;
    let mut warnings = Vec::new();
    let session = load_session(&conn, session_id, &mut warnings)?
        .ok_or_else(|| format!("Session '{session_id}' was not found."))?;
    let character = load_character_prompt(&conn, &session.character_id)?
        .ok_or_else(|| format!("Character '{}' was not found.", session.character_id))?;
    let mut profile = load_active_profile(&conn)?
        .ok_or_else(|| "No active provider profile was found.".to_string())?;
    apply_send_message_env_overrides(&mut profile, &mut warnings);

    let messages = load_messages(&conn, session_id, &mut warnings)?;
    let Some(user_message) = messages.last().cloned() else {
        return Err(format!(
            "Session '{session_id}' has no messages to retry. Append or send a user message first."
        ));
    };
    if user_message.role != "user" {
        return Err(format!(
            "Session '{session_id}' cannot be retried because the last message role is '{}', not 'user'.",
            user_message.role
        ));
    }

    let existing_messages = &messages[..messages.len().saturating_sub(1)];
    let assistant_response = match call_chat_completion_for_profile(
        &profile,
        &character,
        &session,
        existing_messages,
        &user_message,
    ) {
        Ok(response) => response,
        Err(error) => {
            let audit_event = AuditEvent {
                timestamp: chrono::Utc::now().to_rfc3339(),
                operation: "practice.message.retry_last".into(),
                actor: actor.into(),
                db_path: db.path.clone(),
                target_profile_id: Some(profile.id.clone()),
                target_session_id: Some(session_id.into()),
                target_message_id: Some(user_message.id.clone()),
                target_character_id: Some(session.character_id.clone()),
                previous_profile_id: None,
                result: "failure".into(),
                details: format!("Model retry failed for last user message: {error}"),
            };
            let _ = append_audit_event(&db, &audit_event);
            return Err(error);
        }
    };

    let assistant_message = append_session_message(
        &conn,
        session_id,
        "assistant",
        &assistant_response,
        &mut warnings,
    )?;
    let session = load_session(&conn, session_id, &mut warnings)?;

    let audit_event = AuditEvent {
        timestamp: chrono::Utc::now().to_rfc3339(),
        operation: "practice.message.retry_last".into(),
        actor: actor.into(),
        db_path: db.path.clone(),
        target_profile_id: Some(profile.id.clone()),
        target_session_id: Some(session_id.into()),
        target_message_id: Some(assistant_message.id.clone()),
        target_character_id: session.as_ref().map(|session| session.character_id.clone()),
        previous_profile_id: None,
        result: "success".into(),
        details: "Last user message retried and assistant response appended.".into(),
    };
    let audit_log = append_audit_event(&db, &audit_event)?;

    Ok(SendMessageReport {
        db,
        profile,
        session,
        user_message,
        assistant_message,
        audit_log,
        warnings,
    })
}

fn build_session_export_report(
    options: &CliOptions,
    session_id: &str,
) -> Result<SessionExportReport, String> {
    let db = resolve_db(options.db_override.as_deref());
    let mut warnings = Vec::new();
    let mut session = None;
    let mut messages = Vec::new();
    let mut summary = None;
    let mut transcript_markdown = None;

    if let Some(path) = selected_existing_path(&db) {
        let conn = open_readonly(path)?;
        session = load_session(&conn, session_id, &mut warnings)?;
        if session.is_some() {
            messages = load_messages(&conn, session_id, &mut warnings)?;
            if let Some(found_session) = &session {
                summary = Some(build_session_export_summary(found_session, &messages));
                transcript_markdown =
                    Some(build_session_transcript_markdown(found_session, &messages));
            }
        } else if table_exists(&conn, "chat_sessions")? {
            warnings.push(format!("Session '{session_id}' was not found."));
        }
    } else {
        warnings.push("No SpeakMate database was found. Start the app once or pass --db.".into());
    }

    Ok(SessionExportReport {
        db,
        session,
        messages,
        summary,
        transcript_markdown,
        warnings,
    })
}

fn build_session_coaching_report(
    options: &CliOptions,
    session_id: &str,
    allow_network: bool,
) -> Result<SessionCoachingReport, String> {
    if !allow_network {
        return Err("session coach-report requires --allow-network.".into());
    }

    let db = resolve_db(options.db_override.as_deref());
    let Some(path) = selected_existing_path(&db) else {
        return Err("No SpeakMate database was found. Start the app once or pass --db.".into());
    };

    let conn = open_readonly(path)?;
    let mut warnings = Vec::new();
    let session = load_session(&conn, session_id, &mut warnings)?
        .ok_or_else(|| format!("Session '{session_id}' was not found."))?;
    let messages = load_messages(&conn, session_id, &mut warnings)?;
    let mut profile = load_active_profile(&conn)?
        .ok_or_else(|| "No active provider profile was found.".to_string())?;
    apply_send_message_env_overrides(&mut profile, &mut warnings);

    let summary = build_session_export_summary(&session, &messages);
    let transcript_markdown = build_session_transcript_markdown(&session, &messages);
    let coaching_markdown =
        call_session_coaching_report(&profile, &session, &summary, &transcript_markdown)?;

    Ok(SessionCoachingReport {
        db,
        profile,
        session,
        summary,
        transcript_markdown,
        coaching_markdown,
        warnings,
    })
}

fn build_session_export_summary(
    session: &SessionInfo,
    messages: &[MessageInfo],
) -> SessionExportSummary {
    let user_message_count = messages
        .iter()
        .filter(|message| message.role == "user")
        .count();
    let assistant_message_count = messages
        .iter()
        .filter(|message| message.role == "assistant")
        .count();
    let other_message_count = messages
        .len()
        .saturating_sub(user_message_count + assistant_message_count);

    SessionExportSummary {
        generated_at: chrono::Utc::now().to_rfc3339(),
        title: session.title.clone(),
        character_id: session.character_id.clone(),
        character_name: session.character_name.clone(),
        mode_id: session.mode_id.clone().or_else(|| Some("free_talk".into())),
        mode_name: session
            .mode_name
            .clone()
            .or_else(|| Some("Free Talk".into())),
        scenario_id: session.scenario_id.clone(),
        scenario_name: session.scenario_name.clone(),
        message_count: messages.len(),
        user_message_count,
        assistant_message_count,
        other_message_count,
        first_message_at: messages.first().map(|message| message.timestamp),
        last_message_at: messages.last().map(|message| message.timestamp),
    }
}

fn build_session_transcript_markdown(session: &SessionInfo, messages: &[MessageInfo]) -> String {
    let title = session.title.as_deref().unwrap_or("Untitled");
    let character = session
        .character_name
        .as_deref()
        .unwrap_or(session.character_id.as_str());
    let mut markdown = String::new();

    markdown.push_str("# SpeakMate Session Report\n\n");
    markdown.push_str(&format!("- Session: `{}`\n", session.id));
    markdown.push_str(&format!("- Title: {title}\n"));
    markdown.push_str(&format!(
        "- Character: {character} (`{}`)\n",
        session.character_id
    ));
    markdown.push_str(&format!(
        "- Practice Mode: {} (`{}`)\n",
        session.mode_name.as_deref().unwrap_or("Free Talk"),
        session.mode_id.as_deref().unwrap_or("free_talk")
    ));
    if let Some(scenario_id) = session.scenario_id.as_deref() {
        markdown.push_str(&format!(
            "- Scenario: {} (`{scenario_id}`)\n",
            session.scenario_name.as_deref().unwrap_or(scenario_id)
        ));
    } else {
        markdown.push_str("- Scenario: none\n");
    }
    markdown.push_str(&format!("- Created: {}\n", session.created_at));
    markdown.push_str(&format!("- Updated: {}\n", session.updated_at));
    markdown.push_str(&format!("- Messages: {}\n\n", messages.len()));

    if messages.is_empty() {
        markdown.push_str("_No messages recorded for this session yet._\n");
        return markdown;
    }

    for message in messages {
        markdown.push_str(&format!(
            "## {} ({})\n\n",
            message.role,
            format_message_timestamp(message.timestamp)
        ));
        markdown.push_str(message.content.trim());
        markdown.push_str("\n\n");
    }

    markdown
}

fn format_message_timestamp(timestamp: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp_millis(timestamp)
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| timestamp.to_string())
}

fn build_audit_log_report(
    options: &CliOptions,
    limit: Option<usize>,
    filter: &AuditFilter,
) -> Result<AuditLogReport, String> {
    let db = resolve_db(options.db_override.as_deref());
    let mut warnings = Vec::new();
    let mut events = Vec::new();

    let Some(audit_path) = resolve_audit_log_path(&db) else {
        warnings.push(
            "No SpeakMate database path was available, so no audit log could be resolved.".into(),
        );
        return Ok(AuditLogReport {
            db,
            audit_log: None,
            filters: filter.clone(),
            events,
            warnings,
        });
    };

    if !audit_path.exists() {
        warnings.push(format!(
            "Audit log '{}' does not exist yet.",
            audit_path.display()
        ));
        return Ok(AuditLogReport {
            db,
            audit_log: Some(audit_path.display().to_string()),
            filters: filter.clone(),
            events,
            warnings,
        });
    }

    events = read_audit_events(&audit_path, &mut warnings)?;
    if !filter.is_empty() {
        events.retain(|event| audit_event_matches_filter(event, filter));
    }
    if let Some(limit) = limit {
        let keep = limit.min(500);
        if events.len() > keep {
            events = events.split_off(events.len() - keep);
        }
    }

    Ok(AuditLogReport {
        db,
        audit_log: Some(audit_path.display().to_string()),
        filters: filter.clone(),
        events,
        warnings,
    })
}

fn audit_event_matches_filter(event: &AuditEvent, filter: &AuditFilter) -> bool {
    optional_case_insensitive_match(&event.operation, filter.operation.as_deref())
        && optional_case_insensitive_match(&event.actor, filter.actor.as_deref())
        && optional_case_insensitive_match(&event.result, filter.result.as_deref())
        && optional_exact_match(
            event.target_profile_id.as_deref(),
            filter.target_profile_id.as_deref(),
        )
        && optional_exact_match(
            event.target_session_id.as_deref(),
            filter.target_session_id.as_deref(),
        )
        && optional_exact_match(
            event.target_message_id.as_deref(),
            filter.target_message_id.as_deref(),
        )
        && optional_exact_match(
            event.target_character_id.as_deref(),
            filter.target_character_id.as_deref(),
        )
}

fn optional_case_insensitive_match(actual: &str, expected: Option<&str>) -> bool {
    expected
        .map(|expected| actual.eq_ignore_ascii_case(expected))
        .unwrap_or(true)
}

fn optional_exact_match(actual: Option<&str>, expected: Option<&str>) -> bool {
    expected
        .map(|expected| actual == Some(expected))
        .unwrap_or(true)
}

fn build_diagnostics_report(options: &CliOptions) -> Result<DiagnosticsReport, String> {
    let db = resolve_db(options.db_override.as_deref());
    let mut warnings = Vec::new();
    let mut profiles = Vec::new();
    let mut provider_profiles_schema = false;
    let mut app_config_schema = false;

    if let Some(path) = selected_existing_path(&db) {
        let conn = open_readonly(path)?;
        provider_profiles_schema = table_exists(&conn, "provider_profiles")?;
        app_config_schema = table_exists(&conn, "app_config")?;
        profiles = load_profiles(&conn)?;

        if !provider_profiles_schema {
            warnings.push(
                "provider_profiles table is missing; using legacy app_config fallback.".into(),
            );
        }
    } else {
        warnings.push("No SpeakMate database was found. Start the app once or pass --db.".into());
    }

    let active_profile = profiles
        .iter()
        .find(|profile| profile.is_default)
        .or_else(|| profiles.first());

    Ok(DiagnosticsReport {
        ok: db.exists && warnings.is_empty(),
        version: env!("CARGO_PKG_VERSION").into(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        mode: "read-only".into(),
        db: DbDiagnostics {
            path: db.path.clone(),
            source: db.source.clone(),
            exists: db.exists,
            provider_profiles_schema,
            app_config_schema,
            candidates: db.candidates.clone(),
        },
        profiles: ProfileDiagnostics {
            count: profiles.len(),
            active_profile_id: active_profile.map(|profile| profile.id.clone()),
            active_profile_name: active_profile.map(|profile| profile.name.clone()),
        },
        runtime: RuntimeDiagnostics {
            keychain_access: "not_read".into(),
            ollama_base_url: env::var("OLLAMA_BASE_URL")
                .ok()
                .or_else(|| env::var("LV4_OLLAMA_BASE_URL").ok()),
        },
        warnings,
    })
}

fn build_agent_tools_report(
    allow_writes: bool,
    allow_network: bool,
    allowed_tools: &[String],
) -> AgentToolsReport {
    let mut tools = vec![
        AgentToolInfo {
            name: "get_app_status".into(),
            description: "Return local database discovery, active profile, and data counts.".into(),
            read_only: true,
            arguments: Vec::new(),
        },
        AgentToolInfo {
            name: "list_provider_profiles".into(),
            description: "List provider profiles without reading keychain secrets.".into(),
            read_only: true,
            arguments: Vec::new(),
        },
        AgentToolInfo {
            name: "list_characters".into(),
            description: "List local speaking-practice characters without exporting prompts."
                .into(),
            read_only: true,
            arguments: vec![AgentToolArgument {
                name: "limit".into(),
                kind: "number".into(),
                required: false,
                description: "Maximum number of characters to return, capped at 500.".into(),
            }],
        },
        AgentToolInfo {
            name: "run_runtime_diagnostics".into(),
            description: "Return read-only CLI diagnostics and schema warnings.".into(),
            read_only: true,
            arguments: Vec::new(),
        },
        AgentToolInfo {
            name: "list_sessions".into(),
            description: "List recent chat sessions with character and message counts.".into(),
            read_only: true,
            arguments: vec![AgentToolArgument {
                name: "limit".into(),
                kind: "number".into(),
                required: false,
                description: "Maximum number of sessions to return, capped at 500.".into(),
            }],
        },
        AgentToolInfo {
            name: "export_session".into(),
            description:
                "Export one chat session with ordered messages, local summary, and Markdown transcript."
                    .into(),
            read_only: true,
            arguments: vec![AgentToolArgument {
                name: "session_id".into(),
                kind: "string".into(),
                required: true,
                description: "Session id to export.".into(),
            }],
        },
        AgentToolInfo {
            name: "list_audit_events".into(),
            description:
                "List recent CLI audit events without exposing secrets, optionally filtered by operation, actor, result, or target ids."
                    .into(),
            read_only: true,
            arguments: vec![
                AgentToolArgument {
                    name: "limit".into(),
                    kind: "number".into(),
                    required: false,
                    description: "Maximum number of audit events to return, capped at 500.".into(),
                },
                AgentToolArgument {
                    name: "operation".into(),
                    kind: "string".into(),
                    required: false,
                    description: "Optional operation filter, for example practice.message.send."
                        .into(),
                },
                AgentToolArgument {
                    name: "actor".into(),
                    kind: "string".into(),
                    required: false,
                    description: "Optional actor filter, for example speakmate-cli or speakmate-agent."
                        .into(),
                },
                AgentToolArgument {
                    name: "result".into(),
                    kind: "string".into(),
                    required: false,
                    description: "Optional result filter, for example success or failure.".into(),
                },
                AgentToolArgument {
                    name: "session_id".into(),
                    kind: "string".into(),
                    required: false,
                    description: "Optional target session id filter.".into(),
                },
            ],
        },
    ];

    if allow_network {
        tools.push(AgentToolInfo {
            name: "generate_session_coaching_report".into(),
            description:
                "Network read-only operation: export one local session and ask the active model for a Markdown coaching report without writing the database or audit log."
                    .into(),
            read_only: true,
            arguments: vec![AgentToolArgument {
                name: "session_id".into(),
                kind: "string".into(),
                required: true,
                description: "Session id to analyze with the active model.".into(),
            }],
        });
    }

    if allow_writes {
        if write_tool_enabled("start_practice_session", allowed_tools) {
            tools.push(AgentToolInfo {
            name: "start_practice_session".into(),
            description:
                "Write operation: create a practice session for a character and append an audit event."
                    .into(),
            read_only: false,
            arguments: vec![
                AgentToolArgument {
                    name: "character_id".into(),
                    kind: "string".into(),
                    required: true,
                    description: "Character id to use for the new practice session.".into(),
                },
                AgentToolArgument {
                    name: "title".into(),
                    kind: "string".into(),
                    required: false,
                    description: "Optional session title.".into(),
                },
                AgentToolArgument {
                    name: "mode_id".into(),
                    kind: "string".into(),
                    required: false,
                    description: "Optional practice mode id, defaults to free_talk.".into(),
                },
                AgentToolArgument {
                    name: "scenario_id".into(),
                    kind: "string".into(),
                    required: false,
                    description: "Optional scenario id to bind to the new session.".into(),
                },
            ],
            });
        }
        if write_tool_enabled("append_session_message", allowed_tools) {
            tools.push(AgentToolInfo {
                name: "append_session_message".into(),
            description:
                "Write operation: append a local user or assistant message to a session and append an audit event."
                    .into(),
            read_only: false,
            arguments: vec![
                AgentToolArgument {
                    name: "session_id".into(),
                    kind: "string".into(),
                    required: true,
                    description: "Session id to append the message to.".into(),
                },
                AgentToolArgument {
                    name: "role".into(),
                    kind: "string".into(),
                    required: true,
                    description: "Message role: user or assistant.".into(),
                },
                AgentToolArgument {
                    name: "content".into(),
                    kind: "string".into(),
                    required: true,
                    description: "Message content to store locally.".into(),
                },
                ],
            });
        }
        if allow_network && write_tool_enabled("send_message", allowed_tools) {
            tools.push(AgentToolInfo {
                name: "send_message".into(),
                description:
                    "Network write operation: append a user message, call the active model, append the assistant response, and audit the run."
                        .into(),
                read_only: false,
                arguments: vec![
                    AgentToolArgument {
                        name: "session_id".into(),
                        kind: "string".into(),
                        required: true,
                        description: "Session id to send the message in.".into(),
                    },
                    AgentToolArgument {
                        name: "content".into(),
                        kind: "string".into(),
                        required: true,
                        description: "User message content to send to the active model.".into(),
                    },
                ],
            });
        }
        if allow_network && write_tool_enabled("retry_last_message", allowed_tools) {
            tools.push(AgentToolInfo {
                name: "retry_last_message".into(),
                description:
                    "Network recovery operation: retry the last user message in a session, append only the assistant response, and audit the run."
                        .into(),
                read_only: false,
                arguments: vec![AgentToolArgument {
                    name: "session_id".into(),
                    kind: "string".into(),
                    required: true,
                    description: "Session id whose last user message should be retried.".into(),
                }],
            });
        }
        if write_tool_enabled("switch_provider_profile", allowed_tools) {
            tools.push(AgentToolInfo {
                name: "switch_provider_profile".into(),
                description:
                    "Write operation: switch the active provider profile and append an audit event."
                        .into(),
                read_only: false,
                arguments: vec![AgentToolArgument {
                    name: "profile_id".into(),
                    kind: "string".into(),
                    required: true,
                    description: "Provider profile id to activate.".into(),
                }],
            });
        }
    }

    AgentToolsReport {
        mode: if allow_writes {
            "allow-writes".into()
        } else {
            "read-only".into()
        },
        protocol: "speakmate-jsonl-rpc-v1+mcp-jsonrpc-2.0".into(),
        network_enabled: allow_network,
        write_allowlist: if allow_writes {
            allowed_tools.to_vec()
        } else {
            Vec::new()
        },
        tools,
    }
}

fn write_tool_enabled(tool_name: &str, allowed_tools: &[String]) -> bool {
    allowed_tools.is_empty() || allowed_tools.iter().any(|allowed| allowed == tool_name)
}

fn ensure_write_tool_enabled(
    tool_name: &str,
    allow_writes: bool,
    allowed_tools: &[String],
) -> Result<(), String> {
    if !allow_writes {
        return Err(format!(
            "{tool_name} requires agent serve --allow-writes --yes."
        ));
    }

    if !write_tool_enabled(tool_name, allowed_tools) {
        return Err(format!(
            "{tool_name} is not enabled by this agent server allowlist."
        ));
    }

    Ok(())
}

fn ensure_network_enabled(tool_name: &str, allow_network: bool) -> Result<(), String> {
    if allow_network {
        Ok(())
    } else {
        Err(format!("{tool_name} requires agent serve --allow-network."))
    }
}

fn serve_agent(
    options: &CliOptions,
    read_only: bool,
    stdio: bool,
    allow_writes: bool,
    yes: bool,
    allow_network: bool,
    allowed_tools: &[String],
) -> Result<(), String> {
    if !stdio {
        return Err("agent serve currently requires --stdio.".into());
    }

    if read_only && allow_writes {
        return Err("agent serve cannot combine --read-only and --allow-writes.".into());
    }

    if !allow_writes && !allowed_tools.is_empty() {
        return Err("agent serve --allow-tool requires --allow-writes.".into());
    }

    if allow_network && !allow_writes && !read_only {
        return Err(
            "agent serve --allow-network requires --read-only or --allow-writes --yes.".into(),
        );
    }

    if allow_writes {
        if !yes {
            return Err("agent serve --allow-writes requires --yes.".into());
        }
    } else if !read_only {
        return Err(
            "agent serve currently requires either --read-only or --allow-writes --yes.".into(),
        );
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line.map_err(|error| format!("Failed to read agent request: {error}"))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<AgentRequest>(trimmed) {
            Ok(request) => {
                handle_agent_request(options, request, allow_writes, allow_network, allowed_tools)
            }
            Err(error) => Some(AgentWireResponse::JsonRpc(JsonRpcResponse {
                jsonrpc: "2.0",
                id: None,
                result: None,
                error: Some(JsonRpcError {
                    code: -32700,
                    message: format!("Invalid JSON request: {error}"),
                }),
            })),
        };

        let Some(response) = response else {
            continue;
        };

        let should_shutdown = agent_response_requests_shutdown(&response);

        let output = serde_json::to_string(&response)
            .map_err(|error| format!("Failed to serialize agent response: {error}"))?;
        writeln!(stdout, "{output}")
            .map_err(|error| format!("Failed to write agent response: {error}"))?;
        stdout
            .flush()
            .map_err(|error| format!("Failed to flush agent response: {error}"))?;

        if should_shutdown {
            break;
        }
    }

    Ok(())
}

fn handle_agent_request(
    options: &CliOptions,
    request: AgentRequest,
    allow_writes: bool,
    allow_network: bool,
    allowed_tools: &[String],
) -> Option<AgentWireResponse> {
    if request.jsonrpc.as_deref() == Some("2.0") {
        return handle_mcp_request(options, request, allow_writes, allow_network, allowed_tools);
    }

    let id = request.id.clone();
    let method = request.method.as_deref().unwrap_or_default();

    let result = match method {
        "initialize" => json_result(serde_json::json!({
            "server": "speakmate",
            "version": env!("CARGO_PKG_VERSION"),
            "protocol": "speakmate-jsonl-rpc-v1",
            "mode": if allow_writes { "allow-writes" } else { "read-only" },
            "network_enabled": allow_network,
            "write_allowlist": allowed_tools
        })),
        "tools/list" | "list_tools" => json_result(build_agent_tools_report(
            allow_writes,
            allow_network,
            allowed_tools,
        )),
        "tools/call" => {
            let params = request.params.as_ref();
            let tool_name = params
                .and_then(|value| value.get("name"))
                .and_then(|value| value.as_str());
            let arguments = params
                .and_then(|value| value.get("arguments"))
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));

            match tool_name {
                Some(tool_name) => call_agent_tool(
                    options,
                    tool_name,
                    Some(arguments),
                    allow_writes,
                    allow_network,
                    allowed_tools,
                ),
                None => Err("tools/call requires params.name.".into()),
            }
        }
        "shutdown" => json_result(serde_json::json!({ "shutdown": true })),
        _ => {
            if let Some(tool_name) = request.tool.as_deref().or(request.method.as_deref()) {
                if tool_name == "shutdown" {
                    json_result(serde_json::json!({ "shutdown": true }))
                } else {
                    call_agent_tool(
                        options,
                        tool_name,
                        request.arguments,
                        allow_writes,
                        allow_network,
                        allowed_tools,
                    )
                }
            } else {
                Err("Agent request requires method or tool.".into())
            }
        }
    };

    Some(AgentWireResponse::Legacy(match result {
        Ok(result) => AgentResponse {
            id,
            ok: true,
            result: Some(result),
            error: None,
        },
        Err(error) => AgentResponse {
            id,
            ok: false,
            result: None,
            error: Some(error),
        },
    }))
}

fn handle_mcp_request(
    options: &CliOptions,
    request: AgentRequest,
    allow_writes: bool,
    allow_network: bool,
    allowed_tools: &[String],
) -> Option<AgentWireResponse> {
    let id = request.id.clone();
    let method = request.method.as_deref().unwrap_or_default();

    if method == "notifications/initialized" {
        return None;
    }

    let result = match method {
        "initialize" => json_result(serde_json::json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "speakmate",
                "version": env!("CARGO_PKG_VERSION")
            },
            "instructions": if allow_writes {
                "SpeakMate exposes local speaking-practice diagnostics, profiles, sessions, audit events, and a guarded provider-profile switch tool. Tools never return API keys. Write mode is enabled only because the server was started with --allow-writes --yes."
            } else {
                "SpeakMate exposes local speaking-practice diagnostics, profiles, sessions, and audit events. Tools are read-only in MCP mode and never return API keys."
            },
            "networkEnabled": allow_network,
            "writeAllowlist": allowed_tools
        })),
        "tools/list" => json_result(serde_json::json!({
            "tools": build_mcp_tools(allow_writes, allow_network, allowed_tools)
        })),
        "tools/call" => {
            let params = request.params.as_ref();
            let tool_name = params
                .and_then(|value| value.get("name"))
                .and_then(|value| value.as_str())
                .ok_or_else(|| "tools/call requires params.name.".to_string());
            let arguments = params
                .and_then(|value| value.get("arguments"))
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));

            match tool_name {
                Ok(tool_name) => call_agent_tool(
                    options,
                    tool_name,
                    Some(arguments),
                    allow_writes,
                    allow_network,
                    allowed_tools,
                )
                .and_then(mcp_tool_result),
                Err(error) => Err(error),
            }
        }
        "ping" => json_result(serde_json::json!({})),
        "shutdown" => json_result(serde_json::json!({ "shutdown": true })),
        _ => Err(format!("Unknown MCP method: {method}")),
    };

    Some(AgentWireResponse::JsonRpc(match result {
        Ok(result) => JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        },
        Err(error) => JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code: jsonrpc_error_code(&error),
                message: error,
            }),
        },
    }))
}

fn agent_response_requests_shutdown(response: &AgentWireResponse) -> bool {
    match response {
        AgentWireResponse::Legacy(response) => response
            .result
            .as_ref()
            .and_then(|result| result.get("shutdown"))
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
        AgentWireResponse::JsonRpc(response) => response
            .result
            .as_ref()
            .and_then(|result| result.get("shutdown"))
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
    }
}

fn jsonrpc_error_code(message: &str) -> i64 {
    if message.contains("requires")
        || message.contains("Invalid")
        || message.contains("not enabled")
    {
        -32602
    } else if message.contains("Unknown MCP method")
        || message.contains("Unknown read-only agent tool")
    {
        -32601
    } else {
        -32603
    }
}

fn build_mcp_tools(
    allow_writes: bool,
    allow_network: bool,
    allowed_tools: &[String],
) -> Vec<serde_json::Value> {
    build_agent_tools_report(allow_writes, allow_network, allowed_tools)
        .tools
        .into_iter()
        .map(|tool| {
            serde_json::json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": build_mcp_input_schema(&tool.arguments)
            })
        })
        .collect()
}

fn build_mcp_input_schema(arguments: &[AgentToolArgument]) -> serde_json::Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for argument in arguments {
        properties.insert(
            argument.name.clone(),
            serde_json::json!({
                "type": match argument.kind.as_str() {
                    "number" => "number",
                    "boolean" => "boolean",
                    _ => "string",
                },
                "description": argument.description
            }),
        );

        if argument.required {
            required.push(argument.name.clone());
        }
    }

    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

fn mcp_tool_result(value: serde_json::Value) -> Result<serde_json::Value, String> {
    let text = serde_json::to_string_pretty(&value)
        .map_err(|error| format!("Failed to serialize MCP tool result: {error}"))?;

    Ok(serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ],
        "structuredContent": value,
        "isError": false
    }))
}

fn call_agent_tool(
    options: &CliOptions,
    tool_name: &str,
    arguments: Option<serde_json::Value>,
    allow_writes: bool,
    allow_network: bool,
    allowed_tools: &[String],
) -> Result<serde_json::Value, String> {
    match tool_name {
        "get_app_status" | "status" => json_result(build_status_report(options)?),
        "list_provider_profiles" | "list_config_profiles" => {
            json_result(build_profile_list_report(options)?)
        }
        "list_characters" | "characters_list" => {
            let limit = arguments
                .as_ref()
                .and_then(|value| value.get("limit"))
                .and_then(|value| value.as_u64())
                .map(|value| value as usize)
                .unwrap_or(50)
                .clamp(1, 500);
            json_result(build_character_list_report(options, limit)?)
        }
        "run_runtime_diagnostics" | "diagnostics" => {
            json_result(build_diagnostics_report(options)?)
        }
        "list_sessions" => {
            let limit = arguments
                .as_ref()
                .and_then(|value| value.get("limit"))
                .and_then(|value| value.as_u64())
                .map(|value| value as usize)
                .unwrap_or(50)
                .clamp(1, 500);
            json_result(build_session_list_report(options, limit)?)
        }
        "export_session" => {
            let session_id = arguments
                .as_ref()
                .and_then(|value| value.get("session_id").or_else(|| value.get("session")))
                .and_then(|value| value.as_str())
                .ok_or_else(|| "export_session requires arguments.session_id.".to_string())?;
            json_result(build_session_export_report(options, session_id)?)
        }
        "generate_session_coaching_report" | "session_coach_report" | "coach_report" => {
            ensure_network_enabled("generate_session_coaching_report", allow_network)?;

            let session_id = arguments
                .as_ref()
                .and_then(|value| value.get("session_id").or_else(|| value.get("session")))
                .and_then(|value| value.as_str())
                .ok_or_else(|| {
                    "generate_session_coaching_report requires arguments.session_id.".to_string()
                })?;
            json_result(build_session_coaching_report(options, session_id, true)?)
        }
        "list_audit_events" | "audit_list" => {
            let limit = arguments
                .as_ref()
                .and_then(|value| value.get("limit"))
                .and_then(|value| value.as_u64())
                .map(|value| value as usize)
                .unwrap_or(50)
                .clamp(1, 500);
            let filter = audit_filter_from_agent_arguments(arguments.as_ref());
            json_result(build_audit_log_report(options, Some(limit), &filter)?)
        }
        "start_practice_session" => {
            ensure_write_tool_enabled("start_practice_session", allow_writes, allowed_tools)?;

            let character_id = arguments
                .as_ref()
                .and_then(|value| value.get("character_id").or_else(|| value.get("character")))
                .and_then(|value| value.as_str())
                .ok_or_else(|| {
                    "start_practice_session requires arguments.character_id.".to_string()
                })?;
            let title = arguments
                .as_ref()
                .and_then(|value| value.get("title"))
                .and_then(|value| value.as_str());
            let mode_id = arguments
                .as_ref()
                .and_then(|value| value.get("mode_id").or_else(|| value.get("mode")))
                .and_then(|value| value.as_str());
            let scenario_id = arguments
                .as_ref()
                .and_then(|value| value.get("scenario_id").or_else(|| value.get("scenario")))
                .and_then(|value| value.as_str());
            json_result(build_session_start_report_for_actor(
                options,
                character_id,
                title,
                mode_id,
                scenario_id,
                true,
                "speakmate-agent",
            )?)
        }
        "append_session_message" => {
            ensure_write_tool_enabled("append_session_message", allow_writes, allowed_tools)?;

            let session_id = arguments
                .as_ref()
                .and_then(|value| value.get("session_id").or_else(|| value.get("session")))
                .and_then(|value| value.as_str())
                .ok_or_else(|| {
                    "append_session_message requires arguments.session_id.".to_string()
                })?;
            let role = arguments
                .as_ref()
                .and_then(|value| value.get("role"))
                .and_then(|value| value.as_str())
                .ok_or_else(|| "append_session_message requires arguments.role.".to_string())?;
            let content = arguments
                .as_ref()
                .and_then(|value| value.get("content"))
                .and_then(|value| value.as_str())
                .ok_or_else(|| "append_session_message requires arguments.content.".to_string())?;
            json_result(build_message_append_report_for_actor(
                options,
                session_id,
                role,
                content,
                true,
                "speakmate-agent",
            )?)
        }
        "send_message" => {
            ensure_write_tool_enabled("send_message", allow_writes, allowed_tools)?;
            ensure_network_enabled("send_message", allow_network)?;

            let session_id = arguments
                .as_ref()
                .and_then(|value| value.get("session_id").or_else(|| value.get("session")))
                .and_then(|value| value.as_str())
                .ok_or_else(|| "send_message requires arguments.session_id.".to_string())?;
            let content = arguments
                .as_ref()
                .and_then(|value| value.get("content"))
                .and_then(|value| value.as_str())
                .ok_or_else(|| "send_message requires arguments.content.".to_string())?;
            json_result(build_send_message_report_for_actor(
                options,
                session_id,
                content,
                true,
                true,
                "speakmate-agent",
            )?)
        }
        "retry_last_message" | "retry_last" => {
            ensure_write_tool_enabled("retry_last_message", allow_writes, allowed_tools)?;
            ensure_network_enabled("retry_last_message", allow_network)?;

            let session_id = arguments
                .as_ref()
                .and_then(|value| value.get("session_id").or_else(|| value.get("session")))
                .and_then(|value| value.as_str())
                .ok_or_else(|| "retry_last_message requires arguments.session_id.".to_string())?;
            json_result(build_retry_last_message_report_for_actor(
                options,
                session_id,
                true,
                true,
                "speakmate-agent",
            )?)
        }
        "switch_provider_profile" => {
            ensure_write_tool_enabled("switch_provider_profile", allow_writes, allowed_tools)?;

            let profile_id = arguments
                .as_ref()
                .and_then(|value| value.get("profile_id").or_else(|| value.get("profile")))
                .and_then(|value| value.as_str())
                .ok_or_else(|| {
                    "switch_provider_profile requires arguments.profile_id.".to_string()
                })?;
            json_result(build_profile_switch_report_for_actor(
                options,
                profile_id,
                true,
                "speakmate-agent",
            )?)
        }
        _ => Err(format!("Unknown read-only agent tool: {tool_name}")),
    }
}

fn audit_filter_from_agent_arguments(arguments: Option<&serde_json::Value>) -> AuditFilter {
    build_audit_filter(
        json_string_argument(arguments, "operation"),
        json_string_argument(arguments, "actor"),
        json_string_argument(arguments, "result"),
        json_string_argument(arguments, "profile_id")
            .or_else(|| json_string_argument(arguments, "profile")),
        json_string_argument(arguments, "session_id")
            .or_else(|| json_string_argument(arguments, "session")),
        json_string_argument(arguments, "message_id")
            .or_else(|| json_string_argument(arguments, "message")),
        json_string_argument(arguments, "character_id")
            .or_else(|| json_string_argument(arguments, "character")),
    )
}

fn json_string_argument(arguments: Option<&serde_json::Value>, name: &str) -> Option<String> {
    arguments
        .and_then(|value| value.get(name))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
}

fn json_result<T: Serialize>(value: T) -> Result<serde_json::Value, String> {
    serde_json::to_value(value).map_err(|error| format!("Failed to serialize result: {error}"))
}

fn resolve_db(db_override: Option<&Path>) -> DbInfo {
    let mut candidates = Vec::new();

    if let Some(path) = db_override {
        candidates.push(make_candidate("--db", path));
    } else if let Some(path) = env::var_os("SPEAKMATE_DB").map(PathBuf::from) {
        candidates.push(make_candidate("SPEAKMATE_DB", &path));
    }

    if let Some(local_app_data) = env::var_os("LOCALAPPDATA").map(PathBuf::from) {
        candidates.push(make_candidate(
            CURRENT_APP_ID,
            &local_app_data.join(CURRENT_APP_ID).join(DB_FILE_NAME),
        ));
        candidates.push(make_candidate(
            LEGACY_APP_ID,
            &local_app_data.join(LEGACY_APP_ID).join(DB_FILE_NAME),
        ));
        candidates.push(make_candidate(
            "SpeakMate",
            &local_app_data.join("SpeakMate").join(DB_FILE_NAME),
        ));
    }

    let selected = candidates
        .iter()
        .find(|candidate| candidate.exists)
        .or_else(|| candidates.first());

    DbInfo {
        path: selected.map(|candidate| candidate.path.clone()),
        source: selected.map(|candidate| candidate.source.clone()),
        exists: selected.map(|candidate| candidate.exists).unwrap_or(false),
        candidates,
    }
}

fn make_candidate(source: &str, path: &Path) -> DbCandidate {
    DbCandidate {
        source: source.into(),
        path: path.display().to_string(),
        exists: path.exists(),
    }
}

fn selected_existing_path(db: &DbInfo) -> Option<&Path> {
    if !db.exists {
        return None;
    }

    db.path.as_deref().map(Path::new)
}

fn open_readonly(path: &Path) -> Result<Connection, String> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| format!("Failed to open database '{}': {error}", path.display()))
}

fn open_readwrite(path: &Path) -> Result<Connection, String> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE).map_err(|error| {
        format!(
            "Failed to open database '{}' for writing: {error}",
            path.display()
        )
    })
}

fn load_profiles(conn: &Connection) -> Result<Vec<ProfileInfo>, String> {
    if table_exists(conn, "provider_profiles")? {
        load_provider_profiles(conn)
    } else if table_exists(conn, "app_config")? {
        Ok(vec![load_legacy_profile(conn)?])
    } else {
        Ok(Vec::new())
    }
}

fn load_active_profile(conn: &Connection) -> Result<Option<ProfileInfo>, String> {
    let profiles = load_profiles(conn)?;
    Ok(profiles
        .iter()
        .find(|profile| profile.is_default)
        .cloned()
        .or_else(|| profiles.first().cloned()))
}

fn load_provider_profiles(conn: &Connection) -> Result<Vec<ProfileInfo>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, provider, base_url, model, is_default
             FROM provider_profiles
             ORDER BY is_default DESC, updated_at DESC, name ASC",
        )
        .map_err(|error| format!("Failed to prepare profile query: {error}"))?;

    let rows = stmt
        .query_map([], |row| {
            let provider_raw: String = row.get(2)?;
            let base_url: String = row.get(3)?;
            let provider = normalize_provider(&provider_raw, &base_url);
            let is_default: i64 = row.get(5)?;

            Ok(ProfileInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                provider: provider.clone(),
                base_url,
                model: row.get(4)?,
                is_default: is_default == 1,
                requires_api_key: requires_api_key(&provider),
                supports_stt: supports_stt(&provider),
                keychain_status: "not_read".into(),
            })
        })
        .map_err(|error| format!("Failed to read profiles: {error}"))?;

    let mut profiles = Vec::new();
    for row in rows {
        profiles.push(row.map_err(|error| format!("Failed to decode profile row: {error}"))?);
    }
    Ok(profiles)
}

fn load_provider_profile(
    conn: &Connection,
    profile_id: &str,
) -> Result<Option<ProfileInfo>, String> {
    let result = conn.query_row(
        "SELECT id, name, provider, base_url, model, is_default
         FROM provider_profiles
         WHERE id = ?1",
        params![profile_id],
        |row| {
            let provider_raw: String = row.get(2)?;
            let base_url: String = row.get(3)?;
            let provider = normalize_provider(&provider_raw, &base_url);
            let is_default: i64 = row.get(5)?;

            Ok(ProfileInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                provider: provider.clone(),
                base_url,
                model: row.get(4)?,
                is_default: is_default == 1,
                requires_api_key: requires_api_key(&provider),
                supports_stt: supports_stt(&provider),
                keychain_status: "not_read".into(),
            })
        },
    );

    match result {
        Ok(profile) => Ok(Some(profile)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(format!("Failed to read profile '{profile_id}': {error}")),
    }
}

fn switch_provider_profile(conn: &Connection, profile_id: &str) -> Result<ProfileInfo, String> {
    let target = load_provider_profile(conn, profile_id)?
        .ok_or_else(|| format!("Config profile '{profile_id}' was not found."))?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute_batch("BEGIN IMMEDIATE")
        .map_err(|error| format!("Failed to start profile switch transaction: {error}"))?;

    let update_result = (|| -> Result<(), String> {
        conn.execute("UPDATE provider_profiles SET is_default = 0", [])
            .map_err(|error| format!("Failed to clear active profile: {error}"))?;
        conn.execute(
            "UPDATE provider_profiles SET is_default = 1, updated_at = ?2 WHERE id = ?1",
            params![profile_id, now],
        )
        .map_err(|error| format!("Failed to activate profile '{profile_id}': {error}"))?;

        save_app_config_value(conn, "active_profile_id", &target.id)?;
        save_app_config_value(conn, "provider", &target.provider)?;
        save_app_config_value(conn, "base_url", &target.base_url)?;
        save_app_config_value(conn, "model", &target.model)?;

        Ok(())
    })();

    if update_result.is_ok() {
        conn.execute_batch("COMMIT")
            .map_err(|error| format!("Failed to commit profile switch: {error}"))?;
    } else {
        let _ = conn.execute_batch("ROLLBACK");
        update_result?;
    }

    load_provider_profile(conn, profile_id)?
        .ok_or_else(|| format!("Config profile '{profile_id}' was not found after switch."))
}

fn load_legacy_profile(conn: &Connection) -> Result<ProfileInfo, String> {
    let provider_raw = read_app_config(conn, "provider")?.unwrap_or_else(|| "openai".into());
    let base_url =
        read_app_config(conn, "base_url")?.unwrap_or_else(|| "https://api.openai.com/v1".into());
    let provider = normalize_provider(&provider_raw, &base_url);
    let model = read_app_config(conn, "model")?.unwrap_or_else(|| "gpt-4o-mini".into());

    Ok(ProfileInfo {
        id: "default".into(),
        name: "Default".into(),
        provider: provider.clone(),
        base_url,
        model,
        is_default: true,
        requires_api_key: requires_api_key(&provider),
        supports_stt: supports_stt(&provider),
        keychain_status: "not_read".into(),
    })
}

fn load_characters(
    conn: &Connection,
    limit: usize,
    warnings: &mut Vec<String>,
) -> Result<Vec<CharacterInfo>, String> {
    if !table_exists(conn, "characters")? {
        warnings.push("characters table is missing.".into());
        return Ok(Vec::new());
    }

    let mut stmt = conn
        .prepare(
            "SELECT id, name, language, style, voice_lang, voice_name
             FROM characters
             ORDER BY name ASC
             LIMIT ?1",
        )
        .map_err(|error| format!("Failed to prepare character query: {error}"))?;

    let rows = stmt
        .query_map([limit as i64], |row| {
            Ok(CharacterInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                language: row.get(2)?,
                style: row.get(3)?,
                voice_lang: row.get(4)?,
                voice_name: row.get(5)?,
            })
        })
        .map_err(|error| format!("Failed to read characters: {error}"))?;

    let mut characters = Vec::new();
    for row in rows {
        characters.push(row.map_err(|error| format!("Failed to decode character row: {error}"))?);
    }
    Ok(characters)
}

fn load_character(conn: &Connection, character_id: &str) -> Result<Option<CharacterInfo>, String> {
    if !table_exists(conn, "characters")? {
        return Ok(None);
    }

    let result = conn.query_row(
        "SELECT id, name, language, style, voice_lang, voice_name
         FROM characters
         WHERE id = ?1",
        [character_id],
        |row| {
            Ok(CharacterInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                language: row.get(2)?,
                style: row.get(3)?,
                voice_lang: row.get(4)?,
                voice_name: row.get(5)?,
            })
        },
    );

    match result {
        Ok(character) => Ok(Some(character)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(format!(
            "Failed to read character '{character_id}': {error}"
        )),
    }
}

fn load_character_prompt(
    conn: &Connection,
    character_id: &str,
) -> Result<Option<CharacterPromptInfo>, String> {
    if !table_exists(conn, "characters")? {
        return Ok(None);
    }

    let result = conn.query_row(
        "SELECT system_prompt
         FROM characters
         WHERE id = ?1",
        [character_id],
        |row| {
            Ok(CharacterPromptInfo {
                system_prompt: row.get(0)?,
            })
        },
    );

    match result {
        Ok(character) => Ok(Some(character)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(format!(
            "Failed to read character prompt '{character_id}': {error}"
        )),
    }
}

fn create_practice_session(
    conn: &Connection,
    character_id: &str,
    title: Option<&str>,
    mode_id: Option<&str>,
    scenario_id: Option<&str>,
    warnings: &mut Vec<String>,
) -> Result<SessionInfo, String> {
    if !table_exists(conn, "chat_sessions")? {
        return Err("chat_sessions table is missing; cannot start a practice session.".into());
    }
    ensure_v02_schema(conn, warnings)?;

    if !table_exists(conn, "characters")? {
        return Err("characters table is missing; cannot validate the requested character.".into());
    }

    let character = load_character(conn, character_id)?
        .ok_or_else(|| format!("Character '{character_id}' was not found."))?;
    let now = chrono::Utc::now();
    let session_id = format!("sess-{}", now.timestamp_millis());
    let timestamp = now.to_rfc3339();
    let normalized_title = title
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let normalized_mode_id = mode_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("free_talk");
    let normalized_scenario_id = scenario_id.map(str::trim).filter(|value| !value.is_empty());

    validate_practice_mode(conn, normalized_mode_id)?;
    if let Some(scenario_id) = normalized_scenario_id {
        validate_scenario(conn, scenario_id)?;
    }

    conn.execute(
        "INSERT INTO chat_sessions (id, character_id, title, mode_id, scenario_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            &session_id,
            &character.id,
            normalized_title.as_deref(),
            normalized_mode_id,
            normalized_scenario_id,
            &timestamp,
            &timestamp
        ],
    )
    .map_err(|error| format!("Failed to create practice session: {error}"))?;

    load_session(conn, &session_id, warnings)?
        .ok_or_else(|| format!("Session '{session_id}' was not found after creation."))
}

fn validate_practice_mode(conn: &Connection, mode_id: &str) -> Result<(), String> {
    if !table_exists(conn, "practice_modes")? {
        return Err("practice_modes table is missing; cannot validate practice mode.".into());
    }

    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM practice_modes WHERE id = ?1)",
            [mode_id],
            |row| row.get(0),
        )
        .map_err(|error| format!("Failed to validate practice mode '{mode_id}': {error}"))?;

    if exists {
        Ok(())
    } else {
        Err(format!("Practice mode '{mode_id}' was not found."))
    }
}

fn validate_scenario(conn: &Connection, scenario_id: &str) -> Result<(), String> {
    if !table_exists(conn, "scenarios")? {
        return Err("scenarios table is missing; cannot validate scenario.".into());
    }

    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM scenarios WHERE id = ?1)",
            [scenario_id],
            |row| row.get(0),
        )
        .map_err(|error| format!("Failed to validate scenario '{scenario_id}': {error}"))?;

    if exists {
        Ok(())
    } else {
        Err(format!("Scenario '{scenario_id}' was not found."))
    }
}

fn append_session_message(
    conn: &Connection,
    session_id: &str,
    role: &str,
    content: &str,
    warnings: &mut Vec<String>,
) -> Result<MessageInfo, String> {
    if !table_exists(conn, "chat_sessions")? {
        return Err("chat_sessions table is missing; cannot append a message.".into());
    }

    if !table_exists(conn, "messages")? {
        return Err("messages table is missing; cannot append a message.".into());
    }

    let Some(_) = load_session(conn, session_id, warnings)? else {
        return Err(format!("Session '{session_id}' was not found."));
    };

    let normalized_role = normalize_message_role(role)?;
    let normalized_content = content.trim();
    if normalized_content.is_empty() {
        return Err("Message content cannot be empty.".into());
    }

    let now = chrono::Utc::now();
    let message_id = format!("msg-{}", now.timestamp_millis());
    let timestamp = now.timestamp_millis();
    let updated_at = now.to_rfc3339();

    conn.execute_batch("BEGIN IMMEDIATE")
        .map_err(|error| format!("Failed to start message append transaction: {error}"))?;

    let append_result = (|| -> Result<(), String> {
        conn.execute(
            "INSERT INTO messages (id, session_id, role, content, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &message_id,
                session_id,
                &normalized_role,
                normalized_content,
                timestamp
            ],
        )
        .map_err(|error| format!("Failed to append message: {error}"))?;

        conn.execute(
            "UPDATE chat_sessions SET updated_at = ?2 WHERE id = ?1",
            params![session_id, &updated_at],
        )
        .map_err(|error| format!("Failed to update session timestamp: {error}"))?;

        Ok(())
    })();

    if append_result.is_ok() {
        conn.execute_batch("COMMIT")
            .map_err(|error| format!("Failed to commit message append: {error}"))?;
    } else {
        let _ = conn.execute_batch("ROLLBACK");
        append_result?;
    }

    Ok(MessageInfo {
        id: message_id,
        role: normalized_role,
        content: normalized_content.into(),
        timestamp,
    })
}

fn normalize_message_role(role: &str) -> Result<String, String> {
    match role.trim().to_ascii_lowercase().as_str() {
        "user" => Ok("user".into()),
        "assistant" => Ok("assistant".into()),
        other => Err(format!(
            "Unsupported message role '{other}'. Use 'user' or 'assistant'."
        )),
    }
}

fn call_chat_completion_for_profile(
    profile: &ProfileInfo,
    character: &CharacterPromptInfo,
    session: &SessionInfo,
    existing_messages: &[MessageInfo],
    user_message: &MessageInfo,
) -> Result<String, String> {
    let mut messages = Vec::new();

    messages.push(LlmChatMessage {
        role: "system".into(),
        content: build_cli_system_prompt(character, session),
    });

    for message in existing_messages {
        if message.role == "user" || message.role == "assistant" {
            messages.push(LlmChatMessage {
                role: message.role.clone(),
                content: message.content.clone(),
            });
        }
    }

    messages.push(LlmChatMessage {
        role: user_message.role.clone(),
        content: user_message.content.clone(),
    });

    call_chat_completion_with_messages(profile, messages)
}

fn build_cli_system_prompt(character: &CharacterPromptInfo, session: &SessionInfo) -> String {
    let mut prompt = character.system_prompt.clone();

    let mode_name = session.mode_name.as_deref().unwrap_or("Free Talk");
    let mode_prompt = session
        .mode_prompt_suffix
        .as_deref()
        .unwrap_or("Let the conversation flow naturally and help the learner practice speaking.");
    prompt.push_str(&format!("\n\n[Practice Mode: {mode_name}]\n{mode_prompt}"));

    if let Some(scenario_name) = session.scenario_name.as_deref() {
        prompt.push_str(&format!("\n\n[Current Scenario: {scenario_name}]"));
        if let Some(description) = session.scenario_description.as_deref() {
            prompt.push('\n');
            prompt.push_str(description);
        }
        prompt
            .push_str("\nPlease stay in character for this scenario throughout the conversation.");
    }

    prompt
}

fn call_session_coaching_report(
    profile: &ProfileInfo,
    session: &SessionInfo,
    summary: &SessionExportSummary,
    transcript_markdown: &str,
) -> Result<String, String> {
    let title = session.title.as_deref().unwrap_or("Untitled");
    let character = session
        .character_name
        .as_deref()
        .unwrap_or(session.character_id.as_str());
    let mode = session.mode_name.as_deref().unwrap_or("Free Talk");
    let scenario = session.scenario_name.as_deref().unwrap_or("none");
    let transcript_for_model = truncate_prompt_text(transcript_markdown, 12000);
    let user_prompt = format!(
        "Session title: {title}\nCharacter: {character} ({character_id})\nPractice mode: {mode}\nScenario: {scenario}\nMessages: {message_count} total, {user_count} user, {assistant_count} assistant.\n\nTranscript:\n{transcript_for_model}",
        character_id = session.character_id.as_str(),
        message_count = summary.message_count,
        user_count = summary.user_message_count,
        assistant_count = summary.assistant_message_count,
    );

    call_chat_completion_with_messages(
        profile,
        vec![
            LlmChatMessage {
                role: "system".into(),
                content: "You are SpeakMate's speaking coach. Produce a concise Markdown coaching report for the learner. Focus on practical speaking improvements, useful phrases, grammar or pronunciation risks visible from the transcript, and a short next-practice plan. Use Chinese unless the learner clearly used another language.".into(),
            },
            LlmChatMessage {
                role: "user".into(),
                content: user_prompt,
            },
        ],
    )
}

fn truncate_prompt_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.into();
    }

    let mut truncated: String = value.chars().take(max_chars).collect();
    truncated.push_str("\n\n[Transcript truncated for coaching prompt.]");
    truncated
}

fn call_chat_completion_with_messages(
    profile: &ProfileInfo,
    messages: Vec<LlmChatMessage>,
) -> Result<String, String> {
    let api_key = get_api_key_for_profile(profile)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("Failed to start async runtime: {error}"))?;

    runtime.block_on(call_chat_completion(
        &profile.base_url,
        &api_key,
        &profile.model,
        messages,
    ))
}

fn apply_send_message_env_overrides(profile: &mut ProfileInfo, warnings: &mut Vec<String>) {
    if let Ok(base_url) = env::var("SPEAKMATE_CLI_SEND_BASE_URL") {
        let trimmed = base_url.trim();
        if !trimmed.is_empty() {
            profile.base_url = trimmed.into();
            profile.provider = infer_provider(&profile.base_url);
            warnings.push("SPEAKMATE_CLI_SEND_BASE_URL override is active.".into());
        }
    }

    if let Ok(provider) = env::var("SPEAKMATE_CLI_SEND_PROVIDER") {
        let trimmed = provider.trim();
        if !trimmed.is_empty() {
            profile.provider = normalize_provider(trimmed, &profile.base_url);
            warnings.push("SPEAKMATE_CLI_SEND_PROVIDER override is active.".into());
        }
    }

    if let Ok(model) = env::var("SPEAKMATE_CLI_SEND_MODEL") {
        let trimmed = model.trim();
        if !trimmed.is_empty() {
            profile.model = trimmed.into();
            warnings.push("SPEAKMATE_CLI_SEND_MODEL override is active.".into());
        }
    }

    profile.requires_api_key = requires_api_key(&profile.provider);
    profile.supports_stt = supports_stt(&profile.provider);
}

fn get_api_key_for_profile(profile: &ProfileInfo) -> Result<String, String> {
    if let Ok(api_key) = env::var("SPEAKMATE_CLI_SEND_API_KEY") {
        return Ok(api_key);
    }

    if let Ok(entry) = Entry::new("speakmate", &format!("api-key:{}", profile.id)) {
        if let Ok(password) = entry.get_password() {
            return Ok(password);
        }
    }

    if profile.id == "default" {
        if let Ok(entry) = Entry::new("speakmate", "api-key") {
            if let Ok(password) = entry.get_password() {
                return Ok(password);
            }
        }
    }

    if profile.requires_api_key {
        Err(format!(
            "Active profile '{}' requires an API key, but no keychain entry was found.",
            profile.id
        ))
    } else {
        Ok(String::new())
    }
}

async fn call_chat_completion(
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: Vec<LlmChatMessage>,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(CLI_CHAT_COMPLETION_TIMEOUT)
        .build()
        .map_err(|error| format!("Failed to create HTTP client: {error}"))?;
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let request_body = ChatCompletionRequest {
        model: model.into(),
        messages,
        stream: false,
    };

    let request = client.post(url).header("Content-Type", "application/json");
    let request = if api_key.trim().is_empty() {
        request
    } else {
        request.header("Authorization", format!("Bearer {api_key}"))
    };

    let response = request
        .json(&request_body)
        .send()
        .await
        .map_err(|error| format!("Chat completion request failed: {error}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Chat completion API returned {status}: {body}"));
    }

    let parsed = response
        .json::<ChatCompletionResponse>()
        .await
        .map_err(|error| format!("Failed to decode chat completion response: {error}"))?;

    parsed
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .filter(|content| !content.trim().is_empty())
        .ok_or_else(|| "Chat completion response did not include assistant content.".into())
}

fn load_sessions(
    conn: &Connection,
    limit: usize,
    warnings: &mut Vec<String>,
) -> Result<Vec<SessionInfo>, String> {
    if !table_exists(conn, "chat_sessions")? {
        warnings.push("chat_sessions table is missing.".into());
        return Ok(Vec::new());
    }

    let has_context = session_context_columns_exist(conn)?;
    let mut stmt = if has_context {
        conn.prepare(
            "SELECT
                s.id,
                s.character_id,
                c.name,
                s.title,
                s.mode_id,
                pm.name,
                s.scenario_id,
                sc.name,
                pm.prompt_suffix,
                sc.description,
                s.created_at,
                s.updated_at,
                COUNT(m.id) AS message_count
             FROM chat_sessions s
             LEFT JOIN characters c ON c.id = s.character_id
             LEFT JOIN practice_modes pm ON pm.id = COALESCE(s.mode_id, 'free_talk')
             LEFT JOIN scenarios sc ON sc.id = s.scenario_id
             LEFT JOIN messages m ON m.session_id = s.id
             GROUP BY s.id, s.character_id, c.name, s.title, s.mode_id, pm.name, s.scenario_id, sc.name, pm.prompt_suffix, sc.description, s.created_at, s.updated_at
             ORDER BY s.updated_at DESC
             LIMIT ?1",
        )
    } else {
        conn.prepare(
            "SELECT
                s.id,
                s.character_id,
                c.name,
                s.title,
                s.created_at,
                s.updated_at,
                COUNT(m.id) AS message_count
             FROM chat_sessions s
             LEFT JOIN characters c ON c.id = s.character_id
             LEFT JOIN messages m ON m.session_id = s.id
             GROUP BY s.id, s.character_id, c.name, s.title, s.created_at, s.updated_at
             ORDER BY s.updated_at DESC
             LIMIT ?1",
        )
    }
    .map_err(|error| format!("Failed to prepare session query: {error}"))?;

    let rows = stmt
        .query_map([limit as i64], |row| {
            if has_context {
                row_to_session(row)
            } else {
                row_to_legacy_session(row)
            }
        })
        .map_err(|error| format!("Failed to read sessions: {error}"))?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row.map_err(|error| format!("Failed to decode session row: {error}"))?);
    }
    Ok(sessions)
}

fn load_session(
    conn: &Connection,
    session_id: &str,
    warnings: &mut Vec<String>,
) -> Result<Option<SessionInfo>, String> {
    if !table_exists(conn, "chat_sessions")? {
        warnings.push("chat_sessions table is missing.".into());
        return Ok(None);
    }

    let has_context = session_context_columns_exist(conn)?;
    let result = if has_context {
        conn.query_row(
            "SELECT
                s.id,
                s.character_id,
                c.name,
                s.title,
                s.mode_id,
                pm.name,
                s.scenario_id,
                sc.name,
                pm.prompt_suffix,
                sc.description,
                s.created_at,
                s.updated_at,
                COUNT(m.id) AS message_count
             FROM chat_sessions s
             LEFT JOIN characters c ON c.id = s.character_id
             LEFT JOIN practice_modes pm ON pm.id = COALESCE(s.mode_id, 'free_talk')
             LEFT JOIN scenarios sc ON sc.id = s.scenario_id
             LEFT JOIN messages m ON m.session_id = s.id
             WHERE s.id = ?1
             GROUP BY s.id, s.character_id, c.name, s.title, s.mode_id, pm.name, s.scenario_id, sc.name, pm.prompt_suffix, sc.description, s.created_at, s.updated_at",
            [session_id],
            row_to_session,
        )
    } else {
        conn.query_row(
            "SELECT
                s.id,
                s.character_id,
                c.name,
                s.title,
                s.created_at,
                s.updated_at,
                COUNT(m.id) AS message_count
             FROM chat_sessions s
             LEFT JOIN characters c ON c.id = s.character_id
             LEFT JOIN messages m ON m.session_id = s.id
             WHERE s.id = ?1
             GROUP BY s.id, s.character_id, c.name, s.title, s.created_at, s.updated_at",
            [session_id],
            row_to_legacy_session,
        )
    };

    match result {
        Ok(session) => Ok(Some(session)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(format!("Failed to read session '{session_id}': {error}")),
    }
}

fn row_to_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionInfo> {
    Ok(SessionInfo {
        id: row.get(0)?,
        character_id: row.get(1)?,
        character_name: row.get(2)?,
        title: row.get(3)?,
        mode_id: row.get(4)?,
        mode_name: row.get(5)?,
        scenario_id: row.get(6)?,
        scenario_name: row.get(7)?,
        mode_prompt_suffix: row.get(8)?,
        scenario_description: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
        message_count: row.get(12)?,
    })
}

fn row_to_legacy_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionInfo> {
    Ok(SessionInfo {
        id: row.get(0)?,
        character_id: row.get(1)?,
        character_name: row.get(2)?,
        title: row.get(3)?,
        mode_id: None,
        mode_name: Some("Free Talk".into()),
        scenario_id: None,
        scenario_name: None,
        mode_prompt_suffix: None,
        scenario_description: None,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        message_count: row.get(6)?,
    })
}

fn load_messages(
    conn: &Connection,
    session_id: &str,
    warnings: &mut Vec<String>,
) -> Result<Vec<MessageInfo>, String> {
    if !table_exists(conn, "messages")? {
        warnings.push("messages table is missing.".into());
        return Ok(Vec::new());
    }

    let mut stmt = conn
        .prepare(
            "SELECT id, role, content, timestamp
             FROM messages
             WHERE session_id = ?1
             ORDER BY timestamp ASC",
        )
        .map_err(|error| format!("Failed to prepare message query: {error}"))?;

    let rows = stmt
        .query_map([session_id], |row| {
            Ok(MessageInfo {
                id: row.get(0)?,
                role: row.get(1)?,
                content: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })
        .map_err(|error| format!("Failed to read messages: {error}"))?;

    let mut messages = Vec::new();
    for row in rows {
        messages.push(row.map_err(|error| format!("Failed to decode message row: {error}"))?);
    }
    Ok(messages)
}

fn table_exists(conn: &Connection, table_name: &str) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table_name],
            |row| row.get(0),
        )
        .map_err(|error| format!("Failed to inspect database schema: {error}"))?;

    Ok(count > 0)
}

fn column_exists(conn: &Connection, table_name: &str, column_name: &str) -> Result<bool, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table_name})"))
        .map_err(|error| format!("Failed to inspect {table_name} columns: {error}"))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("Failed to read {table_name} columns: {error}"))?;

    for row in rows {
        if row.map_err(|error| format!("Failed to decode {table_name} column: {error}"))?
            == column_name
        {
            return Ok(true);
        }
    }

    Ok(false)
}

fn session_context_columns_exist(conn: &Connection) -> Result<bool, String> {
    Ok(column_exists(conn, "chat_sessions", "mode_id")?
        && column_exists(conn, "chat_sessions", "scenario_id")?
        && table_exists(conn, "practice_modes")?)
}

fn ensure_column(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
    column_definition: &str,
) -> Result<(), String> {
    if column_exists(conn, table_name, column_name)? {
        return Ok(());
    }

    let sql = format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {column_definition}");
    conn.execute(&sql, [])
        .map_err(|error| format!("Failed to add {table_name}.{column_name}: {error}"))?;
    Ok(())
}

fn ensure_v02_schema(conn: &Connection, warnings: &mut Vec<String>) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS practice_modes (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            prompt_suffix TEXT NOT NULL,
            is_preset INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|error| format!("Failed to ensure practice_modes schema: {error}"))?;
    seed_cli_practice_modes(conn)?;
    ensure_column(conn, "chat_sessions", "mode_id", "TEXT")?;
    ensure_column(conn, "chat_sessions", "scenario_id", "TEXT")?;
    warnings.push("V0.2 session context schema is available.".into());
    Ok(())
}

fn seed_cli_practice_modes(conn: &Connection) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let presets = [
        (
            "free_talk",
            "Free Talk",
            "Open-ended conversation for relaxed speaking practice.",
            "Let the conversation flow naturally. Keep the tone friendly, ask follow-up questions, and gently model better English without over-correcting every sentence.",
        ),
        (
            "roleplay",
            "Roleplay",
            "A focused roleplay where the learner practices real-life interaction.",
            "Stay in a clear roleplay setup. Drive the scene with realistic turns, ask context-aware questions, and help the learner practice useful phrases for the situation.",
        ),
        (
            "scenario_drill",
            "Scenario Drill",
            "Structured repetition for one practical situation.",
            "Keep the learner inside the selected scenario. Repeat and vary the same practical language pattern so the learner can improve accuracy and fluency.",
        ),
        (
            "ielts_speaking",
            "IELTS Speaking",
            "IELTS-style spoken answer practice with coaching.",
            "Act as an IELTS speaking examiner. Ask concise Part 1, Part 2, or Part 3 style prompts, then give practical feedback on fluency, vocabulary, grammar, and coherence.",
        ),
    ];

    for (id, name, description, prompt_suffix) in presets {
        conn.execute(
            "INSERT OR IGNORE INTO practice_modes
                (id, name, description, prompt_suffix, is_preset, created_at)
             VALUES (?1, ?2, ?3, ?4, 1, ?5)",
            params![id, name, description, prompt_suffix, now],
        )
        .map_err(|error| format!("Failed to seed practice mode '{id}': {error}"))?;
    }
    Ok(())
}

fn read_app_config(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    let result = conn.query_row(
        "SELECT value FROM app_config WHERE key = ?1",
        [key],
        |row| row.get(0),
    );

    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(format!("Failed to read app_config.{key}: {error}")),
    }
}

fn save_app_config_value(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO app_config (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(|error| format!("Failed to save app_config.{key}: {error}"))?;
    Ok(())
}

fn count_table_rows(conn: &Connection, table_name: &str) -> Result<Option<i64>, String> {
    if !table_exists(conn, table_name)? {
        return Ok(None);
    }

    let sql = format!("SELECT COUNT(*) FROM {table_name}");
    let count = conn
        .query_row(&sql, [], |row| row.get(0))
        .map_err(|error| format!("Failed to count {table_name}: {error}"))?;
    Ok(Some(count))
}

fn append_audit_event(db: &DbInfo, event: &AuditEvent) -> Result<String, String> {
    let audit_path = resolve_audit_log_path(db)
        .ok_or_else(|| "Cannot write audit log without a selected database path.".to_string())?;
    let line = serde_json::to_string(event)
        .map_err(|error| format!("Failed to serialize audit event: {error}"))?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&audit_path)
        .map_err(|error| {
            format!(
                "Failed to open audit log '{}': {error}",
                audit_path.display()
            )
        })?;
    writeln!(file, "{line}").map_err(|error| {
        format!(
            "Failed to write audit log '{}': {error}",
            audit_path.display()
        )
    })?;

    Ok(audit_path.display().to_string())
}

fn resolve_audit_log_path(db: &DbInfo) -> Option<PathBuf> {
    let db_path = db.path.as_deref()?;
    let db_parent = Path::new(db_path).parent()?;
    Some(db_parent.join(AUDIT_LOG_FILE_NAME))
}

fn read_audit_events(
    audit_path: &Path,
    warnings: &mut Vec<String>,
) -> Result<Vec<AuditEvent>, String> {
    let file = File::open(audit_path).map_err(|error| {
        format!(
            "Failed to open audit log '{}': {error}",
            audit_path.display()
        )
    })?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line.map_err(|error| {
            format!(
                "Failed to read audit log '{}' line {}: {error}",
                audit_path.display(),
                index + 1
            )
        })?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match serde_json::from_str::<AuditEvent>(trimmed) {
            Ok(event) => events.push(event),
            Err(error) => warnings.push(format!(
                "Skipped malformed audit log line {}: {}",
                index + 1,
                error
            )),
        }
    }

    Ok(events)
}

fn normalize_provider(stored_provider: &str, base_url: &str) -> String {
    match stored_provider.trim().to_ascii_lowercase().as_str() {
        "openai" => "openai".into(),
        "ollama" => "ollama".into(),
        "custom" => "custom".into(),
        _ => infer_provider(base_url),
    }
}

fn infer_provider(base_url: &str) -> String {
    let normalized = base_url.trim().to_ascii_lowercase();

    if normalized.contains("localhost:11434")
        || normalized.contains("127.0.0.1:11434")
        || normalized.contains("0.0.0.0:11434")
        || normalized.contains("ollama")
    {
        "ollama".into()
    } else if normalized.contains("api.openai.com") {
        "openai".into()
    } else {
        "custom".into()
    }
}

fn requires_api_key(provider: &str) -> bool {
    provider == "openai"
}

fn supports_stt(provider: &str) -> bool {
    provider != "ollama"
}

fn print_json<T: Serialize>(value: &T) -> Result<(), String> {
    let output = serde_json::to_string_pretty(value)
        .map_err(|error| format!("Failed to serialize JSON: {error}"))?;
    println!("{output}");
    Ok(())
}

fn print_status_text(report: &StatusReport) {
    println!("SpeakMate CLI {}", report.version);
    println!(
        "Database: {}",
        report.db.path.as_deref().unwrap_or("not discovered")
    );
    println!("Database exists: {}", report.db.exists);
    println!("Profiles: {}", report.profile_count);

    if let Some(profile) = &report.active_profile {
        println!(
            "Active profile: {} ({}, {})",
            profile.name, profile.provider, profile.model
        );
    }

    print_optional_count("Characters", report.character_count);
    print_optional_count("Sessions", report.session_count);
    print_optional_count("Messages", report.message_count);
    print_warnings(&report.warnings);
}

fn print_profiles_text(report: &ProfileListReport) {
    if report.profiles.is_empty() {
        println!("No config profiles found.");
    } else {
        for profile in &report.profiles {
            let marker = if profile.is_default { "*" } else { " " };
            println!(
                "{marker} {} [{}] {} / {}",
                profile.name, profile.provider, profile.base_url, profile.model
            );
        }
    }

    print_warnings(&report.warnings);
}

fn print_profile_switch_text(report: &ProfileSwitchReport) {
    println!(
        "Active profile: {} ({})",
        report.active_profile.name, report.active_profile.id
    );
    println!("Changed: {}", report.changed);
    println!("Audit log: {}", report.audit_log);
    print_warnings(&report.warnings);
}

fn print_characters_text(report: &CharacterListReport) {
    if report.characters.is_empty() {
        println!("No characters found.");
    } else {
        for character in &report.characters {
            let voice = character
                .voice_name
                .as_deref()
                .or(character.voice_lang.as_deref())
                .unwrap_or("default voice");
            println!(
                "{} ({}) [{}] {} - {}",
                character.name, character.id, character.language, character.style, voice
            );
        }
    }

    print_warnings(&report.warnings);
}

fn print_sessions_text(report: &SessionListReport) {
    if report.sessions.is_empty() {
        println!("No sessions found.");
    } else {
        for session in &report.sessions {
            let title = session.title.as_deref().unwrap_or("Untitled");
            let character = session
                .character_name
                .as_deref()
                .unwrap_or(&session.character_id);
            println!(
                "{} [{}] {} messages - {}",
                session.id, character, session.message_count, title
            );
        }
    }

    print_warnings(&report.warnings);
}

fn print_session_start_text(report: &SessionStartReport) {
    let title = report.session.title.as_deref().unwrap_or("Untitled");
    let character = report
        .session
        .character_name
        .as_deref()
        .unwrap_or(&report.session.character_id);
    println!("Started session: {} - {}", report.session.id, title);
    println!("Character: {character}");
    println!("Audit log: {}", report.audit_log);
    print_warnings(&report.warnings);
}

fn print_message_append_text(report: &MessageAppendReport) {
    println!(
        "Appended message: {} [{}]",
        report.message.id, report.message.role
    );
    if let Some(session) = &report.session {
        println!("Session: {}", session.id);
    }
    println!("Audit log: {}", report.audit_log);
    print_warnings(&report.warnings);
}

fn print_send_message_text(report: &SendMessageReport) {
    println!("Sent message: {}", report.user_message.id);
    println!("Assistant response: {}", report.assistant_message.id);
    println!(
        "Profile: {} ({})",
        report.profile.name, report.profile.model
    );
    if let Some(session) = &report.session {
        println!("Session: {}", session.id);
    }
    println!("Audit log: {}", report.audit_log);
    print_warnings(&report.warnings);
}

fn print_retry_last_message_text(report: &SendMessageReport) {
    println!("Retried user message: {}", report.user_message.id);
    println!("Assistant response: {}", report.assistant_message.id);
    println!(
        "Profile: {} ({})",
        report.profile.name, report.profile.model
    );
    if let Some(session) = &report.session {
        println!("Session: {}", session.id);
    }
    println!("Audit log: {}", report.audit_log);
    print_warnings(&report.warnings);
}

fn print_session_export_text(report: &SessionExportReport) {
    if let Some(session) = &report.session {
        let title = session.title.as_deref().unwrap_or("Untitled");
        println!("Session: {} - {}", session.id, title);
        if let Some(summary) = &report.summary {
            println!(
                "Messages: {} (user: {}, assistant: {}, other: {})",
                summary.message_count,
                summary.user_message_count,
                summary.assistant_message_count,
                summary.other_message_count
            );
        } else {
            println!("Messages: {}", report.messages.len());
        }

        if let Some(transcript) = &report.transcript_markdown {
            println!();
            println!("{transcript}");
        } else {
            for message in &report.messages {
                println!("[{}] {}", message.role, message.content);
            }
        }
    } else {
        println!("Session not found.");
    }

    print_warnings(&report.warnings);
}

fn print_session_coaching_report_text(report: &SessionCoachingReport) {
    let title = report.session.title.as_deref().unwrap_or("Untitled");
    println!("Session: {} - {}", report.session.id, title);
    println!(
        "Profile: {} ({})",
        report.profile.name, report.profile.model
    );
    println!(
        "Messages: {} (user: {}, assistant: {}, other: {})",
        report.summary.message_count,
        report.summary.user_message_count,
        report.summary.assistant_message_count,
        report.summary.other_message_count
    );
    println!();
    println!("{}", report.coaching_markdown.trim());
    print_warnings(&report.warnings);
}

fn print_audit_log_text(report: &AuditLogReport) {
    println!(
        "Audit log: {}",
        report.audit_log.as_deref().unwrap_or("not resolved")
    );
    if !report.filters.is_empty() {
        println!("Filters: {}", format_audit_filter(&report.filters));
    }
    if report.events.is_empty() {
        println!("No audit events found.");
    } else {
        for event in &report.events {
            let target = event
                .target_profile_id
                .as_deref()
                .or(event.target_session_id.as_deref())
                .or(event.target_message_id.as_deref())
                .or(event.target_character_id.as_deref())
                .unwrap_or("n/a");
            println!(
                "{} {} target={} result={}",
                event.timestamp, event.operation, target, event.result
            );
        }
    }
    print_warnings(&report.warnings);
}

fn format_audit_filter(filter: &AuditFilter) -> String {
    let mut parts = Vec::new();
    if let Some(value) = &filter.operation {
        parts.push(format!("operation={value}"));
    }
    if let Some(value) = &filter.actor {
        parts.push(format!("actor={value}"));
    }
    if let Some(value) = &filter.result {
        parts.push(format!("result={value}"));
    }
    if let Some(value) = &filter.target_profile_id {
        parts.push(format!("profile={value}"));
    }
    if let Some(value) = &filter.target_session_id {
        parts.push(format!("session={value}"));
    }
    if let Some(value) = &filter.target_message_id {
        parts.push(format!("message={value}"));
    }
    if let Some(value) = &filter.target_character_id {
        parts.push(format!("character={value}"));
    }
    parts.join(", ")
}

fn print_agent_tools_text(report: &AgentToolsReport) {
    println!("SpeakMate agent tools ({})", report.mode);
    for tool in &report.tools {
        println!("- {}: {}", tool.name, tool.description);
    }
}

fn print_diagnostics_text(report: &DiagnosticsReport) {
    println!("SpeakMate diagnostics {}", report.version);
    println!("OK: {}", report.ok);
    println!(
        "Database: {}",
        report.db.path.as_deref().unwrap_or("not discovered")
    );
    println!(
        "Provider profile schema: {}",
        report.db.provider_profiles_schema
    );
    println!("Legacy app_config schema: {}", report.db.app_config_schema);
    println!("Profiles: {}", report.profiles.count);
    println!("Keychain access: {}", report.runtime.keychain_access);
    print_warnings(&report.warnings);
}

fn print_optional_count(label: &str, value: Option<i64>) {
    if let Some(count) = value {
        println!("{label}: {count}");
    }
}

fn print_warnings(warnings: &[String]) {
    if warnings.is_empty() {
        return;
    }

    println!("Warnings:");
    for warning in warnings {
        println!("- {warning}");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        audit_event_matches_filter, build_audit_filter, build_cli_system_prompt,
        ensure_network_enabled, parse_args, AuditEvent, CharacterPromptInfo, Command, SessionInfo,
    };

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).into()).collect()
    }

    #[test]
    fn parses_session_coach_report_as_network_read_command() {
        let options = parse_args(args(&[
            "session",
            "coach-report",
            "--session",
            "sess-1",
            "--allow-network",
            "--json",
        ]))
        .expect("coach-report args should parse");

        assert!(options.json);
        match options.command {
            Command::SessionCoachReport {
                session_id,
                allow_network,
            } => {
                assert_eq!(session_id, "sess-1");
                assert!(allow_network);
            }
            _ => panic!("expected SessionCoachReport"),
        }
    }

    #[test]
    fn parses_agent_write_allowlist_from_comma_separated_value() {
        let options = parse_args(args(&[
            "agent",
            "tools",
            "list",
            "--allow-writes",
            "--allow-tool",
            "send_message,retry_last_message",
        ]))
        .expect("agent tools args should parse");

        match options.command {
            Command::AgentToolsList { allowed_tools, .. } => {
                assert_eq!(allowed_tools, vec!["send_message", "retry_last_message"]);
            }
            _ => panic!("expected AgentToolsList"),
        }
    }

    #[test]
    fn parses_session_start_context_flags() {
        let options = parse_args(args(&[
            "session",
            "start",
            "--character",
            "emily",
            "--title",
            "Mock interview",
            "--mode",
            "ielts_speaking",
            "--scenario",
            "interview",
            "--yes",
            "--json",
        ]))
        .expect("session start args should parse");

        assert!(options.json);
        match options.command {
            Command::SessionStart {
                character_id,
                title,
                mode_id,
                scenario_id,
                yes,
            } => {
                assert_eq!(character_id, "emily");
                assert_eq!(title.as_deref(), Some("Mock interview"));
                assert_eq!(mode_id.as_deref(), Some("ielts_speaking"));
                assert_eq!(scenario_id.as_deref(), Some("interview"));
                assert!(yes);
            }
            _ => panic!("expected SessionStart"),
        }
    }

    #[test]
    fn cli_system_prompt_orders_character_mode_then_scenario() {
        let character = CharacterPromptInfo {
            system_prompt: "You are Emily.".into(),
        };
        let session = SessionInfo {
            id: "sess-1".into(),
            character_id: "emily".into(),
            character_name: Some("Emily".into()),
            title: Some("Interview".into()),
            mode_id: Some("roleplay".into()),
            mode_name: Some("Roleplay".into()),
            scenario_id: Some("interview".into()),
            scenario_name: Some("Job Interview".into()),
            mode_prompt_suffix: Some("Stay in roleplay.".into()),
            scenario_description: Some("Ask interview questions.".into()),
            created_at: "2026-05-16T00:00:00Z".into(),
            updated_at: "2026-05-16T00:00:00Z".into(),
            message_count: 0,
        };

        let prompt = build_cli_system_prompt(&character, &session);
        let character_index = prompt.find("You are Emily.").unwrap();
        let mode_index = prompt.find("[Practice Mode: Roleplay]").unwrap();
        let scenario_index = prompt.find("[Current Scenario: Job Interview]").unwrap();

        assert!(character_index < mode_index);
        assert!(mode_index < scenario_index);
        assert!(prompt.contains("Stay in roleplay."));
        assert!(prompt.contains("Ask interview questions."));
    }

    #[test]
    fn rejects_network_tool_when_network_gate_is_closed() {
        let error = ensure_network_enabled("send_message", false)
            .expect_err("network gate should reject disabled network access");
        assert!(error.contains("--allow-network"));
        assert!(ensure_network_enabled("send_message", true).is_ok());
    }

    #[test]
    fn audit_filter_matches_operation_actor_and_target_session() {
        let event = AuditEvent {
            timestamp: "2026-04-28T00:00:00Z".into(),
            operation: "practice.message.send".into(),
            actor: "speakmate-cli".into(),
            db_path: Some("speakmate.db".into()),
            target_profile_id: Some("default".into()),
            target_session_id: Some("sess-1".into()),
            target_message_id: Some("msg-1".into()),
            target_character_id: Some("emily".into()),
            previous_profile_id: None,
            result: "success".into(),
            details: "ok".into(),
        };
        let filter = build_audit_filter(
            Some("practice.message.send".into()),
            Some("SPEAKMATE-CLI".into()),
            Some("success".into()),
            None,
            Some("sess-1".into()),
            None,
            None,
        );

        assert!(audit_event_matches_filter(&event, &filter));
    }
}
