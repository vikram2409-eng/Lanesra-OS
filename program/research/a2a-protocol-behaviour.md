# A2A protocol — behaviour note

Plain-language behaviour note for the implementers of WP-0502 (A2A server)
and WP-0503 (A2A client). Clean room: behaviour only, written from the
protocol's observable behaviour. No code, prompts or text copied from any
specification. Lanesra vocabulary (Team, Run, Step) is what the code uses;
the words below describe what the other side does, not what Lanesra calls it.

## What it is

A2A is a way for agents built by different people, on different frameworks,
to work together over ordinary web requests. One side asks for work; the
other side does it and reports back. Neither side needs to know how the
other is built — only what it offers and how to ask.

## The agent card

Before anything else, an agent publishes a card: a public description of
itself. The card says the agent's name, what it can do (its skills, in
plain words), what kind of input it accepts (text, files, structured data),
what kind of output it produces, and how a caller proves who it is. A
client reads the card first and decides whether this agent is the right one
for the job. If the card is missing, or the caller cannot prove itself the
way the card asks, there is no conversation.

## Tasks

Work is a **task**. The caller creates one with an id of its own choosing
and sends it to the agent. The task carries the request as a message, and a
message is made of **parts**: a piece of text, a file, or a block of
structured data. A task can carry several messages over its life — the
caller asks, the agent answers, the caller adds more.

A task is always in one state, and the caller can read it:

- **submitted** — received, not yet started.
- **working** — the agent is on it.
- **input-required** — the agent is stuck and needs something from the
  caller: a missing detail, a choice, an approval. The task waits; it does
  not fail.
- **completed** — done, with results attached.
- **failed** — the agent could not finish, with a reason.
- **cancelled** — stopped, by the caller or by the agent.

## Results

When an agent finishes, it attaches **artifacts** to the task: named
results, each made of parts (text, a file, structured data). The caller
reads the artifacts, not the agent's internal workings. How the agent got
there — what model it used, what tools it called, what it thought — is not
part of the answer unless the agent chooses to send a trace as well.

## Updates while it works

A long task does not leave the caller guessing. The agent can send updates
as it works, over a stream, so the caller sees progress and partial
results. For very long tasks the agent can also call back: the caller gives
it a web address when it creates the task, and the agent pings that address
when the task changes state.

## How this maps to Lanesra

- A published Team is an agent with a card and an endpoint (WP-0502). A
  task sent to it becomes a run; the run's steps are Lanesra's business,
  not the caller's.
- When a run reaches an approval, the task goes to **input-required**: the
  caller is told the run is waiting for a person, and waits. The caller is
  never given a made-up answer in place of the approval (ADR-0027).
- When Lanesra calls an outside agent (WP-0503), the agent fills a seat.
  Its reply is fenced as untrusted (ADR-0027 decision 3); its trace, if it
  sends one, is ingested as externally reported (ADR-0027 decision 5).
- A caller that disappears does not kill the run. The run continues, and
  the caller can ask for the task again later and read its state.
