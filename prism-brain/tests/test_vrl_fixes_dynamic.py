import pytest
from coder.coder import VrlCoder
from unittest.mock import patch, MagicMock

def test_dynamic_vrl_prompt():
    c = VrlCoder({"device": "cpu", "coder": {"enabled": True, "host": "http://127.0.0.1:8088"}})
    with patch("requests.post") as mock_post:
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = {"choices": [{"message": {"content": ".ip = 1"}}]}
        mock_post.return_value = mock_resp
        
        c.generate_vrl("some log", "Authentication")
        
        args, kwargs = mock_post.call_args
        json_data = kwargs["json"]
        
        system_prompt = next(m["content"] for m in json_data["messages"] if m["role"] == "system")
        assert "Vector Remap Language (VRL) mapping logs to the OCSF taxonomy" in system_prompt
        assert ".class_uid" in system_prompt
        assert ".category_uid" in system_prompt
