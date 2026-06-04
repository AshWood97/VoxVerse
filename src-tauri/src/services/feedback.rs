//! Feedback service: constructs specialized prompts for learning assistance.
//! These prompts are sent as independent LLM requests that don't pollute the main conversation history.

#[derive(Debug, Clone)]
pub struct SummaryCorrection {
    pub original_text: String,
    pub corrected_text: String,
    pub explanation: Option<String>,
    pub better_expression: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SummaryContext {
    pub practice_mode_name: String,
    pub practice_mode_description: String,
    pub scenario_name: Option<String>,
    pub scenario_description: Option<String>,
    pub recent_corrections: Vec<SummaryCorrection>,
}

/// Build a prompt that analyzes user text for grammar, word choice, and naturalness.
/// Returns structured JSON for the frontend to render as a diff view.
pub fn build_correction_prompt(user_text: &str) -> Vec<super::llm::ChatMessage> {
    vec![
        super::llm::ChatMessage {
            role: "system".into(),
            content: "You are an expert English language tutor. Analyze the user's text and return ONLY valid JSON (no markdown fences). Be encouraging but thorough.".into(),
        },
        super::llm::ChatMessage {
            role: "user".into(),
            content: format!(
                r#"Analyze the following English text for grammar, word choice, and naturalness.

Return a JSON object with EXACTLY these fields:
- "hasErrors": boolean - whether any issues were found
- "corrected": string - the corrected version (identical to original if no errors)
- "explanation": string - brief explanation of all changes, in Chinese (中文)
- "betterExpression": string - a more natural/native way to express the same meaning
- "vocabulary": array of objects with "word" and "meaning" (Chinese) for useful vocabulary

Text to analyze: "{}"

Respond ONLY with valid JSON."#,
                user_text
            ),
        },
    ]
}

/// Build a prompt to translate text to a target language.
pub fn build_translation_prompt(text: &str, target_lang: &str) -> Vec<super::llm::ChatMessage> {
    vec![
        super::llm::ChatMessage {
            role: "system".into(),
            content:
                "You are a professional translator. Return ONLY valid JSON (no markdown fences)."
                    .into(),
        },
        super::llm::ChatMessage {
            role: "user".into(),
            content: format!(
                r#"Translate the following text to {target_lang}.

Return a JSON object with EXACTLY these fields:
- "translation": string - the translated text
- "notes": string - any important nuances, cultural context, or alternative translations (in {target_lang})

Text: "{text}"

Respond ONLY with valid JSON."#,
                target_lang = target_lang,
                text = text
            ),
        },
    ]
}

/// Build a prompt to polish/improve user's text to sound more native.
pub fn build_polish_prompt(user_text: &str) -> Vec<super::llm::ChatMessage> {
    vec![
        super::llm::ChatMessage {
            role: "system".into(),
            content: "You are a native English speaker helping a learner improve their expression. Return ONLY valid JSON (no markdown fences).".into(),
        },
        super::llm::ChatMessage {
            role: "user".into(),
            content: format!(
                r#"Improve the following English text to sound more natural and native-like.

Return a JSON object with EXACTLY these fields:
- "polished": string - the improved version
- "changes": array of objects, each with:
  - "original": string - the original phrase
  - "improved": string - the improved phrase
  - "reason": string - why this change makes it better (in Chinese 中文)

Text: "{}"

Respond ONLY with valid JSON."#,
                user_text
            ),
        },
    ]
}

/// Build a prompt to generate a learning summary report for a conversation.
pub fn build_summary_prompt(
    conversation_messages: &[(String, String)],
    context: Option<&SummaryContext>,
) -> Vec<super::llm::ChatMessage> {
    let conversation_text: String = conversation_messages
        .iter()
        .map(|(role, content)| format!("{}: {}", role, content))
        .collect::<Vec<_>>()
        .join("\n");
    let context_text = build_summary_context_text(context);

    vec![
        super::llm::ChatMessage {
            role: "system".into(),
            content: "You are an English learning coach. Analyze the student's conversation and provide a learning report. Return ONLY valid JSON (no markdown fences).".into(),
        },
        super::llm::ChatMessage {
            role: "user".into(),
            content: format!(
                r#"Analyze this English conversation between a student (user) and an AI tutor (assistant).

Practice context:
{context_text}

Generate a learning report as a JSON object with EXACTLY these fields:
- "overallScore": number (1-10) - overall English proficiency shown
- "scoreBreakdown": object with numeric 1-10 scores for "fluency", "grammar", "vocabulary", "coherence", and optional "pronunciation"
- "strengths": array of strings - what the student did well (in Chinese 中文)
- "improvements": array of strings - areas to improve (in Chinese 中文)
- "commonErrors": array of objects with "error" and "correction" - recurring mistakes
- "vocabularyUsed": number - approximate count of unique words used
- "suggestedTopics": array of strings - recommended topics for next practice
- "nextDrills": array of strings - 2-4 concrete short drills the student should practice next

Conversation:
{conversation_text}

Respond ONLY with valid JSON."#,
                context_text = context_text,
                conversation_text = conversation_text
            ),
        },
    ]
}

fn build_summary_context_text(context: Option<&SummaryContext>) -> String {
    let Some(context) = context else {
        return "Practice mode: Free Talk\nScenario: none\nSaved corrections: none".into();
    };

    let mut lines = vec![
        format!("Practice mode: {}", context.practice_mode_name),
        format!("Mode goal: {}", context.practice_mode_description),
    ];

    if let Some(scenario_name) = context.scenario_name.as_deref() {
        lines.push(format!("Scenario: {scenario_name}"));
        if let Some(description) = context.scenario_description.as_deref() {
            lines.push(format!("Scenario detail: {description}"));
        }
    } else {
        lines.push("Scenario: none".into());
    }

    if context.recent_corrections.is_empty() {
        lines.push("Saved corrections: none".into());
    } else {
        lines.push("Saved corrections:".into());
        for correction in &context.recent_corrections {
            let explanation = correction
                .explanation
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("no explanation");
            let better_expression = correction
                .better_expression
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("none");
            lines.push(format!(
                "- original: {}; corrected: {}; note: {}; better expression: {}",
                correction.original_text, correction.corrected_text, explanation, better_expression
            ));
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{
        build_correction_prompt, build_polish_prompt, build_summary_prompt,
        build_translation_prompt,
    };

    #[test]
    fn correction_prompt_demands_json_and_contains_source_text() {
        let messages = build_correction_prompt("I has a apple.");
        assert_eq!(messages.len(), 2);
        assert!(messages[0].content.contains("ONLY valid JSON"));
        assert!(messages[1].content.contains("hasErrors"));
        assert!(messages[1].content.contains("I has a apple."));
    }

    #[test]
    fn translation_prompt_includes_target_language() {
        let messages = build_translation_prompt("Good morning", "中文");
        assert!(messages[1]
            .content
            .contains("Translate the following text to 中文"));
        assert!(messages[1].content.contains("Good morning"));
    }

    #[test]
    fn summary_prompt_preserves_role_order() {
        let messages = build_summary_prompt(
            &[
                ("user".into(), "Hello".into()),
                ("assistant".into(), "Hi there".into()),
            ],
            None,
        );
        let prompt = &messages[1].content;
        assert!(prompt.contains("user: Hello\nassistant: Hi there"));
        assert!(prompt.contains("overallScore"));
        assert!(prompt.contains("scoreBreakdown"));
        assert!(prompt.contains("nextDrills"));
    }

    #[test]
    fn polish_prompt_requests_change_rationale() {
        let messages = build_polish_prompt("I want speak better.");
        assert!(messages[1].content.contains("\"polished\""));
        assert!(messages[1].content.contains("\"reason\""));
    }
}
