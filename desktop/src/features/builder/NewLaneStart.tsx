/**
 * Guided start for a new lane (WP-0722, REQ-LANE-024): name, object and
 * start point, then into the builder with the choice applied.
 *
 * An admin creates a lane from the thing it is about — the object —
 * instead of starting blank and picking the object second. The object
 * arrives pre-filled when the start comes from an object's page; the
 * blank-lane Object card stays as the fallback for lanes started from
 * the Lanes section. The start point is blank, describe it with the
 * Assistant (the WP-0715 draft — still always a draft, never published),
 * or a template when one exists; there are no templates yet (WP-0717,
 * scope decision pending), so that choice is offered disabled with the
 * reason, not hidden. No new mandatory screens inside the builder: this
 * dialog is the whole guided start, and the builder it lands in is
 * unchanged.
 */
import { useState } from "react";

import type { CustomObjectDefinition } from "../../lib/types";

export type NewLaneStartPoint = "blank" | "describe" | "template";

export interface NewLaneStartChoice {
  /** Empty input falls back to "Untitled lane" (the blank definition's name). */
  name: string;
  objectKey: string;
  start: NewLaneStartPoint;
  /** Only for `describe`: the process in the person's own words. */
  description: string;
}

export function NewLaneStart({
  initialObjectKey,
  objects,
  busy,
  error,
  onCancel,
  onCreate,
}: {
  /** Pre-filled when the start comes from an object's page. */
  initialObjectKey?: string | null;
  objects: Pick<CustomObjectDefinition, "key" | "plural_label">[];
  /** A draft is being fetched (describe path); the buttons wait. */
  busy?: boolean;
  /** A describe failure to show inline, keeping the dialog open. */
  error?: string | null;
  onCancel: () => void;
  onCreate: (choice: NewLaneStartChoice) => void;
}) {
  const [name, setName] = useState("");
  const [objectKey, setObjectKey] = useState(
    initialObjectKey ?? objects[0]?.key ?? "",
  );
  const [start, setStart] = useState<NewLaneStartPoint>("blank");
  const [description, setDescription] = useState("");
  const [localError, setLocalError] = useState<string | null>(null);

  function submit() {
    if (!objectKey) {
      setLocalError("Pick the object this lane runs on.");
      return;
    }
    if (start === "describe" && !description.trim()) {
      setLocalError("Describe the process first — an empty description drafts nothing.");
      return;
    }
    setLocalError(null);
    onCreate({
      name: name.trim() || "Untitled lane",
      objectKey,
      start,
      description: description.trim(),
    });
  }

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0, 0, 0, 0.45)",
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "center",
        padding: "48px 16px",
        overflowY: "auto",
        zIndex: 50,
      }}
    >
      <div className="card" style={{ width: "100%", maxWidth: 560, padding: 24 }}>
        <h2 style={{ marginTop: 0 }}>New lane</h2>
        <p style={{ color: "var(--text-muted)" }}>
          A lane is a guided, step-by-step process for one record. Name it,
          pick the object it runs on, and choose how to start — the builder
          opens with those choices applied.
        </p>

        <div style={{ marginBottom: 16 }}>
          <label htmlFor="new-lane-name" style={{ display: "block", marginBottom: 4 }}>
            Lane name
          </label>
          <input
            id="new-lane-name"
            aria-label="Lane name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Untitled lane"
            style={{ width: "100%" }}
          />
        </div>

        <div style={{ marginBottom: 16 }}>
          <label htmlFor="new-lane-object" style={{ display: "block", marginBottom: 4 }}>
            Runs on
          </label>
          {objects.length === 0 ? (
            <p style={{ color: "var(--text-muted)" }}>
              No objects yet — create a Custom Object first, then come back to
              start a lane on it.
            </p>
          ) : (
            <select
              id="new-lane-object"
              aria-label="Lane object"
              value={objectKey}
              onChange={(e) => setObjectKey(e.target.value)}
              style={{ width: "100%" }}
            >
              {objects.map((o) => (
                <option key={o.key} value={o.key}>
                  {o.plural_label}
                </option>
              ))}
            </select>
          )}
        </div>

        <fieldset style={{ marginBottom: 16, border: "1px solid var(--border)", borderRadius: 8, padding: 12 }}>
          <legend>How to start</legend>
          <label style={{ display: "block", marginBottom: 8 }}>
            <input
              type="radio"
              name="new-lane-start"
              value="blank"
              checked={start === "blank"}
              onChange={() => setStart("blank")}
            />{" "}
            Start blank — an empty lane on this object
          </label>
          <label style={{ display: "block", marginBottom: 8 }}>
            <input
              type="radio"
              name="new-lane-start"
              value="describe"
              checked={start === "describe"}
              onChange={() => setStart("describe")}
            />{" "}
            Describe it — the Admin Assistant drafts the lane from your words
          </label>
          <label style={{ display: "block", color: "var(--text-muted)" }}>
            <input type="radio" name="new-lane-start" value="template" disabled />{" "}
            Start from a template — no templates yet (templates arrive with
            WP-0717)
          </label>
        </fieldset>

        {start === "describe" && (
          <div style={{ marginBottom: 16 }}>
            <label htmlFor="new-lane-description" style={{ display: "block", marginBottom: 4 }}>
              Describe the process
            </label>
            <textarea
              id="new-lane-description"
              aria-label="Process description"
              rows={4}
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Receive a purchase order at the warehouse. Verify the delivery against the order. Record what arrived…"
              style={{ width: "100%" }}
            />
            <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
              The draft is always a draft, never published — you review it in
              the builder and publish it yourself.
            </p>
          </div>
        )}

        {(localError || error) && (
          <p role="alert" style={{ color: "var(--destructive)" }}>
            {localError ?? error}
          </p>
        )}

        <div style={{ display: "flex", justifyContent: "flex-end", gap: 8 }}>
          <button className="btn" onClick={onCancel} disabled={busy}>
            Cancel
          </button>
          <button
            className="btn btn-primary"
            onClick={submit}
            disabled={busy || objects.length === 0}
            title={objects.length === 0 ? "No objects yet — create a Custom Object first" : undefined}
          >
            {busy ? "Drafting…" : "Create lane"}
          </button>
        </div>
      </div>
    </div>
  );
}
