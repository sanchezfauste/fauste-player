"""Tests for scripts/package/snap-build.sh: telling a Snap Store outage
(exit 75) from a real failure, with a fake sudo on PATH."""

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "snap-build.sh"
OUTAGE = 75

STORE_DOWN = "error: unable to contact snap store\n"
TIMEOUT = (
    'error: cannot perform the following tasks:\n- Fetch and check assertions for snap "snapd" '
    '(28332) (Get "https://api.snapcraft.io/v2/assertions/snap-declaration/16/x": net/http: '
    "request canceled while waiting for connection (Client.Timeout exceeded while awaiting headers))\n"
)

# Fails `snap install <name>` the first FAILS times with MESSAGE, counting
# the attempts in COUNT; every other command succeeds.
FAKE_SUDO = """#!/usr/bin/env bash
if [ "$1" = snap ] && [ "$2" = install ]; then
  n=$(( $(cat "$COUNT" 2>/dev/null || echo 0) + 1 ))
  echo "$n" > "$COUNT"
  if [ "$n" -le "$FAILS" ] && [ -n "$HANG" ]; then sleep 5; fi
  if [ "$n" -le "$FAILS" ]; then printf '%s' "$MESSAGE" >&2; exit 1; fi
fi
exit 0
"""


class StoreErrorTest(unittest.TestCase):
    def classify(self, text):
        with tempfile.NamedTemporaryFile("w", suffix=".log", delete=False) as f:
            f.write(text)
        self.addCleanup(os.unlink, f.name)
        return subprocess.run([SCRIPT, "store-error", f.name]).returncode

    def test_an_unreachable_store_is_a_store_error(self):
        self.assertEqual(self.classify("Installing...\n" + STORE_DOWN), 0)

    def test_a_store_timeout_is_a_store_error(self):
        self.assertEqual(self.classify(TIMEOUT), 0)

    def test_a_compile_error_is_not(self):
        log = "error[E0425]: cannot find value `x` in this scope\nFailed to build 'fauste-player'.\n"
        self.assertEqual(self.classify(log), 1)

    def test_only_the_end_of_the_log_counts(self):
        # A store hiccup early on that snapcraft recovered from does not
        # excuse a compile error at the end.
        log = STORE_DOWN + "ok\n" * 200 + "error: could not compile `fp-app`\n"
        self.assertEqual(self.classify(log), 1)


class SetupTest(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.dir = Path(tmp.name)
        bin_dir = self.dir / "bin"
        bin_dir.mkdir()
        sudo = bin_dir / "sudo"
        sudo.write_text(FAKE_SUDO)
        sudo.chmod(0o755)
        self.count = self.dir / "count"
        self.env = dict(
            os.environ,
            PATH=f"{bin_dir}:{os.environ['PATH']}",
            COUNT=str(self.count),
            SNAP_BIN_DIR=str(self.dir / "snap-bin"),  # empty: nothing installed
            SNAP_STORE_ATTEMPTS="3",
            SNAP_STORE_DELAY="0",
        )

    def setup(self, fails, message, **extra):
        env = dict(self.env, FAILS=str(fails), MESSAGE=message, **extra)
        status = subprocess.run(
            [SCRIPT, "setup"], env=env, capture_output=True, text=True
        ).returncode
        return status, int(self.count.read_text())

    def test_a_short_outage_is_retried(self):
        # lxd fails twice, then lxd and snapcraft install.
        self.assertEqual(self.setup(2, STORE_DOWN), (0, 4))

    def test_a_long_outage_exits_75(self):
        self.assertEqual(self.setup(99, TIMEOUT), (OUTAGE, 3))

    def test_a_stalled_install_counts_as_an_outage(self):
        status = self.setup(1, "", HANG="1", SNAP_STORE_TIMEOUT="1")
        self.assertEqual(status, (0, 3))

    def test_another_failure_is_not_retried(self):
        self.assertEqual(self.setup(99, "error: snap \"lxd\" has bad plugs\n"), (1, 1))


if __name__ == "__main__":
    unittest.main()
