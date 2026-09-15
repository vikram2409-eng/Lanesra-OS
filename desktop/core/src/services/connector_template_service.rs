//! Connector Template Library: a small bundled catalog of curated,
//! hand-trimmed OpenAPI 3.x documents an admin can start `Import
//! connector` from instead of always pasting their own spec text (see
//! `desktop/core/src/connector_templates/*.json`). Every template's
//! request-body schemas are fully inline - no `$ref` - so they import
//! cleanly through the exact same `connector_service::preview_import`/
//! `import` path a hand-pasted spec already uses, with nothing degraded,
//! and every action stays eligible for the Integration Hub Tool Bridge
//! (`connector_tool_service.rs`) the moment an admin opts a connector in.
//!
//! Deliberately not every popular API: a template is only added here
//! once its real auth shape and body content-type are confirmed to fit
//! this codebase's existing `Connection` model - `Authorization: Bearer`
//! (or another of the 7 supported `auth_mode`s) plus a JSON request
//! body. Stripe/Twilio (`application/x-www-form-urlencoded` bodies) and
//! Google Gemini (an API key passed as a `?key=` query parameter, which
//! no existing `auth_mode` can inject) are deliberately not here yet for
//! exactly that reason - see Issue #130 for the fuller list of
//! candidates still pending a fit check.
//!
//! `list`/`get_spec` carry no admin gate of their own - this is static,
//! bundled content, the same trust level as `chat_service::
//! record_tools()`'s own hardcoded catalog. `connector_service::import`,
//! which the frontend calls immediately after picking a template,
//! already enforces `require_admin` exactly as it does for a hand-pasted
//! spec today.

use crate::domain::{AppError, AppResult};
use crate::models::integration::{ConnectorTemplateSpec, ConnectorTemplateSummary};

struct ConnectorTemplate {
    key: &'static str,
    name: &'static str,
    category: &'static str,
    description: &'static str,
    auth_mode: &'static str,
    setup_notes: &'static str,
    spec_format: &'static str,
    spec_text: &'static str,
}

const TEMPLATES: &[ConnectorTemplate] = &[
    ConnectorTemplate {
        key: "openai",
        name: "OpenAI",
        category: "ai_model",
        description: "Chat completions and model listing against the OpenAI API.",
        auth_mode: "bearer",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://api.openai.com/v1', auth_mode 'bearer', and an API key from platform.openai.com/api-keys as the secret.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/openai.json"),
    },
    ConnectorTemplate {
        key: "cohere",
        name: "Cohere",
        category: "ai_model",
        description: "Chat completions against the Cohere v2 API.",
        auth_mode: "bearer",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://api.cohere.com/v2', auth_mode 'bearer', and an API key from dashboard.cohere.com/api-keys as the secret.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/cohere.json"),
    },
    ConnectorTemplate {
        key: "slack",
        name: "Slack",
        category: "saas",
        description: "Post messages and list conversations via the Slack Web API.",
        auth_mode: "bearer",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://slack.com/api', auth_mode 'bearer', and a Bot User OAuth Token (starts with 'xoxb-') from your Slack app's OAuth & Permissions page as the secret - scope it to at least chat:write and channels:read.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/slack.json"),
    },
    ConnectorTemplate {
        key: "github",
        name: "GitHub",
        category: "saas",
        description: "Create and list issues on a repository via the GitHub REST API.",
        auth_mode: "bearer",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://api.github.com', auth_mode 'bearer', and a personal access token with 'issues' read/write permission as the secret.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/github.json"),
    },
    ConnectorTemplate {
        key: "sendgrid",
        name: "SendGrid",
        category: "saas",
        description: "Send transactional email via the Twilio SendGrid Mail Send API.",
        auth_mode: "bearer",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://api.sendgrid.com/v3', auth_mode 'bearer', and an API key (Settings -> API Keys, Mail Send permission) as the secret.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/sendgrid.json"),
    },
];

pub fn list() -> Vec<ConnectorTemplateSummary> {
    TEMPLATES
        .iter()
        .map(|t| ConnectorTemplateSummary {
            key: t.key.to_string(),
            name: t.name.to_string(),
            category: t.category.to_string(),
            description: t.description.to_string(),
            auth_mode: t.auth_mode.to_string(),
            setup_notes: t.setup_notes.to_string(),
        })
        .collect()
}

pub fn get_spec(key: &str) -> AppResult<ConnectorTemplateSpec> {
    TEMPLATES
        .iter()
        .find(|t| t.key == key)
        .map(|t| ConnectorTemplateSpec { spec_text: t.spec_text.to_string(), spec_format: t.spec_format.to_string() })
        .ok_or_else(|| AppError::NotFound("Connector template".into()))
}
