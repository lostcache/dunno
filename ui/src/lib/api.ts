async function handleResponse<T>(r: Response): Promise<T> {
  if (!r.ok) {
    const e = await r.json().catch(() => ({ error: r.statusText }));
    throw new Error(e.error || r.statusText);
  }
  return r.json();
}

export async function api<T = unknown>(url: string): Promise<T> {
  const r = await fetch(url);
  return handleResponse(r);
}

export async function apiPost<T = unknown>(url: string, body: unknown): Promise<T> {
  const r = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return handleResponse(r);
}

export async function apiPatch<T = unknown>(url: string, body: unknown): Promise<T> {
  const r = await fetch(url, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return handleResponse(r);
}

export async function apiDel(url: string): Promise<void> {
  const r = await fetch(url, { method: "DELETE" });
  if (!r.ok && r.status !== 204) {
    const e = await r.json().catch(() => ({ error: r.statusText }));
    throw new Error(e.error || r.statusText);
  }
}
