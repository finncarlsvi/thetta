export async function requestJson<T>(
  path: string,
  method = "POST",
  body?: unknown,
): Promise<T> {
  const multipart = body instanceof FormData;
  const response = await fetch("/api/" + path, {
    method,
    headers:
      body && !multipart ? { "Content-Type": "application/json" } : undefined,
    body: multipart ? body : body ? JSON.stringify(body) : undefined,
  });
  if (!response.ok) {
    if (response.status === 401)
      window.dispatchEvent(new Event("thetta-signed-out"));
    const result = await response
      .json()
      .catch(() => ({ error: response.statusText }));
    throw new Error(result.error || response.statusText);
  }
  return response.json();
}
