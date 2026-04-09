In index.html, the Freighter wallet connection is broken because it only checks
for `window.freighter` (legacy API), but Freighter v5+ injects `window.freighterApi`
instead.

Fix the wallet connection by:

1. In `waitForFreighter()`, poll for BOTH `window.freighterApi` AND `window.freighter`
   so it works on all Freighter versions.

2. At the top of `connectWallet()`, resolve a single `freighter` variable:
     const freighter = window.freighterApi ?? window.freighter;
   Then use `freighter` everywhere instead of `window.freighter` throughout
   the entire function.

3. The `signTx()` function also hardcodes `window.freighter.signTransaction`.
   Change it to:
     const freighter = window.freighterApi ?? window.freighter;
     return freighter.signTransaction(xdr, { networkPassphrase: NETWORK_PASS });

4. For `window.freighterApi`, the `requestAccess()` call returns an object like
   `{ error: "..." }` on failure (not a plain string), so update the error check:
     const result = await freighter.requestAccess();
     const err = result?.error ?? result;
     if (err) { showConnectError(...); return; }

5. Similarly, `getNetworkDetails()` on `window.freighterApi` returns
   `{ networkPassphrase, network, ... }` directly (no `.networkPassphrase` nesting
   on some versions), so keep the existing optional chaining `net?.networkPassphrase`
   which already handles this safely.

Do not change anything else — only the wallet connection and sign logic.