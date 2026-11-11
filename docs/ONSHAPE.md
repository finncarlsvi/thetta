# Onshape login and imports

The public website is https://thetta.finn.xyz. All workspace and manufacturing
APIs require an authenticated, approved Onshape account. The initial allowlist
contains only `finncarlsvi@gmail.com`. This applies to direct LAN requests too.
Public endpoints are the login shell/assets, OAuth start/callback/status, and the
minimal health check. Signed-in users can upload STEP files or paste a
Part Studio/assembly URL under **New workspace** in the project explorer. Studio
mode retains the linked source and Refresh control.

## OAuth registration

In Onshape **My account → Developer → OAuth applications**, register:

| Field | Value |
| --- | --- |
| Name | Thetta |
| Primary format | xyz.finn.thetta |
| Type | Integrated Cloud App (current registration) |
| Summary | Import Onshape Part Studios and assemblies into Thetta for nesting and CAM. |
| Redirect URL | https://thetta.finn.xyz/api/auth/onshape/callback |
| OAuth URL | https://thetta.finn.xyz/api/auth/onshape/start |
| Permissions | Read profile information; read documents |

The account owner must accept the API Agreement and authorize the app. Keep this
a private application for personal use. Export uses the OAuth-compatible STEP
translation endpoints with `storeInDocument=false`; it does not modify the source
document or create blob tabs. Standard `cad.onshape.com` workspace/version links
and their configuration parameter are supported. Imports replace the current
source only after a successful download and STEP parse. Failed imports leave
existing work intact. Refresh is manual; there is no background design polling.

The Free plan supports API access, with 2,500 annual calls. Private OAuth app
calls count toward the app owner's allowance. Rate limits are separate; imports
use bounded polling/backoff and report quota exhaustion. See [Onshape API
limits](https://onshape-public.github.io/docs/auth/limits/).

## Host secrets and deployment

`scripts/setup-web-auth.sh` creates host-only secret files with restrictive
permissions under `/home/finn/.config/thetta/`. It preserves existing values.

- `token_key`: 32 random binary bytes, used to encrypt persisted OAuth tokens.
- `onshape_client_id`: the OAuth client ID as text.
- `onshape_client_secret`: the OAuth secret as text.

Place the real client ID/secret in these files without checking them into source
control, then recreate the container so Compose refreshes the secret mounts:

```sh
ssh finn@192.168.5.5 'cd /home/finn/thetta && docker compose up -d --force-recreate --wait'
```

With empty registration files the site stays locked and shows that registration
is pending. It never falls back to anonymous access. Production startup requires
the encryption key. Back up the key and `/data/auth.sqlite` together with the
workspace volume; do not replace the key for an existing database.

Authentication uses short-lived, cookie-bound OAuth state, an opaque 30-day
session, and Secure/HttpOnly/SameSite=Lax cookies. Sessions are stored hashed;
OAuth tokens are encrypted in SQLite and never included in browser snapshots or
logs. Logout invalidates the browser session and closes its event stream. Token
refreshes are serialized and updated refresh tokens are saved atomically.
Mutating endpoints require the configured public origin.
The sign-in request expires after ten minutes. An expired browser callback
returns to the login page with a retry message; start a fresh sign-in there.

The approved owner inherits the pre-login workspace UUID directories without
rewriting their source files. The project explorer lets the owner select existing
work or create an isolated new workspace. **Workspaces** in the editor returns
to that page. Ownership is enforced independently of
the workspace ID; the old anonymous cookie provides no access.

## HAProxy

pfSense at `192.168.5.1:445` terminates TLS using the existing `finn_xyz` frontend
and `WILDCARD_FINN_XYZ` certificate. The `thetta.finn.xyz` host ACL routes to the
`thetta_web` backend at `192.168.5.5:3007`. The backend health check is
`GET /api/health`, with an extended server timeout for imports and event streams.
The app uses its configured canonical HTTPS origin instead of trusting arbitrary
forwarded host/scheme headers. Other frontend rules and the default backend are
preserved.

Before exposure, verify that unauthenticated data requests return 401 on both
addresses. Backups are held under `/home/finn/thetta-backups/` on the server.
Use the prior container image and router backup to undo a deployment if needed;
do not restore an anonymous image while leaving the public HAProxy route enabled.

## Validation

`cargo test -p thetta-web` exercises account rejection, callback state binding and
replay/expiry, provider identity verification, refresh token rotation, encrypted
storage, session persistence/logout, cross-origin rejection, workspace ownership,
real STEP/CAM workflows, and Onshape import/refresh against a local mock provider.
Real Onshape login/export still requires the registered app and owner consent.
