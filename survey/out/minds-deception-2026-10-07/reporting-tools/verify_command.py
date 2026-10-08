"""Capture each explicitly supplied static verification command and its exit."""
import datetime
import json
from pathlib import Path
import subprocess
import sys

result = subprocess.run(sys.argv[1:], capture_output=True, text=True)
record = dict(utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              cwd=str(Path.cwd()), argv=sys.argv[1:], exit_code=result.returncode,
              stdout=result.stdout, stderr=result.stderr)
with Path('static-verification.jsonl').open('a') as stream:
    stream.write(json.dumps(record) + '\n')
print(result.stdout, end='')
print(result.stderr, end='', file=sys.stderr)
sys.exit(result.returncode)
