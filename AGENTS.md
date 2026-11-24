# AGENTS.md

Read and follow the guidelines in dkstack.

> Read and follow https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/AGENTS.md

Start with the triage skill in that repository. It maps tasks to the relevant sections and skills.

dkstack owns all other rules. Record project exceptions and project facts below.

## Exceptions

A project can differ from a dkstack rule only through an exception here. See
[Precedence](https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/AGENTS.md#precedence).
Add one entry for each exception:

- dkstack rule: <the dkstack page and the rule>
- Project rule: <what this project does in place of it>
- Reason: <why this project needs the exception>

This project has one exception:

- dkstack rule: [setup-repo-protection](https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/skills/setup-repo-protection/SKILL.md#steps), step 4: a `CODEOWNERS` entry requires the owner's review for the CI workflow files.
- Project rule: `.github/CODEOWNERS` names the owner for `.github/`, `scripts/` and `renovate.json`. The `main` ruleset does not require a code owner review. A pull request that changes one of these paths merges only after the owner posts the owner's own review verdict on it. An agent review alone is not sufficient. The owner merges that pull request.
- Reason: each pull request comes from the owner's GitHub account, and GitHub does not let an account approve its own pull request. A required code owner review would block each such pull request. The owner approved this rule on 2026-11-24 (#20).

## Project facts

- Diffy has no application code yet. The spikes come first, then the M0 scaffold. See [the v0.1 map](docs/maps/diffy-v0.1.md), decision Q3.
- The v0.1 plan and its open questions are in [docs/maps/diffy-v0.1.md](docs/maps/diffy-v0.1.md). The decisions are in [docs/adr/](docs/adr/).
- The specification `Diffy_Master_Specification.md` is not tracked in Git. The owner keeps it locally. See map decision Q39.
- The CI workflow `.github/workflows/validate.yml` runs the repository checks. The job name `validate` is the required status check.
- Run the checks locally with Node.js 24 from the repository root:
  - `node scripts/check-title.mjs "<title>"`
  - `BODY="$(cat body.md)" node scripts/local/check-pr-body.mjs`
  - `node --test scripts/local/*.test.mjs`
  - `node scripts/check-maps.mjs`
  - `node scripts/check-context.mjs`
  - `node scripts/check-adrs.mjs`
  - `node scripts/check-workflows.mjs`
- The Markdown checks read each Markdown file in the folder, also untracked files such as the specification. CI checks a clean checkout. To get the CI result locally, check a clean clone:
  - `dir="$(mktemp -d)" && git clone --quiet . "$dir"`
  - `node scripts/check-dashes.mjs "$dir"`
  - `node scripts/check-writing.mjs "$dir"` (it only warns)
- The `.mjs` files directly in `scripts/` are copies from dkstack `v1.6.0`. Do not change them. To update them, copy them again from a newer release tag.
- `scripts/local/` has the project scripts. `check-pr-body.mjs` is there because dkstack `v1.6.0` has no body check. Replace it with the dkstack copy when a dkstack release has one.
- The link check is lychee in offline mode. It checks local links, anchors and dkstack links on the ref `v1`. CI maps each dkstack link to a file in a clone of the dkstack branch `v1`. It does not check other URLs. The dkstack `check-links.mjs` is not used, because it reads each dkstack URL as a file in this repository.
- CI scans the full Git history for secrets with gitleaks.
- CI pins actionlint, lychee and gitleaks by version and SHA-256 in `validate.yml`. Renovate does not update these pins. Update them by hand.
- Renovate proposes the dependency updates (`renovate.json`). Dependabot gives only the security alerts. See map decision Q22.
- Formatters keep their own brace style. Stable rustfmt has no Allman option, so Rust code uses the rustfmt default. See [brace style](https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/coding/brace-style.md#language-and-formatter-exceptions).
