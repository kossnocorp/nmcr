# Contributing

## Testing MCP server

### Dev Container

When running the project in a dev container, you can use `docker` to run the `stdio` server.

First, get the container name:

```bash
docker ps --format "table {{.Names}}\t{{.Image}}"
```

It would print a table looking like this:

```
NAMES             IMAGE
amazing_wozniak   vsc-nmcr-06907efde6f46cc4152645c83f56beb1b5ad4abdd933374158713f195e4567f3
```

Grab the container name, then to verify if everything is set up, run the command, replacing `eager_wing` with your container name and `/wrkspc/nmcr` with the path to the repo:

```bash
docker exec -it amazing_wozniak bash -lc 'cd /wrkspc/nmcr && eval "$(mise activate bash --shims)" && RUSTFLAGS=-Awarnings cargo run --quiet -- mcp --project ./examples/basic/
```
