"""Exercise readiness retries and failure reporting without a Docker daemon."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class DatabaseReadinessTests(unittest.TestCase):
    def run_wait(self, behavior, driver="postgres", wait_seconds="2"):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docker = root / "docker"
            docker.write_text(
                f"#!{sys.executable}\n" + '''
import os
from pathlib import Path
import sys
import time

state = Path(os.environ["PROBE_STATE"])
mode = os.environ["PROBE_BEHAVIOR"]
command = sys.argv[1]
if command == "exec":
    attempts = int(state.read_text()) if state.exists() else 0
    state.write_text(str(attempts + 1))
    if mode == "hang":
        time.sleep(60)
    sys.exit(0 if mode == "ready" or (mode == "retry" and attempts > 0) else 1)
if command == "inspect":
    print("false" if mode == "stopped" else "true")
if command == "logs":
    print("database startup diagnostics")
'''
            )
            docker.chmod(0o755)
            env = dict(os.environ, PATH=f"{root}{os.pathsep}{os.environ['PATH']}",
                       PROBE_STATE=str(root / "attempts"), PROBE_BEHAVIOR=behavior)
            result = subprocess.run(
                ["bash", str(Path(__file__).with_name("wait-for-db.sh")),
                 "test-container", driver, wait_seconds],
                env=env, text=True, capture_output=True, timeout=8,
            )
            attempts = int((root / "attempts").read_text()) if (root / "attempts").exists() else 0
            return result, attempts

    def test_supported_drivers_succeed_when_ready(self):
        for driver in ("postgres", "mysql", "mssql"):
            with self.subTest(driver=driver):
                result, attempts = self.run_wait("ready", driver)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(attempts, 1)

    def test_retries_until_schema_is_ready(self):
        result, attempts = self.run_wait("retry", wait_seconds="5")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertGreater(attempts, 1)

    def test_stopped_container_fails_with_logs(self):
        result, attempts = self.run_wait("stopped", wait_seconds="60")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(attempts, 1)
        self.assertIn("database startup diagnostics", result.stderr)

    def test_hung_probe_is_bounded_and_reports_logs(self):
        result, _ = self.run_wait("hang", wait_seconds="1")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("database startup diagnostics", result.stderr)

    def test_invalid_arguments_never_probe_database(self):
        for driver, wait in (("unknown", "2"), ("postgres", "0")):
            with self.subTest(driver=driver, wait=wait):
                result, attempts = self.run_wait("ready", driver, wait)
                self.assertEqual(result.returncode, 2)
                self.assertEqual(attempts, 0)


if __name__ == "__main__":
    unittest.main()
