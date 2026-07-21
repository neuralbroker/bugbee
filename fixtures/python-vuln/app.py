"""Intentional vulnerable fixture for Bugbee demos and evals. DO NOT deploy."""

import os
import pickle
import hashlib
import subprocess


# CWE-798 — hardcoded password (fixture only)
password = "fixture-only-not-real"


def insecure_eval(user_input: str):
    # CWE-95
    return eval(user_input)


def insecure_exec(user_input: str):
    # CWE-95
    exec(user_input)


def command_injection(cmd: str):
    # CWE-78
    return subprocess.check_output(cmd, shell=True)


def weak_hash(data: bytes) -> str:
    # CWE-328
    return hashlib.md5(data).hexdigest()


def unsafe_pickle(blob: bytes):
    # CWE-502
    return pickle.loads(blob)


def os_system(cmd: str):
    return os.system(cmd)
