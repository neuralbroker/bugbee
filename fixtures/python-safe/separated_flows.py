from flask import request
import os


def read_user_input():
    # This handler accepts user input but never sends it to a command sink.
    return request.args.get("name", "guest")


def run_fixed_maintenance_task():
    # This command is fixed and does not use request data.
    return os.system("echo maintenance")
