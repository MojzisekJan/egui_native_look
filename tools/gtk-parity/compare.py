#!/usr/bin/python3
"""Compare native measurements, keeping deliberate API differences explicit."""
import argparse
import json
import math
from pathlib import Path

# These expectations describe documented API choices, not GTK compatibility.
# Any additional difference, including another field in these cases, still fails.
INTENTIONAL = {
    "number.commit.'-'": ({'number': 0.0}, {'number': 4.0}),
    "number.commit.'abc'": ({'number': 0.0}, {'number': 4.0}),
    "number.commit.''": ({'number': 0.0}, {'number': 4.0}),
    "number.commit.'1,5'": ({'number': 1.0}, {'number': 4.0}),
    "number.commit.'inf'": ({'number': 10.0}, {'number': 4.0}),
    'list.disabled.1.Down': ({'list_selected': 2, 'list_activated': 0}, {'list_selected': 3, 'list_activated': 0}),
    'list.disabled.2.Page_Down': ({'list_selected': 6, 'list_activated': 0}, {'list_selected': 7, 'list_activated': 0}),
    'list.disabled.3.Page_Down': ({'list_selected': 10, 'list_activated': 0}, {'list_selected': 11, 'list_activated': 0}),
    'list.disabled.4.Page_Up': ({'list_selected': 5, 'list_activated': 0}, {'list_selected': 6, 'list_activated': 0}),
    'list.disabled.5.ctrl+Down': ({'list_selected': 5, 'list_activated': 0}, {'list_selected': 6, 'list_activated': 0}),
    'list.disabled.6.space': ({'list_selected': 6, 'list_activated': 1}, {'list_selected': 7, 'list_activated': 1}),
}


def equivalent(left, right):
    if isinstance(left, dict) and isinstance(right, dict):
        return left.keys() == right.keys() and all(equivalent(left[k], right[k]) for k in left)
    if isinstance(left, list) and isinstance(right, list):
        return len(left) == len(right) and all(equivalent(a, b) for a, b in zip(left, right))
    if type(left) in (int, float) and type(right) in (int, float):
        return math.isclose(left, right, rel_tol=0, abs_tol=0.0001)
    return type(left) is type(right) and left == right


def normalized(result):
    result = dict(result)
    # A native popup's children remain descendants of the launcher; egui uses a
    # separate popup layer. Retain raw focus evidence without equating these IDs.
    result.pop('focus')
    # Compare selection extents, and the insertion position when collapsed.
    # The two APIs expose different ends of a nonempty selection as position.
    if result.get('selection'):
        result.pop('cursor', None)
    return result


def compare(directory):
    traces = json.loads((directory / 'traces.json').read_text())
    gtk = [t for t in traces if t['app'] == 'gtk']
    egui = [t for t in traces if t['app'] == 'egui']
    if len(gtk) != len(egui) or not gtk:
        raise ValueError('Incomplete native/egui trace pair')
    report = dict(cases=len(gtk), matched=0, intentional=[], unexpected=[])
    for native, project in zip(gtk, egui):
        if native['case'] != project['case']:
            raise ValueError('The fixture sequences differ')
        case = native['case']
        left, right = normalized(native['result']), normalized(project['result'])
        if equivalent(left, right):
            report['matched'] += 1
            continue
        difference = dict(case=case, gtk=left, egui=right)
        expected = INTENTIONAL.get(case)
        target = 'intentional' if expected and equivalent(left, expected[0]) and equivalent(right, expected[1]) else 'unexpected'
        report[target].append(difference)
    (directory / 'comparison.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')
    print(f"{report['cases']} cases: {report['matched']} matched, "
          f"{len(report['intentional'])} documented policy differences, "
          f"{len(report['unexpected'])} unexpected differences", flush=True)
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('directory', type=Path)
    report = compare(parser.parse_args().directory)
    raise SystemExit(bool(report['unexpected']))
