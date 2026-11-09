#!/usr/bin/env python3
"""Smoke-test the built ExApp in isolated, rootless TCP and Unix-socket containers.

The mock supplies only the OCS lifecycle registration contract. Real Nextcloud compatibility
is verified separately by the Playwright suite; this test never changes a Nextcloud instance.
"""
import argparse
import base64
import json
import os
import secrets
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--image", default="mealshop.local/mealshop:0.1.0")
args = parser.parse_args()
network = f"mealshop-smoke-{os.getpid()}"
containers = []
secret = secrets.token_hex(32)


def docker(*arguments):
    result = subprocess.run(["docker", *arguments], capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(f"Docker {arguments[0]} failed: {result.stderr.strip()}")
    return result.stdout.strip()


def request(container, unix, path, *, user=None, method="GET", data=None, marker=True, mime="application/json"):
    command = ["exec", container, "curl", "--silent", "--show-error", "--max-time", "10", "-X", method, "-w", "\n%{http_code}"]
    if unix:
        command += ["--unix-socket", "/tmp/exapp.sock"]
    if user is not None:
        auth = base64.b64encode(f"{user}:{secret}".encode()).decode()
        command += ["-H", "EX-APP-ID: mealshop", "-H", f"AUTHORIZATION-APP-API: {auth}"]
    if marker:
        command += ["-H", "X-Mealshop-Request: 1"]
    if data is not None:
        command += ["-H", f"Content-Type: {mime}", "--data", json.dumps(data)]
    command += [f"http://127.0.0.1:{3339 if not unix else 80}{path}"]
    body, status = docker(*command).rsplit("\n", 1)
    return int(status), body


try:
    docker("network", "create", network)
    mock = f"{network}-ocs"
    script = "require('http').createServer((q,r)=>{r.writeHead(200,{'Content-Type':'application/json'});r.end(JSON.stringify({ocs:{meta:{statuscode:100},data:false}}))}).listen(8080,'0.0.0.0')"
    docker("run", "-d", "--name", mock, "--network", network, "--network-alias", "ocs", "node:24-alpine", "node", "-e", script)
    containers.append(mock)
    for unix in (False, True):
        name = f"{network}-{'unix' if unix else 'tcp'}"
        command = ["run", "-d", "--name", name, "--network", network, "--user", "1000:1000", "--read-only", "--cap-drop", "ALL", "--security-opt", "no-new-privileges:true", "--tmpfs", "/tmp:mode=1777", "--tmpfs", "/data:uid=1000,gid=1000,mode=700"]
        environment = {"APP_SECRET": secret, "APP_ID": "mealshop", "APP_HOST": "0.0.0.0", "APP_PORT": "3339", "NEXTCLOUD_URL": "http://ocs:8080", "APP_PERSISTENT_STORAGE": "/data"}
        if unix:
            environment.update(HP_SHARED_KEY=secret, HP_FRP_ADDRESS="ocs", HP_FRP_PORT="8782")
        for key, value in environment.items():
            command += ["-e", f"{key}={value}"]
        command += [args.image]
        docker(*command)
        containers.append(name)
        for attempt in range(40):
            try:
                if request(name, unix, "/heartbeat")[0] == 200:
                    break
            except RuntimeError:
                pass
            time.sleep(0.2)
        else:
            raise RuntimeError("Container did not become ready; inspect its lifecycle startup")
        assert request(name, unix, "/api/households")[0] == 401
        assert request(name, unix, "/enabled?enabled=1", user="", method="PUT")[0] == 200
        assert request(name, unix, "/api/households", user="alice", method="POST", data={"name":"Rejected CSRF"}, marker=False)[0] == 403
        assert request(name, unix, "/api/households", user="alice", method="POST", data={"name":"Rejected non-JSON"}, mime="text/plain")[0] == 415
        status, body = request(name, unix, "/api/households", user="alice", method="POST", data={"name":"Container verification"})
        assert status == 200 and json.loads(body)["id"].startswith("home-")
        status, body = request(name, unix, "/js/mealshop.js", user="alice")
        assert status == 200 and len(body) > 100000
        if unix:
            assert docker("exec", name, "stat", "-c", "%a", "/tmp/frpc.toml") == "600"
        print(f"PASS: {'Unix socket' if unix else 'TCP'}; rootless lifecycle, assets, auth, CSRF and private storage")
finally:
    for container in reversed(containers):
        subprocess.run(["docker", "stop", "--time", "10", container], capture_output=True)
        subprocess.run(["docker", "rm", container], capture_output=True)
    subprocess.run(["docker", "network", "rm", network], capture_output=True)
