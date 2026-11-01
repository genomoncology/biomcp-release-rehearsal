"""Check source-produced iCite observations with the unchanged nightly classifier."""
import copy
import json
from pathlib import Path
import sys

from nightly_live_check import classify_case


def prove(path):
    witness = json.loads(Path(path).read_text())
    row = {'case': 'article/citations/citing.yaml',
           'result': {'drift': ['icite: request-plan-changed'],
                      'commands': [{'ordinal': 1, 'exchanges': {
                          'omitted': 0, 'drift_omitted': 0}}]},
           'witness': [witness]}
    assert classify_case(row) == 'churn', row
    bad = copy.deepcopy(row)
    bad['witness'][0]['live']['requests'][-1]['provenance']['inputs']['ids'][-1] = '99999999'
    assert classify_case(bad) == 'failure', bad
    print('iCite source discovery: +2 batch boundary churn; unrelated PMID failure')


if __name__ == '__main__':
    prove(sys.argv[1])
