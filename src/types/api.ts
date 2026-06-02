export type AiProvider = 'openai' | 'ollama' | 'custom';

export interface ApiConfig {
  provider: AiProvider;
  baseUrl: string;
  apiKey: string;
  model: string;
}

export interface ConfigInfo {
  active_profile_id: string;
  active_profile_name: string;
  provider: AiProvider;
  base_url: string;
  model: string;
  has_api_key: boolean;
  requires_api_key: boolean;
  supports_stt: boolean;
}

export interface ConfigProfileInfo {
  id: string;
  name: string;
  provider: AiProvider;
  base_url: string;
  model: string;
  is_default: boolean;
  has_api_key: boolean;
  requires_api_key: boolean;
  supports_stt: boolean;
}

export interface StreamEvent {
  token: string;
  done: boolean;
}

export type DiagnosticStatus = 'success' | 'warning' | 'error';

export interface DiagnosticCheck {
  status: DiagnosticStatus;
  summary: string;
  details: string;
}

export interface RuntimeDiagnostics {
  chat: DiagnosticCheck;
  stt: DiagnosticCheck;
  tts: DiagnosticCheck;
}
