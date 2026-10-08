"""Memory-bounded, full-hash-bound P4 presentation projection.

Every biological frame is parsed and offered to the callback; it remains in the
immutable source file rather than being silently discarded or filtered by effect.
"""
import hashlib
from pathlib import Path
import ijson
from ijson.common import ObjectBuilder

REQUIRED = {'schema', 'census', 'cells', 'endpoints', 'estimates', 'biological_groups',
            'repeated_endpoints', 'effective_aliases'}


class HashingReader:
    def __init__(self, stream):
        self.stream = stream
        self.digest = hashlib.sha256()
        self.bytes = 0

    def read(self, size=-1):
        data = self.stream.read(size)
        self.digest.update(data)
        self.bytes += len(data)
        return data


def load(path, *, group_callback=None):
    path = Path(path)
    projection, seen = {}, set()
    builder, capture = None, None
    groups, frames = 0, 0
    with path.open('rb') as stream:
        reader = HashingReader(stream)
        for prefix, event, value in ijson.parse(reader, use_float=True):
            if prefix == '' and event == 'map_key':
                if value in seen:
                    raise ValueError(f'duplicate top-level key: {value}')
                seen.add(value)
                continue
            if builder is not None:
                builder.event(event, value)
                if prefix == capture and event in ('end_map', 'end_array'):
                    if capture == 'biological_groups.item':
                        group = builder.value
                        groups += 1
                        frames += len(group['frames'])
                        if group_callback is not None:
                            group_callback(group)
                    else:
                        projection[capture] = builder.value
                    builder, capture = None, None
                continue
            if prefix == 'biological_groups.item' and event == 'start_map':
                capture, builder = prefix, ObjectBuilder()
                builder.event(event, value)
            elif '.' not in prefix and prefix and prefix != 'biological_groups':
                if event in ('start_map', 'start_array'):
                    capture, builder = prefix, ObjectBuilder()
                    builder.event(event, value)
                elif event in ('string', 'number', 'boolean', 'null'):
                    projection[prefix] = value
        missing = REQUIRED - seen
        if missing:
            raise ValueError(f'missing required analysis fields: {sorted(missing)}')
        if builder is not None or reader.bytes != path.stat().st_size:
            raise ValueError('incomplete source parse')
        binding = dict(path=str(path.resolve()), sha256=reader.digest.hexdigest(), bytes=reader.bytes,
                       biological_groups=groups, frames=frames, top_level_fields=sorted(seen),
                       projection_semantics='complete endpoint/cell/estimate/alias/duplicate fields; '
                       'all full biological frames retained in immutable hash-bound original')
    projection['biological_groups'] = []
    projection['presentation_projection'] = binding
    return projection, binding
