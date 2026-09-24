# Golden Fixture — `golden_mixed.log` + `ocsf_expected.jsonl`

**Files**

| File | Lines | Content |
| :--- | :--- | :--- |
| `golden_mixed.log` | 35 | raw input: one log event per line (F-1…F-35) |
| `ocsf_expected.jsonl` | 35 | expected output: one compact JSON object per line (E-1…E-35), line-aligned with the `.log` |
| `README.md` | — | this file |

**Line alignment contract.** Line N of `golden_mixed.log` corresponds to line N of
`ocsf_expected.jsonl`. The harness reads both files in order (`zip(log, expected)`)
and compares the parser's OCSF output for input line N against expected line N.
Valid lines expect a full OCSF object (`category_uid`, `class_uid`, `activity_id`,
`time`, `src/dst_endpoint`, `connection_info`, `traffic`, `metadata`, …);
alien lines expect exactly `{"action":"REJECT"}`.

**Layout (35 = 30 valid + 5 alien)**

| Lines | Format | Class |
| :--- | :--- | :--- |
| F-1…F-3 | Fortinet FortiGate KV (RFC 5424): forward-close allow, forward deny, UTM/IPS drop | 4001 |
| F-4…F-7 | Cisco ASA (RFC 3164): 302013 build, 302014 teardown, 106100 ACL deny, 106023 deny | 4001 |
| F-8…F-10 | Palo Alto PAN-OS CSV: TRAFFIC allow, TRAFFIC drop (icmp), THREAT vuln-block | 4001 |
| F-11…F-15 | Suricata EVE JSON: alert→4001, http→4002, dns→4003, tls→4001, flow→4001 | mixed |
| F-16…F-18 | NGINX combined: GET 200, POST /login 401, SQLi-probe 403 | 4002 |
| F-19 | CEF (PAN-OS threat via ArcSight envelope) | 4001 |
| F-20 | LEEF 2.0 (`^`-delimited, Check Point session) | 4001 |
| F-21 | Check Point Log Exporter accept | 4001 |
| F-22…F-23 | VPN auth AnyConnect success (alice) / failure (bob) | 3002 |
| F-24…F-30 | Second-pass variants (no new syntax): Fortinet close, ASA UDP teardown, PAN drop/dns, Suricata Log4Shell alert, NGINX 404, Check Point Drop, VPN failure (carol) | mixed |
| F-31…F-35 | **Alien/ambiguous → `{"action":"REJECT"}`**: unknown `FROBNICATOR` vendor, unknown JSON schema, unknown `weirddaemon` free text, `TRUNCATED BINARY` marker, dots-only line | REJECT |

Full per-line syntax, field tables, and pretty-printed OCSF for every pattern live in
`docs/datasets/VENDOR_OCSF_TRUTH.md` (§1–§11). Placeholder values
`BLAKE3:<computed-at-ingest>` and `parquet:<block>#<offset>` in `metadata` are filled
by the pipeline at ingest/vault time — the harness must ignore or substitute them
before comparison.

**Validate**

```bash
wc -l datasets/fixtures/golden_mixed.log datasets/fixtures/ocsf_expected.jsonl
python3 -c "import json; [json.loads(l) for l in open('datasets/fixtures/ocsf_expected.jsonl',encoding='utf-8')]; print('JSONL OK')"
```

**How to extend**

1. Append new raw line(s) to `golden_mixed.log` (one event per line, UTF-8, LF, no
   embedded newlines; keep timestamps at/after `2024-09-22T14:30:01Z`).
2. Append the matching expected object(s) to `ocsf_expected.jsonl` in the same
   position — full OCSF for parseable events, exactly `{"action":"REJECT"}` for
   DLQ-bound alien lines. Keep keys sorted and separators compact
   (`json.dumps(obj, separators=(",",":"), sort_keys=True)`).
3. Document the new pattern in `docs/datasets/VENDOR_OCSF_TRUTH.md` (sample lines
   marked SYNTHETIC-BUT-VALID + field table + full OCSF JSON) and update the table
   above.
4. Re-run the Validate block; both files must stay line-count-equal and the JSONL
   must parse. Do NOT commit unless asked.
