"""Exercise release validation and real Helm rendering without contacting a cluster."""
import base64
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

import yaml

ROOT = Path(__file__).resolve().parents[1]
CHART = ROOT / "charts/mealshop"
spec = importlib.util.spec_from_file_location("release", ROOT / "scripts/release.py")
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)
VALUES = {"nextcloud": {"url": "https://cloud.example.org/nextcloud/"},
          "appApi": {"existingSecret": "synthetic-installation"}}


def render(values=None, *, chart=CHART, name="mealshop", success=True):
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "values.json"
        path.write_text(json.dumps(VALUES if values is None else values))
        result = subprocess.run(["helm", "template", name, str(chart), "--namespace", "cloud",
                                 "--values", str(path)], capture_output=True, text=True)
    if not success:
        if result.returncode == 0:
            raise AssertionError("Invalid chart configuration was accepted")
        return None
    if result.returncode:
        raise AssertionError(result.stderr)
    return [value for value in yaml.safe_load_all(result.stdout) if value]


def resource(documents, kind):
    return next(document for document in documents if document["kind"] == kind)


class ReleaseTests(unittest.TestCase):
    def test_source_versions_and_routes_match(self):
        version = release.validate(ROOT)
        self.assertEqual(release.validate(ROOT, "v" + version), version)

    def test_noncanonical_and_mismatched_tags_rejected(self):
        for tag in ["0.1.0", "v00.1.0", "v0.1.0-rc.1", "v0.1.0+build", "v9.9.9", "v0.1.0\n"]:
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                release.validate(ROOT, tag)

    def test_route_drift_and_version_drift_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ["Cargo.toml", "Cargo.lock", "appinfo/info.xml", "charts/mealshop/Chart.yaml", "charts/mealshop/files/routes.json"]:
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / name, target)
            for name in ["Cargo.lock", "appinfo/info.xml", "charts/mealshop/Chart.yaml"]:
                path = root / name
                original = path.read_text()
                path.write_text(original.replace(release.validate(ROOT), "9.9.9"))
                with self.subTest(file=name), self.assertRaises(ValueError):
                    release.validate(root)
                path.write_text(original)
            (root / "charts/mealshop/files/routes.json").write_text("[]")
            with self.assertRaises(ValueError):
                release.validate(root)

    def test_only_tag_workflow_can_publish_after_all_gates(self):
        workflow = yaml.load((ROOT / ".github/workflows/release.yml").read_text(), Loader=yaml.BaseLoader)
        self.assertEqual(list(workflow["on"]), ["push"])
        self.assertEqual(list(workflow["on"]["push"]), ["tags"])
        jobs = workflow["jobs"]
        self.assertEqual(set(jobs["images"]["needs"]), {"metadata", "quality", "security", "android"})
        self.assertEqual(set(jobs["publish"]["needs"]), {"metadata", "images"})
        self.assertEqual(workflow["permissions"], {"contents": "read"})
        for job in ["metadata", "quality", "security", "android"]:
            self.assertNotEqual(jobs[job].get("permissions", {}).get("packages"), "write")


class ChartTests(unittest.TestCase):
    def test_default_chart_lints_with_required_configuration(self):
        result = subprocess.run(["helm", "lint", str(CHART), "--strict", "--set",
                                 "nextcloud.url=https://cloud.example.org", "--set",
                                 "appApi.existingSecret=synthetic-installation"], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)

    def test_required_and_invalid_configuration_rejected(self):
        invalid = [{}, {"nextcloud": VALUES["nextcloud"]}, {"appApi": VALUES["appApi"]},
                   {**VALUES, "replicaCount": 2},
                   {**VALUES, "service": {"port": 80}},
                   {**VALUES, "image": {"digest": "not-a-digest"}},
                   {**VALUES, "nextcloud": {"url": "https://user:password@cloud.example.org"}},
                   {**VALUES, "nextcloud": {"url": "https://cloud.example.org/?token=bad"}}]
        for values in invalid:
            with self.subTest(values=values):
                render(values, success=False)

    def test_internal_service_and_one_rootless_writer(self):
        documents = render()
        deployment = resource(documents, "Deployment")["spec"]
        pod = deployment["template"]["spec"]
        container = pod["containers"][0]
        self.assertEqual(deployment["replicas"], 1)
        self.assertEqual(deployment["strategy"], {"type": "Recreate"})
        self.assertFalse(pod["automountServiceAccountToken"])
        self.assertEqual(pod["securityContext"]["runAsUser"], 1000)
        self.assertTrue(container["securityContext"]["readOnlyRootFilesystem"])
        self.assertFalse(container["securityContext"]["allowPrivilegeEscalation"])
        self.assertEqual(container["securityContext"]["capabilities"]["drop"], ["ALL"])
        self.assertEqual(resource(documents, "Service")["spec"]["type"], "ClusterIP")
        self.assertFalse(any(d["kind"] in ["Secret", "Ingress", "Role", "ClusterRole"] for d in documents))
        self.assertEqual(pod["volumes"][1]["name"], "tmp")
        self.assertEqual(container["readinessProbe"]["httpGet"]["path"], "/heartbeat")

    def test_retained_pvc_and_existing_claim(self):
        pvc = resource(render(), "PersistentVolumeClaim")
        self.assertEqual(pvc["metadata"]["annotations"]["helm.sh/resource-policy"], "keep")
        self.assertNotIn("storageClassName", pvc["spec"])
        values = {**VALUES, "persistence": {"existingClaim": "existing-data"}}
        documents = render(values)
        self.assertFalse(any(d["kind"] == "PersistentVolumeClaim" for d in documents))
        pod = resource(documents, "Deployment")["spec"]["template"]["spec"]
        self.assertEqual(pod["volumes"][0]["persistentVolumeClaim"]["claimName"], "existing-data")

    def test_pvc_retention_cannot_be_overridden(self):
        values = {**VALUES, "persistence": {"annotations": {"helm.sh/resource-policy": "delete", "example.org/note": "retained"}}}
        pvc = resource(render(values), "PersistentVolumeClaim")
        self.assertEqual(pvc["metadata"]["annotations"], {"helm.sh/resource-policy": "keep", "example.org/note": "retained"})

    def test_image_digest_pull_secret_and_longhorn_configuration(self):
        digest = "sha256:" + "a" * 64
        values = {**VALUES, "image": {"tag": "ignored", "digest": digest},
                  "imagePullSecrets": [{"name": "registry-auth"}],
                  "persistence": {"storageClass": "longhorn", "labels": {"recurring-job.longhorn.io/source": "enabled"}}}
        documents = render(values)
        pod = resource(documents, "Deployment")["spec"]["template"]["spec"]
        self.assertEqual(pod["containers"][0]["image"], "ghcr.io/th3link/mealshop@" + digest)
        self.assertEqual(pod["imagePullSecrets"], [{"name": "registry-auth"}])
        self.assertEqual(resource(documents, "PersistentVolumeClaim")["spec"]["storageClassName"], "longhorn")

    def test_url_port_and_registration_contract(self):
        documents = render({**VALUES, "service": {"port": 23456}})
        pod = resource(documents, "Deployment")["spec"]["template"]["spec"]
        env = {e["name"]: e for e in pod["containers"][0]["env"]}
        template = json.loads(resource(documents, "ConfigMap")["data"]["registration.json"])
        self.assertEqual(env["NEXTCLOUD_URL"]["value"], "https://cloud.example.org/nextcloud")
        self.assertNotIn("HP_SHARED_KEY", env)
        self.assertNotIn("secret", template)
        self.assertEqual(env["APP_PORT"]["value"], str(template["port"]))
        self.assertEqual(template["port"], 23456)
        self.assertEqual(template["routes"], json.loads((CHART / "files/routes.json").read_text()))
        self.assertEqual(env["APP_SECRET"]["valueFrom"]["secretKeyRef"]["name"], "synthetic-installation")

    def test_releases_have_distinct_workload_and_storage_names(self):
        first, second = render(name="first"), render(name="second")
        for kind in ["Deployment", "Service", "PersistentVolumeClaim"]:
            self.assertNotEqual(resource(first, kind)["metadata"]["name"], resource(second, kind)["metadata"]["name"])

    def test_packaged_chart_retains_its_registration_files(self):
        with tempfile.TemporaryDirectory() as directory:
            packaged = subprocess.run(["helm", "package", str(CHART), "--destination", directory], capture_output=True, text=True)
            self.assertEqual(packaged.returncode, 0, packaged.stderr)
            documents = render(chart=next(Path(directory).glob("*.tgz")))
            self.assertTrue(json.loads(resource(documents, "ConfigMap")["data"]["registration.json"])["routes"])

    def test_registration_helper_validates_secret_without_error_disclosure(self):
        template = resource(render(), "ConfigMap")["data"]["registration.json"]
        secret = "synthetic-registration-secret-0001"
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "public.json"
            path.write_text(template)
            for value, valid in [(secret, True), ("bad-\u00fc-secret", False), ("too-short", False)]:
                encoded = base64.b64encode(value.encode()).decode()
                result = subprocess.run(["python3", str(CHART / "files/registration.py"), str(path)],
                                        input=json.dumps({"data": {"APP_SECRET": encoded}}), capture_output=True, text=True)
                self.assertEqual(result.returncode == 0, valid)
                if valid:
                    self.assertEqual(json.loads(result.stdout)["secret"], value)
                else:
                    self.assertNotIn(value, result.stderr + result.stdout)
                    self.assertNotIn(encoded, result.stderr + result.stdout)


if __name__ == "__main__":
    unittest.main()
