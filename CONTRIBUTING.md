<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# Contributing

> This is the default contributing guide for all `hyperpolymath` projects. A
> repository may override or extend it with its own `CONTRIBUTING.md`. Always
> defer to a repository's own README and build files for project-specific steps.

Thank you for considering a contribution. These projects value careful,
well-grounded work over speed.

## Ground rules

- **Foundations first.** Soundness holes and broken foundations (non-compiling
  proofs, a falsely-"done" status, a claimed-passing test that doesn't) are fixed
  *before* new features are built on top. If you find one, file a
  **🕳️ Soundness hole** issue.
- **Ground truth over status docs.** Establish what is true by *running the tool*
  (e.g. `idris2`, `cargo`, `coqc`, `agda`), not by reading a status file.
- **Honest claims.** If a public claim overstates what the code does, that's
  drift — file an **🪞 Affirmation / claim drift** issue. Reporting it is welcome.

## Getting set up

Reproducible builds use **GNU Guix** (this estate is Guix-only — please do not
add Nix files or propose Nix-based workflows):

```sh
git clone https://github.com/hyperpolymath/<repo>.git
cd <repo>

# Enter a reproducible dev shell (if the repo ships a manifest / guix.scm):
guix shell -m manifest.scm      # or: guix shell -f guix.scm

# Most repos expose a task runner:
just --list
```

If a repository documents a different toolchain in its README, follow that.

## Making changes

1. **Open or claim an issue first** for anything non-trivial, so the work isn't
   duplicated and the approach can be agreed.
2. **Branch** from up-to-date `main`. Keep changes focused.
3. **Sign your commits.** Signed commits are expected:
   ```sh
   git commit -S -m "your message"
   ```
4. **Licensing.** Code and config are `MPL-2.0`; prose documentation is
   `CC-BY-SA-4.0`. New files should carry the appropriate `SPDX-License-Identifier`
   header. Please do **not** change the licence of existing files — licence
   changes are handled manually by the maintainer.
5. **Tests and checks** should pass locally before you open a PR. Re-run the
   project's own checks; don't rely on a status doc saying they pass.

## Opening a pull request

- Fill out the pull-request template.
- Describe *what* changed and *why*; link the issue it closes.
- Keep the PR reviewable — small, coherent diffs merge faster.
- Be patient and kind in review; see the [Code of Conduct](CODE_OF_CONDUCT.md).

## Questions

Open a **Discussion** for open-ended questions, or a **❓ Question** issue if you
want a tracked answer.
