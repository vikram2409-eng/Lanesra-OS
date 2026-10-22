//! Argument parsing only - see `lib.rs::run` for what each command does.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "lanesra",
    version,
    about = "Scriptable access to a Lanesra OS Team Workspace server's REST API"
)]
pub struct Cli {
    /// Team Workspace server base URL, e.g. http://localhost:8787
    #[arg(long, env = "LANESRA_API_URL")]
    pub base_url: String,

    /// API client key ("{client_id}.{secret}") issued from Admin -> Integration Hub -> API Access
    #[arg(long, env = "LANESRA_API_KEY")]
    pub api_key: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// This API client's rate limit and scopes (GET /api/v1/limits)
    Limits,
    /// List and inspect this workspace's objects
    Objects {
        #[command(subcommand)]
        command: ObjectsCommand,
    },
    /// List, read, and write records through the generic object dispatcher
    Records {
        #[command(subcommand)]
        command: RecordsCommand,
    },
    /// Manage webhooks (needs the webhooks.manage scope)
    Webhooks {
        #[command(subcommand)]
        command: WebhooksCommand,
    },
    /// Find records by text across every readable object (GET /api/v1/search; needs the search.read scope)
    Search {
        /// The text to look for, e.g. a name, number, email or phone
        text: String,
        /// Only search this object, e.g. Company or a custom object's key
        #[arg(long)]
        object: Option<String>,
        #[arg(long)]
        page: Option<i64>,
        #[arg(long = "page-size")]
        page_size: Option<i64>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ObjectsCommand {
    /// List every built-in and custom object this workspace exposes
    List,
    /// Get an object's label and its custom field definitions
    Metadata { object_key: String },
}

#[derive(Subcommand, Debug)]
pub enum RecordsCommand {
    /// List records for an object, paginated
    List {
        object_key: String,
        /// A JSON object of exact-match field/value pairs, e.g. '{"status":"Prospect"}'
        #[arg(long)]
        filter: Option<String>,
        /// Comma-separated field names; prefix a field with '-' for descending
        #[arg(long)]
        sort: Option<String>,
        #[arg(long)]
        page: Option<i64>,
        #[arg(long = "page-size")]
        page_size: Option<i64>,
    },
    /// Get a single record by id
    Get { object_key: String, id: String },
    /// Create a record. --data is the record's fields as a JSON object
    Create {
        object_key: String,
        #[arg(long)]
        data: String,
    },
    /// Update a record's fields by id. --data holds only the fields being changed
    Update {
        object_key: String,
        id: String,
        #[arg(long)]
        data: String,
    },
    /// Archive (soft-delete) a record by id
    Archive { object_key: String, id: String },
    /// Export every record of an object (needs bulk.read); prints NDJSON or CSV
    Export {
        object_key: String,
        /// ndjson (default) or csv
        #[arg(long, default_value = "ndjson")]
        format: String,
    },
    /// Apply up to 1000 create/update/archive operations at once (needs bulk.write).
    /// --data is a JSON array of {"op","id"?,"data"?}, or @path to read it from a file
    Bulk {
        object_key: String,
        #[arg(long)]
        data: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum WebhooksCommand {
    /// List this workspace's webhooks (never shows a signing secret)
    List,
    /// Get one webhook
    Get { id: String },
    /// Create a webhook; prints its signing secret, the only time it is shown.
    /// --data is {"name","connection_id","event_types",...}
    Create {
        #[arg(long)]
        data: String,
    },
    /// Replace a webhook's settings; --data as for create
    Update {
        id: String,
        #[arg(long)]
        data: String,
    },
    /// Delete a webhook
    Delete { id: String },
    /// Stop deliveries without deleting the webhook
    Pause { id: String },
    /// Resume deliveries
    Reactivate { id: String },
    /// Replace the signing secret; prints the new one, once
    RotateSecret { id: String },
    /// Send a signed test delivery
    Test { id: String },
    /// Page through delivery attempts, newest first
    Deliveries {
        id: String,
        #[arg(long)]
        page: Option<i64>,
        #[arg(long = "page-size")]
        page_size: Option<i64>,
    },
}
