"""Install retained public CIViC evidence and declared synthetic stream-boundary replies."""
import copy
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
URL = 'https://civicdb.org/api/graphql'
COMMON = 'id status significance disease{name doid} therapies{id name} therapyInteractionType'
EVIDENCE = 'evidenceType evidenceLevel evidenceDirection source{citationId sourceType}'
MEMBERS = 'molecularProfile{id name rawName link isComplex variants{id name link feature{id name} ... on GeneVariant{alleleRegistryId hgvsDescriptions coordinates{referenceBuild}}}}'


def record(case, request, data):
    body = json.dumps(request, separators=(',', ':'), ensure_ascii=False).encode()
    digest = hashlib.sha256(body).hexdigest()
    key = hashlib.sha256(b'POST\0')
    for value in [URL, 'application/json', digest]:
        value = value.encode()
        key.update(len(value).to_bytes(8, 'big'))
        key.update(value)
    root = ROOT / 'tests/replies' / case / 'civic'
    root.mkdir(parents=True, exist_ok=True)
    value = {'method': 'POST', 'source': 'civic', 'url': URL,
             'post': {'request_content_type': 'application/json', 'request_body_sha256': digest},
             'exchanges': [{'status': 200, 'content_type': 'application/json', 'delay_ms': 0,
                            'retry_after_ms': None, 'body': {'kind': 'json', 'value': data}}]}
    (root / (key.hexdigest() + '.json')).write_text(json.dumps(value, indent=2) + '\n')


def drug_request(level=None, status='ACCEPTED', count=2, after=None):
    filters = 'status:' + status + (',evidenceLevel:' + level if level else '')
    query = ('query($id:Int!,$after:String){therapy(id:$id){id name link} '
             'evidenceItems(therapyId:$id,' + filters + ',first:' + str(count) + ',after:$after)'
             '{totalCount pageInfo{hasNextPage endCursor} nodes{' + COMMON + ' ' + EVIDENCE + ' ' + MEMBERS + '}}}')
    return {'query': query, 'variables': {'id': 5, 'after': after}}


def connection(rows, total, cursor=None):
    return {'totalCount': total, 'nodes': rows,
            'pageInfo': {'hasNextPage': cursor is not None, 'endCursor': cursor}}


def main():
    fixtures = ROOT / 'tests/fixtures/drug/civic-evidence'
    source = json.loads((fixtures / 'accepted-A.body').read_bytes())
    provenance = json.loads((fixtures / 'provenance.json').read_text())
    record('drug/civic-evidence-public', provenance['request'], source)
    therapy = source['data']['therapy']
    template = source['data']['evidenceItems']['nodes'][0]

    def row(number, level='A', status='ACCEPTED'):
        value = copy.deepcopy(template)
        value.update(id=900000 + number, evidenceLevel=level, status=status)
        return value

    def put(level, count, after, rows, total, cursor=None, status='ACCEPTED'):
        record('drug/civic-evidence-streams', drug_request(level, status, count, after),
               {'data': {'therapy': therapy, 'evidenceItems': connection(rows, total, cursor)}})

    put('A', 2, None, [row(1), row(2)], 3, 'A2')
    put('B', 0, None, [], 2, 'B0')
    put('A', 2, 'A2', [row(3)], 3)
    put('B', 1, None, [row(4, 'B')], 2, 'B1')
    put('A', 0, None, [], 3, 'A0')
    put('B', 2, 'B1', [row(5, 'B')], 2)
    put('C', 2, None, [row(6, 'C', 'SUBMITTED')], 1, status='SUBMITTED')
    put(None, 2, None, [row(7, 'D', 'REJECTED')], 1, status='REJECTED')
    record('drug/civic-evidence-streams', drug_request(count=2),
           {'data': {'therapy': therapy, 'evidenceItems': connection([row(1), row(2)], 5, 'plain2')}})


if __name__ == '__main__':
    main()
