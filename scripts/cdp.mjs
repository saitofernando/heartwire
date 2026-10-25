// Dependency-free Chromium/Edge test helper. Requires Node.js 22+.
export async function connectCdp(port, { timeout = 20000, selectTarget = (tab) => tab.type === "page" } = {}) {
  const until = Date.now() + timeout;
  let target;
  while (Date.now() < until) {
    try {
      const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      target = targets.find(selectTarget);
      if (target) break;
    } catch { /* Chromium may still be starting. */ }
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
  if (!target) throw new Error(`No Chromium page on debugging port ${port}`);
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });
  let next = 1;
  const pending = new Map();
  const exceptions = [];
  socket.addEventListener("message", (event) => {
    const message = JSON.parse(event.data);
    if (message.id) {
      const item = pending.get(message.id);
      if (item) {
        pending.delete(message.id); clearTimeout(item.timer);
        message.error ? item.reject(new Error(JSON.stringify(message.error))) : item.resolve(message.result);
      }
    }
    if (message.method === "Runtime.exceptionThrown") exceptions.push(message.params.exceptionDetails);
  });
  socket.addEventListener("close", () => {
    for (const item of pending.values()) { clearTimeout(item.timer); item.reject(new Error("Chromium debugging connection closed")); }
    pending.clear();
  });
  const command = (method, params = {}) => new Promise((resolve, reject) => {
    const id = next++;
    const timer = setTimeout(() => { pending.delete(id); reject(new Error(`CDP command timed out: ${method}`)); }, 20000);
    pending.set(id, { resolve, reject, timer });
    socket.send(JSON.stringify({ id, method, params }));
  });
  const evaluate = async (expression) => {
    const result = await command("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true, userGesture: true });
    if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
    return result.result.value;
  };
  const waitFor = async (expression, waitTimeout = 10000) => {
    const end = Date.now() + waitTimeout;
    while (Date.now() < end) {
      if (await evaluate(expression)) return;
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
    throw new Error(`Browser assertion timed out: ${expression}`);
  };
  await command("Runtime.enable");
  await command("Page.enable");
  return { target, command, evaluate, waitFor, exceptions, close: () => socket.close() };
}
