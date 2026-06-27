# onnda-subscribe

Standalone Vercel serverless function that captures emails for the onnda launch list. Stores subscribers in Vercel KV with deduplication. This is an independently deployable mini-project — it has no relation to the Tauri desktop app build.

## Deploy

```bash
cd vercel-subscribe
npm install
vercel link          # link to a Vercel project (create new or select existing)
# In the Vercel dashboard: Storage → Create → KV → link to this project
# Or via CLI: vercel env pull to get the KV env vars after provisioning in dashboard
vercel deploy --prod
```

Secrets (KV_URL, KV_REST_API_URL, KV_REST_API_TOKEN, KV_REST_API_READ_ONLY_TOKEN) are injected by Vercel automatically when KV is linked. They never live in this repo.

## Endpoint

```
POST /api/subscribe
Content-Type: application/json

{ "email": "user@example.com", "name": "Optional Name" }
```

### Responses

| Status | Body | Meaning |
|--------|------|---------|
| 200 | `{ "ok": true }` | Subscribed (or already subscribed — no double-count) |
| 400 | `{ "ok": false, "error": "invalid email" }` | Missing or malformed email |
| 405 | `{ "ok": false }` | Non-POST method |

## Smoke test (after deploy)

```bash
DEPLOY_URL=https://your-project.vercel.app

# First call — should subscribe and count once
curl -s -X POST "$DEPLOY_URL/api/subscribe" \
  -H 'content-type: application/json' \
  -d '{"email":"a@b.com","name":"Test"}' | jq .
# → { "ok": true }

# Second identical call — must NOT increment sub:count again
curl -s -X POST "$DEPLOY_URL/api/subscribe" \
  -H 'content-type: application/json' \
  -d '{"email":"a@b.com","name":"Test"}' | jq .
# → { "ok": true }  (idempotent, count stays at 1)
```

## Integration with the app

After deploy, the resulting URL goes into `src/lib/subscribe.ts` (Task 10). Example:

```ts
const SUBSCRIBE_URL = "https://your-project.vercel.app/api/subscribe";
```
