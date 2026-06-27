import { kv } from "@vercel/kv";

function validEmail(e) {
  return typeof e === "string" && /^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(e);
}

export default async function handler(req, res) {
  if (req.method !== "POST") return res.status(405).json({ ok: false });
  const { email, name } = req.body ?? {};
  if (!validEmail(email)) return res.status(400).json({ ok: false, error: "invalid email" });
  const key = `sub:${email.toLowerCase()}`;
  const exists = await kv.get(key);
  if (!exists) {
    await kv.set(key, { email, name: typeof name === "string" ? name : "", ts: Date.now() });
    await kv.incr("sub:count");
  }
  return res.status(200).json({ ok: true });
}
