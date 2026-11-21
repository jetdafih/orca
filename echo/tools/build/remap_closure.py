#!/usr/bin/env python3
"""Keep every app class; derive only the public remapper hierarchy/mapping classpath."""
import hashlib
import json
from pathlib import Path
import struct
import zipfile

ROOT = Path(__file__).resolve().parents[2]
load = lambda p: json.loads(p.read_text())
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()

def header(data):
    assert data[:4] == b'\xca\xfe\xba\xbe'
    count = struct.unpack_from('>H', data, 8)[0]; pool = {}; offset = 10; i = 1
    while i < count:
        tag = data[offset]; offset += 1
        if tag == 1:
            n = struct.unpack_from('>H', data, offset)[0]; offset += 2
            pool[i] = data[offset:offset+n].decode('utf8', errors='replace'); offset += n
        elif tag == 7: pool[i] = struct.unpack_from('>H', data, offset)[0]; offset += 2
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18): offset += 4
        elif tag in (5, 6): offset += 8; i += 1
        elif tag in (8, 16, 19, 20): offset += 2
        elif tag == 15: offset += 3
        else: raise ValueError('Unknown class-pool tag ' + str(tag))
        i += 1
    _, owner, parent, count = struct.unpack_from('>HHHH', data, offset); offset += 8
    parents = [pool[pool[parent]]] if parent else []
    parents += [pool[pool[struct.unpack_from('>H', data, offset+n*2)[0]]] for n in range(count)]
    return pool[pool[owner]], parents

def main():
    deps = load(ROOT / 'build/reports/foundation/dependencies.json')
    jars = {}; index = {}; mapped = set()
    rows = [r for r in deps['compileClasspath'] if r['file'].endswith('.jar') and '-natives-' not in r['file']]
    try:
        for row in rows:
            path = Path(row['file']); assert sha(path) == row['sha256']
            jar = zipfile.ZipFile(path); jars[str(path)] = jar
            for name in jar.namelist():
                if name.endswith('.class') and not name.startswith('META-INF/') and name != 'module-info.class':
                    index.setdefault(name[:-6], (str(path), name))
            if row.get('loomMappingData'):
                for line in jar.read('mappings/mappings.tiny').decode().splitlines():
                    if line.startswith('c\t'): mapped.update(line.split('\t')[1:])
        mapping = load(ROOT / 'build/reports/candidate/compile.json')['mapping_jar']
        path = Path(mapping['file']); assert sha(path) == mapping['sha256']
        with zipfile.ZipFile(path) as mapping_jar:
            for line in mapping_jar.read('mappings/mappings.tiny').decode().splitlines():
                if line.startswith('c\t'): mapped.update(line.split('\t')[1:])
        assert mapped, 'Resolved real Yarn mappings required'
        named = ROOT / 'build/candidate/echo-clean-candidate-named.jar'
        app_jar = zipfile.ZipFile(named); jars[str(named)] = app_jar
        app = {n[:-6] for n in app_jar.namelist() if n.endswith('.class')}
        for name in app: index[name] = (str(named), name + '.class')
        game_jar = jars[deps['namedMinecraft']['file']]
        game = {n[:-6] for n in game_jar.namelist() if n.endswith('.class') and not n.startswith('META-INF/') and n != 'module-info.class'}
        todo = list(app | game); visited = set()
        while todo:
            name = todo.pop()
            if name in visited: continue
            visited.add(name)
            if name not in index: continue
            jar, entry = index[name]; owner, parents = header(jars[jar].read(entry))
            assert owner == name; todo.extend(parents)
        required = {index[n][0] for n in visited | mapped if n in index and index[n][0] != str(named)}
        result = {'named_sha256': sha(named), 'application_classes': len(app), 'mapped_game_classes': len(game),
            'hierarchy_types_checked': len(visited), 'resolution_classpath': [r for r in rows if r['file'] in required],
            'omitted_no_hierarchy_or_mapping_participation': [r for r in rows if r['file'] not in required],
            'application_input_reduced': False, 'compiler_or_runtime_dependencies_reduced': False}
        out = ROOT / 'build/reports/candidate/remap-classpath.json'; out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(result, indent=2) + '\n')
        print(f'{len(app)} application classes retained; {len(required)} hierarchy/mapping resolution JARs')
    finally:
        for jar in jars.values(): jar.close()

if __name__ == '__main__': main()
