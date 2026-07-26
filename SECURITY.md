<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# Security Policy

> This is the default security policy for all `hyperpolymath` projects. A
> repository may override it with its own `SECURITY.md`.

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public issues, pull
requests, or discussions.**

Instead, use GitHub's private vulnerability reporting:

1. Go to the **Security** tab of the affected repository.
2. Click **Report a vulnerability**.
3. Fill out the advisory form with as much detail as you can — affected version,
   reproduction steps, and impact.

If private reporting is unavailable on a particular repository, email
**j.d.a.jewell@open.ac.uk** with the details.

## What to Expect

- **Acknowledgement** within 48 hours.
- An initial assessment and severity triage shortly after.
- Coordinated disclosure: we will agree a timeline with you and credit you in the
  advisory unless you prefer to remain anonymous.

## Supported Versions

Unless a repository states otherwise, security fixes target the latest `main` and
the most recent tagged release.

| Version          | Supported          |
| ---------------- | ------------------ |
| latest `main`    | :white_check_mark: |
| latest release   | :white_check_mark: |
| older            | :x:                |

## Scope

This policy covers vulnerabilities in the project's own code. For vulnerabilities
in third-party dependencies, please open a public **🛡️ Dependency / advisory**
issue referencing the upstream advisory (CVE / GHSA / RUSTSEC) instead — those are
already public.
