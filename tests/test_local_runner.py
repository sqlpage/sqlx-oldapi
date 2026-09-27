"""Check local test isolation and cleanup without running databases or Cargo."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class LocalRunnerTests(unittest.TestCase):
    def run_backend(self, backend, cargo_status=0, docker_status=0):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            log = root / "commands.jsonl"
            program = f"#!{sys.executable}\n" + '''
import json
import os
from pathlib import Path
import sys

name = Path(sys.argv[0]).name
with open(os.environ["COMMAND_LOG"], "a") as log:
    log.write(json.dumps([name, sys.argv[1:], os.environ.get("DATABASE_URL")]) + "\\n")
if name == "cargo":
    sys.exit(int(os.environ["CARGO_STATUS"]))
if sys.argv[1] == "compose":
    if int(os.environ["DOCKER_STATUS"]):
        sys.exit(int(os.environ["DOCKER_STATUS"]))
    print("owned-container")
elif sys.argv[1] == "port":
    print("127.0.0.1:49123")
elif sys.argv[1] == "inspect":
    print("true")
'''
            for name in ("docker", "cargo"):
                tool = root / name
                tool.write_text(program)
                tool.chmod(0o755)
            # A user's saved DSN must neither be overwritten nor selected.
            config = root / ".odbc.ini"
            config.write_text("[SNOWFLAKE]\nDatabase=production\n")
            env = dict(os.environ, PATH=f"{root}{os.pathsep}{os.environ['PATH']}",
                       COMMAND_LOG=str(log), CARGO_STATUS=str(cargo_status),
                       DOCKER_STATUS=str(docker_status), ODBCINI=str(config),
                       DATABASE_URL="DSN=SNOWFLAKE")
            result = subprocess.run(
                ["bash", str(Path(__file__).resolve().parent.parent / "test.sh"), backend],
                cwd=root, env=env, text=True, capture_output=True, timeout=10,
            )
            self.assertEqual(config.read_text(), "[SNOWFLAKE]\nDatabase=production\n")
            commands = [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
            return result, commands

    def test_database_backends_use_owned_containers_and_random_loopback_ports(self):
        for backend in ("postgres", "mysql", "mssql", "odbc"):
            with self.subTest(backend=backend):
                result, commands = self.run_backend(backend)
                self.assertEqual(result.returncode, 0, result.stderr)
                compose = next(args for name, args, _ in commands if name == "docker" and args[0] == "compose")
                self.assertIn("-d", compose)
                self.assertIn("--rm", compose)
                self.assertTrue(compose[compose.index("-p", compose.index("run")) + 1].startswith("127.0.0.1::"))
                cargo = next(command for command in commands if command[0] == "cargo")
                self.assertIn("--locked", cargo[1])
                self.assertIn("49123", cargo[2])
                self.assertNotIn("DSN=", cargo[2])
                self.assertEqual(commands[-1][1][-2:], ["down", "--remove-orphans"])

    def test_test_failure_is_preserved_after_cleanup(self):
        result, commands = self.run_backend("odbc", cargo_status=17)
        self.assertEqual(result.returncode, 17)
        self.assertEqual(commands[-1][1][-2:], ["down", "--remove-orphans"])

    def test_failed_container_start_never_runs_tests(self):
        result, commands = self.run_backend("postgres", docker_status=19)
        self.assertEqual(result.returncode, 19)
        self.assertFalse(any(name == "cargo" for name, _, _ in commands))

    def test_sqlite_does_not_start_docker(self):
        result, commands = self.run_backend("sqlite")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([name for name, _, _ in commands], ["cargo"])
        self.assertTrue(commands[0][2].startswith("sqlite:"))
        self.assertFalse(Path(commands[0][2].removeprefix("sqlite://")).exists())

    def test_invalid_backend_does_not_start_anything(self):
        result, commands = self.run_backend("production")
        self.assertEqual(result.returncode, 2)
        self.assertEqual(commands, [])


if __name__ == "__main__":
    unittest.main()
