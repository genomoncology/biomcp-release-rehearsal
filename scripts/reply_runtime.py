"""Bind declared runtime values across one complete pair of captures."""
import hashlib
import json
import re
import shlex

from reply_acquisitions import acquisition_events, catalog_root


def owner(key):
    if isinstance(key, str):
        if key.startswith('case/'):
            parts = key.rsplit('/step/', 1)
            if len(parts) == 2 and re.fullmatch(r'0|[1-9][0-9]*', parts[1]):
                name = parts[0]
                if all(p not in ('', '.', '..') for p in name.split('/')[1:]):
                    return name
        elif key.startswith('trial/'):
            parts = key.rsplit('/', 2)
            if (len(parts) == 3 and re.fullmatch(r'[0-9a-f]{64}', parts[1])
                    and re.fullmatch(r'0|[1-9][0-9]*', parts[2])
                    and parts[0].startswith('trial/')
                    and all(p not in ('', '.', '..') for p in parts[0].split('/')[1:])):
                return parts[0]
    raise ValueError('invalid transcript key')


class Bindings:
    def __init__(self):
        self.forward = {}
        self.reverse = {}
        self.pools = ({}, {})
        self.snapshot_steps = {}
        self.acquisitions = ({}, {})

    def bind(self, kind, left, right):
        a, b = (kind, left), (kind, right)
        if self.forward.setdefault(a, right) != right or self.reverse.setdefault(b, left) != left:
            raise ValueError(f'inconsistent runtime binding: {kind}')
        return '<runtime>'

    def values(self, left, right, scope, path=(), encoded=(), clocks=True, catalog=None, acquisition=None, event=None):
        """Return only authorized scalar substitutions, indexed by JSON path."""
        changes = ({}, {})
        def envelope(value):
            if isinstance(value, dict) and value.get('jsonrpc') == '2.0':
                result = value.get('result')
                value = result.get('structuredContent') if isinstance(result, dict) else None
            return value if isinstance(value, dict) else {}
        documents = [envelope(value) for value in (left, right)]
        def schema(prefix):
            return all(str(value.get('schema', '')).startswith(prefix) for value in documents)
        def pgx_page(position, a, b):
            # PgxAnnotations::SCHEMA; pgx/annotations.rs binds the stored collection
            # before pgx/next.rs emits its typed page command. Only this slot declares.
            flags = {3: '--gene', 5: '--kind', 7: '--section-limit',
                     9: '--section-offset', 11: '--snapshot', 13: '--source'}
            return (len(position) == 3 and position[0] == 'next'
                    and all(d.get('schema') == 'biomcp.pgx-annotations/5'
                            and d['next'][position[1]].get('kind') == 'page'
                            for d in documents)
                    and all(len(args) == 15 and args[:3] == ['get', 'pgx', 'annotations']
                            and args[14] == 'clinpgx'
                            and all(args[i] == flag for i, flag in flags.items())
                            for args in (a, b)))
        def put(p, a, b, kind):
            if type(a) is not type(b):
                return
            self.bind(kind, a, b)
            changes[0][p] = changes[1][p] = '<runtime>'
        def visit(a, b, p):
            position = p[len(path):]
            if position[:2] == ('result', 'structuredContent'):
                position = position[2:]
            if isinstance(a, dict) and isinstance(b, dict):
                fetched = (position == ('result',)
                           or (position[:2] == ('result', 'items') and len(position) in (3, 4)
                               and isinstance(position[2], int)
                               and (len(position) == 3 or position[3] == 'answer')))
                pool = (position[-2:] == ('page', 'pool')
                        and {'id', 'created', 'expires', 'size'} <= a.keys() & b.keys())
                if pool:
                    if not all(type(x[k]) is int for x in (a, b) for k in ('created', 'expires')):
                        raise ValueError('invalid pool clocks')
                    if a['expires'] - a['created'] != b['expires'] - b['created']:
                        raise ValueError('changed pool lifetime')
                    for index, value in enumerate((a, b)):
                        pool_clocks = (value['created'], value['expires'])
                        if self.pools[index].setdefault((scope, acquisition, value['id']), pool_clocks) != pool_clocks:
                            raise ValueError('inconsistent pool clocks')
                    put(p + ('id',), a['id'], b['id'], (scope, 'pool'))
                    for k in ('created', 'expires'):
                        changes[0][p + (k,)] = changes[1][p + (k,)] = '<runtime>'
                for k in a.keys() & b.keys():
                    child = p + (k,)
                    if pool and k in ('id', 'created', 'expires'):
                        continue
                    if clocks and ((k == 'fetched' and fetched)
                                   or (k == 'acquired' and position == ('result', 'snapshot'))):
                        if type(a[k]) is int and type(b[k]) is int:
                            kind = (scope, k)
                            if k == 'fetched':
                                if not acquisition:
                                    continue
                                kind += (acquisition,)
                            if k == 'acquired':
                                if (not catalog or not all(catalog) or not isinstance(a.get('revision'), str)
                                        or a.get('revision') != b.get('revision')):
                                    continue
                                self.bind((scope, 'catalog-root'), *catalog)
                                kind += (catalog[0], a['revision'])
                                if event is not None:
                                    kind += ('sync-event', event)
                            put(child, a[k], b[k], kind)
                    elif (k == 'pool' and position[-1:] == ('query',)
                          and isinstance(a[k], str) and isinstance(b[k], str)):
                        if (((scope, 'pool'), a[k]) in self.forward
                                or ((scope, 'pool'), b[k]) in self.reverse):
                            put(child, a[k], b[k], (scope, 'pool'))
                    elif (k == 'pool' and schema('biomcp.trial-cards/') and len(position) == 7
                          and position[:2] == ('result', 'items') and isinstance(position[2], int)
                          and position[3] == 'sections' and position[5:] == ('data', 'continuation')
                          and isinstance(a[k], str) and isinstance(b[k], str)):
                        put(child, a[k], b[k], (scope, 'pool'))
                    elif (k == 'snapshot' and position == ('query',)
                          and isinstance(a[k], str) and isinstance(b[k], str)
                          and ((scope, 'snapshot'), a[k]) in self.forward):
                        put(child, a[k], b[k], (scope, 'snapshot'))
                    else:
                        visit(a[k], b[k], child)
            elif isinstance(a, list) and isinstance(b, list):
                argv = (not p or p == ('params', 'arguments', 'args')
                        or (len(position) == 3 and position[0] == 'next'
                            and isinstance(position[1], int) and position[2] == 'args'))
                for i, (x, y) in enumerate(zip(a, b)):
                    if (argv and i and a[i-1] == b[i-1] == '--pool'
                            and isinstance(x, str) and isinstance(y, str)):
                        if (len(position) == 3 and position[0] == 'next'
                                or ((scope, 'pool'), x) in self.forward
                                or ((scope, 'pool'), y) in self.reverse):
                            put(p + (i,), x, y, (scope, 'pool'))
                    elif (argv and i and a[i-1] == b[i-1] == '--snapshot'
                          and isinstance(x, str) and isinstance(y, str)):
                        pgx = i == 12 and pgx_page(position, a, b)
                        # Retained phenotype sections and ClinPGx pages declare snapshots.
                        declared = (len(position) == 3 and position[0] == 'next'
                                    and schema('biomcp.phenotype-card/')) or pgx
                        if declared or ((scope, 'snapshot'), x) in self.forward:
                            put(p + (i,), x, y, (scope, 'snapshot'))
                            if pgx:
                                self.snapshot_steps[scope, tuple(a)] = tuple(b)
                    else:
                        visit(x, y, p + (i,))
            elif p in encoded and isinstance(a, str) and isinstance(b, str):
                try:
                    aa, bb = json.loads(a), json.loads(b)
                except ValueError:
                    return
                # MCP embeds complete JSON replies inside text content.
                if isinstance(aa, (dict, list)) and isinstance(bb, (dict, list)):
                    nested = self.values(aa, bb, scope, p, clocks=clocks, catalog=catalog,
                                         acquisition=acquisition, event=event)
                    for index in range(2):
                        if nested[index]:
                            changes[index][p] = nested[index]
        visit(left, right, path)
        return changes


def rewrite(text, changes, path=(), spans=False):
    """Replace selected JSON leaves while preserving every other original byte."""
    decoder = json.JSONDecoder()
    def space(i):
        while i < len(text) and text[i].isspace():
            i += 1
        return i
    edits = []
    def parse(i, p):
        i = space(i)
        if text[i] == '{':
            i = space(i + 1)
            while text[i] != '}':
                key, end = decoder.raw_decode(text, i)
                i = parse(space(end) + 1, p + (key,))
                i = space(i)
                if text[i] != ',':
                    break
                i = space(i + 1)
            return i + 1
        if text[i] == '[':
            i, n = space(i + 1), 0
            while text[i] != ']':
                i = space(parse(i, p + (n,)))
                n += 1
                if text[i] != ',':
                    break
                i = space(i + 1)
            return i + 1
        value, end = decoder.raw_decode(text, i)
        if p in changes:
            replacement = changes[p]
            if isinstance(replacement, dict):
                # Map decoded characters back to their original JSON escapes.
                # Only runtime leaf spans may change inside an MCP text string.
                offsets, cursor = [i + 1], i + 1
                while cursor < end - 1:
                    if text[cursor] == '\\':
                        if text[cursor + 1] == 'u':
                            code = int(text[cursor + 2:cursor + 6], 16)
                            cursor += 6
                            if (0xD800 <= code <= 0xDBFF and text[cursor:cursor + 2] == '\\u'
                                    and 0xDC00 <= int(text[cursor + 2:cursor + 6], 16) <= 0xDFFF):
                                cursor += 6
                        else:
                            cursor += 2
                    else:
                        cursor += 1
                    offsets.append(cursor)
                for start, finish, segment in rewrite(value, replacement, p, spans=True):
                    edits.append((offsets[start], offsets[finish],
                                  json.dumps(segment, ensure_ascii=False)[1:-1]))
            else:
                edits.append((i, end, json.dumps(replacement, ensure_ascii=False)))
        return end
    parse(0, path)
    if spans:
        return edits
    for start, end, value in reversed(edits):
        text = text[:start] + value + text[end:]
    return text


def context_roots(roots, key, index):
    return dict(roots.get(owner(key), {}), **roots.get((key, index), {}))


def record_roots(records):
    roots = {}
    recordings = {}
    for key, record in records.items():
        for execution in record['executions']:
            args = execution['args']
            if (execution.get('root') and execution['exit'] == 0
                    and args.count('--record') == 1
                    and args.index('--record') + 1 < len(args)
                    and ('--' not in args or args.index('--') > args.index('--record'))):
                recordings.setdefault((owner(key), execution['root']), set()).add(
                    args[args.index('--record') + 1])
            for flag in ('--record', '--replay'):
                if flag in args:
                    value = args[args.index(flag) + 1]
                    root = execution.get('root', '')
                    if root and value.startswith(root + '/'):
                        roots.setdefault(owner(key), {})[value] = value[len(root):]
    for key, record in records.items():
        for index, execution in enumerate(record['executions']):
            if 'replay_owner_root' not in execution:
                continue
            root, args = execution['replay_owner_root'], execution['args']
            if (not isinstance(root, str) or not root.startswith('/')
                    or any(part in ('', '.', '..') for part in root.split('/')[1:])):
                raise ValueError('invalid captured replay owner root')
            if (args.count('--replay') != 1 or args.index('--replay') + 1 >= len(args)
                    or ('--' in args and args.index('--') < args.index('--replay'))):
                raise ValueError('captured replay owner requires one effective replay argument')
            path = args[args.index('--replay') + 1]
            if path not in (root + '/recordings', root + '/all'):
                raise ValueError('replay argument is outside its captured owner')
            if path == root + '/all':
                inputs = recordings.get((owner(key), root), set())
                if not {root + '/first', root + '/second'} <= inputs:
                    raise ValueError('combined replay lacks successful owned recording inputs')
            roots[key, index] = {
                path: {'owner_root': root, 'relative': path[len(root):]}}
    return roots


def declared_args(execution, roots):
    args = execution['args']
    result = []
    for i, arg in enumerate(args):
        for path, name in roots.items():
            # A declared record directory may be passed by another isolated process.
            if isinstance(name, dict):
                if i and args[i - 1] == '--replay' and arg == path:
                    arg = '<owned-replay>' + name['relative']
            elif arg == path:
                arg = '<record-root>' + name
        result.append(arg)
    return result


def _mcp_arguments(execution):
    """Yield only captured BioMCP tools/call request frames and their argv."""
    args = execution['args']
    if execution['surface'] != 'mcp' or '<requests>' not in args:
        return
    for i in range(args.index('<requests>') + 1, len(args)):
        try:
            frame = json.loads(args[i])
            if (frame.get('jsonrpc') == '2.0' and frame.get('method') == 'tools/call'
                    and frame['params'].get('name') == 'biomcp'):
                vector = frame['params']['arguments']['args']
                if isinstance(vector, list):
                    yield i, frame, vector
        except (ValueError, KeyError, TypeError, AttributeError):
            continue


def align(base, head, normalize, bindings=None):
    """Alias changed trial hashes through declared paths and bound opaque IDs."""
    roots = (record_roots(base), record_roots(head))
    bindings = bindings if bindings is not None else Bindings()
    bindings.acquisitions = tuple(acquisition_events(records, owner, _mcp_arguments) for records in (base, head))
    # Shared capture keys declare runtime IDs before hashed requests are aligned.
    for key in sorted(base.keys() & head.keys()):
        for i, (a, b) in enumerate(zip(base[key]['executions'], head[key]['executions'])):
            declared = [context_roots(r, key, i) for r in roots]
            pair(a, b, owner(key), bindings, declared, normalize)
    def signature(key, record, index):
        values = []
        for i, e in enumerate(record['executions']):
            n = normalize(dict(e, args=declared_args(e, context_roots(roots[index], key, i))))
            def tokens(args):
                changes = {}
                for j, arg in enumerate(args):
                    if j and args[j-1] in ('--pool', '--snapshot') and isinstance(arg, str):
                        kind = (owner(key), args[j-1][2:])
                        if index == 0 and (kind, arg) in bindings.forward:
                            changes[j,] = '<bound>/' + arg
                        elif index == 1 and (kind, arg) in bindings.reverse:
                            changes[j,] = '<bound>/' + bindings.reverse[kind, arg]
                return changes
            for j, value in tokens(n['args']).items():
                n['args'][j[0]] = value
            for j, frame, vector in _mcp_arguments(n):
                changes = {('params', 'arguments', 'args') + p: v
                           for p, v in tokens(vector).items()}
                n['args'][j] = rewrite(n['args'][j], changes)
            values.append({k: n[k] for k in ('args', 'surface')})
        return owner(key), key.rsplit('/', 1)[-1], json.dumps(values, sort_keys=True)
    def valid_hash(key, record):
        parts = key.rsplit('/', 2)
        def stable(e):
            args = e['args']
            fields = [('root', '<root>'), ('checkout', '<checkout>/crates/biomcp')]
            for field, replacement in fields:
                if e.get(field):
                    args = [a.replace(e[field], replacement) for a in args]
            return args
        return (key.startswith('trial/') and len(parts) == 3 and
                all(hashlib.sha256(json.dumps(stable(e), ensure_ascii=False,
                    separators=(',', ':')).encode()).hexdigest() == parts[1]
                    for e in record['executions']))
    unmatched = (base.keys() - head.keys(), head.keys() - base.keys())
    aliases = {}
    while True:
        candidates = {}
        for key in sorted(unmatched[1]):
            if valid_hash(key, head[key]):
                candidates.setdefault(signature(key, head[key], 1), []).append(key)
        newly_paired = {}
        for key in sorted(unmatched[0]):
            if not valid_hash(key, base[key]):
                continue
            matches = candidates.get(signature(key, base[key], 0), [])
            if len(matches) == 1:
                newly_paired[key] = matches.pop()
        if not newly_paired:
            break
        # An aligned reply may declare the token needed by a later request.
        # Every round consumes complete unique pairs; unproven values stay literal.
        for key, target in newly_paired.items():
            for i, (a, b) in enumerate(zip(base[key]['executions'], head[target]['executions'])):
                declared = [context_roots(r, k, i) for r, k in zip(roots, (key, target))]
                pair(a, b, owner(key), bindings, declared, normalize)
        aliases.update(newly_paired)
        unmatched = (unmatched[0] - newly_paired.keys(),
                     unmatched[1] - set(newly_paired.values()))
    return roots, aliases


def pair(left, right, scope, bindings, roots, normalize):
    owned = [[value['owner_root'] for value in declared.values() if isinstance(value, dict)]
             for declared in roots]
    if all(len(values) == 1 for values in owned):
        bindings.bind((scope, 'replay-owner-root'), owned[0][0], owned[1][0])
    recorded = tuple(e.get('root') for e in (left, right))
    acquisition = None
    if all(isinstance(root, str) and root.startswith('/') for root in recorded):
        bindings.bind((scope, 'execution-root'), *recorded)
        acquisition = recorded[0]
    a, b = [normalize(dict(e, args=declared_args(e, declared)))
            for e, declared in zip((left, right), roots)]
    args = bindings.values(a['args'], b['args'], scope, clocks=False)
    for n, changes in ((a, args[0]), (b, args[1])):
        for p, value in changes.items():
            # Executed argv can also contain a JSON MCP request token.
            n['args'][p[0]] = value
    # Only captured MCP request frames own JSON-encoded argument vectors.
    for (i, _, av), (j, _, bv) in zip(_mcp_arguments(a), _mcp_arguments(b)):
        if i == j:
            path = ('params', 'arguments', 'args')
            changes = bindings.values(av, bv, scope, path, clocks=False)
            for n, change in zip((a, b), changes):
                n['args'][i] = rewrite(n['args'][i], change)
    try:
        values = [json.loads(n['stdout']) for n in (a, b)]
    except ValueError:
        # Markdown can reference an already declared typed page command. Match the
        # complete renderer-owned block, never a source string containing its token.
        for (step_scope, before), after in bindings.snapshot_steps.items():
            if step_scope != scope:
                continue
            for n, args in ((a, before), (b, after)):
                command = ' '.join(shlex.quote(arg) for arg in args)
                canonical = list(args)
                canonical[12] = '<runtime>'
                replacement = ' '.join(shlex.quote(arg) for arg in canonical)
                prefix = '**page:**\n\n```sh\nbiomcp '
                n['stdout'] = n['stdout'].replace(prefix + command + '\n```',
                                                prefix + replacement + '\n```')
        # Markdown clocks reference a JSON declaration in this owner's cache root.
        for before, after in bindings.forward.items():
            kind, value = before
            if isinstance(kind, tuple) and kind[0] == scope and kind[1:] == ('fetched', acquisition):
                for n, clock in ((a, value), (b, after)):
                    n['stdout'] = n['stdout'].replace(
                        '**fetched:** ' + str(clock) + '\n', '**fetched:** <runtime>\n')
        return a, b
    def embedded(value, execution):
        if (execution['surface'] != 'mcp' or not isinstance(value, dict)
                or value.get('jsonrpc') != '2.0'):
            return set()
        result = value.get('result', {})
        if not isinstance(result, dict):
            return set()
        paths = set()
        for i, item in enumerate(result.get('content', [])):
            if isinstance(item, dict) and item.get('type') == 'text':
                try:
                    if json.loads(item.get('text', '')) == result.get('structuredContent'):
                        paths.add(('result', 'content', i, 'text'))
                except (ValueError, TypeError):
                    pass
        return paths
    encoded = embedded(values[0], left) & embedded(values[1], right)
    events = tuple(events.get(id(execution)) for events, execution
                   in zip(bindings.acquisitions, (left, right)))
    if events[0] != events[1]:
        raise ValueError('changed active catalog acquisition event')
    changes = bindings.values(*values, scope, encoded=encoded,
                              catalog=tuple(catalog_root(e) for e in (left, right)),
                              acquisition=acquisition, event=events[0])
    # Saved paths are owned only when the literal destination was declared.
    for index, execution in enumerate((left, right)):
        args = execution['args']
        if '--save' in args:
            name = args[args.index('--save') + 1]
            path = execution.get('root', '') + '/' + name
            target = ('result', 'file', 'data', 'path')
            value = values[index]
            for key in target:
                value = value.get(key, {}) if isinstance(value, dict) else None
            if value == path and '..' not in name.split('/') and not name.startswith('/'):
                changes[index][target] = '<saved>/' + name
    for n, change in ((a, changes[0]), (b, changes[1])):
        if change:
            n['stdout'] = rewrite(n['stdout'], change)
    return a, b
