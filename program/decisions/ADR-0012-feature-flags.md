# ADR-0012: Feature flags: code-defined, workspace-scoped, short-lived

- Status: Proposed. Accepted when the owner merges the WP-0009 pull request.
- Date: 2026-10-16
- Work package: WP-0009 (implemented by WP-0010)
- Requirements: REQ-PRE-006, REQ-PRE-007, REQ-PRE-015 (editions)
- Constrains: WP-0010, and every work package that ships a user-visible capability before it is finished

## Context

The Playground build lasts months and merges to `main` continuously, while `main` keeps shipping desktop releases, Team Workspace and the live online demo. Nothing in the code can hide unfinished work today: there is no flag table, setting or helper. Desktop and Team Workspace must release a feature together, and the online demo must show it **in the same release and never earlier** (owner decision, ADR-0013 #3).

## Decision

### 1. Flags are defined in code

All flags are listed in one Rust registry, `lanesra_core::feature_flags::REGISTRY`. Each entry has:

| Field | Meaning |
|---|---|
| `key` | `area.feature`, lower snake case, for example `playground.compare` |
| `description` | One line, shown on the admin screen |
| `work_package` | The work package that introduced it |
| `remove_by` | The work package or release by which the flag and its old code path are deleted |
| `default` | Per edition: `desktop`, `workspace`, each `on` or `off`. The demo has no default of its own: it shows a feature exactly when the release that turns the feature on for desktop and Team Workspace is published |
| `stage` | `building` (hidden; only a developer setting can turn it on), `preview` (admins may turn it on), or `ga` (on by default; the flag now only exists to be removed) |

A flag that is not in the registry does not exist. Reading an unknown key is a compile-time error, because each flag is a constant.

### 2. Overrides are per workspace

A new table, `feature_flag_overrides`, has the columns `workspace_id`, `key`, `enabled`, `updated_by_principal_id` and `updated_at`, following ADR-0010. A row overrides the edition default for that workspace. Every change is audited.

### 3. One way to check a flag

- **Core:** `feature_flags::is_enabled(conn, workspace_id, KEY) -> bool`.
- **Server and Tauri:** one command, `list_feature_flags`, returns the resolved flags for the current workspace. Routes and commands behind a flag answer `404 feature_disabled` when it is off, so a flag hides the API as well as the screen.
- **Frontend:** a `useFeature(KEY)` hook. Menus, routes and buttons for a disabled feature are not rendered at all, not merely greyed out.
- **Online demo:** reads a generated copy of the registry, published with each release (`app.js` data, built by a script). A feature appears in the demo only when it is on for desktop and Team Workspace in a published release. On `main` between releases the demo keeps the last release's state, so the live demo never shows unreleased work.

### 4. What flags may not do

- A flag never gates a **migration**: migrations always run, and new tables are simply unused while a flag is off.
- A flag never gates a **security or correctness fix**, such as the gap-audit defects I-12 to I-18. Those ship on.
- A flag never gates **audit or policy enforcement**. Firewall, approvals and audit apply whether or not a feature is flagged.
- Flags are not for per-customer licensing or plans. Those need their own model.

### 5. Lifecycle and tests

1. A work package adds the flag as `building` with every default `off`. The demo version is built at the same time (REQ-PRE-015) but stays hidden until release.
2. At its gate the flag becomes `preview`, then `ga`, with defaults turned `on` per edition.
3. **Removal:** within two releases of `ga`, the flag and the old code path are deleted, as the `remove_by` field requires. CI warns when a `ga` flag passes its `remove_by` release.
4. **Tests:** every flagged behaviour is tested in both states, and the API test asserts `404 feature_disabled` when the flag is off.
5. **Release builds** ignore the developer setting, so a `building` flag can never be turned on in a shipped desktop app.

## Example

```rust
pub const PLAYGROUND_COMPARE: Flag = Flag {
    key: "playground.compare",
    description: "Compare two runs side by side in the Playground",
    work_package: "WP-0209",
    remove_by: "WP-0223",
    default: Defaults { desktop: Off, workspace: Off }, // the demo follows the release
    stage: Stage::Building,
};
```

## Consequences

- `main` stays releasable while the Playground is half built, and each edition can be ahead or behind as needed.
- The registry and the `remove_by` field stop flags from piling up.
- Each flag adds two code paths until it is removed, which is why both must be tested and why flags are kept short-lived.
