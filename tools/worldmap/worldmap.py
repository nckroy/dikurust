#!/usr/bin/env python3
"""Draw an interactive map of every room in the world files.

Reads lib/tinyworld.wld and lib/tinyworld.zon, lays rooms out on a grid by
exit direction (each zone on its own, then zones placed next to the zones
they connect to), and writes a single self-contained HTML page.

    python3 tools/worldmap/worldmap.py                # writes target/worldmap.html
    python3 tools/worldmap/worldmap.py -o map.html
"""
import argparse, collections, json, pathlib

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent.parent
ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
ap.add_argument('--wld', default=ROOT / 'lib/tinyworld.wld', type=pathlib.Path)
ap.add_argument('--zon', default=ROOT / 'lib/tinyworld.zon', type=pathlib.Path)
ap.add_argument('-o', '--out', default=ROOT / 'target/worldmap.html', type=pathlib.Path)
ap.add_argument('--json', action='store_true', help='write the layout as JSON instead of HTML')
args = ap.parse_args()
WLD, ZON = args.wld, args.zon
VEC = {0: (0, -1), 1: (1, 0), 2: (0, 1), 3: (-1, 0), 4: (1, -1), 5: (-1, 1)}
REV = {0: 2, 1: 3, 2: 0, 3: 1, 4: 5, 5: 4}

def tilde_blocks(lines, i):
    """Read lines until one ending in ~; return (text, next index)."""
    buf = []
    while True:
        l = lines[i]; i += 1
        if l.rstrip().endswith('~'):
            buf.append(l.rstrip()[:-1]); return '\n'.join(buf), i
        buf.append(l)

lines = open(WLD, encoding='latin-1').read().split('\n')
rooms, i = {}, 0
while i < len(lines):
    l = lines[i].strip()
    if not l.startswith('#'): i += 1; continue
    v = int(l[1:]); i += 1
    if lines[i].strip() == '$~': break
    name, i = tilde_blocks(lines, i)
    _, i = tilde_blocks(lines, i)
    zone_n, flags, sector = (int(x) for x in lines[i].split()[:3]); i += 1
    exits = []
    while True:
        l = lines[i].strip(); i += 1
        if l == 'S': break
        if l.startswith('D'):
            d = int(l[1:]); _, i = tilde_blocks(lines, i); kw, i = tilde_blocks(lines, i)
            while not lines[i].split(): i += 1
            t, key, to = (int(x) for x in lines[i].split()[:3]); i += 1
            exits.append({'d': d, 'to': to, 'door': t > 0, 'kw': kw.strip()})
        elif l.startswith('E'):
            _, i = tilde_blocks(lines, i); _, i = tilde_blocks(lines, i)
    rooms[v] = {'v': v, 'name': name.strip(), 'flags': flags, 'sector': sector, 'exits': exits}

zl = open(ZON, encoding='latin-1').read().split('\n')
zones, i = [], 0
while i < len(zl):
    l = zl[i].strip()
    if l.startswith('#'):
        name = zl[i+1].strip().rstrip('~')
        if name == '$' or l == '#$': break
        top = int(zl[i+2].split()[0]); zones.append((top, name)); i += 3
    elif l == '$~' or l == '$': break
    else: i += 1
zones.sort()
def zone_of(v):
    for zi, (top, _) in enumerate(zones):
        if v <= top: return zi
    return len(zones) - 1
for r in rooms.values(): r['zone'] = zone_of(r['v'])

# 1. lay out each zone's intra-zone components on a grid
def layout(seed, members):
    pos, occ, q = {seed: (0, 0)}, {(0, 0): seed}, collections.deque([seed])
    while q:
        a = q.popleft(); ax, ay = pos[a]
        for e in sorted(rooms[a]['exits'], key=lambda e: e['d']):
            b = e['to']
            if b not in members or b in pos: continue
            dx, dy = VEC[e['d']]
            k = 1
            while (ax + dx*k, ay + dy*k) in occ: k += 1
            p = (ax + dx*k, ay + dy*k); pos[b] = p; occ[p] = b; q.append(b)
    return pos

blocks = []  # each: {rooms:set, pos:{v:(x,y)}, bbox}
assigned = {}
by_zone = collections.defaultdict(set)
for v, r in rooms.items(): by_zone[r['zone']].add(v)
for z, members in by_zone.items():
    left = set(members)
    # undirected adjacency within zone to find components; seed with most-connected low vnum
    while left:
        seed = min(left)
        if z == zone_of(3001) and 3001 in left: seed = 3001
        pos = layout(seed, left)
        bi = len(blocks); blocks.append({'pos': pos, 'zone': z})
        for v in pos: assigned[v] = bi
        left -= set(pos)
        # also pull in rooms only reachable inward (reverse exits) by reseeding: handled by loop
for b in blocks:
    xs = [p[0] for p in b['pos'].values()]; ys = [p[1] for p in b['pos'].values()]
    b['bbox'] = (min(xs), min(ys), max(xs), max(ys))

# 2. place blocks relative to each other via inter-zone exits
links = collections.defaultdict(list)
for a, r in rooms.items():
    for e in r['exits']:
        if e['to'] in rooms and assigned[e['to']] != assigned[a]:
            links[assigned[a]].append((a, e['d'], e['to']))
            links[assigned[e['to']]].append((e['to'], REV[e['d']], a))

GAP = 2
placed = {}
def rect(bi, off):
    x0, y0, x1, y1 = blocks[bi]['bbox']
    return (x0 + off[0], y0 + off[1], x1 + off[0], y1 + off[1])
def collides(r, cluster):
    for bj in cluster:
        s = rect(bj, placed[bj])
        if not (r[2] + GAP <= s[0] or s[2] + GAP <= r[0] or r[3] + GAP <= s[1] or s[3] + GAP <= r[1]):
            return True
    return False

order = sorted(range(len(blocks)), key=lambda b: -len(blocks[b]['pos']))
start = assigned[3001]
clusters = []
for root in [start] + [b for b in order if b != start]:
    if root in placed: continue
    cluster = [root]; placed[root] = (0, 0)
    q = collections.deque([root])
    while q:
        bi = q.popleft()
        for a, d, b in links[bi]:
            bj = assigned[b]
            if bj in placed: continue
            ax, ay = blocks[bi]['pos'][a]; ax += placed[bi][0]; ay += placed[bi][1]
            bx, by = blocks[bj]['pos'][b]
            dx, dy = VEC[d]
            k = 1
            while True:
                off = (ax + dx*k - bx, ay + dy*k - by)
                if not collides(rect(bj, off), cluster): break
                k += 1
            placed[bj] = off; cluster.append(bj); q.append(bj)
    rs = [rect(bj, placed[bj]) for bj in cluster]
    clusters.append((cluster, (min(r[0] for r in rs), min(r[1] for r in rs), max(r[2] for r in rs), max(r[3] for r in rs))))

# Stack the clusters that can't be reached from the main map in columns just
# to its right, top-aligned with it, starting a new column when one would run
# past the bottom of the main map. Most are single rooms (Limbo, private rooms),
# so this keeps them from stretching the page far wider than the world itself.
main, mb = clusters[0]
cx, cy, colw = mb[2] + 4, mb[1], 0
for cluster, bb in sorted(clusters[1:], key=lambda c: (-(c[1][3]-c[1][1]), min(c[0]))):
    w, h = bb[2]-bb[0], bb[3]-bb[1]
    if cy + h > mb[3] and cy > mb[1]:
        cx += colw + 4; cy = mb[1]; colw = 0
    dx, dy = cx - bb[0], cy - bb[1]
    for bj in cluster: placed[bj] = (placed[bj][0] + dx, placed[bj][1] + dy)
    cy += h + 2; colw = max(colw, w)

out_rooms = []
for v, r in rooms.items():
    bi = assigned[v]; x, y = blocks[bi]['pos'][v]; ox, oy = placed[bi]
    out_rooms.append({'v': v, 'n': r['name'], 'z': r['zone'], 's': r['sector'], 'f': r['flags'],
                      'x': x + ox, 'y': y + oy})
edges, seen = [], set()
for a, r in rooms.items():
    for e in r['exits']:
        b = e['to']
        if b not in rooms: continue
        back = any(x['to'] == a and x['d'] == REV[e['d']] for x in rooms[b]['exits'])
        key = (min(a, b), max(a, b), min(e['d'], REV[e['d']]))
        if back and key in seen: continue
        seen.add(key)
        edges.append({'a': a, 'b': b, 'd': e['d'], 'one': not back, 'door': e['door']})
data = {'zones': [z[1] for z in zones], 'rooms': out_rooms, 'edges': edges}
args.out.parent.mkdir(parents=True, exist_ok=True)
if args.json:
    args.out.write_text(json.dumps(data))
else:
    tpl = (HERE / 'template.html').read_text().replace('__DATA__', json.dumps(data, separators=(',', ':')))
    cut = tpl.index('<header>')  # title, fonts and styles go in <head>; the rest is the body
    page = ('<!doctype html>\n<html lang="en">\n<head>\n<meta charset="utf-8">\n'
            '<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">\n'
            '<style>body{margin:0}[hidden]{display:none!important}</style>\n'
            + tpl[:cut] + '</head>\n<body>\n' + tpl[cut:] + '</body>\n</html>\n')
    args.out.write_text(page)
print(f'{args.out}: {len(out_rooms)} rooms, {len(edges)} exits, {sum(1 for z in by_zone)} zones with rooms')
