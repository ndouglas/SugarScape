"""Strict JSON export of the unchanged registered polarity report.

JSON object keys represent Python None as the string "null"; actual null episode
end_cause values remain JSON null. Dictionary ordering does not alter judges.
"""
import json


def serialize_report(result):
    return json.dumps(result, indent=2, sort_keys=False, allow_nan=False)+'\n'


def main():
    from analysis import main as registered_main
    registered_main()


if __name__ == '__main__':
    main()
