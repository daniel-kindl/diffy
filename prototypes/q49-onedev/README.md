# Q49 prototype: S4 OneDev capabilities

Throwaway code for map question Q49 in `docs/maps/diffy-v0.1.md`. It is not Diffy code.

Charter question: Which review actions can Diffy do through the latest stable OneDev REST API, and
how does Diffy confirm each one?

## Server

- The Q14 homelab instance, `http://192.168.0.102:6610`, on OneDev 17.2.4. `GET /~api/server/version`
  returns `17.2.4`. It is the `1dev/server:17.2.4` image,
  `sha256:da5add944fc39270ea8f3cf05a98b9a589429e2e45e5ba3b99bc162bbd092fdd`. On 2026-11-25 this is
  also the `latest` tag on Docker Hub.
- The instance ran 16.9.8 before. The agent upgraded it on 2026-11-25 with the owner's approval.
  The data folder was backed up first, to `/home/tango/onedev-backup-16.9.8-20261125.tar.gz` on
  the homelab. The only listed incompatibility from 16.9.8 to 17.2.4 is the 17.0.2 webhook
  signature.
- The CI container is the same image. Q49 ran no separate container.

## Files

- `probe.mjs`: the API probe. It reads an administrator access token on standard input and
  never writes a token.
  - It creates the project `diffy-q49-sandbox` and the ordinary user `diffy-q49-reviewer`, or
    reuses them. The user gets the `Code Writer` role on the project.
  - It creates a reviewer access token that expires after 2 hours, and deletes the token at the
    end.
  - Each run pushes a branch that edits rows 10 and 50 of `src/lib.rs`. The administrator opens a
    pull request with the reviewer as reviewer. The reviewer then does each review action.
  - Each comment body ends with the Q34 marker `<!-- diffy:f=<finding-id> a=<attempt-id> -->`.
- `ui-check.mjs`: logs in to the web UI as the reviewer in headless Chromium 158. It checks the
  visible text and the HTML of three pages for the marker, and saves a screenshot of each.
- `results/homelab.json`: the last probe run, pull request #5 (ID 74). `results/homelab-first-run.json` is
  the first complete run, before the status change variants.
- `results/ui-check.json` and `screenshots/`: the UI check of pull request #5.

Run:

```sh
node probe.mjs http://192.168.0.102:6610 results/homelab.json < ~/.config/diffy-q49/admin-token
node ui-check.mjs http://192.168.0.102:6610 <PR number> results/ui-check.json screenshots/homelab-pr<N> <code comment ID>
```

## Results

Each row passed on the homelab instance (`results/homelab.json`). The source column points to
OneDev `v17.2.4`, `server-core/src/main/java/io/onedev/server/rest/resource/`.

| Action | Endpoint | Source | Read call that confirms it before a retry |
|--------|----------|--------|-------------------------------------------|
| Authenticate | `Authorization: Bearer <access token>`, `GET /~api/users/me` | `UserResource.java:153` | n/a |
| Get a pull request | `GET /~api/pulls/{id}` | `PullRequestResource.java:110` | n/a |
| Inline comment | `POST /~api/code-comments` | `CodeCommentResource.java:155` | `GET /~api/pulls/{id}/code-comments`, the marker in `content` |
| Reply | `POST /~api/code-comment-replies` | `CodeCommentReplyResource.java:59` | `GET /~api/code-comments/{id}/replies`, the marker in `content` |
| Resolve, unresolve | `POST /~api/code-comments/{id}/resolve`, `/unresolve` | `CodeCommentResource.java:265` | `GET /~api/code-comments/{id}`, field `resolved` |
| Pull request comment | `POST /~api/pull-request-comments` | `PullRequestCommentResource.java:50` | `GET /~api/pulls/{id}/comments`, the marker in `content` |
| Approve | `POST /~api/pulls/{id}/approve` | `PullRequestResource.java:408` | `GET /~api/pulls/{id}/reviews`, the entry with the user's `userId`, field `status` |
| Request changes | `POST /~api/pulls/{id}/request-for-changes` | `PullRequestResource.java:425` | the same, `status` is `REQUESTED_FOR_CHANGES` |

Each write took 14 to 102 ms.

## Findings

1. Each review action has a REST endpoint. Each write has a read call that shows the result
   before a retry.
2. The marker is not visible. On the activity page, on the changes page and in the open code
   comment, the visible text has no `diffy:f=`. The HTML has no escaped marker.
   The HTML of the activity page has the marker, but the page does not show it.
3. Only a reviewer of the pull request can approve or request changes. The submitter gets 406
   `You are not reviewer of this pull request. Add your option as comment instead`. A submitter
   cannot be added as a reviewer (`PullRequestResource.java:383`).
4. An inline comment must be in a OneDev diff hunk, with its context rows. A comment on row 7,
   3 rows above the change, passes. A comment on row 30 gets 406
   `Comment range must be within a diff hunk`. The compare context of each comment has
   `whitespaceOption` `IGNORE_TRAILING`, so OneDev hunks can differ from the canonical hunks.
5. A repeated approve is not a no-op. It updates `statusDate` and adds a second "approved" entry
   to the pull request activity. A repeated resolve is a no-op.
6. Each resolve and unresolve adds a reply when the body is not null. The endpoint takes the raw
   request body as the note and accepts only `application/json`. `text/plain` gets 415.
   - An empty body adds an empty reply.
   - The body `null` adds the reply `null`.
   - The body `Resolved.` followed by a marker adds that text as the reply. The marker then
     confirms the note.
   No tested body changes the state without a reply.
7. The comment, reply and review objects have the user ID, so the read calls can also check the
   author.

## Not tested

- OAuth or a user login other than an access token.
- Multi-line comment ranges and comments on the old side.
- A comment on a pull request after a new push (the mark of an older commit).
- Email notifications and webhooks.
- A second OneDev instance (Q14 names one).
