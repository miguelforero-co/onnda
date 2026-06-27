// Replace the placeholder below with your deployed Vercel function URL after deploying vercel-subscribe/.
// See vercel-subscribe/README.md for deploy instructions.
// Until replaced, subscribe() is a no-op — no requests are fired, no errors thrown.
const ENDPOINT = "https://REPLACE-WITH-YOUR-VERCEL-APP.vercel.app/api/subscribe";

/** Best-effort marketing-list capture. Never blocks onboarding, never throws. */
export async function subscribe(email: string, name: string): Promise<void> {
  if (ENDPOINT.includes("REPLACE-WITH-YOUR-VERCEL-APP")) return; // not configured yet
  try {
    await fetch(ENDPOINT, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ email, name }),
    });
  } catch {
    // silent — analytics/marketing must never break the UX
  }
}
