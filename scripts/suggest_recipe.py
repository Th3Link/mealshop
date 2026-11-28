#!/usr/bin/env python3
"""Hand unreviewed recipes to a household's suggestion inbox; standard library only.

Reads one JSON object per line from stdin (title, language, yield_text, ingredients, steps,
source_url, note) and calls the `recipe_suggest` MCP tool for each. Use it as the last step of
your own pipeline, for example after transcribing a saved video on your machine. It fetches
nothing itself. Configuration is the same protected environment as `mcp_stdio.py`.
"""
import json
import os
import pathlib
import sys
import urllib.request

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import mcp_stdio as bridge  # noqa: E402

FIELDS = {"title", "language", "yield_text", "ingredients", "steps", "source_url", "note"}


def suggest(opener, url, credentials, identifier, candidate):
    """Returns (suggestion_id, replayed) or raises ValueError with a message safe to print."""
    if not isinstance(candidate, dict) or set(candidate) - FIELDS:
        raise ValueError("expected an object with only " + ", ".join(sorted(FIELDS)))
    arguments = {key: value for key, value in candidate.items() if value not in (None, "", [])}
    message = {"jsonrpc": "2.0", "id": identifier, "method": "tools/call", "params": {
        "name": "recipe_suggest", "arguments": arguments,
        "_meta": {"io.modelcontextprotocol/protocolVersion": bridge.MODERN,
                  "io.modelcontextprotocol/clientCapabilities": {}}}}
    reply, _ = bridge.forward(opener, url, credentials, message, bridge.LEGACY)
    if "error" in reply:
        raise ValueError(str(reply["error"].get("message", "request failed")))
    content = reply["result"].get("structuredContent") or {}
    if reply["result"].get("isError") or not isinstance(content.get("suggestion_id"), str):
        raise ValueError(str(content.get("code", "unusable response")))
    return content["suggestion_id"], bool(content.get("replayed"))


def run(source, sink, environ):
    try:
        url, credentials = bridge.configuration(environ)
    except ValueError as failure:
        print(str(failure), file=sys.stderr)
        return 2
    opener = urllib.request.build_opener(bridge.NoRedirect)
    failed = False
    for number, line in enumerate(source, 1):
        if not line.strip():
            continue
        try:
            if len(line) > bridge.LIMIT:
                raise ValueError("line exceeds 128 KiB")
            identifier, replayed = suggest(opener, url, credentials, number, json.loads(line))
            sink.write(f"{number}\t{'waiting' if replayed else 'added'}\t{identifier}\n")
        except (UnicodeError, ValueError, TypeError, KeyError, AttributeError) as failure:
            failed = True
            sink.write(f"{number}\tfailed\t{failure}\n")
        sink.flush()
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(run(sys.stdin.buffer, sys.stdout, os.environ))
