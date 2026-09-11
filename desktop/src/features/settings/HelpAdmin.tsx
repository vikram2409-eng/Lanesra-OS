import { useEffect, useMemo, useState } from "react";

import { HELP_CATEGORIES, type HelpTopic } from "../../lib/helpContent";

// Product Help - detailed admin user guides, static and shipped with the
// app (see helpContent.ts for why this is duplicated from, not imported
// from, the website's own /help section). A left-hand category/topic list
// plus a content pane, the same list+detail shape every other admin screen
// on this panel already uses, with a plain client-side search box and
// "on this page" anchors for a long article - no backend, no new table.
//
// `initialTopicSlug` lets another admin screen deep-link straight into a
// specific article (see AiAgentsAdmin.tsx's own "📖 Help" link) instead of
// landing an admin on the bare hub every time.
export function HelpAdmin({ initialTopicSlug }: { initialTopicSlug?: string | null }) {
  const allTopics = useMemo(
    () => HELP_CATEGORIES.flatMap((c) => c.topics.map((t) => ({ ...t, catKey: c.key, catLabel: c.label }))),
    [],
  );

  const [selectedSlug, setSelectedSlug] = useState<string | null>(initialTopicSlug ?? allTopics[0]?.slug ?? null);
  const [query, setQuery] = useState("");

  // A new deep-link (a fresh click of the same "📖 Help" button with a
  // different slug) should re-select even if this component never
  // unmounted in between - it only unmounts when leaving the Help tab
  // entirely, which the caller already handles by remounting via `tab`.
  useEffect(() => {
    if (initialTopicSlug) setSelectedSlug(initialTopicSlug);
  }, [initialTopicSlug]);

  const q = query.trim().toLowerCase();
  const matches = (t: HelpTopic) => !q || `${t.title} ${t.summary}`.toLowerCase().includes(q);

  const selected = allTopics.find((t) => t.slug === selectedSlug) ?? null;
  const siblingTopics = selected ? allTopics.filter((t) => t.catKey === selected.catKey && t.slug !== selected.slug) : [];

  const tocId = (heading: string) =>
    heading
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/(^-|-$)/g, "");

  return (
    <div className="help-admin-layout">
      <div>
        <input
          type="text"
          placeholder="Search help..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          style={{ width: "100%", marginBottom: 10 }}
        />
        <nav className="help-admin-nav">
          {HELP_CATEGORIES.map((cat) => {
            const visibleTopics = cat.topics.filter(matches);
            if (visibleTopics.length === 0) return null;
            return (
              <div key={cat.key}>
                <div className="help-admin-cat-label">
                  <span>{cat.icon}</span>
                  {cat.label}
                </div>
                {visibleTopics.map((t) => (
                  <button
                    key={t.slug}
                    className={`help-admin-topic-btn${t.slug === selectedSlug ? " active" : ""}`}
                    onClick={() => setSelectedSlug(t.slug)}
                  >
                    {t.title}
                  </button>
                ))}
              </div>
            );
          })}
          {allTopics.filter(matches).length === 0 && (
            <p style={{ color: "var(--text-muted)", fontSize: 13 }}>No help articles match "{query}".</p>
          )}
        </nav>
      </div>

      <div>
        {!selected && <p style={{ color: "var(--text-muted)" }}>Pick a topic from the list.</p>}
        {selected && (
          <div style={{ display: "grid", gridTemplateColumns: "1fr 180px", gap: 20, alignItems: "start" }}>
            <div>
              <h3 style={{ marginTop: 0, marginBottom: 4 }}>{selected.title}</h3>
              <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>{selected.summary}</p>
              <div
                className="help-admin-article"
                dangerouslySetInnerHTML={{
                  __html: selected.sections
                    .map((s) => `<h4 id="${tocId(s.heading)}">${s.heading}</h4>${s.bodyHtml}`)
                    .join(""),
                }}
              />
              {siblingTopics.length > 0 && (
                <div className="help-admin-related">
                  {siblingTopics.map((t) => (
                    <button key={t.slug} className="link-button" onClick={() => setSelectedSlug(t.slug)}>
                      {t.title} →
                    </button>
                  ))}
                </div>
              )}
            </div>
            <div className="card help-admin-toc">
              <div>On this page</div>
              {selected.sections.map((s) => (
                <a key={s.heading} href={`#${tocId(s.heading)}`}>
                  {s.heading}
                </a>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
