"""Import an immutable PalCalc snapshot; preserve every gender-specific outcome.

Run with --source-dir for previously downloaded db.json and breeding.json.
The default fetches the pinned files directly from their original repository.
"""
from __future__ import annotations

import argparse
import collections
import hashlib
import json
from pathlib import Path
from urllib.request import urlopen

COMMIT = '66f18a000e2107640b219562437357ced5aff7eb'
ORIGIN = f'https://raw.githubusercontent.com/tylercamp/palcalc/{COMMIT}'
ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('--source-dir', type=Path)
    args = parser.parse_args()
    raw = {}
    for name in ('db.json', 'breeding.json'):
        raw[name] = ((args.source_dir / name).read_bytes() if args.source_dir else
                     urlopen(f'{ORIGIN}/PalCalc.Model/{name}', timeout=60).read())
    pals = json.loads(raw['db.json'])['Pals']
    rows = json.loads(raw['breeding.json'])['Breeding']
    counts = collections.Counter(p['Name'] for p in pals)
    def display(p):
        return p['Name'] + (' (Flower)' if p['InternalName'] == 'PlantSlime_Flower' else '')
    assert all(n == 1 or name == 'Gumoss' for name, n in counts.items()), 'Unexpected duplicate display name'
    pals.sort(key=lambda p: display(p))
    names = [display(p) for p in pals]
    assert len(set(names)) == len(pals)
    index = {p['InternalName']: i for i, p in enumerate(pals)}
    n = len(pals)
    children = [-2] * (n * (n + 1) // 2)
    gender_outcomes = []
    child_counts = collections.Counter(r['ChildInternalName'] for r in rows
                                     if r['Parent1InternalName'] != r['Parent2InternalName'])
    special = []
    def offset(a, b):
        a, b = sorted((a, b))
        return a*n - a*(a-1)//2 + b-a
    for row in rows:
        a, b, c = (index[row[k]] for k in ('Parent1InternalName', 'Parent2InternalName', 'ChildInternalName'))
        ga, gb = row['Parent1Gender'].lower(), row['Parent2Gender'].lower()
        if a > b:
            a, b, ga, gb = b, a, gb, ga
        pos = offset(a, b)
        if ga != 'wildcard' or gb != 'wildcard':
            assert children[pos] in (-2, -1)
            children[pos] = -1
            gender_outcomes.append(dict(a=a, b=b, child=c, a_gender=ga, b_gender=gb))
        else:
            assert children[pos] == -2, 'Duplicate ungendered pair'
            children[pos] = c
            if a != b and child_counts[row['ChildInternalName']] == 1:
                special.append(dict(parent_a=names[a], parent_b=names[b], child=names[c]))
    assert -2 not in children, 'Source does not cover every parent pair'
    assert all(children[offset(i, i)] == i for i in range(n)), 'Invalid same-species result'
    table = dict(pal_names=names, children=children, gender_outcomes=gender_outcomes)
    # Replay every source row through the exported lookup, including swapped parent genders.
    for row in rows:
        a,b,c=(index[row[k]] for k in ('Parent1InternalName','Parent2InternalName','ChildInternalName'))
        if children[offset(a,b)] >= 0:
            assert children[offset(a,b)] == c
        else:
            ga,gb=row['Parent1Gender'].lower(),row['Parent2Gender'].lower()
            if a>b:a,b,ga,gb=b,a,gb,ga
            assert any(x == dict(a=a,b=b,child=c,a_gender=ga,b_gender=gb) for x in gender_outcomes)
    metadata = dict(source='PalCalc', source_url=f'https://github.com/tylercamp/palcalc/tree/{COMMIT}',
                    source_commit=COMMIT, imported_on='2026-10-25', pal_count=n,
                    parent_pair_count=len(children), outcome_count=len(rows),
                    gender_specific_outcome_count=len(gender_outcomes),
                    sha256={k:hashlib.sha256(v).hexdigest() for k,v in raw.items()},
                    scope='Species outcomes from the pinned community dataset; game build is not declared by this snapshot. Mutations and trait inheritance are not modeled.')
    data=ROOT/'data'
    (data/'pals.json').write_text(json.dumps(dict(pals=[dict(name=display(p),power=p['BreedingPower']) for p in pals]),indent=2)+'\n')
    (data/'special_combos.json').write_text(json.dumps(dict(combos=sorted(special,key=lambda x:(x['parent_a'],x['parent_b']))),indent=2)+'\n')
    (data/'breeding_table.json').write_text(json.dumps(table,separators=(',',':'))+'\n')
    (data/'provenance.json').write_text(json.dumps(metadata,indent=2)+'\n')
    print(f'Replayed {len(rows):,} outcomes across {len(children):,} parent pairs for {n} species; {len(gender_outcomes)} gender-specific outcomes.')


if __name__ == '__main__':
    main()
