// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! MCP Prompt Templates
//!
//! Prompts are clinical note templates with variable substitution, per
//! [`spec/mcp.md`](../../../../../spec/mcp.md#mcp-prompts). They generate
//! *drafting instructions* for an LLM client to act on - they never read or
//! write repository data themselves, unlike resources and tools.

use serde::{Deserialize, Serialize};

/// A single prompt argument definition, as returned by `prompts/list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// MCP Prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

/// List prompts response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsList {
    pub prompts: Vec<Prompt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptContent {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: PromptContent,
}

/// `prompts/get` response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResult {
    pub description: String,
    pub messages: Vec<PromptMessage>,
}

/// Prompt handler for GitEHR's clinical note templates.
///
/// Stateless: prompts render generic drafting instructions from their
/// arguments only, so unlike `ResourceHandler`/`ToolHandler` there is no
/// repository path to hold.
pub struct PromptHandler;

impl PromptHandler {
    pub fn new() -> Self {
        Self
    }

    /// List all available prompts.
    pub fn list_prompts(&self) -> PromptsList {
        PromptsList {
            prompts: vec![
                Prompt {
                    name: "soap_note".to_string(),
                    description: "Generate a SOAP (Subjective, Objective, Assessment, Plan) note template".to_string(),
                    arguments: vec![
                        PromptArgument {
                            name: "chief_complaint".to_string(),
                            description: "Patient's chief complaint".to_string(),
                            required: true,
                        },
                        PromptArgument {
                            name: "specialty".to_string(),
                            description: "Medical specialty (e.g., cardiology, pediatrics)".to_string(),
                            required: false,
                        },
                    ],
                },
                Prompt {
                    name: "discharge_summary".to_string(),
                    description: "Generate a hospital discharge summary template".to_string(),
                    arguments: vec![
                        PromptArgument {
                            name: "diagnosis".to_string(),
                            description: "Primary diagnosis at discharge".to_string(),
                            required: true,
                        },
                        PromptArgument {
                            name: "admission_date".to_string(),
                            description: "Date of admission".to_string(),
                            required: false,
                        },
                        PromptArgument {
                            name: "discharge_date".to_string(),
                            description: "Date of discharge".to_string(),
                            required: false,
                        },
                    ],
                },
                Prompt {
                    name: "referral_letter".to_string(),
                    description: "Generate a specialist referral letter template".to_string(),
                    arguments: vec![
                        PromptArgument {
                            name: "specialty".to_string(),
                            description: "Specialty being referred to (e.g., cardiology, dermatology)".to_string(),
                            required: true,
                        },
                        PromptArgument {
                            name: "reason".to_string(),
                            description: "Reason for referral".to_string(),
                            required: true,
                        },
                        PromptArgument {
                            name: "urgency".to_string(),
                            description: "Urgency of the referral (e.g., routine, urgent, two-week-wait)".to_string(),
                            required: false,
                        },
                    ],
                },
                Prompt {
                    name: "consultation".to_string(),
                    description: "Generate a general consultation note template".to_string(),
                    arguments: vec![PromptArgument {
                        name: "chief_complaint".to_string(),
                        description: "Patient's chief complaint".to_string(),
                        required: true,
                    }],
                },
                Prompt {
                    name: "medication_review".to_string(),
                    description: "Generate a systematic medication review template".to_string(),
                    arguments: vec![PromptArgument {
                        name: "focus".to_string(),
                        description: "Optional focus for the review (e.g., polypharmacy, a specific drug class)".to_string(),
                        required: false,
                    }],
                },
            ],
        }
    }

    /// Render a prompt by name into drafting instructions for an LLM client.
    pub fn get_prompt(
        &self,
        name: &str,
        arguments: &serde_json::Value,
    ) -> anyhow::Result<GetPromptResult> {
        match name {
            "soap_note" => self.soap_note(arguments),
            "discharge_summary" => self.discharge_summary(arguments),
            "referral_letter" => self.referral_letter(arguments),
            "consultation" => self.consultation(arguments),
            "medication_review" => self.medication_review(arguments),
            _ => Err(anyhow::anyhow!("Unknown prompt: {}", name)),
        }
    }

    fn soap_note(&self, arguments: &serde_json::Value) -> anyhow::Result<GetPromptResult> {
        let chief_complaint = required_arg(arguments, "chief_complaint")?;
        let specialty = optional_arg(arguments, "specialty");

        let description = match &specialty {
            Some(s) => format!("SOAP note template for {chief_complaint} ({s})"),
            None => format!("SOAP note template for {chief_complaint}"),
        };
        let specialty_phrase = specialty.map(|s| format!("{s} ")).unwrap_or_default();

        let text = format!(
            "Generate a {specialty_phrase}SOAP note for a patient presenting with {chief_complaint}. Include:\n\n\
            **Subjective**: Symptom description, onset, character, duration, associated symptoms, risk factors\n\n\
            **Objective**: Vital signs, physical exam findings, relevant investigations\n\n\
            **Assessment**: Differential diagnosis, risk stratification where relevant\n\n\
            **Plan**: Investigations, treatment, disposition, follow-up"
        );

        text_prompt_result(description, text)
    }

    fn discharge_summary(&self, arguments: &serde_json::Value) -> anyhow::Result<GetPromptResult> {
        let diagnosis = required_arg(arguments, "diagnosis")?;
        let admission_date = optional_arg(arguments, "admission_date");
        let discharge_date = optional_arg(arguments, "discharge_date");

        let description = format!("Discharge summary template for {diagnosis}");
        let dates_line = match (admission_date, discharge_date) {
            (Some(a), Some(d)) => format!("Admitted {a}, discharged {d}.\n\n"),
            (Some(a), None) => format!("Admitted {a}.\n\n"),
            (None, Some(d)) => format!("Discharged {d}.\n\n"),
            (None, None) => String::new(),
        };

        let text = format!(
            "Generate a hospital discharge summary for a patient with a discharge diagnosis of {diagnosis}. {dates_line}Include:\n\n\
            **Diagnosis**: Primary and secondary diagnoses\n\n\
            **Hospital Course**: Summary of presentation, investigations, and treatment during admission\n\n\
            **Discharge Medications**: Full list with doses, including any changes from admission\n\n\
            **Follow-up Plan**: Outstanding investigations, appointments, and responsible clinicians\n\n\
            **Discharge Instructions**: Advice for the patient, including red-flag symptoms to seek help for"
        );

        text_prompt_result(description, text)
    }

    fn referral_letter(&self, arguments: &serde_json::Value) -> anyhow::Result<GetPromptResult> {
        let specialty = required_arg(arguments, "specialty")?;
        let reason = required_arg(arguments, "reason")?;
        let urgency = optional_arg(arguments, "urgency");

        let description = format!("Referral letter template to {specialty} for {reason}");
        let urgency_line = urgency
            .map(|u| format!(" This referral is {u}."))
            .unwrap_or_default();

        let text = format!(
            "Generate a referral letter to {specialty} for {reason}.{urgency_line} Include:\n\n\
            **Reason for Referral**: Clinical question being asked of the specialist\n\n\
            **History**: Relevant presenting complaint and history\n\n\
            **Examination and Investigations**: Relevant findings and results to date\n\n\
            **Current Management**: Treatment already tried\n\n\
            **Request**: What is being asked of the receiving specialist (opinion, procedure, shared care)"
        );

        text_prompt_result(description, text)
    }

    fn consultation(&self, arguments: &serde_json::Value) -> anyhow::Result<GetPromptResult> {
        let chief_complaint = required_arg(arguments, "chief_complaint")?;

        let description = format!("Consultation note template for {chief_complaint}");
        let text = format!(
            "Generate a general consultation note for a patient presenting with {chief_complaint}. Include:\n\n\
            **History**: Presenting complaint, relevant past medical, drug, and social history\n\n\
            **Examination**: Relevant examination findings\n\n\
            **Assessment**: Clinical impression and differential diagnosis\n\n\
            **Plan**: Investigations, treatment, safety-netting, and follow-up"
        );

        text_prompt_result(description, text)
    }

    fn medication_review(&self, arguments: &serde_json::Value) -> anyhow::Result<GetPromptResult> {
        let focus = optional_arg(arguments, "focus");

        let description = match &focus {
            Some(f) => format!("Medication review template (focus: {f})"),
            None => "Medication review template".to_string(),
        };
        let focus_line = focus
            .map(|f| format!(" with a particular focus on {f}"))
            .unwrap_or_default();

        let text = format!(
            "Generate a systematic medication review{focus_line}. For each current medication, consider:\n\n\
            **Indication**: Is there still a valid indication?\n\n\
            **Effectiveness**: Is it achieving its therapeutic goal?\n\n\
            **Safety**: Interactions, adverse effects, and monitoring requirements\n\n\
            **Adherence**: Any practical barriers to taking it as prescribed?\n\n\
            **Deprescribing**: Candidates for dose reduction or stopping, and how to do so safely"
        );

        text_prompt_result(description, text)
    }
}

impl Default for PromptHandler {
    fn default() -> Self {
        Self::new()
    }
}

fn required_arg(arguments: &serde_json::Value, name: &str) -> anyhow::Result<String> {
    arguments
        .get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("Missing '{}' argument", name))
}

fn optional_arg(arguments: &serde_json::Value, name: &str) -> Option<String> {
    arguments
        .get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn text_prompt_result(description: String, text: String) -> anyhow::Result<GetPromptResult> {
    Ok(GetPromptResult {
        description,
        messages: vec![PromptMessage {
            role: "user".to_string(),
            content: PromptContent::Text { text },
        }],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_prompts_returns_all_five() {
        let handler = PromptHandler::new();
        let list = handler.list_prompts();
        let names: Vec<_> = list.prompts.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "soap_note",
                "discharge_summary",
                "referral_letter",
                "consultation",
                "medication_review",
            ]
        );
    }

    #[test]
    fn test_unknown_prompt_is_an_error() {
        let handler = PromptHandler::new();
        let err = handler
            .get_prompt("nonesuch", &serde_json::json!({}))
            .unwrap_err();
        assert!(err.to_string().contains("Unknown prompt"));
    }

    #[test]
    fn test_soap_note_requires_chief_complaint() {
        let handler = PromptHandler::new();
        let err = handler
            .get_prompt("soap_note", &serde_json::json!({}))
            .unwrap_err();
        assert!(err.to_string().contains("chief_complaint"));
    }

    #[test]
    fn test_soap_note_includes_specialty_when_given() {
        let handler = PromptHandler::new();
        let result = handler
            .get_prompt(
                "soap_note",
                &serde_json::json!({"chief_complaint": "chest pain", "specialty": "cardiology"}),
            )
            .unwrap();
        assert!(result.description.contains("chest pain"));
        assert!(result.description.contains("cardiology"));
        let PromptContent::Text { text } = &result.messages[0].content;
        assert!(text.contains("cardiology SOAP note"));
        assert!(text.contains("chest pain"));
        assert_eq!(result.messages[0].role, "user");
    }

    #[test]
    fn test_soap_note_without_specialty() {
        let handler = PromptHandler::new();
        let result = handler
            .get_prompt(
                "soap_note",
                &serde_json::json!({"chief_complaint": "headache"}),
            )
            .unwrap();
        let PromptContent::Text { text } = &result.messages[0].content;
        assert!(text.contains("Generate a SOAP note"));
    }

    #[test]
    fn test_discharge_summary_requires_diagnosis() {
        let handler = PromptHandler::new();
        let err = handler
            .get_prompt("discharge_summary", &serde_json::json!({}))
            .unwrap_err();
        assert!(err.to_string().contains("diagnosis"));
    }

    #[test]
    fn test_discharge_summary_with_dates() {
        let handler = PromptHandler::new();
        let result = handler
            .get_prompt(
                "discharge_summary",
                &serde_json::json!({
                    "diagnosis": "community-acquired pneumonia",
                    "admission_date": "2026-09-01",
                    "discharge_date": "2026-09-05",
                }),
            )
            .unwrap();
        let PromptContent::Text { text } = &result.messages[0].content;
        assert!(text.contains("Admitted 2026-09-01, discharged 2026-09-05"));
    }

    #[test]
    fn test_referral_letter_requires_specialty_and_reason() {
        let handler = PromptHandler::new();
        assert!(
            handler
                .get_prompt("referral_letter", &serde_json::json!({"reason": "x"}))
                .is_err()
        );
        assert!(
            handler
                .get_prompt("referral_letter", &serde_json::json!({"specialty": "x"}))
                .is_err()
        );
    }

    #[test]
    fn test_referral_letter_with_urgency() {
        let handler = PromptHandler::new();
        let result = handler
            .get_prompt(
                "referral_letter",
                &serde_json::json!({
                    "specialty": "dermatology",
                    "reason": "a suspicious skin lesion",
                    "urgency": "two-week-wait",
                }),
            )
            .unwrap();
        let PromptContent::Text { text } = &result.messages[0].content;
        assert!(text.contains("This referral is two-week-wait."));
    }

    #[test]
    fn test_consultation_requires_chief_complaint() {
        let handler = PromptHandler::new();
        let err = handler
            .get_prompt("consultation", &serde_json::json!({}))
            .unwrap_err();
        assert!(err.to_string().contains("chief_complaint"));
    }

    #[test]
    fn test_medication_review_has_no_required_arguments() {
        let handler = PromptHandler::new();
        let result = handler
            .get_prompt("medication_review", &serde_json::json!({}))
            .unwrap();
        assert_eq!(result.description, "Medication review template");
    }

    #[test]
    fn test_medication_review_with_focus() {
        let handler = PromptHandler::new();
        let result = handler
            .get_prompt(
                "medication_review",
                &serde_json::json!({"focus": "polypharmacy"}),
            )
            .unwrap();
        assert!(result.description.contains("polypharmacy"));
        let PromptContent::Text { text } = &result.messages[0].content;
        assert!(text.contains("focus on polypharmacy"));
    }

    #[test]
    fn test_prompt_serialization_uses_camel_case_free_shape() {
        let prompt = Prompt {
            name: "test".to_string(),
            description: "Test".to_string(),
            arguments: vec![PromptArgument {
                name: "arg".to_string(),
                description: "An argument".to_string(),
                required: true,
            }],
        };
        let json = serde_json::to_value(&prompt).unwrap();
        assert_eq!(json["arguments"][0]["required"], true);
    }
}
