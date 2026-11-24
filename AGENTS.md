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

This project has no exceptions yet.

## Project facts

- Diffy has no application code yet. The spikes come first, then the M0 scaffold. See [the v0.1 map](docs/maps/diffy-v0.1.md), decision Q3.
- The v0.1 plan and its open questions are in [docs/maps/diffy-v0.1.md](docs/maps/diffy-v0.1.md). The decisions are in [docs/adr/](docs/adr/).
- The specification `Diffy_Master_Specification.md` is not tracked in Git. The owner keeps it locally. See map decision Q39.
- The CI workflow `.github/workflows/validate.yml` runs the repository checks. The job name `validate` is the required status check.
- Run the checks locally with Node.js 24 from the repository root:
  - `node scripts/check-title.mjs "<title>"`
  - `node scripts/check-maps.mjs`
  - `node scripts/check-context.mjs`
  - `node scripts/check-adrs.mjs`
  - `node scripts/check-dashes.mjs`
  - `node scripts/check-writing.mjs` (it only warns)
  - `node scripts/check-workflows.mjs`
- The scripts in `scripts/` are copies from dkstack `v1.6.0`. Do not change them. To update them, copy them again from a newer release tag.
- The link check is lychee in offline mode. It checks local links and anchors only. The dkstack `check-links.mjs` is not used, because it reads each dkstack URL as a file in this repository.
- Renovate proposes the dependency updates (`renovate.json`). Dependabot gives only the security alerts. See map decision Q22.
- Formatters keep their own brace style. Stable rustfmt has no Allman option, so Rust code uses the rustfmt default. See [brace style](https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/coding/brace-style.md#language-and-formatter-exceptions).
