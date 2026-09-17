import { useEffect, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { ConfirmVoicePlanInput, VoiceCommandOutcome } from "../../lib/types";
import { useVoiceContext } from "./VoiceContext";

// The browser's own Speech Recognition API - not typed in lib.dom.d.ts, and
// vendor-prefixed in every browser that ships it today. This is the one
// real (not simulated) STT/TTS adapter PR 1 ships (spec §18's Speech
// Provider Abstraction: real when the browser/OS supports it, with the
// "Type instead" field as a first-class, always-present fallback - never a
// hidden one - so the whole feature works with no microphone at all).
type SpeechRecognitionLike = {
  lang: string;
  continuous: boolean;
  interimResults: boolean;
  start: () => void;
  stop: () => void;
  onresult: ((e: any) => void) | null;
  onerror: ((e: any) => void) | null;
  onend: (() => void) | null;
};
function getSpeechRecognitionCtor(): (new () => SpeechRecognitionLike) | null {
  const w = window as any;
  return w.SpeechRecognition || w.webkitSpeechRecognition || null;
}

function speak(text: string) {
  try {
    if ("speechSynthesis" in window) {
      window.speechSynthesis.speak(new SpeechSynthesisUtterance(text));
    }
  } catch {
    // TTS is a nice-to-have, never worth failing a command over.
  }
}

/**
 * Voice-First Mode, PR 1: the always-mounted mic button (spec §2's UX
 * flow, §3's session states, §17's "unlocked ≠ listening" visibility
 * requirement). Mounted once in `AppShell`'s topbar, next to the
 * notification bell - never inside a per-section view, so it survives
 * every navigation.
 */
export function VoiceModeButton() {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const queryClient = useQueryClient();
  const { objectKey, recordId } = useVoiceContext();

  const settings = useQuery({ queryKey: ["voiceSettings"], queryFn: () => api.getVoiceSettings(), enabled: open });
  const session = useQuery({ queryKey: ["voiceSession"], queryFn: () => api.getCurrentVoiceSession(), enabled: open, refetchInterval: open ? 30_000 : false });

  useEffect(() => {
    function onOutsideClick(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) setOpen(false);
    }
    document.addEventListener("mousedown", onOutsideClick);
    return () => document.removeEventListener("mousedown", onOutsideClick);
  }, []);

  const activeSession = session.data && session.data.state !== "expired" ? session.data : null;

  // Keep the session's own context in sync with wherever the user actually
  // is (spec §7) - only while unlocked and only when it actually changed,
  // so opening/closing the panel never spams the backend.
  const lastSyncedContext = useRef<string>("");
  useEffect(() => {
    if (!activeSession) return;
    const key = `${objectKey ?? ""}:${recordId ?? ""}`;
    if (key === lastSyncedContext.current) return;
    lastSyncedContext.current = key;
    api.setVoiceSessionContext(activeSession.id, objectKey, recordId ?? null).catch(() => {});
  }, [activeSession, objectKey, recordId]);

  return (
    <div ref={containerRef} style={{ position: "relative" }}>
      <button
        className="btn"
        onClick={() => setOpen((v) => !v)}
        title="Voice Mode"
        style={{ position: "relative" }}
      >
        🎙️
      </button>
      {open && (
        <div
          className="card"
          style={{ position: "absolute", right: 0, top: "calc(100% + 4px)", width: 380, maxHeight: 520, overflowY: "auto", zIndex: 30, boxShadow: "0 4px 16px rgba(0,0,0,0.15)" }}
        >
          {settings.isLoading || session.isLoading ? (
            <p style={{ fontSize: 13 }}>Loading Voice Mode...</p>
          ) : !settings.data?.pin_set ? (
            <NoPinYet />
          ) : !activeSession ? (
            <UnlockPanel onUnlocked={() => queryClient.invalidateQueries({ queryKey: ["voiceSession"] })} />
          ) : (
            <UnlockedPanel session={activeSession} autoSpeak={settings.data.auto_speak_confirmations} onClose={() => setOpen(false)} />
          )}
        </div>
      )}
    </div>
  );
}

function NoPinYet() {
  return (
    <div>
      <h4 style={{ marginTop: 0 }}>Voice Mode</h4>
      <p style={{ fontSize: 13, color: "var(--text-muted)" }}>
        Set a 4-digit Voice PIN under Personal Settings → Voice PIN to start using Voice Mode.
      </p>
    </div>
  );
}

function UnlockPanel({ onUnlocked }: { onUnlocked: () => void }) {
  const [pin, setPin] = useState("");
  const unlock = useMutation({
    mutationFn: () => api.unlockVoiceSession(pin),
    onSuccess: onUnlocked,
  });
  return (
    <div>
      <h4 style={{ marginTop: 0 }}>Voice Mode is locked</h4>
      <p style={{ fontSize: 13, color: "var(--text-muted)" }}>Enter your Voice PIN to unlock.</p>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          unlock.mutate();
        }}
        style={{ display: "flex", gap: 8 }}
      >
        <input
          type="password"
          inputMode="numeric"
          maxLength={4}
          value={pin}
          onChange={(e) => setPin(e.target.value.replace(/\D/g, ""))}
          placeholder="••••"
          style={{ width: 80, textAlign: "center", fontSize: 18, letterSpacing: 4 }}
          autoFocus
        />
        <button className="btn btn-primary" disabled={pin.length !== 4 || unlock.isPending}>
          {unlock.isPending ? "Unlocking..." : "Unlock"}
        </button>
      </form>
      {unlock.isError && (
        <div className="error-banner" style={{ marginTop: 8, fontSize: 13 }}>
          {unlock.error instanceof ApiError ? unlock.error.message : "Could not unlock Voice Mode"}
        </div>
      )}
    </div>
  );
}

function riskBadgeClass(risk: string) {
  if (risk === "high" || risk === "critical") return "badge-danger";
  if (risk === "medium") return "badge-warning";
  return "";
}

function UnlockedPanel({ session, autoSpeak, onClose }: { session: { id: string; context_object_key: string | null; context_record_id: string | null }; autoSpeak: boolean; onClose: () => void }) {
  const queryClient = useQueryClient();
  const [listening, setListening] = useState(false);
  const [transcript, setTranscript] = useState("");
  const [typedText, setTypedText] = useState("");
  const [outcome, setOutcome] = useState<VoiceCommandOutcome | null>(null);
  const recognitionRef = useRef<SpeechRecognitionLike | null>(null);
  const speechSupported = !!getSpeechRecognitionCtor();

  const activity = useQuery({ queryKey: ["myVoiceActivity"], queryFn: () => api.listMyVoiceActivity(5) });

  const submit = useMutation({
    mutationFn: (args: { text: string; confidence: number | null }) => api.submitVoiceCommand(session.id, args.text, "en-US", args.confidence),
    onSuccess: (result) => {
      setOutcome(result);
      setTranscript("");
      setTypedText("");
      queryClient.invalidateQueries({ queryKey: ["myVoiceActivity"] });
      const plan = result.plan;
      if (autoSpeak && plan) speak(plan.plan.steps.map((s) => s.description).join(". "));
      else if (autoSpeak && result.clarification_question) speak(result.clarification_question);
      else if (autoSpeak && result.unsupported_reason) speak(result.unsupported_reason);
    },
  });

  const confirm = useMutation({
    mutationFn: (input: ConfirmVoicePlanInput) => api.confirmVoicePlan(session.id, input),
    onSuccess: (result) => {
      setOutcome((prev) => (prev ? { ...prev, plan: prev.plan ? { ...prev.plan, status: result.status } : prev.plan } : prev));
      queryClient.invalidateQueries({ queryKey: ["myVoiceActivity"] });
    },
  });

  function startListening() {
    const Ctor = getSpeechRecognitionCtor();
    if (!Ctor) return;
    const recognition = new Ctor();
    recognition.lang = "en-US";
    recognition.continuous = false;
    recognition.interimResults = true;
    recognition.onresult = (e: any) => {
      let finalText = "";
      let interim = "";
      let confidence: number | null = null;
      for (let i = 0; i < e.results.length; i++) {
        const result = e.results[i];
        if (result.isFinal) {
          finalText += result[0].transcript;
          confidence = result[0].confidence ?? null;
        } else {
          interim += result[0].transcript;
        }
      }
      setTranscript(finalText || interim);
      if (finalText.trim()) {
        submit.mutate({ text: finalText.trim(), confidence });
      }
    };
    recognition.onerror = () => setListening(false);
    recognition.onend = () => setListening(false);
    recognitionRef.current = recognition;
    recognition.start();
    setListening(true);
  }
  function stopListening() {
    recognitionRef.current?.stop();
    setListening(false);
  }

  function sendTyped() {
    if (!typedText.trim()) return;
    submit.mutate({ text: typedText.trim(), confidence: null });
  }

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h4 style={{ margin: 0 }}>Voice Mode</h4>
        <button className="link-button" style={{ fontSize: 12 }} onClick={() => api.resetVoiceConversation(session.id).then(() => setOutcome(null))}>
          Reset context
        </button>
      </div>
      {session.context_object_key && (
        <p style={{ fontSize: 12, color: "var(--text-muted)", margin: "2px 0 8px" }}>
          Context: this {session.context_object_key} record
        </p>
      )}

      <div style={{ display: "flex", gap: 8, alignItems: "center", marginBottom: 8 }}>
        <button
          className="btn"
          disabled={!speechSupported || submit.isPending}
          onClick={listening ? stopListening : startListening}
          style={listening ? { background: "var(--danger, #d64545)", color: "white", animation: "pulse 1.2s infinite" } : undefined}
          title={speechSupported ? "Tap to talk" : "Speech recognition isn't supported in this browser - use Type instead"}
        >
          {listening ? "● Listening..." : "🎙️ Tap to talk"}
        </button>
        {submit.isPending && <span style={{ fontSize: 12, color: "var(--text-muted)" }}>Processing...</span>}
      </div>
      {transcript && <p style={{ fontSize: 13, fontStyle: "italic" }}>"{transcript}"</p>}

      <form
        onSubmit={(e) => {
          e.preventDefault();
          sendTyped();
        }}
        style={{ display: "flex", gap: 6, marginBottom: 12 }}
      >
        <input
          value={typedText}
          onChange={(e) => setTypedText(e.target.value)}
          placeholder="Type instead..."
          style={{ flex: 1, fontSize: 13 }}
        />
        <button className="btn" disabled={submit.isPending || !typedText.trim()}>
          Send
        </button>
      </form>

      {submit.isError && (
        <div className="error-banner" style={{ fontSize: 13, marginBottom: 8 }}>
          {submit.error instanceof ApiError ? submit.error.message : "Could not process that command"}
        </div>
      )}

      {outcome && <OutcomePanel outcome={outcome} onConfirm={(input) => confirm.mutate(input)} confirming={confirm.isPending} />}

      <div style={{ marginTop: 12, borderTop: "1px solid var(--border, #eee)", paddingTop: 8 }}>
        <strong style={{ fontSize: 12 }}>My Voice Activity</strong>
        {(activity.data ?? []).length === 0 && <p className="empty-state" style={{ fontSize: 12 }}>No commands yet</p>}
        {(activity.data ?? []).map((a) => (
          <div key={a.command_id} style={{ fontSize: 12, padding: "3px 0", color: "var(--text-muted)" }}>
            "{a.transcript}" {a.plan_status ? `— ${a.plan_status}` : ""}
          </div>
        ))}
      </div>
      <button className="link-button" style={{ fontSize: 12, marginTop: 8 }} onClick={onClose}>
        Close
      </button>
    </div>
  );
}

function OutcomePanel({ outcome, onConfirm, confirming }: { outcome: VoiceCommandOutcome; onConfirm: (input: ConfirmVoicePlanInput) => void; confirming: boolean }) {
  if (outcome.unsupported_reason) {
    return <p style={{ fontSize: 13 }}>{outcome.unsupported_reason}</p>;
  }
  if (outcome.clarification_question) {
    return (
      <div>
        <p style={{ fontSize: 13 }}>{outcome.clarification_question}</p>
        {outcome.candidates.length > 0 && (
          <ul style={{ fontSize: 13, margin: "4px 0", paddingLeft: 20 }}>
            {outcome.candidates.map((c) => (
              <li key={c.record_id}>{c.label}</li>
            ))}
          </ul>
        )}
        <p style={{ fontSize: 12, color: "var(--text-muted)" }}>Try again naming which one you mean.</p>
      </div>
    );
  }
  const plan = outcome.plan;
  if (!plan) return null;

  return (
    <div className="panel" style={{ padding: 8 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 6, marginBottom: 4 }}>
        <span className={`badge ${riskBadgeClass(plan.risk)}`}>{plan.risk}</span>
        <span style={{ fontSize: 12, color: "var(--text-muted)" }}>{plan.status}</span>
      </div>
      <ul style={{ fontSize: 13, margin: "4px 0", paddingLeft: 20 }}>
        {plan.plan.steps.map((s, i) => (
          <li key={i}>{s.description}</li>
        ))}
      </ul>
      {plan.status === "awaiting_confirmation" && (
        <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
          <button className="btn btn-secondary" disabled={confirming} onClick={() => onConfirm({ plan_id: plan.id, method: "reject", edited_plan: null })}>
            Cancel
          </button>
          <button className="btn btn-primary" disabled={confirming} onClick={() => onConfirm({ plan_id: plan.id, method: "tap", edited_plan: null })}>
            {confirming ? "Confirming..." : "Confirm"}
          </button>
        </div>
      )}
      {plan.status === "awaiting_approval" && <p style={{ fontSize: 12, color: "var(--text-muted)" }}>Waiting for an Administrator to approve this action.</p>}
      {plan.status === "succeeded" && <p style={{ fontSize: 12, color: "var(--success, #2e7d32)" }}>Done.</p>}
      {plan.status === "partially_failed" && <p style={{ fontSize: 12, color: "var(--danger, #d64545)" }}>Only some steps completed - see My Voice Activity for details.</p>}
    </div>
  );
}
