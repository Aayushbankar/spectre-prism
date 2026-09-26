import os

def test_vrl_object_initialization():
    rules_dir = "/mnt/work/projects/sih/prism/rules"
    vrl_files = ["fortinet.vrl", "cisco_asa.vrl", "paloalto.vrl"]
    
    for filename in vrl_files:
        path = os.path.join(rules_dir, filename)
        assert os.path.exists(path), f"{filename} missing"
        
        with open(path, "r") as f:
            content = f.read()
            
        assert ".device = {}" in content, f"{filename} missing .device = {{}}"
        assert ".src_endpoint = {}" in content, f"{filename} missing .src_endpoint = {{}}"
        assert ".dst_endpoint = {}" in content, f"{filename} missing .dst_endpoint = {{}}"

def test_vrl_fortinet_extended_fields():
    with open("/mnt/work/projects/sih/prism/rules/fortinet.vrl", "r") as f:
        content = f.read()
        
    assert ".src_endpoint.port = .srcport" in content
    assert ".dst_endpoint.port = .dstport" in content
    assert ".action = .action" in content
    assert ".network.sent_bytes = .sentbyte" in content

def test_vrl_cisco_asa_extended_fields():
    with open("/mnt/work/projects/sih/prism/rules/cisco_asa.vrl", "r") as f:
        content = f.read()
        
    assert "parse_regex!(.message," in content
    assert ".src_endpoint.ip = parsed.src_ip" in content

def test_vrl_paloalto_extended_fields():
    with open("/mnt/work/projects/sih/prism/rules/paloalto.vrl", "r") as f:
        content = f.read()
        
    assert ".src_endpoint.ip = .[7]" in content
    assert ".dst_endpoint.ip = .[8]" in content
    assert ".src_endpoint.port = .[24]" in content
    assert ".dst_endpoint.port = .[25]" in content
