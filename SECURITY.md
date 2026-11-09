<!--
SPDX-FileCopyrightText: 2026 Antonio <https://github.com/kepler-shell>
SPDX-License-Identifier: GPL-3.0-or-later
-->

# Security Policy

## Supported Versions

Security updates apply to the following versions of Kepler Shell:

| Version | Supported |
|---|---|
| 0.1.x (GNOME 45+) | Yes |
| < 0.1.0 | No |

## Reporting a Vulnerability

If you identify a security vulnerability in Kepler Shell:

1. **Do not create a public issue**.
2. Send an email report to the project maintainers with technical details, steps to reproduce, and impact evaluation.
3. Maintainers acknowledge reports within 48 hours and coordinate a security patch before public disclosure.

## Security Controls and Invariants

- **Command Execution**: All process invocations use explicit argument vectors with `GLib.spawn_async`. Shell command interpreters (`sh -c`) are prohibited.
- **Data Privacy**: No telemetry, analytics, or user identifiers are collected, transmitted, or logged.
- **Input Sanitization**: Search bar arithmetic evaluation uses a deterministic scanner without `eval` or dynamic code generators.
