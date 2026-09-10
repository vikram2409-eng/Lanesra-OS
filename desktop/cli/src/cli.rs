//! Argument parsing only - see `lib.rs::run` for what each command does.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "lanesra", version, about = "Scriptable access to a Lanesra OS Team Workspace server's REST API")]
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
}
