"""Aurotype Engine - entry point with free port detection and parent process monitoring."""

import json
import os
import signal
import socket
import sys
import threading
import time

import uvicorn

from aurotype_engine.server import app


def find_free_port() -> int:
    """Find a free port by binding to port 0 and reading the assigned port."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("", 0))
        port = s.getsockname()[1]
    return port


def monitor_parent_pid(parent_pid: int) -> None:
    """Monitor parent PID and terminate self if parent dies."""
    while True:
        time.sleep(2)
        try:
            # Check if parent process still exists by sending signal 0
            os.kill(parent_pid, 0)
        except (ProcessLookupError, OSError):
            # Parent died (or invalid PID on Windows), self-terminate
            os.kill(os.getpid(), signal.SIGTERM)
            break


if __name__ == "__main__":
    # Find free port and output it immediately
    port = find_free_port()
    print(json.dumps({"port": port}), flush=True)

    # Parent PID monitoring only works reliably on Unix.
    # On Windows, os.getppid() may return invalid values, and os.kill(pid, 0)
    # raises OSError for non-existent PIDs, causing immediate self-termination.
    # PyInstaller onefile mode doesn't provide reliable parent detection on Windows.
    if sys.platform != "win32":
        parent_pid = os.getppid()
        if parent_pid > 0:
            monitor_thread = threading.Thread(
                target=monitor_parent_pid, args=(parent_pid,), daemon=True
            )
            monitor_thread.start()

    # Start uvicorn with suppressed logs
    uvicorn.run(app, host="127.0.0.1", port=port, log_level="warning")
