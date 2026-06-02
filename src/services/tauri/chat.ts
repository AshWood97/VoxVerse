import { invoke, Channel } from '@tauri-apps/api/core';
import type {
  ApiConfig,
  ConfigInfo,
  ConfigProfileInfo,
  RuntimeDiagnostics,
  StreamEvent,
} from '../../types/api';

interface MessagePayload {
  role: string;
  content: string;
}

/**
 * Send a message to the LLM via Rust backend and stream the response.
 * Returns the full response string after streaming completes.
 */
export async function sendMessage(
  systemPrompt: string,
  messages: MessagePayload[],
  onToken: (token: string) => void,
): Promise<string> {
  const onEvent = new Channel<StreamEvent>();
  onEvent.onmessage = (event) => {
    if (!event.done && event.token) {
      onToken(event.token);
    }
  };

  const result = await invoke<string>('send_message', {
    systemPrompt,
    messages,
    onEvent,
  });

  return result;
}

/**
 * Save API configuration to the Rust backend.
 * API keys are stored in the OS keychain via the Rust layer.
 */
export async function saveConfig(config: ApiConfig): Promise<void> {
  await invoke('save_config', {
    provider: config.provider,
    baseUrl: config.baseUrl,
    apiKey: config.apiKey,
    model: config.model,
  });
}

/**
 * Get current API configuration (API key is not returned for security).
 */
export async function getConfig(): Promise<ConfigInfo> {
  return invoke<ConfigInfo>('get_config');
}

export async function listConfigProfiles(): Promise<ConfigProfileInfo[]> {
  return invoke<ConfigProfileInfo[]>('list_config_profiles');
}

export async function switchConfigProfile(profileId: string): Promise<ConfigInfo> {
  return invoke<ConfigInfo>('switch_config_profile', { profileId });
}

export async function createConfigProfile(
  name: string,
  config: ApiConfig,
): Promise<ConfigInfo> {
  return invoke<ConfigInfo>('create_config_profile', {
    name,
    provider: config.provider,
    baseUrl: config.baseUrl,
    apiKey: config.apiKey,
    model: config.model,
  });
}

export async function renameConfigProfile(profileId: string, name: string): Promise<ConfigInfo> {
  return invoke<ConfigInfo>('rename_config_profile', { profileId, name });
}

export async function deleteConfigProfile(profileId: string): Promise<ConfigInfo> {
  return invoke<ConfigInfo>('delete_config_profile', { profileId });
}

export async function runRuntimeDiagnostics(): Promise<RuntimeDiagnostics> {
  return invoke<RuntimeDiagnostics>('run_runtime_diagnostics');
}
