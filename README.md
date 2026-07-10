# halcyon

Named for the kingfisher of Greek myth — calm seas, a well-run flock.

Front-door Caddy plus an orchestrator for every app this server hosts (`blog`,
`kana-quiz`, `feedbot`, ...). Caddy terminates TLS and routes by hostname to
per-app containers on the shared `web` Docker network. Nothing but Caddy
publishes a port.

## How this works

**Each app is a git checkout sitting in this directory, and that checkout is the
deployment.** You edit `feedbot/`, run `bin/sync feedbot`, and what you edited is
what ships. There is no pull-from-origin release step; `bin/sync` builds the
image from the tree in front of you. Push to GitHub when you like — that's
backup and history, not deploy.

Two consequences worth internalizing:

- `bin/sync` **always builds**. `docker compose up -d` on its own will start a
  stale image and tell you nothing, which is the single most reliable way to
  spend an hour debugging a fix you already made.
- `bin/sync` **never pulls** unless you pass `--pull`, which it refuses on a
  dirty tree. Uncommitted work in an app directory is normal here, so sync says
  so before building it rather than clobbering it.

## Layout

| Path                        | What                                                        |
| --------------------------- | ----------------------------------------------------------- |
| `config/docker-compose.yml` | The Caddy front door.                                        |
| `config/caddy/Caddyfile`    | Host → container routing. Hand-edited; `bin/new` appends.    |
| `config/apps.toml`          | Manifest: which apps exist, their container, port and hosts. |
| `bin/new`                   | Scaffold a new app, repo and all, and serve it.              |
| `bin/sync`                  | Build + bring up apps, then reconcile and verify Caddy.      |
| `bin/templates/`            | The `rust` and `python` app scaffolds `bin/new` renders.     |
| `secrets/<name>.env`        | Per-app KEY=VALUE secrets (gitignored, 0600).                |
| `<name>/`                   | The app's checkout (gitignored).                             |

## A new app

```sh
bin/new toybox                       # rust + axum, https://toybox.rjp.io
bin/new toybox --lang python         # uv + fastapi
```

That scaffolds a working server (`/healthz`, static files, SPA fallback, non-root
Dockerfile, healthcheck, graceful SIGTERM), locks its dependencies, creates the
public GitHub repo and pushes, registers the app in `apps.toml`, appends a vhost
to the Caddyfile, and runs `bin/sync` to build and serve it.

The one thing it can't do is DNS — point the hostname at this server yourself.

Useful flags: `--host` (repeatable), `--port`, `--private`, `--description`,
`--no-repo` / `--no-sync` to stop short, and `--from <git-url>` to adopt an
existing repo instead of scaffolding one.

## Day to day

```sh
bin/sync                # build + up every app, then the front door
bin/sync feedbot        # just one
bin/sync --check        # read-only: what's drifted, what's down
```

| Flag           | Effect                                                  |
| -------------- | ------------------------------------------------------- |
| `--check`      | Read-only report. Non-zero exit if anything is broken.  |
| `--no-build`   | Start the existing image instead of rebuilding.         |
| `--pull`       | `git pull --ff-only` first. Refused on a dirty tree.    |
| `--recreate`   | Force-recreate the app containers.                      |
| `--skip-caddy` | Leave the front door alone.                             |
| `--force`      | Build every app even if its tree is dirty.              |

Naming an app is a statement of intent: `bin/sync blog` deploys blog's working
tree, uncommitted changes and all, and says so. A bare `bin/sync` reconciles
everything and **refuses** if any tree is dirty, because a routine reconcile
should never be the thing that ships work you never committed.

`--no-build` is not an escape hatch from that: `compose up -d` also applies the
working tree's `docker-compose.yml`, so an uncommitted volume or environment
change reaches the running container even when the image doesn't move.

## Cold start on a new server

```sh
git clone git@github.com:rjpower/halcyon.git ~/code/halcyon
cd ~/code/halcyon
bin/sync                         # clones each app from apps.toml, then builds
$EDITOR secrets/*.env            # sync creates empty ones; fill them in
bin/sync
```

`repo` in `apps.toml` exists only for this: to clone an app directory that
isn't there yet. After that it is never consulted.

## The front door

`bin/sync` brings Caddy up, reloads it, and then asks Caddy's admin API which
hosts it is *actually* serving, comparing that against the `hosts` declared in
`apps.toml`. If a host is missing it recreates the container and checks again,
and fails loudly if it still isn't served. A Caddyfile you edited but never
loaded is the failure this catches.

Note `config/docker-compose.yml` bind-mounts the `config/caddy` **directory**,
not the Caddyfile. A single-file bind mount pins that file's inode at container
start, and editors replace a file rather than rewrite it in place — so Caddy
would read the original forever while `caddy reload` cheerfully reported
success. Don't "simplify" it back.

Hosts Caddy serves that no app in `apps.toml` claims are reported as
`unmanaged`, not as errors: loom (`weaver.rjp.io`, `loom.rjp.io`) is deployed
out of band from its own checkout.

## Secrets

Each app gets one file at `secrets/<name>.env` (`KEY=VALUE` per line, no quotes).
`bin/sync` symlinks it into the checkout as `.env`, so the app's own
`docker-compose.yml` picks it up with the standard `env_file: .env`. The values
show up in `docker inspect` — fine for a single-tenant homelab, where anyone
with docker access can read `secrets/` anyway.
