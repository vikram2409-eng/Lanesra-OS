# Security Policy

## Supported versions

Lanesra OS Desktop is in Early Access; only the latest released version
(see [Releases](https://github.com/vikram2409-eng/Lanesra-OS/releases)) is
supported with security fixes. There is no long-term support branch yet.

## Reporting a vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Instead, report it privately using one of these:

- **GitHub Private Vulnerability Reporting**: open the [Security tab](https://github.com/vikram2409-eng/Lanesra-OS/security/advisories/new) on this repository and click "Report a vulnerability."
- **Email**: vikram2409@gmail.com — include steps to reproduce, the affected version, and the potential impact.

You should expect an initial response within a few days. Once a fix is
confirmed, we'll coordinate on a disclosure timeline before any public
details are published, and credit the reporter in the release notes unless
you'd prefer to stay anonymous.

## Scope notes

A few things worth knowing given how Lanesra OS is designed:

- **Personal Workspace** (the desktop app) trusts the OS process boundary —
  it has no network-facing surface of its own by default.
- **Team Workspace** (the local-network server mode) is designed for a
  trusted LAN, not the public internet, and has no built-in TLS termination.
  Exposing it directly to the internet without a reverse proxy in front of it
  is a misconfiguration, not something the server itself defends against —
  if you find a way to compromise it *within its intended LAN threat model*,
  that's a legitimate report.
- Passwords are stored as salted Argon2 hashes; there is no plaintext or
  reversible storage.
