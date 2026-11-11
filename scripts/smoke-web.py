#!/usr/bin/env python3
"""Exercise a running Thetta website with disposable, isolated workspaces."""
import argparse
import http.cookiejar
import io
import json
from pathlib import Path
import time
import urllib.error
import urllib.request
import zipfile

parser = argparse.ArgumentParser()
parser.add_argument('--url', default='https://thetta.finn.xyz')
parser.add_argument('--cookie-file', type=Path, help='File containing an authenticated Thetta session cookie')
args = parser.parse_args()
jar = http.cookiejar.CookieJar()
client = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))
root = Path(__file__).resolve().parents[1]
session_cookie = args.cookie_file.read_text().strip() if args.cookie_file else None
if not session_cookie:
    raise SystemExit('Sign in first, then supply a private --cookie-file. This test creates and modifies an isolated workspace.')


def call(path, data=None, method='POST', content_type='application/json'):
    body = json.dumps(data).encode() if data is not None else None
    req = urllib.request.Request(args.url + '/api/' + path, body, method=method)
    req.add_header('Cookie', session_cookie)
    req.add_header('Origin', args.url.rstrip('/'))
    if body:
        req.add_header('Content-Type', content_type)
    with client.open(req, timeout=60) as response:
        return json.load(response)


def upload(name):
    boundary = 'thetta-smoke-boundary'
    body = (f'--{boundary}\r\nContent-Disposition: form-data; name="file"; filename="{name}"\r\n'
            'Content-Type: application/octet-stream\r\n\r\n').encode()
    body += (root / 'test_data/step' / name).read_bytes()
    body += f'\r\n--{boundary}--\r\n'.encode()
    req = urllib.request.Request(args.url + '/api/upload', body,
                                 {'Content-Type': f'multipart/form-data; boundary={boundary}', 'Cookie': session_cookie, 'Origin': args.url.rstrip('/')})
    with client.open(req, timeout=60) as response:
        return json.load(response)


def nest():
    call('nesting/start')
    for _ in range(150):
        time.sleep(.1)
        snapshot = call('workspace', method='GET')
        if not snapshot['running']:
            assert snapshot['workspace']['result'], snapshot['status']
            return snapshot
    raise AssertionError('Nesting did not finish')


assert call('health', method='GET')['status'] == 'ok'
call('workspaces', {'id': None})
call('workspace')
w = upload('multi_thickness.step')['workspace']
assert len(w['parts']) == 4
assert {round(p['thickness_mm']) for p in w['parts']} == {2, 5}
w = upload('single_box_20x10x2.step')['workspace']
s = w['settings']
s['sheet'] = {'width': 15, 'height': 25, 'separation': 1}
s['optimizer']['time_limit_secs'] = .3
s['tools'] = {'tools': [{'number': 1, 'name': '6 mm end mill', 'diameter': 6,
                        'flute_count': 2, 'feed_rate': 1500, 'plunge_rate': 500, 'spindle_speed': 18000}]}
s['gcode']['tool_number'] = 1
s['gcode']['cut_depth'] = 2
s['drill']['enabled'] = False
s['pocket']['enabled'] = False
s['tabs'].update(auto_size=False, tab_width=2, tab_height=1, material_thickness=2,
                 corner_exclusion=1, tab_distance=10)
call('settings', s, 'PUT')
call('parts/0', {'quantity': 3, 'enabled': True, 'rotation': 'fixed'}, 'PUT')
v = nest()
sheets = v['workspace']['result']['sheets']
assert len(sheets) == 3, f'Expected three sheets, got {len(sheets)}; profiles: {v["pieces"]}'
assert sum(len(sheet['placements']) for sheet in sheets) == 3
v = call('tabs')
assert sum(len(t['tabs']) for t in v['workspace']['tabs']) > 0
for kind in ['step', 'gcode']:
    with client.open(urllib.request.Request(args.url + '/api/export/' + kind, method='POST', headers={'Cookie':session_cookie,'Origin':args.url.rstrip('/')}), timeout=60) as response:
        archive = zipfile.ZipFile(io.BytesIO(response.read()))
        assert len(archive.namelist()) == 3
        for name in archive.namelist():
            data = archive.read(name)
            assert len(data) > 100
            if kind == 'step':
                assert b'ISO-10303-21' in data
            else:
                assert b'G54' in data
# Source-too-large path: report unplaced parts without crashing.
s['sheet'] = {'width': 5, 'height': 5, 'separation': 1}
call('settings', s, 'PUT')
v = nest()
assert len(v['workspace']['result']['unplaced']) == 3
# Exercise strip mode on uploaded STEP profiles as well.
s['sheet'] = {'width': 100, 'height': 30, 'separation': 1}
s['mode'] = 'strip'
call('settings', s, 'PUT')
v = nest()
assert len(v['workspace']['result']['sheets']) == 1
assert len(v['workspace']['result']['sheets'][0]['placements']) == 3
assert v['workspace']['result']['sheets'][0]['strip_length'] > 0
print('PASS: STEP import, thickness filters, quantities, multi-sheet nesting, tabs, both exports, oversized parts, strip mode')
print('Smoke workspace is saved; session credentials were not printed.')
