# tenmin

Discord bot using Serenity and Poise. It needs an always-on process because it
connects to Discord's gateway; it is not an HTTP service.

## Required configuration

Set these runtime environment variables in your host's secret/variable UI:

- `DISCORD_TOKEN` — the Discord bot token
- `RIOT_TOKEN` — a Riot Games API key

The existing `Secrets.toml` is kept for reference but is no longer read by the
application. Do not commit it or any `.env` file containing real tokens.

## Deploy with Railway

1. Push this `dev/tenmin` directory to a private GitHub repository.
2. In Railway, create a project, choose **Deploy from GitHub repo**, and select
   the repository. Railway detects the root `Dockerfile` automatically.
3. In the service's **Variables** tab, add `DISCORD_TOKEN` and `RIOT_TOKEN`.
4. Deploy. Check the service logs for `is connected!`.

Railway should be configured as a worker/container service; no public domain,
port, database, or volume is needed.

## Deploy with Fly.io

From this directory, after installing and authenticating `flyctl`:

```bash
fly launch --no-deploy
fly secrets set DISCORD_TOKEN='your-discord-token' RIOT_TOKEN='your-riot-token'
fly deploy
```

No HTTP service configuration is necessary. Keep one machine running so the
Discord gateway connection remains online.

## Run locally with Docker

```bash
docker build -t tenmin .
docker run --rm \
  -e DISCORD_TOKEN='your-discord-token' \
  -e RIOT_TOKEN='your-riot-token' \
  tenmin
```

In the Discord Developer Portal, make sure **Message Content Intent** is
enabled for prefix commands (the `!` commands). Slash commands are registered
globally on startup and may take a little time to appear in Discord.
