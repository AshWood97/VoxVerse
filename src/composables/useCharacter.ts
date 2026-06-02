import { ref, computed } from 'vue';
import type { Character } from '../types/chat';
import { getCharacters, saveCharacter, deleteCharacter } from '../services/tauri/character';
import { logError } from '../utils/errors';

const PNG_SIGNATURE = 0x89504E47;
const MAX_CHARACTER_IMPORT_BYTES = 2 * 1024 * 1024;
const MAX_CHARACTER_CARD_CHUNK_BYTES = 512 * 1024;
const MAX_AVATAR_CHARS = 3_000_000;
const FIELD_LIMITS = {
  name: 80,
  language: 80,
  personality: 4_000,
  style: 2_000,
  systemPrompt: 12_000,
  greeting: 2_000,
  voiceLang: 32,
  voiceNamePattern: 160,
};

function createCharacterId(): string {
  return `char-${Date.now()}`;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function getStringValue(record: Record<string, unknown>, key: string): string | undefined {
  const value = record[key];
  return typeof value === 'string' ? value : undefined;
}

function getVoiceConfigValue(record: Record<string, unknown>): Character['voiceConfig'] {
  const voiceConfig = record.voiceConfig;
  if (!isRecord(voiceConfig)) {
    return undefined;
  }

  const lang = getStringValue(voiceConfig, 'lang');
  if (!lang) {
    return undefined;
  }

  const namePattern = getStringValue(voiceConfig, 'namePattern');
  return namePattern ? { lang, namePattern } : { lang };
}

function assertMaxLength(label: string, value: string, maxLength: number) {
  if (value.length > maxLength) {
    throw new Error(`${label} must be ${maxLength} characters or fewer.`);
  }
}

function validateCharacterPayload(character: Character): Character {
  const normalized: Character = {
    ...character,
    name: character.name.trim(),
    language: character.language.trim(),
    personality: character.personality.trim(),
    style: character.style.trim(),
    systemPrompt: character.systemPrompt.trim(),
    greeting: character.greeting.trim(),
  };

  if (!normalized.name) {
    throw new Error('Character name is required.');
  }

  if (!normalized.systemPrompt) {
    throw new Error('Character system prompt is required.');
  }

  assertMaxLength('Character name', normalized.name, FIELD_LIMITS.name);
  assertMaxLength('Character language', normalized.language, FIELD_LIMITS.language);
  assertMaxLength('Character personality', normalized.personality, FIELD_LIMITS.personality);
  assertMaxLength('Character style', normalized.style, FIELD_LIMITS.style);
  assertMaxLength('Character system prompt', normalized.systemPrompt, FIELD_LIMITS.systemPrompt);
  assertMaxLength('Character greeting', normalized.greeting, FIELD_LIMITS.greeting);
  assertMaxLength('Character avatar', normalized.avatar, MAX_AVATAR_CHARS);

  if (normalized.voiceConfig) {
    assertMaxLength('Voice language', normalized.voiceConfig.lang, FIELD_LIMITS.voiceLang);
    if (normalized.voiceConfig.namePattern) {
      assertMaxLength(
        'Voice name pattern',
        normalized.voiceConfig.namePattern,
        FIELD_LIMITS.voiceNamePattern,
      );
    }
  }

  return normalized;
}

function validateImportFile(file: File) {
  const lowerName = file.name.toLowerCase();
  const isSupported =
    lowerName.endsWith('.json')
    || lowerName.endsWith('.png')
    || file.type === 'application/json'
    || file.type === 'image/png';

  if (!isSupported) {
    throw new Error('Only JSON and PNG character files are supported.');
  }

  if (file.size > MAX_CHARACTER_IMPORT_BYTES) {
    throw new Error('Character import file is too large. Please keep it under 2 MB.');
  }
}

function readFileAsText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new Error('Failed to read text file'));
    reader.readAsText(file);
  });
}

function readFileAsArrayBuffer(file: File): Promise<ArrayBuffer> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as ArrayBuffer);
    reader.onerror = () => reject(new Error('Failed to read binary file'));
    reader.readAsArrayBuffer(file);
  });
}

function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new Error('Failed to read image preview'));
    reader.readAsDataURL(file);
  });
}

function decodeBase64Utf8(base64: string): string {
  const binary = atob(base64);
  const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));
  return new TextDecoder().decode(bytes);
}

function unwrapCharacterPayload(payload: unknown): Record<string, unknown> {
  if (!isRecord(payload)) {
    throw new Error('Invalid character payload');
  }

  return isRecord(payload.data) ? payload.data : payload;
}

function buildCharacterFromJson(payload: unknown): Character {
  const record = unwrapCharacterPayload(payload);
  const name = getStringValue(record, 'name');
  const systemPrompt = getStringValue(record, 'systemPrompt');

  if (!name || !systemPrompt) {
    throw new Error('Invalid character JSON schema. Missing name or systemPrompt.');
  }

  return validateCharacterPayload({
    id: createCharacterId(),
    name,
    language: getStringValue(record, 'language') || '',
    personality: getStringValue(record, 'personality') || '',
    style: getStringValue(record, 'style') || '',
    avatar: getStringValue(record, 'avatar') || '👤',
    systemPrompt,
    greeting: getStringValue(record, 'greeting') || '',
    voiceConfig: getVoiceConfigValue(record),
  });
}

function buildCharacterFromCard(payload: unknown, avatar: string): Character {
  const record = unwrapCharacterPayload(payload);

  return validateCharacterPayload({
    id: createCharacterId(),
    name: getStringValue(record, 'name') || 'Unknown',
    language: '',
    personality: (getStringValue(record, 'description') || getStringValue(record, 'personality') || '').slice(0, 1000),
    style: '',
    avatar,
    systemPrompt: [
      getStringValue(record, 'description'),
      getStringValue(record, 'personality'),
      getStringValue(record, 'scenario'),
      getStringValue(record, 'mes_example'),
    ].filter(Boolean).join('\n\n').slice(0, FIELD_LIMITS.systemPrompt),
    greeting: getStringValue(record, 'first_mes') || '',
  });
}

function extractCharaChunkBase64(buffer: ArrayBuffer): string {
  const dataView = new DataView(buffer);
  const bytes = new Uint8Array(buffer);

  if (buffer.byteLength < 8) {
    throw new Error('Invalid PNG file');
  }

  if (dataView.getUint32(0) !== PNG_SIGNATURE) {
    throw new Error('Invalid PNG signature');
  }

  let offset = 8;
  while (offset + 8 <= buffer.byteLength) {
    const length = dataView.getUint32(offset);
    offset += 4;

    const type = String.fromCharCode(
      bytes[offset],
      bytes[offset + 1],
      bytes[offset + 2],
      bytes[offset + 3],
    );
    offset += 4;

    if (offset + length + 4 > buffer.byteLength) {
      throw new Error('Invalid PNG chunk boundary');
    }

    if (type === 'tEXt') {
      if (length > MAX_CHARACTER_CARD_CHUNK_BYTES) {
        throw new Error('Character card metadata chunk is too large.');
      }

      const chunkData = bytes.subarray(offset, offset + length);
      const keywordEnd = chunkData.indexOf(0);

      if (keywordEnd !== -1) {
        const keyword = new TextDecoder().decode(chunkData.subarray(0, keywordEnd));
        if (keyword === 'chara') {
          return new TextDecoder().decode(chunkData.subarray(keywordEnd + 1));
        }
      }
    }

    offset += length + 4;
  }

  throw new Error('No character data (chara chunk) found in this PNG');
}

const PRESET_CHARACTERS: Character[] = [
  {
    id: 'emily',
    name: 'Emily',
    language: 'English (US)',
    personality: 'Warm, outgoing, casual. Speaks at a moderate pace with natural American slang.',
    style: 'Friendly barista who loves small talk. Uses contractions and casual expressions.',
    avatar: '☕',
    systemPrompt: `You are Emily, a friendly barista at a cozy coffee shop in New York City. You are warm, outgoing, and love chatting with customers. 

Key behaviors:
- Speak naturally in American English with casual expressions and slang
- Keep your responses conversational and relatively short (2-4 sentences usually)
- If the user makes grammar mistakes, gently correct them by naturally using the correct form in your response
- Occasionally ask follow-up questions to keep the conversation going
- Stay in character as a barista - reference coffee, the shop, your day, etc.
- Adjust your language complexity based on the user's apparent level`,
    greeting: "Hey there! Welcome to Joe's Coffee! ☕ What can I get started for you today?",
    voiceConfig: { lang: 'en-US', namePattern: '(female|zira|samantha|aria)' },
  },
  {
    id: 'kenji',
    name: 'Kenji',
    language: 'English (JP accent)',
    personality: 'Patient, methodical, encouraging. Focuses on grammar accuracy.',
    style: 'Experienced English teacher who gives clear explanations and examples.',
    avatar: '📚',
    systemPrompt: `You are Kenji, an experienced English teacher from Tokyo who has taught English for 15 years. You are patient, methodical, and encouraging.

Key behaviors:
- Speak clearly and at a slightly slower pace
- When the user makes mistakes, point them out kindly and explain the grammar rule
- Provide example sentences to illustrate correct usage
- Use encouraging phrases like "Good try!", "You're improving!", "Almost perfect!"
- Occasionally teach useful phrases or idioms
- Keep responses educational but not overwhelming
- Ask the user to try again when they make repeated mistakes`,
    greeting: "Hello! I'm Kenji, your English practice partner. 📚 Let's have a conversation to improve your English skills. What topic would you like to discuss today?",
    voiceConfig: { lang: 'en-US', namePattern: '(male|david|guy|andrew)' },
  },
  {
    id: 'sophie',
    name: 'Sophie',
    language: 'English (FR accent)',
    personality: 'Lively, curious, cultured. Loves discussing art, food, and travel.',
    style: 'French exchange student who mixes in occasional French expressions.',
    avatar: '🎨',
    systemPrompt: `You are Sophie, a lively French exchange student studying art history in London. You are curious, cultured, and love discussing art, food, movies, and travel.

Key behaviors:
- Speak enthusiastically with occasional French expressions (then translate them)
- Be curious about the user's culture and experiences
- Share interesting facts about French culture, art, or cuisine
- Keep the conversation fun and engaging
- If the user makes mistakes, subtly model the correct form without being preachy
- Use expressions like "C'est magnifique!" (That's magnificent!), "Mon Dieu!" (My God!)
- Suggest more expressive or colorful ways to say things`,
    greeting: "Bonjour! I'm Sophie! 🎨 I just came from an amazing exhibition at the Tate Modern. Do you like art? Or maybe we can talk about something else — I'm curious about everything!",
    voiceConfig: { lang: 'en-GB', namePattern: '(female|hazel|mia|chloe)' }, // Use en-GB as fallback for Sophie as French accent English is rare in basic TTS
  },
];

const charactersState = ref<Character[]>([]);

export function useCharacter() {
  async function loadCharacters() {
    try {
      let dbChars = await getCharacters();
      if (dbChars.length === 0) {
        // Init defaults
        for (const preset of PRESET_CHARACTERS) {
          await saveCharacter(preset);
        }
        dbChars = await getCharacters(); // Reload after save
      }
      charactersState.value = dbChars;
    } catch (error) {
      logError('Failed to load characters from DB, falling back to presets', error);
      if (charactersState.value.length === 0) {
        charactersState.value = [...PRESET_CHARACTERS];
      }
    }
  }

  async function addOrUpdateCharacter(char: Character) {
    await saveCharacter(char);
    await loadCharacters();
  }

  async function removeCharacter(id: string) {
    if (PRESET_CHARACTERS.some(p => p.id === id)) {
      throw new Error('Cannot delete preset characters.');
    }
    await deleteCharacter(id);
    await loadCharacters();
  }

  function getCharacterById(id: string): Character | undefined {
    return charactersState.value.find((c) => c.id === id);
  }

  function exportCharacter(id: string) {
    const char = getCharacterById(id);
    if (!char) return;

    const { id: _unused, ...exportData } = char;

    const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(exportData, null, 2));
    const downloadAnchorNode = document.createElement('a');
    downloadAnchorNode.setAttribute("href", dataStr);
    downloadAnchorNode.setAttribute("download", `${char.name.replace(/\s+/g, '_').toLowerCase()}_character.json`);
    document.body.appendChild(downloadAnchorNode);
    downloadAnchorNode.click();
    downloadAnchorNode.remove();
  }

  async function importCharacter(file: File) {
    validateImportFile(file);
    const isPng = file.name.toLowerCase().endsWith('.png') || file.type === 'image/png';
    const importedChar = isPng
      ? buildCharacterFromCard(
          JSON.parse(decodeBase64Utf8(extractCharaChunkBase64(await readFileAsArrayBuffer(file)))),
          await readFileAsDataUrl(file),
        )
      : buildCharacterFromJson(JSON.parse(await readFileAsText(file)));

    await addOrUpdateCharacter(importedChar);
    return importedChar;
  }

  return {
    characters: computed(() => charactersState.value),
    loadCharacters,
    addOrUpdateCharacter,
    removeCharacter,
    getCharacterById,
    exportCharacter,
    importCharacter
  };
}
