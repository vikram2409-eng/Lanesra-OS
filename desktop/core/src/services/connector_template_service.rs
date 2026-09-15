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
//! this codebase's `Connection` model - `Authorization: Bearer`/`Basic`/
//! a query-param key, plus a JSON or (Phase 2) flat form-urlencoded
//! request body. Phase 2 added Stripe/Twilio (form-urlencoded bodies,
//! see `connector_service::is_flat_object_schema`) and Google Gemini
//! (the new `"query_param"` auth_mode) once the engine actually
//! supported them, rather than curating a template the product
//! couldn't yet call correctly.
//!
//! `oauth2_authorization_code`/`oauth2_client_credentials` are, in this
//! codebase, just `bearer` under another name - no token exchange or
//! refresh loop exists anywhere (see `connection_service::apply_auth`).
//! A template only belongs in the `"saas"`/`"ai_model"` categories here
//! when the vendor also offers a **long-lived static token** (a Private
//! App token, Personal/Programmatic Access Token, or Internal
//! Integration Secret) - a service whose only option is a genuinely
//! short-lived OAuth2 access token with mandatory refresh (BigQuery,
//! Salesforce, Google Sheets - all confirmed, not assumed) has no
//! template here and won't until that refresh loop is real
//! infrastructure, not a curation problem.
//!
//! `list`/`get_spec` carry no admin gate of their own - this is static,
//! bundled content, the same trust level as `chat_service::
//! record_tools()`'s own hardcoded catalog. `connector_service::import`,
//! which the frontend calls immediately after picking a template,
//! already enforces `require_admin` exactly as it does for a hand-pasted
//! spec today.
//!
//! Phase 3 added six more SaaS templates (Zendesk, Jira Cloud,
//! PagerDuty, Datadog, Mailchimp, Shopify) and, separately, real MySQL/
//! SQL Server `Connection` types (see `mysql_service.rs`/
//! `sqlserver_service.rs` - a different kind of connectivity than the
//! OpenAPI templates here, no template registration needed). Two real
//! findings shaped this batch: **Linear is GraphQL-only** (confirmed
//! against developers.linear.app - no REST resource endpoints exist),
//! a real architectural mismatch with this OpenAPI/REST-parsing
//! gallery, not a curation gap - excluded, not silently dropped.
//! **Datadog needs a second credential** (`DD-APPLICATION-KEY`, in
//! addition to `DD-API-KEY`) for most endpoints, confirmed against
//! Datadog's own published OpenAPI specs - this `Connection` model has
//! exactly one secret slot, so the Datadog template is deliberately
//! scoped to the two operations confirmed to need only the API key
//! (submit event, submit metrics), not a general Datadog client.

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
    ConnectorTemplate {
        key: "snowflake",
        name: "Snowflake",
        category: "data_warehouse",
        description: "Submit a SQL statement via the Snowflake SQL API v2.",
        auth_mode: "bearer",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://YOUR_ACCOUNT.snowflakecomputing.com' (replace with your real account identifier), auth_mode 'bearer', and a Programmatic Access Token (Snowsight -> Governance & security -> Users & roles, or ALTER USER ... ADD PROGRAMMATIC ACCESS TOKEN) as the secret. PATs expire (15-365 days, admin-configurable) - plan to rotate the Connection's secret before it does.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/snowflake.json"),
    },
    ConnectorTemplate {
        key: "databricks",
        name: "Databricks",
        category: "data_warehouse",
        description: "List SQL warehouses and execute a SQL statement via the Databricks Statement Execution API.",
        auth_mode: "bearer",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://YOUR_WORKSPACE.cloud.databricks.com' (replace with your real workspace hostname), auth_mode 'bearer', and a Personal Access Token (workspace User Settings -> Developer -> Access tokens) as the secret. Static until revoked, but Databricks auto-revokes an unused token after 90 days.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/databricks.json"),
    },
    ConnectorTemplate {
        key: "hubspot",
        name: "HubSpot",
        category: "saas",
        description: "List and create contacts via the HubSpot CRM API.",
        auth_mode: "bearer",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://api.hubapi.com', auth_mode 'bearer', and a Private App access token (Settings -> Integrations -> Private Apps, with crm.objects.contacts scope) as the secret - long-lived, no OAuth consent flow needed.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/hubspot.json"),
    },
    ConnectorTemplate {
        key: "airtable",
        name: "Airtable",
        category: "saas",
        description: "List and create records in an Airtable base/table.",
        auth_mode: "bearer",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://api.airtable.com', auth_mode 'bearer', and a Personal Access Token (airtable.com/create/tokens, scoped to the base you'll use) as the secret.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/airtable.json"),
    },
    ConnectorTemplate {
        key: "notion",
        name: "Notion",
        category: "saas",
        description: "Retrieve a database and create a page via the Notion API.",
        auth_mode: "bearer",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://api.notion.com', auth_mode 'bearer', and an Internal Integration Secret (notion.so/my-integrations, shared with the target database) as the secret. Every action also requires a Notion-Version header - pass the current version (e.g. '2026-03-11') as that header parameter on every call, since this codebase has no way to bake in a fixed header value automatically.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/notion.json"),
    },
    ConnectorTemplate {
        key: "stripe",
        name: "Stripe",
        category: "saas",
        description: "List and create customers via the Stripe API.",
        auth_mode: "basic",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://api.stripe.com', auth_mode 'basic', and the secret stored as '<your secret key>:' (your Stripe secret key, e.g. sk_live_..., followed by a colon with nothing after it - Stripe uses the key as the Basic auth username with a blank password).",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/stripe.json"),
    },
    ConnectorTemplate {
        key: "twilio",
        name: "Twilio",
        category: "saas",
        description: "Send an SMS message via the Twilio Messaging API.",
        auth_mode: "basic",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://api.twilio.com', auth_mode 'basic', and the secret stored as '<Account SID>:<Auth Token>' (both from your Twilio Console). Note the Account SID is also needed as the action's own AccountSid path parameter on every call.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/twilio.json"),
    },
    ConnectorTemplate {
        key: "gemini",
        name: "Google Gemini",
        category: "ai_model",
        description: "Generate content via the Google Gemini API.",
        auth_mode: "query_param",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://generativelanguage.googleapis.com', auth_mode 'query_param', and the secret stored as 'key:<your Gemini API key from aistudio.google.com/apikey>' - injected automatically as a ?key= query parameter on every call, never passed as an action parameter.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/gemini.json"),
    },
    ConnectorTemplate {
        key: "zendesk",
        name: "Zendesk",
        category: "saas",
        description: "Create and list support tickets via the Zendesk REST API.",
        auth_mode: "basic",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://YOUR_SUBDOMAIN.zendesk.com' (replace with your real subdomain), auth_mode 'basic', and the secret stored as '<your email>/token:<API token>' (Admin Center -> Apps and integrations -> APIs -> Zendesk API). Zendesk is nudging accounts toward OAuth, but this static API token option remains live and documented.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/zendesk.json"),
    },
    ConnectorTemplate {
        key: "jira",
        name: "Jira Cloud",
        category: "saas",
        description: "Create an issue and search issues via JQL on Jira Cloud.",
        auth_mode: "basic",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://YOUR_DOMAIN.atlassian.net' (replace with your real site), auth_mode 'basic', and the secret stored as '<your Atlassian account email>:<API token>' (id.atlassian.com/manage/api-tokens).",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/jira.json"),
    },
    ConnectorTemplate {
        key: "pagerduty",
        name: "PagerDuty",
        category: "saas",
        description: "Create and list incidents via the PagerDuty REST API v2.",
        auth_mode: "custom_header",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://api.pagerduty.com', auth_mode 'custom_header', and the secret stored as 'Authorization:Token token=<your REST API key>' (from a PagerDuty user's or the account's own API access settings). createIncident's own 'From' header parameter must be a valid PagerDuty user's email on every call.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/pagerduty.json"),
    },
    ConnectorTemplate {
        key: "datadog",
        name: "Datadog",
        category: "saas",
        description: "Submit an event or metrics to Datadog.",
        auth_mode: "custom_header",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://api.datadoghq.com' (or your account's regional site, e.g. 'https://api.us5.datadoghq.com'), auth_mode 'custom_header', and the secret stored as 'DD-API-KEY:<your API key>' (Organization Settings -> API Keys). Scoped to submitEvent/submitMetrics only - most other Datadog endpoints also require a separate Application key this Connection model has no second secret slot for.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/datadog.json"),
    },
    ConnectorTemplate {
        key: "mailchimp",
        name: "Mailchimp",
        category: "saas",
        description: "List and add members to a Mailchimp audience.",
        auth_mode: "bearer",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://YOUR_DC.api.mailchimp.com/3.0' (replace YOUR_DC with the data-center suffix on your own API key, e.g. an API key ending '-us6' means 'us6'), auth_mode 'bearer', and your API key (Account -> Extras -> API keys) as the secret.",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/mailchimp.json"),
    },
    ConnectorTemplate {
        key: "shopify",
        name: "Shopify",
        category: "saas",
        description: "List and create products via the Shopify Admin API.",
        auth_mode: "custom_header",
        setup_notes: "Create a Connection with connection_type 'rest', base_url 'https://YOUR_SHOP.myshopify.com' (replace with your own *.myshopify.com subdomain), auth_mode 'custom_header', and the secret stored as 'X-Shopify-Access-Token:<your custom app's Admin API access token>' (Settings -> Apps and sales channels -> Develop apps, in your own store admin).",
        spec_format: "json",
        spec_text: include_str!("../connector_templates/shopify.json"),
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
