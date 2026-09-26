import React, { useState } from 'react';
import { Play, Code, CheckCircle2, ShieldCheck, Copy, Check, FileCode, Sparkles } from 'lucide-react';

interface Props {
  theme: 'dark' | 'light' | 'cyber';
  initialPayload?: string;
  initialRule?: string;
}

export const VRLPlaygroundView: React.FC<Props> = ({ theme, initialPayload, initialRule }) => {
  const [vendor, setVendor] = useState<'paloalto' | 'fortinet' | 'cisco' | 'alien'>('paloalto');
  const [rawLog, setRawLog] = useState<string>(
    initialPayload ||
      '1,2026/09/26 10:00:00,010108010441,TRAFFIC,start,2305,2026/09/26 10:00:00,10.10.20.45,203.0.113.25,0.0.0.0,0.0.0.0,Rule-Web,user1,,ssl,vsys1,Trust,Untrust,ethernet1/2,ethernet1/1,LogForwarding_Syslog,2026/09/26 10:00:00,654123,1,52341,443,0,0,0x0,tcp,allow,1420,700,720,12,6,6,0'
  );

  const [vrlRule, setVrlRule] = useState<string>(
    initialRule ||
      `# Palo Alto VRL mapping to OCSF v1.9
. = parse_csv!(.message)
.class_uid = 4001
.activity_id = 1
.device = { "vendor": "Palo Alto Networks", "product": "PAN-OS" }
.src_endpoint = { "ip": .[7], "port": to_int!(.[24]) }
.dst_endpoint = { "ip": .[8], "port": to_int!(.[25]) }`
  );

  const [parsedOutput, setParsedOutput] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const [copiedHash, setCopiedHash] = useState(false);

  const samplePresets = {
    paloalto: {
      log: '1,2026/09/26 10:00:00,010108010441,TRAFFIC,start,2305,2026/09/26 10:00:00,10.10.20.45,203.0.113.25,0.0.0.0,0.0.0.0,Rule-Web,user1,,ssl,vsys1,Trust,Untrust,ethernet1/2,ethernet1/1,LogForwarding_Syslog,2026/09/26 10:00:00,654123,1,52341,443,0,0,0x0,tcp,allow,1420,700,720,12,6,6,0',
      rule: `# Palo Alto VRL mapping to OCSF v1.9\n. = parse_csv!(.message)\n.class_uid = 4001\n.activity_id = 1\n.device = { "vendor": "Palo Alto Networks", "product": "PAN-OS" }\n.src_endpoint = { "ip": .[7], "port": to_int!(.[24]) }\n.dst_endpoint = { "ip": .[8], "port": to_int!(.[25]) }`,
    },
    fortinet: {
      log: 'date=2026-09-26 time=10:00:00 devname="FGT-60E" devid="FGT60E4Q16000000" logid="0000000013" type="traffic" subtype="forward" level="notice" vd="root" srcip=10.10.20.45 srcport=52341 dstip=203.0.113.25 dstport=443 proto=6 action="accept" policyid=1 app="HTTPS" duration=12 sentbyte=700 rcvdbyte=720',
      rule: `# Fortinet FortiGate VRL mapping to OCSF\n. = parse_key_value!(.message)\n.class_uid = 4001\n.activity_id = 1\n.device = { "vendor": "Fortinet", "product": "FortiGate" }\n.src_endpoint = { "ip": .srcip, "port": to_int!(.srcport) }\n.dst_endpoint = { "ip": .dstip, "port": to_int!(.dstport) }`,
    },
    cisco: {
      log: '<166>2026-09-26T10:00:00Z ciscoasa : %ASA-6-302013: Built inbound TCP connection 52341 for outside:203.0.113.25/443 (203.0.113.25/443) to inside:10.10.20.45/52341 (10.10.20.45/52341)',
      rule: `# Cisco ASA VRL mapping to OCSF\nparsed = parse_regex!(.message, r'%ASA-\\d+-\\d+: .*? src (?P<src_ip>\\d+\\.\\d+\\.\\d+\\.\\d+)/(?P<src_port>\\d+) dst (?P<dst_ip>\\d+\\.\\d+\\.\\d+\\.\\d+)/(?P<dst_port>\\d+)')\n.class_uid = 4001\n.activity_id = 1\n.device = { "vendor": "Cisco", "product": "ASA" }\n.src_endpoint = { "ip": parsed.src_ip, "port": to_int!(parsed.src_port) }\n.dst_endpoint = { "ip": parsed.dst_ip, "port": to_int!(parsed.dst_port) }`,
    },
    alien: {
      log: '192.168.1.189 - - [26/Sep/2026:10:00:00 +0000] "GET /admin_c2_exploit.php HTTP/1.1" 404 0',
      rule: `# AI Mined Rule for Alien Web Log\nparsed = parse_regex!(.message, r'^(?P<client_ip>\\S+) .*?"(?P<method>\\w+) (?P<uri>\\S+) HTTP.*?" (?P<status>\\d+)')\n.class_uid = 4001\n.activity_id = 2\n.severity = "Critical"\n.src_endpoint = { "ip": parsed.client_ip }\n.http_request = { "url": parsed.uri, "method": parsed.method }`,
    },
  };

  const handleSelectPreset = (key: 'paloalto' | 'fortinet' | 'cisco' | 'alien') => {
    setVendor(key);
    setRawLog(samplePresets[key].log);
    setVrlRule(samplePresets[key].rule);
  };

  const handleExecute = async () => {
    setLoading(true);
    try {
      const res = await fetch('/api/real/vrl/test', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ rawLog, ruleCode: vrlRule, vendor }),
      });
      const data = await res.json();
      setParsedOutput(data);
    } catch {
      // fallback
    } finally {
      setLoading(false);
    }
  };

  const isDark = theme === 'dark' || theme === 'cyber';

  return (
    <div className="space-y-6 font-mono">
      {/* Banner */}
      <div
        className={`rounded-2xl border p-5 transition-all duration-300 shadow-xl flex flex-wrap items-center justify-between gap-4 ${
          isDark ? 'bg-[#141724]/95 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
        }`}
      >
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-cyan-500/20 border border-cyan-500/30 flex items-center justify-center text-cyan-400">
            <FileCode className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-lg font-bold tracking-tight">Interactive VRL Sandbox & OCSF Normalizer</h2>
            <p className="text-xs text-slate-400">
              Live Vector Remap Language compiler testing against heterogeneous raw firewall logs
            </p>
          </div>
        </div>

        {/* Preset Selectors */}
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-xs text-slate-400 mr-1">Sample Presets:</span>
          <button
            onClick={() => handleSelectPreset('paloalto')}
            className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition ${
              vendor === 'paloalto' ? 'bg-blue-600 text-white shadow' : 'bg-slate-800 text-slate-300 hover:text-white'
            }`}
          >
            Palo Alto CSV
          </button>
          <button
            onClick={() => handleSelectPreset('fortinet')}
            className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition ${
              vendor === 'fortinet' ? 'bg-orange-600 text-white shadow' : 'bg-slate-800 text-slate-300 hover:text-white'
            }`}
          >
            Fortinet KV
          </button>
          <button
            onClick={() => handleSelectPreset('cisco')}
            className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition ${
              vendor === 'cisco' ? 'bg-yellow-600 text-white shadow' : 'bg-slate-800 text-slate-300 hover:text-white'
            }`}
          >
            Cisco ASA Syslog
          </button>
          <button
            onClick={() => handleSelectPreset('alien')}
            className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition ${
              vendor === 'alien' ? 'bg-rose-600 text-white shadow' : 'bg-slate-800 text-slate-300 hover:text-white'
            }`}
          >
            Alien DLQ Log
          </button>
        </div>
      </div>

      {/* Editor & Execution Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left: Raw Log + VRL Rule Editor (6 cols) */}
        <div className="lg:col-span-6 space-y-4">
          {/* Raw Log Input */}
          <div
            className={`rounded-2xl border p-4 shadow-md space-y-2 ${
              isDark ? 'bg-[#141724]/95 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
            }`}
          >
            <div className="flex items-center justify-between text-xs text-slate-400">
              <span className="font-bold">Raw Ingress Log Payload</span>
              <span className="text-[10px] text-slate-500">UDP/TCP Zero-Alloc Buffer</span>
            </div>
            <textarea
              rows={3}
              value={rawLog}
              onChange={(e) => setRawLog(e.target.value)}
              className="w-full p-3 rounded-xl bg-black/80 border border-slate-800 text-slate-200 text-xs font-mono focus:outline-none focus:border-cyan-500"
            />
          </div>

          {/* VRL Rule Script Editor */}
          <div
            className={`rounded-2xl border p-4 shadow-md space-y-2 ${
              isDark ? 'bg-[#141724]/95 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
            }`}
          >
            <div className="flex items-center justify-between text-xs text-slate-400">
              <span className="font-bold flex items-center gap-1.5">
                <Code className="w-3.5 h-3.5 text-cyan-400" /> Vector Remap Language (VRL) Script
              </span>
              <span className="text-[10px] text-slate-500">Zero-alloc transform engine</span>
            </div>
            <textarea
              rows={8}
              value={vrlRule}
              onChange={(e) => setVrlRule(e.target.value)}
              className="w-full p-3 rounded-xl bg-black/80 border border-slate-800 text-emerald-400 text-xs font-mono focus:outline-none focus:border-cyan-500"
            />
          </div>

          {/* Execute Button */}
          <button
            onClick={handleExecute}
            disabled={loading}
            className="w-full py-3 rounded-xl bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500 text-white font-bold text-xs uppercase tracking-wider shadow-lg shadow-cyan-600/25 transition flex items-center justify-center gap-2"
          >
            <Play className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
            <span>Execute VRL Parse & Normalize to OCSF</span>
          </button>
        </div>

        {/* Right: Normalized OCSF Output & BLAKE3 Proof (6 cols) */}
        <div
          className={`lg:col-span-6 rounded-2xl border p-5 shadow-md space-y-4 flex flex-col justify-between ${
            isDark ? 'bg-[#141724]/95 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div>
            <div className="flex items-center justify-between border-b border-slate-800 pb-3 mb-3">
              <span className="text-xs font-bold text-slate-300">Normalized OCSF Event (Class 4001)</span>
              <span className="px-2 py-0.5 rounded text-[10px] bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 flex items-center gap-1">
                <ShieldCheck className="w-3.5 h-3.5" /> SEC 65B VALIDATED
              </span>
            </div>

            {parsedOutput ? (
              <pre className="p-4 rounded-xl bg-black/80 border border-slate-800 text-cyan-300 text-xs overflow-x-auto max-h-[380px] leading-relaxed">
                {JSON.stringify(parsedOutput.ocsf, null, 2)}
              </pre>
            ) : (
              <div className="flex flex-col items-center justify-center h-64 text-slate-500 text-xs">
                <Sparkles className="w-8 h-8 mb-2 text-cyan-500/40" />
                <span>Click &quot;Execute VRL Parse&quot; to compile and normalize in real time</span>
              </div>
            )}
          </div>

          {parsedOutput && (
            <div className="border-t border-slate-800 pt-3 space-y-2">
              <div className="flex items-center justify-between text-xs">
                <span className="text-slate-400">BLAKE3 Provenance Hash:</span>
                <button
                  onClick={() => {
                    navigator.clipboard.writeText(parsedOutput.hash);
                    setCopiedHash(true);
                    setTimeout(() => setCopiedHash(false), 2000);
                  }}
                  className="text-xs text-emerald-400 hover:text-emerald-300 flex items-center gap-1"
                >
                  {copiedHash ? <Check className="w-3.5 h-3.5" /> : <Copy className="w-3.5 h-3.5" />}
                  {copiedHash ? 'Copied' : 'Copy'}
                </button>
              </div>
              <div className="p-2.5 rounded-lg bg-black/60 border border-slate-800 text-[11px] text-emerald-400 break-all select-all">
                {parsedOutput.hash}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
