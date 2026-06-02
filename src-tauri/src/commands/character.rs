use crate::error::AppError;
use crate::repositories::db::DbState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

const PRESET_CHARACTER_IDS: [&str; 3] = ["emily", "kenji", "sophie"];

#[derive(Debug, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub lang: String,
    #[serde(rename = "namePattern")]
    pub name_pattern: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub language: String,
    pub personality: String,
    pub style: String,
    pub avatar: String,
    #[serde(rename = "systemPrompt")]
    pub system_prompt: String,
    pub greeting: String,
    #[serde(rename = "voiceConfig")]
    pub voice_config: Option<VoiceConfig>,
}

#[tauri::command]
pub fn get_characters(state: State<'_, DbState>) -> Result<Vec<Character>, AppError> {
    let conn = state.lock_conn()?;
    let mut stmt = conn.prepare("SELECT id, name, language, personality, style, avatar, system_prompt, greeting, voice_lang, voice_name FROM characters")?;

    let character_iter = stmt.query_map([], |row| {
        let voice_lang: Option<String> = row.get(8)?;
        let voice_name: Option<String> = row.get(9)?;

        let voice_config = voice_lang.map(|lang| VoiceConfig {
            lang,
            name_pattern: voice_name,
        });

        Ok(Character {
            id: row.get(0)?,
            name: row.get(1)?,
            language: row.get(2)?,
            personality: row.get(3)?,
            style: row.get(4)?,
            avatar: row.get(5)?,
            system_prompt: row.get(6)?,
            greeting: row.get(7)?,
            voice_config,
        })
    })?;

    let mut characters = Vec::new();
    for character_result in character_iter {
        characters.push(character_result?);
    }

    Ok(characters)
}

#[tauri::command]
pub fn save_character(character: Character, state: State<'_, DbState>) -> Result<(), AppError> {
    let conn = state.lock_conn()?;

    let voice_lang = character.voice_config.as_ref().map(|vc| vc.lang.clone());
    let voice_name = character
        .voice_config
        .as_ref()
        .and_then(|vc| vc.name_pattern.clone());

    conn.execute(
        "INSERT INTO characters (id, name, language, personality, style, avatar, system_prompt, greeting, voice_lang, voice_name)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            name=excluded.name,
            language=excluded.language,
            personality=excluded.personality,
            style=excluded.style,
            avatar=excluded.avatar,
            system_prompt=excluded.system_prompt,
            greeting=excluded.greeting,
            voice_lang=excluded.voice_lang,
            voice_name=excluded.voice_name
        ",
        params![
            character.id,
            character.name,
            character.language,
            character.personality,
            character.style,
            character.avatar,
            character.system_prompt,
            character.greeting,
            voice_lang,
            voice_name
        ],
    )?;

    Ok(())
}

#[tauri::command]
pub fn delete_character(id: String, state: State<'_, DbState>) -> Result<(), AppError> {
    if PRESET_CHARACTER_IDS.contains(&id.as_str()) {
        return Err(AppError::Config(
            "Preset characters cannot be deleted.".into(),
        ));
    }

    let conn = state.lock_conn()?;
    conn.execute("DELETE FROM characters WHERE id = ?1", params![id])?;
    Ok(())
}
