// Loops newsletter-form endpoint (public, no API key — safe to ship in the app).
// Create a Form in Loops (Audience → Forms) and paste its endpoint URL here.
// It looks like: https://app.loops.so/api/newsletter-form/<formId>
// Until set, subscribe() is a no-op — no requests are fired, no errors thrown.
const LOOPS_FORM_ENDPOINT = "https://app.loops.so/api/newsletter-form/cmqzu6a9f00qi0jzdoekx6o67";

/** Best-effort launch-list capture. Never blocks onboarding, never throws. */
export async function subscribe(email: string, name: string): Promise<void> {
  if (LOOPS_FORM_ENDPOINT.includes("REPLACE_WITH_FORM_ID")) return; // not configured yet
  try {
    // Loops' form endpoint accepts URL-encoded fields; `email` is required,
    // `firstName` is the standard Loops contact property.
    const body = new URLSearchParams({ email });
    if (name.trim()) body.set("firstName", name.trim());
    await fetch(LOOPS_FORM_ENDPOINT, {
      method: "POST",
      headers: { "content-type": "application/x-www-form-urlencoded" },
      body,
    });
  } catch {
    // silent — launch-list capture must never break the UX
  }
}
