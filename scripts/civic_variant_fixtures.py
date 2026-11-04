"""Declare filtered root evidence replies for an existing exact CIViC profile."""
import copy
import json
from civic_evidence_fixtures import ROOT, connection, record


def profile_request(profile=False):
    ids_variable, ids_argument = (',$ids:[Int!]', ',ids:$ids') if profile else ('', '')
    query = ('query($allele:String!,$after:String' + ids_variable + ',$evidence_after:String,'
             '$assertion_after:String){molecularProfiles(alleleRegistryId:$allele' + ids_argument + ',first:1,after:$after){totalCount '
             'pageInfo{hasNextPage endCursor} nodes{id name rawName link isComplex variants{id name link '
             'feature{id name} ... on GeneVariant{alleleRegistryId hgvsDescriptions}} '
             'evidenceItems(first:0,after:$evidence_after,includeRejected:false){totalCount '
             'pageInfo{hasNextPage endCursor} nodes{id status evidenceType evidenceLevel evidenceDirection '
             'significance disease{name doid} therapies{name} therapyInteractionType source{citationId sourceType}}} '
             'assertions(first:' + ('0' if profile else '1') + ',after:$assertion_after,includeRejected:false){totalCount '
             'pageInfo{hasNextPage endCursor} nodes{id status assertionType assertionDirection significance '
             'ampLevel disease{name doid} therapies{name} therapyInteractionType variantOrigin}}}}}')
    variables = {'allele': 'CA123643', 'after': None}
    if profile:
        variables['ids'] = [12]
    variables.update(evidence_after=None, assertion_after=None)
    return {'query': query, 'variables': variables}


def main():
    baseline = next((ROOT / 'tests/replies/variant/native-civic/civic').glob('*.json'))
    data = json.loads(baseline.read_text())['exchanges'][0]['body']['value']
    profile = data['data']['molecularProfiles']['nodes'][0]
    rows = copy.deepcopy(profile['evidenceItems']['nodes'])
    for index, row in enumerate(rows):
        row.update(id=910001 + index, status='SUBMITTED', evidenceLevel='A')
    profile['evidenceItems'] = connection([], 218, 'initial-zero')
    case = 'variant/civic-evidence-filters'
    record(case, profile_request(), data)
    selected = copy.deepcopy(data)
    selected['data']['molecularProfiles'] = connection(selected['data']['molecularProfiles']['nodes'], 1)
    selected['data']['molecularProfiles']['nodes'][0]['assertions'] = connection([], 11, 'assertions-zero')
    record(case, profile_request(True), selected)
    query = ('query($id:Int!,$after:String){evidenceItems(molecularProfileId:$id,status:SUBMITTED,evidenceLevel:A,first:2,after:$after)'
             '{totalCount pageInfo{hasNextPage endCursor} nodes{id status evidenceType evidenceLevel evidenceDirection '
             'significance disease{name doid} therapies{name} therapyInteractionType source{citationId sourceType}}}}')
    for after, page in [(None, connection(rows, 3, 'E2')),
                        ('E2', connection([dict(rows[0], id=910003)], 3))]:
        record(case, {'query': query, 'variables': {'id': 12, 'after': after}}, {'data': {'evidenceItems': page}})
    default_query = query.replace('status:SUBMITTED', 'status:NON_REJECTED')
    record(case, {'query': default_query, 'variables': {'id': 12, 'after': None}},
           {'data': {'evidenceItems': connection(rows, 3, 'E2')}})
    steps = []
    base = ['get', 'variant', 'CA123643', 'civic', '--disable-sources', 'myvariant',
            '--section-limit', '2', '--evidence-level', 'A', '--evidence-status', 'submitted']
    for after in [None, 'E2']:
        run = base + (['--civic-profile', '12', '--civic-kind', 'evidence', '--civic-cursor',
                       '0:' + after] if after else [])
        steps.append({'run': run, 'expect': {'/result/candidates/0/sections/civic/data/profiles/0/evidence_page/total': 3,
                                            '/result/candidates/0/sections/civic/data/profiles/0/evidence_page/returned': 1 if after else 2,
                                            '/result/candidates/0/sections/civic/data/profiles/0/statements/0/record/id': 'EID910003' if after else 'EID910001'}})
    default_run = base[:-2]
    steps.append({'run': default_run, 'expect': {'/result/candidates/0/sections/civic/data/profiles/0/evidence_page/total': 3}})
    case = {'feature': 'variant-civic', 'live_check': 'synthetic',
            'story': 'Variant evidence selectors use root native filters and retain source totals and profile-scoped continuation.',
            'verified': 'Synthetic evidence IDs, levels, status and counts; profile metadata reused from the retained public native-civic fixture. No clinical meaning inferred.',
            'replay': case, 'steps': steps}
    (ROOT / 'tests/cases/variant/civic-evidence-filters.yaml').write_text(json.dumps(case, indent=2) + '\n')


if __name__ == '__main__':
    main()
