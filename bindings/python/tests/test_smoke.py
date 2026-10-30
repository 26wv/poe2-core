import _thread
import os
import shutil
import tempfile
import threading
import unittest

LINE = "2026/01/02 03:04:05 123456789 1a2b3c4d [INFO Client 4242] : ExileOne (Smith of Kitava) is now level 12"


class SmokeTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # Never let discover() see this machine's real install: HOME is an empty temp dir and
        # POE2_*, XDG_* and WINEPREFIX are removed.
        cls.home = tempfile.mkdtemp(prefix="poe2-py-")
        os.environ["HOME"] = cls.home
        for key in ("XDG_DATA_HOME", "XDG_CONFIG_HOME", "WINEPREFIX", "POE2_INSTALL_DIR", "POE2_USER_DIR"):
            os.environ.pop(key, None)
        global poe2_core
        import poe2_core

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.home)

    def bounded(self, seconds):
        """Interrupts the main thread after `seconds`, so a blocked next() can't hang the run."""
        timer = threading.Timer(seconds, _thread.interrupt_main)
        timer.start()
        self.addCleanup(timer.cancel)

    def test_discover(self):
        self.assertEqual(
            sorted(poe2_core.discover()),
            ["build_planner", "client_log", "filters", "installs", "user_dir"],
        )

    def test_parse_line(self):
        self.assertEqual(
            poe2_core.parse_line(LINE),
            {
                "timestamp": "2026-01-02T03:04:05",
                "prefix": {
                    "ms_counter": 123456789,
                    "hex_id": "1a2b3c4d",
                    "log_level": "INFO",
                    "log_source": "Client",
                    "pid": 4242,
                },
                "event": {"type": "LevelUp", "name": "ExileOne", "class": "Smith of Kitava", "level": 12},
            },
        )
        self.assertIsNone(poe2_core.parse_line(""))

    def test_log_tail(self):
        with tempfile.TemporaryDirectory() as d:
            p = os.path.join(d, "Client.txt")
            with open(p, "w", newline="") as f:
                f.write("a\nb\n")
            t = poe2_core.LogTail(p, from_start=True)
            self.assertIs(iter(t), t)
            self.bounded(5)
            self.assertEqual([next(t)["event"]["raw"], next(t)["event"]["raw"]], ["a", "b"])
            self.assertIsNone(t.next_line())
            with open(p, "a", newline="") as f:
                f.write("c\n")
            self.assertEqual(t.next_line()["event"]["raw"], "c")
            del t  # the tail may still hold the file open, which blocks the cleanup on Windows
            with self.assertRaises(OSError):
                poe2_core.LogTail(os.path.join(d, "missing.txt"))

    def test_blocked_next_is_interruptible(self):
        # interrupt_main acts like Ctrl-C arriving while next() sleeps with the GIL released.
        with tempfile.TemporaryDirectory() as d:
            p = os.path.join(d, "Client.txt")
            open(p, "w").close()
            t = poe2_core.LogTail(p)
            self.bounded(0.5)
            with self.assertRaises(KeyboardInterrupt):
                next(t)


if __name__ == "__main__":
    unittest.main()
