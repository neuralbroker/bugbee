"""Synthetic vulnerable sample for Bugbee regression (local only)."""

import os
import subprocess

# secrets.generic_password_assign
password = "hunter2-not-real"

# owasp.command.shell_true
def run_user(cmd: str) -> None:
    subprocess.run(cmd, shell=True)


# owasp.python.eval
def calc(expr: str):
    return eval(expr)


# owasp.sql.concatenate
def lookup(user_id: str) -> str:
    return "SELECT * FROM users WHERE id = '" + user_id + "'"


if __name__ == "__main__":
    print(lookup(os.environ.get("UID", "1")))
