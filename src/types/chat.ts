export interface Message {
  id: string;
  sessionId?: string;
  role: 'user' | 'assistant' | 'system' | string;
  content: string;
  timestamp: number;
}

export interface Character {
  id: string;
  name: string;
  language: string;
  personality: string;
  style: string;
  avatar: string;
  systemPrompt: string;
  greeting: string;
  voiceConfig?: {
    lang: string;
    namePattern?: string;
  };
}
