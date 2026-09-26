import pytest
from coder.coder import VrlCoder
from unittest.mock import patch, MagicMock

def test_dynamic_vrl_prompt():
    c = VrlCoder({"device": "cpu", "coder": {"enabled": True, "host": "http://127.0.0.1:8088"}})
    with patch("requests.post") as mock_post:
        # Valid VRL that passes validation
        valid_vrl = '''.message = parse_syslog!(.message)
.ip = parse_regex!(string!(.message), r'(?P<ip>\\d+\\.\\d+\\.\\d+\\.\\d+)').ip
.class_uid = 3001
.category_uid = 3
.type_uid = 300101
.src_endpoint.ip = .ip
.dst_endpoint.ip = .ip'''
        
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = {"choices": [{"message": {"content": valid_vrl}}]}
        mock_post.return_value = mock_resp
        
        c.generate_vrl("some log", "Authentication")
        
        args, kwargs = mock_post.call_args
        json_data = kwargs["json"]
        
        system_prompt = next(m["content"] for m in json_data["messages"] if m["role"] == "system")
        assert "Vector Remap Language (VRL)" in system_prompt
        assert "OCSF Class UIDs" in system_prompt
        assert "3001" in system_prompt  # Authentication class_uid
        assert "parse_regex" in system_prompt
