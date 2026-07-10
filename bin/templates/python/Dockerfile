# uv ships as a single static binary; copy it into a conventional glibc python
# so we get `uv sync` for reproducible installs without uv at runtime.
FROM python:3.13-slim
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /usr/local/bin/

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    UV_LINK_MODE=copy \
    UV_COMPILE_BYTECODE=1 \
    UV_NO_CACHE=1 \
    {{env_prefix}}_STATIC=/app/static \
    {{env_prefix}}_PORT={{port}}

WORKDIR /app

# Dependencies first, so their layer caches independently of our source. `uv
# sync` builds the project itself, so src/ has to be here for it.
COPY pyproject.toml uv.lock ./
COPY src ./src
RUN uv sync --frozen --no-dev

COPY static ./static

RUN useradd --system --uid 1000 app && mkdir -p /data && chown -R app:app /data /app
USER app

EXPOSE {{port}}
VOLUME ["/data"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD python -c "import urllib.request as u, sys; sys.exit(0 if u.urlopen('http://127.0.0.1:{{port}}/healthz', timeout=5).status == 200 else 1)"

# Straight at the venv's uvicorn: no `uv run` at boot, so nothing tries to
# re-resolve the lockfile as a non-root user on a read-only-ish filesystem.
CMD ["/app/.venv/bin/uvicorn", "{{pkg}}.main:app", "--host", "0.0.0.0", "--port", "{{port}}"]
