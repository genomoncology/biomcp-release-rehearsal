"""Recommendation discovery selections used by offline nightly wrapper tests."""


def reranked_tail_case(change='boundary'):
    old = [str(1000 + i) for i in range(23)]
    new = old[:8] + ['2000'] + old[8:]
    if change == 'prefix':
        new = list(reversed(old[:20])) + old[20:] + ['2000']
    else:
        new.reverse()
    def discovery(ids):
        return {'identity': 'discovery', 'source': 'semantic-scholar', 'role': 'Recommendations',
                'url': 'https://example.org/recommendations?limit=24', 'status': 200,
                'provenance': {'discoveries': [{'namespace': 'pmid', 'ids': ids},
                                             {'namespace': 'doi', 'ids': []}]}}
    def batch(ids):
        return {'identity': ','.join(sorted(ids)), 'source': 'pubmed', 'role': 'Summary',
                'status': 200, 'provenance': {'input_context': 'same-filter-offset-limit',
                    'inputs': {'namespace': 'pmid', 'ids': sorted(ids)}}}
    commands = []
    for start in range(0, 23, 10):
        ordinal = len(commands) + 1
        commands.append({'ordinal': ordinal,
            'baseline': {'requests': ([discovery(old)] if start == 0 else []) + [batch(old[start:start+10])]},
            'live': {'requests': ([discovery(new)] if start == 0 else []) + [batch(new[start:start+10])]}})
    first = commands[0]
    provider = first['live']['requests'][0]
    dependent = first['live']['requests'][1]
    if change in ('filter', 'offset', 'limit'):
        dependent['provenance']['input_context'] = 'changed-' + change
    elif change == 'missing-batch':
        commands[1]['live']['requests'] = []
    elif change == 'duplicate-id':
        dependent['provenance']['inputs']['ids'].append(dependent['provenance']['inputs']['ids'][0])
    elif change == 'omitted-id':
        dependent['provenance']['inputs']['ids'].pop()
    elif change == 'unchanged':
        provider['provenance']['discoveries'][0]['ids'] = old
    elif change in ('source', 'role', 'url', 'status'):
        provider[change] = 503 if change == 'status' else 'unrelated'
    elif change == 'over-cap':
        provider['provenance']['discoveries'][0]['ids'] = [str(100000 + i) for i in range(10_001)]
    elif change == 'missing-metadata':
        provider['provenance'] = {}
    elif change == 'duplicate-namespace':
        provider['provenance']['discoveries'].append({'namespace': 'pmid', 'ids': new})
    elif change == 'stale':
        import copy
        for side in ('baseline', 'live'):
            stale = copy.deepcopy(first['baseline']['requests'][0])
            commands[1][side]['requests'].insert(0, stale)
    drift = ['summary: request-plan-changed']
    if change in ('schema', 'absence', 'refusal'):
        drift.append('output ' + change + ' changed')
    return {'case': change, 'witness': commands, 'result': {'drift': drift,
        'commands': [{'ordinal': cmd['ordinal'], 'exchanges': {'omitted': 0, 'drift_omitted': 0}}
                     for cmd in commands]}}


def recommendation_case(story, day='1102', change=None):
    # Identity/provenance selections from Yellow's retained Nov 1 and Nov 2
    # 0068-scheduled reports (827yf6_p and odzt3h2e); no article text retained.
    pools = {
        'multi': ([['42745055', '42685317', '42748921', '42830785'],
                   ['10.64898/2026.09.28.26364204']],
                  {'1101': [['42929675', '42927854', '42685317', '42814929'],
                            ['10.64898/2026.09.28.26364204']],
                   '1102': [['42927854', '42748921', '42964339', '42814929', '42830785'], []]}),
        'single': ([['42745055', '42685317', '42846599', '42891951'],
                    ['10.1158/1538-7445.pancreatic26-b074']],
                   {'1101': [['42929675', '42927884', '42685317', '42891951'],
                             ['10.17650/1994-4098-2026-22-2-85-91']],
                    '1102': [['42927884', '42846599', '42886887'],
                             ['10.17650/1994-4098-2026-22-2-85-91',
                              '10.1158/1538-7445.pancreatic26-b074']]})}
    def requests(selection):
        rows = [{'identity': 'recommendations-' + story, 'source': 'semantic-scholar',
                 'role': 'Recommendations', 'url': 'https://example.org/' + story,
                 'status': 200, 'provenance': {'discoveries': [
                     {'namespace': namespace, 'ids': ids}
                     for namespace, ids in zip(('pmid', 'doi'), selection)]}}]
        for namespace, ids in zip(('pmid', 'doi'), selection):
            if ids:
                rows.append({'identity': namespace + ','.join(sorted(ids)), 'source': 'pubmed',
                             'role': 'Summary' if namespace == 'pmid' else 'Search', 'status': 200,
                             'provenance': {'input_context': 'fixed-' + namespace,
                                 'inputs': {'namespace': namespace, 'ids': sorted(ids)}}})
        return rows
    old, live = pools[story]
    before, after = requests(old), requests(live[day])
    if change == 'unrelated':
        after[1]['provenance']['inputs']['ids'][0] = '99999999'
    elif change == 'context':
        after[1]['provenance']['input_context'] = 'changed-parameter'
    elif change == 'failed-discovery':
        after[0]['status'] = None
    elif change == 'subset':
        after[1]['provenance']['inputs']['ids'].pop()
    drift = ['pubmed: request-plan-changed']
    return {'case': 'article/recommendations/' + story + '.yaml',
            'result': {'drift': drift, 'commands': [{'ordinal': 1,
                'before_status': 'complete', 'after_status': 'partial' if change == 'partial' else 'complete',
                'exchanges': {'omitted': 0, 'drift_omitted': 0}}]},
            'witness': [{'ordinal': 1, 'baseline': {'requests': before}, 'live': {'requests': after}}]}
