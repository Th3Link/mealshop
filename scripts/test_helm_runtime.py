#!/usr/bin/env python3
"""Verify a chart in a fresh disposable Kind cluster with a synthetic OCS service.

Every kubectl/Helm call uses the private kubeconfig created here. Existing kubeconfigs,
Nextcloud instances, clusters and data are never used. Only this script's cluster is removed.
"""
import argparse
import base64
import json
import os
from pathlib import Path
import secrets
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--image", required=True, help="Already-built local image")
    args = parser.parse_args()
    cluster = "mealshop-helm-" + secrets.token_hex(4)
    secret = secrets.token_hex(32)
    with tempfile.TemporaryDirectory(prefix="mealshop-helm-") as temporary:
        directory = Path(temporary)
        environment = {**os.environ, "KUBECONFIG": str(directory / "kubeconfig"),
                       "HELM_PLUGINS": str(directory / "plugins")}

        def run(*command, input=None):
            result = subprocess.run(command, input=input, env=environment, capture_output=True, text=True, timeout=600)
            if result.returncode:
                raise RuntimeError(f"{command[0]} {command[1]} failed: {result.stderr[:2000]}")
            return result.stdout

        try:
            print("Creating an isolated Kind cluster with a private kubeconfig", flush=True)
            run("kind", "create", "cluster", "--name", cluster, "--kubeconfig", environment["KUBECONFIG"], "--wait", "120s")
            run("docker", "pull", "node:24-alpine")
            run("kind", "load", "docker-image", "--name", cluster, args.image, "node:24-alpine")
            mock = "require('http').createServer((q,r)=>{r.writeHead(200,{'Content-Type':'application/json'});r.end(JSON.stringify({ocs:{meta:{statuscode:100},data:q.url.includes('/ex-app/state')?true:false}}))}).listen(8080,'0.0.0.0')"
            objects = [
                {"apiVersion": "v1", "kind": "Secret", "metadata": {"name": "app-secret"},
                 "stringData": {"APP_SECRET": secret}},
                {"apiVersion": "v1", "kind": "Service", "metadata": {"name": "ocs"},
                 "spec": {"selector": {"app": "ocs"}, "ports": [{"port": 8080}]}},
                {"apiVersion": "apps/v1", "kind": "Deployment", "metadata": {"name": "ocs"},
                 "spec": {"selector": {"matchLabels": {"app": "ocs"}}, "template": {
                     "metadata": {"labels": {"app": "ocs"}}, "spec": {"containers": [
                         {"name": "ocs", "image": "node:24-alpine", "imagePullPolicy": "Never",
                          "command": ["node", "-e", mock]}]}}}},
            ]
            run("kubectl", "apply", "-f", "-", input=json.dumps({"apiVersion": "v1", "kind": "List", "items": objects}))
            run("kubectl", "rollout", "status", "deployment/ocs", "--timeout=120s")
            repository, tag = args.image.rsplit(":", 1)
            values = {"image": {"repository": repository, "tag": tag, "pullPolicy": "Never"},
                      "nextcloud": {"url": "http://ocs:8080"}, "appApi": {"existingSecret": "app-secret"},
                      "persistence": {"size": "1Gi"}}
            path = directory / "values.json"
            path.write_text(json.dumps(values))
            chart = str(ROOT / "charts/mealshop")
            print("Installing chart against the synthetic OCS service", flush=True)
            run("helm", "install", "mealshop", chart, "--values", str(path), "--wait", "--timeout", "180s")

            def request(route, *, method="GET", user=None, body=None, marker=True):
                command = ["kubectl", "exec", "deployment/mealshop", "-c", "mealshop", "--", "curl",
                           "--silent", "--show-error", "--max-time", "10", "-X", method, "-w", "\n%{http_code}"]
                if user is not None:
                    auth = base64.b64encode(f"{user}:{secret}".encode()).decode()
                    command += ["-H", "EX-APP-ID: mealshop", "-H", "AUTHORIZATION-APP-API: " + auth]
                if marker:
                    command += ["-H", "X-Mealshop-Request: 1"]
                if body is not None:
                    command += ["-H", "Content-Type: application/json", "--data", json.dumps(body)]
                command += ["http://mealshop:23000" + route]
                response, status = run(*command).rsplit("\n", 1)
                return int(status), response

            assert request("/heartbeat")[0] == 200
            assert request("/api/households")[0] == 401
            assert request("/enabled?enabled=1", method="PUT", user="")[0] == 200
            assert request("/api/households", method="POST", user="synthetic", body={"name": "CSRF fixture"}, marker=False)[0] == 403
            status, response = request("/api/households", method="POST", user="synthetic", body={"name": "Chart persistence fixture"})
            assert status == 200
            identity = json.loads(response)["id"]
            assert run("kubectl", "exec", "deployment/mealshop", "--", "id", "-u").strip() == "1000"
            config = json.loads(run("kubectl", "get", "configmap", "mealshop-appapi", "-o", "json"))
            registration = json.loads(config["data"]["registration.json"])
            assert "secret" not in registration and registration["port"] == 23000
            before = run("kubectl", "get", "pods", "-l", "app.kubernetes.io/instance=mealshop", "-o", "jsonpath={.items[0].metadata.uid}")
            values["resources"] = {"requests": {"cpu": "150m"}}
            path.write_text(json.dumps(values))
            print("Verifying a real writer replacement and persistent household", flush=True)
            run("helm", "upgrade", "mealshop", chart, "--values", str(path), "--wait", "--timeout", "180s")
            after = run("kubectl", "get", "pods", "-l", "app.kubernetes.io/instance=mealshop", "-o", "jsonpath={.items[0].metadata.uid}")
            assert before != after, "Upgrade must actually replace the writer"
            status, response = request("/api/households", user="synthetic")
            assert status == 200 and any(h["id"] == identity for h in json.loads(response)["households"])
            run("helm", "uninstall", "mealshop", "--wait")
            assert run("kubectl", "get", "pvc", "mealshop-data", "-o", "jsonpath={.status.phase}") == "Bound"
            print("PASS: isolated Helm install, rootless/auth/CSRF checks, real Recreate upgrade, persisted household and retained PVC")
        finally:
            subprocess.run(["kind", "delete", "cluster", "--name", cluster, "--kubeconfig", environment["KUBECONFIG"]],
                           env=environment, capture_output=True, text=True, timeout=60)


if __name__ == "__main__":
    main()
