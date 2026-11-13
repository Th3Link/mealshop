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
    if path.endswith('/api/household/mcp') and data is not None:
        command += ['-H', 'Accept: application/json, text/event-stream', '-H', 'MCP-Protocol-Version: 2026-07-28', '-H', f'Mcp-Method: {data["method"]}']
        if data['method'] == 'tools/call':
            command += ['-H', f'Mcp-Name: {data["params"]["name"]}']
    if data is not None:
        command += ["-H", f"Content-Type: {mime}", "--data", json.dumps(data)]
    command += [f"http://127.0.0.1:{3339 if not unix else 80}{path}"]
    body, status = docker(*command).rsplit("\n", 1)
    return int(status), body


def verify_brewing(container, unix, prefix, home):
    root = prefix + "/api/household/brewing"
    def get(suffix=""):
        status, body = request(container, unix, root + suffix, user="alice")
        assert status == 200, body
        return json.loads(body)
    def change(command, key):
        body = {"expected_revision":get()["revision"],"idempotency_key":key,"reviewed":True,"command":command}
        status, result = request(container, unix, root + "/preview", user="alice", method="POST", data=body)
        assert status == 200, result
        status, result = request(container, unix, root, user="alice", method="POST", data=body)
        assert status == 200, result
        return body, json.loads(result)
    catalogue = get("/templates")
    assert len(catalogue["templates"]) == 12
    template = next(t for t in catalogue["templates"] if t["id"] == "biab")
    phase = catalogue["phases"][template["phases"][0]]
    stage = {"id":"water","origin":{"kind":"custom","note":"Synthetic smoke phase"},"name":phase["name"],"instruction":phase["instruction"],"dependencies":[],"active_seconds":0,"passive_seconds":0,"passive_min_seconds":0,"passive_max_seconds":0,"equipment":[],"lead_seconds":0,"repeat":1,"wait_between_seconds":0,"uncertainty":"Declared smoke fixture"}
    recipe = {"beverage":"beer","title":"Container brewing recipe","language":"en","volume":{"amount":"0.100001","unit":"l"},"template":{"id":"biab","version":1,"sources":[s for s in catalogue["sources"] if s["id"] in template["sources"]]},"method":{"strategy":"all_grain","layout":"bag","heat":"manual","sparge":"none","mash":"infusion"},"ingredients":[],"phases":[{"kind":"water","timing_known":False,"stage":stage,"temperature":None,"volume":None}],"notes":"Original synthetic process"}
    _, result = change({"operation":"save_recipe","id":None,"expected_recipe_revision":None,"recipe":recipe},"container-brewing-recipe-0001")
    recipe_id = result["brewing"]["recipes"][0]["record"]["id"]
    _, result = change({"operation":"save_equipment","id":None,"equipment":{"name":"Container brewing vessel","kind":"kettle","model":"Synthetic vessel","capacity":{"amount":"1.000001","unit":"l"},"notes":"Synthetic only"}},"container-brewing-equipment-0001")
    equipment = result["brewing"]["equipment"][0]["id"]
    original, result = change({"operation":"create_batch","draft":{"name":"Container brewing batch","recipe_id":recipe_id,"recipe_revision":1,"brew_day":{"local":"2030-01-15T08:00:00","timezone":"UTC","fold":"reject"},"yeast_preparation":{"choice":"skip"},"equipment":{"water":[equipment]}}},"container-brewing-batch-0001")
    run = result["brewing"]["batches"][0]["run"]
    assert "recipe" not in run and "servings" not in run and run["brew_recipe"]["volume"]["amount"] == "0.100001"
    status, body = request(container, unix, root, user="alice", method="POST", data=original)
    assert status == 200 and json.loads(body)["replayed"] and len(json.loads(body)["brewing"]["batches"]) == 1, body
    assert request(container, unix, root, user="bob", method="POST", data=original)[0] == 403
    reading = {"phase_id":"water-1","measurement":{"value":"1.050001","unit":"sg"},"measured_at":{"local":"2026-11-01T08:00:00","timezone":"UTC","fold":"reject"},"source":{"kind":"manual"},"calibration":{"kind":"unknown"},"notes":"Synthetic transcription"}
    change({"operation":"add_reading","run_id":run["id"],"reading":reading,"corrects":None,"reason":None},"container-brewing-reading-0001")
    detail = get("/batches/"+run["id"])
    assert detail["measurements"]["readings"][0]["input"] == reading
    assert docker("exec",container,"stat","-c","%a",f"/data/{home}/household/cooking.json") == "600"

try:
    docker("network", "create", network)
    mock = f"{network}-ocs"
    script = "require('http').createServer((q,r)=>{r.writeHead(200,{'Content-Type':'application/json'});r.end(JSON.stringify({ocs:{meta:{statuscode:100},data:q.url.startsWith('/ocs/v1.php/apps/app_api/api/v1/users')?['alice','bob']:false}}))}).listen(8080,'0.0.0.0')"
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
        home = json.loads(body)["id"]
        prefix = f"/api/households/{home}/domain"
        declaration = {"expected_revision": 0, "ingredient": {"name": "Container ingredient", "language": "en", "form": "As declared", "recorded_on": "2026-11-09"}}
        status, body = request(name, unix, prefix + "/api/household/ingredients", user="alice", method="POST", data=declaration)
        assert status == 201
        ingredient = json.loads(body)["id"]
        recipe = {"title": "Container recipe", "language": "en", "servings": 1, "preparation": "as_recorded", "lines": [{"ingredient_id": ingredient, "grams": "1"}], "steps": [{"instruction":"Prepare container fixture","active_seconds":1,"passive_seconds":60}]}
        status, body = request(name, unix, prefix + "/api/recipes", user="alice", method="POST", data=recipe)
        assert status == 201, body
        recipe_id = json.loads(body)["id"]
        assert docker("exec", name, "stat", "-c", "%a", f"/data/{home}/household/ingredients.json") == "600"
        shopping = prefix + "/api/household/shopping"
        def shopping_change(revision, command):
            status, body = request(name, unix, shopping, user="alice", method="POST", data={"expected_revision":revision,"command":command})
            assert status == 200, body
            return json.loads(body)
        shopping_change(0, {"operation":"save_shop","name":"Container shop"})
        result = shopping_change(1, {"operation":"add_manual","label":"Container ingredient","ingredient_id":ingredient,"quantity":{"amount":"0.100001","unit":"kg"}})
        assert result["items"][0]["remaining"] == "100.001000"
        result = shopping_change(2, {"operation":"check","key":result["items"][0]["key"],"checked":True})
        assert result["items"][0]["purchased"] == "100.001000"
        product = {"barcode":"034000470693","name":"Container product","brand":"","label_quantity":"100 g","quantity":{"amount":"100","unit":"g"},"ingredient_id":ingredient,"match_confirmed":True}
        shopping_change(3, {"operation":"save_product","product":product})
        result = shopping_change(4, {"operation":"add_product","barcode":product["barcode"],"packages":2})
        assert result["items"][0]["remaining"] == "200.000000"
        assert result["items"][0]["purchased"] == "100.001000"
        assert docker("exec", name, "stat", "-c", "%a", f"/data/{home}/household/shopping.json") == "600"
        status, body = request(name, unix, "/js/mealshop.js", user="alice")
        assert status == 200 and len(body) > 100000
        if unix:
            assert docker("exec", name, "stat", "-c", "%a", "/tmp/frpc.toml") == "600"
        code, body = request(name, unix, prefix + "/api/household/cooking/template?recipe_id=" + recipe_id + "&revision=1&servings=3&timezone=Europe%2FBerlin", user="alice")
        assert code == 200, body
        draft = json.loads(body)["draft"]
        code, body = request(name, unix, prefix + "/api/household/cooking/preview", user="alice", method="POST", data={"expected_revision": 0, "proposal": {"kind": "create", "draft": draft}})
        assert code == 200, body
        review = json.loads(body)
        code, body = request(name, unix, prefix + "/api/household/cooking", user="alice", method="POST", data={"expected_revision": 0, "command": {"operation": "apply", "preview_id": review["preview_id"], "reviewed": True}})
        assert code == 200, body
        cooking_id = json.loads(body)["run_id"]
        code, body = request(name, unix, prefix + "/api/household/cooking/" + cooking_id, user="alice")
        assert code == 200 and json.loads(body)["run"]["schedule"] == review["run"]["schedule"], body
        code, body = request(name, unix, prefix + "/api/household/cooking/internal", user="alice", method="POST", data={"operation": "claim"})
        assert code == 403, body
        collections = f"/api/households/{home}/collections"
        status, body = request(name, unix, collections, user="alice", method="POST", data={"expected_revision":0,"collection":{"name":"Container collection","entries":[{"recipe_id":recipe_id,"revision":1}]}})
        assert status == 200, body
        collection_id = json.loads(body)["id"]
        shares = f"/api/households/{home}/shares"
        status, body = request(name, unix, shares, user="alice", method="POST", data={"expected_revision":1,"resource":{"kind":"collection","collection_id":collection_id},"recipient":{"kind":"account","user":"bob"}})
        assert status == 200, body
        share_id = json.loads(body)["id"]
        shared = f"/api/shared/{home}/{share_id}"
        status, body = request(name, unix, shared, user="bob")
        assert status == 200 and json.loads(body)["content"]["items"][0]["content"]["title"] == "Container recipe", body
        assert request(name, unix, prefix + "/api/recipes", user="bob")[0] == 403
        assert docker("exec", name, "stat", "-c", "%a", "/data/library.json") == "600"
        imported = {**recipe,"origin":{"kind":"imported","source_url":"https://example.org/container-original"}}
        status, body = request(name, unix, prefix + "/api/recipes/" + recipe_id, user="alice", method="PUT", data={"expected_revision":1,"recipe":imported})
        assert status == 200, body
        status, body = request(name, unix, shared + "/recipes/" + recipe_id, user="bob")
        assert status == 200 and json.loads(body)["content"] == {"kind":"source_link","source_url":"https://example.org/container-original"}, body
        status, body = request(name, unix, shares + "/" + share_id, user="alice", method="DELETE", data={"expected_revision":2})
        assert status == 200, body
        assert request(name, unix, shared, user="bob")[0] == 404
        settings = prefix + '/api/household/mcp/settings'
        rpc_path = prefix + '/api/household/mcp'
        status, body = request(name, unix, settings, user='alice', method='PUT', data={'expected_revision':0,'grants':{'alice':'create'}})
        assert status == 200 and json.loads(body)['scope'] == 'create', body
        def rpc(tool, arguments):
            payload = {'jsonrpc':'2.0','id':1,'method':'tools/call','params':{'name':tool,'arguments':arguments,'_meta':{'io.modelcontextprotocol/protocolVersion':'2026-07-28','io.modelcontextprotocol/clientCapabilities':{}}}}
            status, body = request(name, unix, rpc_path, user='alice', method='POST', data=payload)
            assert status == 200, body
            return json.loads(body)['result']
        restricted = rpc('recipe_get', {'id':recipe_id,'revision':1})
        assert restricted['structuredContent']['content'] == {'kind':'source_link','source_url':'https://example.org/container-original'}, restricted
        assert request(name, unix, rpc_path, user='bob', method='POST', data={'jsonrpc':'2.0','id':1,'method':'ping'})[0] == 403
        preview = rpc('recipe_preview_create', {'recipe':{**recipe,'title':'Container MCP recipe'}})
        assert not preview['isError'] and not preview['structuredContent']['saved'], preview
        confirmed = {'preview_id':preview['structuredContent']['preview_id'],'idempotency_key':'container-mcp-request-0001','confirmed':True}
        created = rpc('recipe_create', confirmed)
        assert not created['isError'], created
        retry = rpc('recipe_create', confirmed)
        assert retry['structuredContent']['replayed'] and retry['structuredContent']['record']['id'] == created['structuredContent']['record']['id'], retry
        assert docker('exec', name, 'stat', '-c', '%a', f'/data/{home}/household/mcp.json') == '600'
        status, body = request(name, unix, settings, user='alice', method='PUT', data={'expected_revision':1,'grants':{'alice':'read'}})
        assert status == 200, body
        assert rpc('recipe_create', confirmed)['isError']
        status, body = request(name, unix, prefix + '/api/household/mcp/activity', user='alice')
        assert status == 200 and json.loads(body)['entries'][0]['actor'] == 'alice' and 'idempotency_key' not in body, body
        imports = prefix + '/api/household/imports'
        status, body = request(name, unix, imports + '/preview', user='alice', method='POST', data={'mode':'url','url':'https://127.0.0.1/private','consent':True})
        assert status == 422 and json.loads(body)['code'] == 'import_url_invalid', body
        status, body = request(name, unix, imports + '/preview', user='alice', method='POST', data={'mode':'manual','url':'https://example.org/container-import'})
        assert status == 200 and json.loads(body)['method'] == 'manual', body
        reviewed = {'preview_id':json.loads(body)['preview_id'],'candidate':0,'idempotency_key':'container-import-request-0001','reviewed':True,'recipe':{**recipe,'title':'Container import recipe','origin':{'kind':'imported','source_url':'https://example.org/container-import'}}}
        status, body = request(name, unix, imports, user='alice', method='POST', data=reviewed)
        assert status == 201, body
        inserted = json.loads(body)
        assert inserted['import_source']['requested_url'] == 'https://example.org/container-import' and 'container-import-request-0001' not in body, body
        status, body = request(name, unix, imports, user='alice', method='POST', data=reviewed)
        assert status == 201 and json.loads(body)['id'] == inserted['id'], body
        assert request(name, unix, imports, user='bob', method='POST', data=reviewed)[0] == 403
        assert docker('exec', name, 'stat', '-c', '%a', f"/data/{home}/recipes/{inserted['id']}-1.json") == '600'
        inventory = prefix + "/api/household/inventory"
        status, body = request(name, unix, inventory, user="alice")
        assert status == 200 and not json.loads(body)["lots"], body
        stock = json.loads(body)
        booking = {"expected_revision": stock["revision"], "idempotency_key":"container-stock-request-0001", "reviewed":False, "command":{"operation":"create_lot","target":{"kind":"ingredient","ingredient_id":ingredient},"quantity":{"amount":"0.100001","unit":"kg"},"metadata":{"location":"Container pantry","batch":"synthetic","expiry":None}}}
        status, body = request(name, unix, inventory + "/preview", user="alice", method="POST", data=booking)
        assert status == 200, body
        assert not json.loads(request(name, unix, inventory, user="alice")[1])["lots"]
        booking["reviewed"] = True
        status, body = request(name, unix, inventory, user="alice", method="POST", data=booking)
        assert status == 200 and json.loads(body)["inventory"]["lots"][0]["remaining"] == "100.001000", body
        status, body = request(name, unix, inventory, user="alice", method="POST", data=booking)
        assert status == 200 and json.loads(body)["replayed"] and len(json.loads(body)["inventory"]["movements"]) == 1, body
        assert request(name, unix, inventory, user="bob", method="POST", data=booking)[0] == 403
        assert "container-stock-request-0001" not in body
        history = prefix + "/api/household/history"
        learning = prefix + "/api/household/learning"
        status, body = request(name, unix, history, user="alice")
        assert status == 200 and json.loads(body)["total"] == 0, body
        profile = {"id":"person-a","name":"Container attendee","allergies":[],"intolerances":[],"excluded_ingredients":[],"disliked_ingredients":[]}
        status, body = request(name, unix, prefix + "/api/household/profiles", user="alice", method="PUT", data={"expected_revision":0,"members":[profile]})
        assert status == 200, body
        policy = {"expected_revision":0,"expected_profile_revision":1,"policy":{"enabled":True,"member_ids":["person-a"]}}
        status, body = request(name, unix, learning, user="alice", method="PUT", data=policy)
        assert status == 200 and json.loads(body)["learned_cooks"] == 0, body
        entry = {"expected_revision":1,"expected_profile_revision":1,"confirmed_completed":True,"source":{"kind":"manual","recipe_id":recipe_id,"revision":1,"servings":2},"input":{"date":"2026-11-01","timezone":"Europe/Berlin","attendees":["person-a"],"active_seconds":125,"household_rating":5,"ratings":{"person-a":4}}}
        status, body = request(name, unix, history, user="alice", method="POST", data=entry)
        assert status == 201, body
        history_id = json.loads(body)["id"]
        status, body = request(name, unix, learning, user="alice")
        assert status == 200 and json.loads(body)["learned_cooks"] == 1, body
        assert request(name, unix, history, user="bob")[0] == 403
        assert docker("exec", name, "stat", "-c", "%a", f"/data/{home}/household/history.json") == "600"
        candidate = {"id":recipe_id,"revision":1}
        suggestion = {"template":{"date":"2026-11-02","meal":"lunch","recipe":candidate,"attendees":["person-a"],"calendar":None,"servings":2,"active_seconds":600,"elapsed_seconds":600,"advance_seconds":0},"candidates":[candidate]}
        status, body = request(name, unix, prefix + "/api/household/mealplan/suggestions", user="alice", method="POST", data=suggestion)
        ranked = json.loads(body)
        assert status == 200 and ranked["model_version"] == "history-v1" and ranked["suggestions"][0]["recommendation"]["evidence"]["cooks"] == 1, body
        status, body = request(name, unix, history + "/" + history_id, user="alice", method="DELETE", data={"expected_revision":2,"confirmed":True})
        assert status == 200, body
        status, body = request(name, unix, learning, user="alice")
        assert status == 200 and json.loads(body)["learned_cooks"] == 0, body
        policy.update(expected_revision=3,policy={"enabled":False,"member_ids":[]})
        status, body = request(name, unix, learning, user="alice", method="PUT", data=policy)
        assert status == 200 and json.loads(body)["model_version"] == "rules-v1", body
        verify_brewing(name, unix, prefix, home)
        print(f"PASS: {'Unix socket' if unix else 'TCP'}; rootless lifecycle, assets, auth, CSRF, private shopping/products, cooking review, collections, sharing/redaction/revocation, completed history, preference epochs, scoped MCP, reviewed imports/retries/SSRF rejection, stock preview/booking/idempotence, source-linked brewing/batches/readings/retries and storage")
finally:
    for container in reversed(containers):
        subprocess.run(["docker", "stop", "--time", "10", container], capture_output=True)
        subprocess.run(["docker", "rm", container], capture_output=True)
    subprocess.run(["docker", "network", "rm", network], capture_output=True)
