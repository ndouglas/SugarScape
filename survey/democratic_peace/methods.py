"""Prospective numerical contract, independent of measured histories."""
from copy import deepcopy
import hashlib
import json

from pathlib import Path

METHOD_CONTRACT = json.loads(Path(__file__).with_name('methods.json').read_bytes())


def canonical_bytes(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'),
                      ensure_ascii=False, allow_nan=False).encode('utf-8')


def method_contract():
    return deepcopy(METHOD_CONTRACT)


def contract_sha256():
    return hashlib.sha256(canonical_bytes(METHOD_CONTRACT)).hexdigest()
