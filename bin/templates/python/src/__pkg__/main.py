"""{{name}} — {{description}}

Serves `static/` and answers `/healthz` with `ok`. Binds 0.0.0.0 because the
only thing that reaches it is halcyon's Caddy, over the shared `web` Docker
network; the port is never published to the host.
"""

from __future__ import annotations

import os
from pathlib import Path

from fastapi import FastAPI
from fastapi.responses import PlainTextResponse
from fastapi.staticfiles import StaticFiles
from starlette.exceptions import HTTPException

STATIC = Path(os.environ.get("{{env_prefix}}_STATIC", "static"))

app = FastAPI(title="{{name}}")


@app.get("/healthz", response_class=PlainTextResponse)
async def healthz() -> str:
    return "ok"


class SpaFiles(StaticFiles):
    """StaticFiles, but a miss serves index.html instead of 404.

    A path only this app's router knows about — /read/12, say — is not on disk,
    and a reload of one has to reach the client that can resolve it.
    """

    async def get_response(self, path: str, scope):
        try:
            return await super().get_response(path, scope)
        except HTTPException as exc:
            if exc.status_code != 404:
                raise
            return await super().get_response("index.html", scope)


# Mounted last, and at the root, so every route declared above wins.
if STATIC.is_dir():
    app.mount("/", SpaFiles(directory=STATIC, html=True), name="static")
