import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { ChangeOwnPassword, User, VoicePreferencesInput } from "../../lib/types";

const EMPTY: ChangeOwnPassword = { current_password: "", new_password: "" };

const EMPTY_PREFS: VoicePreferencesInput = {
  response_channel: "voice_and_text",
  spoken_detail: "normal",
  auto_speak_confirmations: true,
  quiet_mode: false,
  unlock_duration_minutes: 15,
};

export function Account({ user }: { user: User }) {
  const [input, setInput] = useState<ChangeOwnPassword>(EMPTY);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const change = useMutation({
    mutationFn: () => api.changeMyPassword(input),
    onSuccess: () => {
      setInput(EMPTY);
      setError(null);
      setSuccess(true);
    },
    onError: (err) => {
      setSuccess(false);
      setError(err instanceof ApiError ? err.message : "Could not change the password");
    },
  });

  return (
    <div>
      <h2>My account</h2>
      <p style={{ color: "var(--text-muted)" }}>
        Signed in as <strong>{user.display_name}</strong> ({user.username}) · {user.roles.join(", ")}
      </p>

      <div className="card" style={{ marginTop: 16, maxWidth: 420 }}>
        <h3 style={{ marginTop: 0 }}>Change my password</h3>
        {error && <div className="error-banner">{error}</div>}
        {success && <div className="success-banner">Password changed.</div>}
        <form
          className="form-grid"
          onSubmit={(e) => {
            e.preventDefault();
            setSuccess(false);
            change.mutate();
          }}
        >
          <div className="form-field full">
            <label>Current password</label>
            <input
              type="password"
              value={input.current_password}
              onChange={(e) => setInput({ ...input, current_password: e.target.value })}
              required
            />
          </div>
          <div className="form-field full">
            <label>New password (min 8 characters)</label>
            <input
              type="password"
              value={input.new_password}
              onChange={(e) => setInput({ ...input, new_password: e.target.value })}
              minLength={8}
              required
            />
          </div>
          <div className="form-field full">
            <button className="btn btn-primary" type="submit" disabled={change.isPending}>
              Change password
            </button>
          </div>
        </form>
      </div>

      <VoicePinSection />
      <MyVoiceActivity />
    </div>
  );
}

function VoicePinSection() {
  const queryClient = useQueryClient();
  const settings = useQuery({ queryKey: ["voiceSettings"], queryFn: () => api.getVoiceSettings() });
  const [pin, setPin] = useState("");
  const [confirmPin, setConfirmPin] = useState("");
  const [pinError, setPinError] = useState<string | null>(null);
  const [pinSuccess, setPinSuccess] = useState(false);
  const [prefs, setPrefs] = useState<VoicePreferencesInput>(EMPTY_PREFS);
  const [prefsSynced, setPrefsSynced] = useState(false);

  if (settings.data && !prefsSynced) {
    setPrefs({
      response_channel: settings.data.response_channel,
      spoken_detail: settings.data.spoken_detail,
      auto_speak_confirmations: settings.data.auto_speak_confirmations,
      quiet_mode: settings.data.quiet_mode,
      unlock_duration_minutes: settings.data.unlock_duration_minutes,
    });
    setPrefsSynced(true);
  }

  const setPinMutation = useMutation({
    mutationFn: () => api.setVoicePin({ pin }),
    onSuccess: (data) => {
      queryClient.setQueryData(["voiceSettings"], data);
      setPin("");
      setConfirmPin("");
      setPinError(null);
      setPinSuccess(true);
    },
    onError: (err) => {
      setPinSuccess(false);
      setPinError(err instanceof ApiError ? err.message : "Could not set the voice PIN");
    },
  });

  const savePrefs = useMutation({
    mutationFn: () => api.updateVoicePreferences(prefs),
    onSuccess: (data) => {
      queryClient.setQueryData(["voiceSettings"], data);
    },
  });

  return (
    <div className="card" style={{ marginTop: 16, maxWidth: 420 }}>
      <h3 style={{ marginTop: 0 }}>Voice PIN</h3>
      <p style={{ color: "var(--text-muted)", marginTop: -8 }}>
        A 4+ digit PIN unlocks Voice Mode on this device. Voice can never do more than your own account
        permissions already allow.
      </p>
      {settings.data?.pin_set && (
        <p style={{ color: "var(--text-muted)" }}>
          PIN is set{settings.data.locked_until ? " · voice unlock is temporarily locked after failed attempts" : "."}
        </p>
      )}
      {pinError && <div className="error-banner">{pinError}</div>}
      {pinSuccess && <div className="success-banner">Voice PIN updated.</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          setPinSuccess(false);
          if (pin !== confirmPin) {
            setPinError("PINs do not match");
            return;
          }
          setPinMutation.mutate();
        }}
      >
        <div className="form-field full">
          <label>{settings.data?.pin_set ? "New PIN" : "Set a PIN"} (4 digits)</label>
          <input
            type="password"
            inputMode="numeric"
            pattern="[0-9]{4}"
            value={pin}
            onChange={(e) => setPin(e.target.value)}
            minLength={4}
            maxLength={4}
            required
          />
        </div>
        <div className="form-field full">
          <label>Confirm PIN</label>
          <input
            type="password"
            inputMode="numeric"
            pattern="[0-9]{4}"
            value={confirmPin}
            onChange={(e) => setConfirmPin(e.target.value)}
            minLength={4}
            maxLength={4}
            required
          />
        </div>
        <div className="form-field full">
          <button className="btn btn-primary" type="submit" disabled={setPinMutation.isPending}>
            {settings.data?.pin_set ? "Reset PIN" : "Set PIN"}
          </button>
        </div>
      </form>

      <h4>Response preferences</h4>
      <div className="form-grid">
        <div className="form-field">
          <label>Response channel</label>
          <select
            value={prefs.response_channel}
            onChange={(e) => setPrefs({ ...prefs, response_channel: e.target.value as VoicePreferencesInput["response_channel"] })}
          >
            <option value="voice_and_text">Voice + text</option>
            <option value="text_only">Text only</option>
          </select>
        </div>
        <div className="form-field">
          <label>Spoken detail</label>
          <select
            value={prefs.spoken_detail}
            onChange={(e) => setPrefs({ ...prefs, spoken_detail: e.target.value as VoicePreferencesInput["spoken_detail"] })}
          >
            <option value="short">Short</option>
            <option value="normal">Normal</option>
            <option value="detailed">Detailed</option>
          </select>
        </div>
        <div className="form-field">
          <label>Unlock duration</label>
          <select
            value={prefs.unlock_duration_minutes}
            onChange={(e) => setPrefs({ ...prefs, unlock_duration_minutes: Number(e.target.value) })}
          >
            <option value={5}>5 minutes</option>
            <option value={15}>15 minutes</option>
            <option value={30}>30 minutes</option>
          </select>
        </div>
        <div className="form-field full">
          <label>
            <input
              type="checkbox"
              checked={prefs.auto_speak_confirmations}
              onChange={(e) => setPrefs({ ...prefs, auto_speak_confirmations: e.target.checked })}
            />{" "}
            Speak confirmations aloud
          </label>
        </div>
        <div className="form-field full">
          <label>
            <input
              type="checkbox"
              checked={prefs.quiet_mode}
              onChange={(e) => setPrefs({ ...prefs, quiet_mode: e.target.checked })}
            />{" "}
            Quiet mode (text responses only, no spoken output)
          </label>
        </div>
        <div className="form-field full">
          <button className="btn btn-secondary" type="button" onClick={() => savePrefs.mutate()} disabled={savePrefs.isPending}>
            Save voice preferences
          </button>
        </div>
      </div>
    </div>
  );
}

function MyVoiceActivity() {
  const activity = useQuery({ queryKey: ["myVoiceActivity"], queryFn: () => api.listMyVoiceActivity(20) });

  return (
    <div className="card" style={{ marginTop: 16, maxWidth: 640 }}>
      <h3 style={{ marginTop: 0 }}>My Voice Activity</h3>
      {!activity.data?.length && <p style={{ color: "var(--text-muted)" }}>No voice commands yet.</p>}
      {!!activity.data?.length && (
        <table className="table">
          <thead>
            <tr>
              <th>When</th>
              <th>Said</th>
              <th>Object</th>
              <th>Status</th>
              <th>Risk</th>
            </tr>
          </thead>
          <tbody>
            {activity.data.map((entry) => (
              <tr key={entry.command_id}>
                <td>{new Date(entry.created_at).toLocaleString()}</td>
                <td>{entry.transcript}</td>
                <td>{entry.object_key ?? "—"}</td>
                <td>{entry.plan_status ?? "—"}</td>
                <td>{entry.risk ?? "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
