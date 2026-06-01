# halcyon

Named for the kingfisher of Greek myth — calm seas, a well-run flock.

Front-door Caddy plus an orchestrator that clones every app this server hosts
(`blog`, `kana-quiz`, ...) and brings them up under a single `bin/sync`. Caddy
terminates TLS and routes by host to per-app containers on the shared `web`
Docker network.

## Layout

| Path                            | What                                                 |
| ------------------------------- | ---------------------------------------------------- |
| `config/docker-compose.yml`     | The Caddy front door.                                |
| `config/Caddyfile`              | Host → container routing. Hand-edited; `bin/add` appends defaults. |
| `config/apps.toml`              | Manifest of apps to clone and run.                   |
| `bin/sync`                      | Clone/pull each app, link its `.env`, compose up.    |
| `bin/add`                       | Interactive: register a new app.                     |
| `secrets/<name>.env`            | Per-app KEY=VALUE secrets (gitignored, 0600).        |
| `<name>/` (e.g. `blog/`)        | Cloned app repo (gitignored).                        |

## Cold start on a new server

```sh
git clone git@github.com:rjpower/halcyon.git ~/code/halcyon
cd ~/code/halcyon
mkdir -p secrets && chmod 700 secrets
$EDITOR secrets/blog.env
$EDITOR secrets/kana-quiz.env
bin/sync
```

`bin/sync` clones each app at the top level of this repo, symlinks
`<name>/.env -> ../secrets/<name>.env`, brings each app's compose stack up,
ensures the external `web` Docker network exists, then starts/reloads Caddy.

## Adding a host

```sh
bin/add                          # prompts for name, repo, port, etc.
$EDITOR secrets/<name>.env       # paste real values
bin/sync <name>                  # clone + bring up just this one
```

Then point the hostname at this server in DNS. `bin/add` appends a default
`reverse_proxy` block to `Caddyfile`; edit if you need auth, custom headers,
or a different upstream port.

## Secrets

Each app gets one file at `secrets/<name>.env` (`KEY=VALUE` per line, no
quotes). `bin/sync` symlinks it into the clone's working directory as `.env`,
so the app's own `docker-compose.yml` can pick it up with the standard
`env_file: .env`. This means the values will appear in `docker inspect`
of the container — fine for a single-tenant homelab where anyone with docker
access can read `secrets/` anyway.

## `bin/sync` flags

| Flag             | Effect                                            |
| ---------------- | ------------------------------------------------- |
| (none)           | Pull + relink + compose up for every app + Caddy. |
| `<name>`         | Same, but only for one app.                       |
| `--no-pull`      | Don't `git pull` existing clones.                 |
| `--pull-only`    | Pull + relink env; skip `compose up`.             |
| `--skip-caddy`   | Don't touch the front door.                       |
