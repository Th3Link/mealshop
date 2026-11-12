#!/usr/bin/env python3
"""MCP stdio bridge to one authenticated Nextcloud household; standard library only.

Set MEALSHOP_MCP_URL, MEALSHOP_MCP_USER and MEALSHOP_MCP_APP_PASSWORD in the
MCP host's protected environment. No credential is accepted as a command argument.
"""
import base64
import http.client
import ipaddress
import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request

LIMIT = 131072
MAX_RESPONSE = 4194304
MODERN = "2026-07-28"
LEGACY = "2025-11-25"


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def error(identifier, code, message):
    return {"jsonrpc": "2.0", "id": identifier, "error": {"code": code, "message": message}}


def configuration(environ):
    url = environ.get("MEALSHOP_MCP_URL", "")
    user = environ.get("MEALSHOP_MCP_USER", "")
    password = environ.get("MEALSHOP_MCP_APP_PASSWORD", "")
    parsed = urllib.parse.urlsplit(url)
    try:
        local = ipaddress.ip_address(parsed.hostname or "").is_loopback
    except ValueError:
        local = parsed.hostname == "localhost"
    if (parsed.scheme != "https" and not (parsed.scheme == "http" and local)
            or not parsed.hostname or parsed.username or parsed.password
            or parsed.query or parsed.fragment or not parsed.path.endswith("/api/household/mcp")
            or not user or ":" in user or not password
            or any(ord(c) < 32 or ord(c) == 127 for c in url + user + password)):
        raise ValueError("Set a valid HTTPS household MCP URL, username and app password in the protected environment; HTTP is allowed only on loopback")
    credentials = base64.b64encode(f"{user}:{password}".encode()).decode()
    return url, credentials


def forward(opener, url, credentials, message, legacy):
    params = message.get("params") or {}
    meta = params.get("_meta") or {}
    version = meta.get("io.modelcontextprotocol/protocolVersion", legacy)
    headers = {"Authorization": "Basic " + credentials, "Content-Type": "application/json",
               "Accept": "application/json, text/event-stream", "X-Mealshop-Request": "1"}
    if message.get("method") != "initialize" or version == MODERN:
        headers["MCP-Protocol-Version"] = version
    if version == MODERN:
        method = message.get("method", "")
        name = params.get("name")
        if not isinstance(method, str) or not method.isascii() or any(ord(c) < 32 for c in method):
            return error(message.get("id"), -32600, "Invalid method"), legacy
        headers["Mcp-Method"] = method
        if isinstance(name, str):
            try:
                visible = name.isascii() and name.strip() == name and all(32 <= ord(c) <= 126 for c in name)
                headers["Mcp-Name"] = name if visible and not (name.startswith("=?base64?") and name.endswith("?=")) else "=?base64?" + base64.b64encode(name.encode()).decode() + "?="
            except UnicodeError:
                return error(message.get("id"), -32602, "Invalid tool name"), legacy
    body = json.dumps(message, ensure_ascii=False, separators=(",", ":")).encode()
    request = urllib.request.Request(url, data=body, headers=headers, method="POST")
    try:
        response = opener.open(request, timeout=30)
    except urllib.error.HTTPError as failure:
        response = failure
    except (OSError, ValueError):
        return error(message.get("id"), -32603, "Nextcloud connection failed; retry confirmed writes only with the original preview ID and idempotency key"), legacy
    with response:
        if 300 <= response.code < 400:
            return error(message.get("id"), -32603, "Nextcloud redirects are disabled; configure the exact MCP endpoint"), legacy
        try:
            raw = response.read(MAX_RESPONSE + 1)
        except (OSError, http.client.HTTPException):
            return error(message.get("id"), -32603, "Nextcloud response was interrupted; retry confirmed writes only with the original key"), legacy
        if response.code == 202 and "id" not in message:
            return None, legacy
        if len(raw) <= MAX_RESPONSE:
            try:
                value = json.loads(raw)
                if (isinstance(value, dict) and value.get("jsonrpc") == "2.0" and ("result" in value or "error" in value)
                        and (value.get("id") == message.get("id") or ("error" in value and value.get("id") is None))):
                    if message.get("method") == "initialize" and isinstance(value.get("result"), dict):
                        legacy = value["result"].get("protocolVersion", LEGACY)
                    return value, legacy
            except (UnicodeError, ValueError):
                pass
        return error(message.get("id"), -32603, f"Nextcloud returned an unusable MCP response (HTTP {response.code}); check URL, app password and granted household scope"), legacy


def run(source, sink, environ):
    try:
        url, credentials = configuration(environ)
    except ValueError as failure:
        print(str(failure), file=sys.stderr)
        return 2
    opener = urllib.request.build_opener(NoRedirect)
    version = LEGACY
    while True:
        line = source.readline(LIMIT + 1)
        if not line:
            return 0
        if len(line) > LIMIT:
            while line and not line.endswith(b"\n"):
                line = source.readline(LIMIT + 1)
            value = error(None, -32600, "MCP request exceeds 128 KiB")
        else:
            try:
                message = json.loads(line)
                if not isinstance(message, dict):
                    raise ValueError()
                value, version = forward(opener, url, credentials, message, version)
            except (UnicodeError, ValueError, TypeError, AttributeError):
                value = error(None, -32700, "Invalid JSON-RPC request")
        if value is not None:
            sink.write(json.dumps(value, ensure_ascii=False, separators=(",", ":")) + "\n")
            sink.flush()


if __name__ == "__main__":
    try:
        raise SystemExit(run(sys.stdin.buffer, sys.stdout, os.environ))
    except BrokenPipeError:
        raise SystemExit(0) from None
