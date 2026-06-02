import { invoke } from '@tauri-apps/api/core';
import type { Message } from '../../types/chat';

export interface ChatSession {
  id: string;
  characterId: string;
  title: string | null;
  modeId?: string | null;
  scenarioId?: string | null;
  createdAt: string;
  updatedAt: string;
  messages: Message[];
}

export interface SessionContext {
  sessionId: string;
  modeId: string | null;
  scenarioId: string | null;
}

export async function getSessions(characterId: string): Promise<ChatSession[]> {
  return invoke<ChatSession[]>('get_sessions', { characterId });
}

export async function getSessionMessages(sessionId: string): Promise<Message[]> {
  return invoke<Message[]>('get_session_messages', { sessionId });
}

export async function createSession(session: ChatSession): Promise<void> {
  await invoke('create_session', { session });
}

export async function getSessionContext(id: string): Promise<SessionContext> {
  return invoke<SessionContext>('get_session_context', { id });
}

export async function updateSessionContext(
  id: string,
  modeId: string | null,
  scenarioId: string | null,
): Promise<SessionContext> {
  return invoke<SessionContext>('update_session_context', { id, modeId, scenarioId });
}

export async function updateSessionTitle(id: string, title: string): Promise<void> {
  await invoke('update_session_title', { id, title });
}

export async function deleteSession(id: string): Promise<void> {
  await invoke('delete_session', { id });
}

export async function saveMessage(message: Message): Promise<void> {
  await invoke('save_message', { message });
}
