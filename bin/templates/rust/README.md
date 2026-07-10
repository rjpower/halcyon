# {{name}}

{{description}}

Served at <https://{{host}}> behind [halcyon](https://github.com/rjpower/halcyon)'s
Caddy front door. Nothing is published to the host; Caddy reaches this container
by name on the shared `web` Docker network.

## Develop

```sh
cargo run                       # http://127.0.0.1:{{port}}
curl -fsS localhost:{{port}}/healthz    # -> ok
```

## Deploy

The checkout under `halcyon/{{name}}/` *is* the deployment — there is no separate
release step. Build from the tree you have and bring it up:

```sh
cd ~/code/halcyon && bin/sync {{name}}
```

`bin/sync` builds the image, brings the stack up, and verifies Caddy is really
serving `{{host}}`.

## Config

`{{env_prefix}}_PORT` (default `{{port}}`) and `{{env_prefix}}_STATIC` (default `static`).
Secrets live in `halcyon/secrets/{{name}}.env`, which halcyon symlinks here as
`.env` — never commit it.
