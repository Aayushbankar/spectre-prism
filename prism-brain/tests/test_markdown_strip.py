import pytest
import requests
from coder.coder import VrlCoder

class MockResponse:
    def __init__(self, content, status_code=200):
        self._content = content
        self.status_code = status_code
        
    def json(self):
        return {
            "choices": [{"message": {"content": self._content}}]
        }

# Valid VRL that passes validation (has .class_uid, .category_uid, .type_uid)
VALID_VRL = '''.message = parse_syslog!(.message)
.ip = parse_regex!(string!(.message), r'(?P<ip>\\d+\\.\\d+\\.\\d+\\.\\d+)').ip
.class_uid = 4001
.category_uid = 4
.type_uid = 400101
.src_endpoint.ip = .ip
.dst_endpoint.ip = .ip'''

@pytest.mark.parametrize("input_content, expected_contains", [
    (f"```vrl\n{VALID_VRL}\n```", '.class_uid = 4001'),
    (f"```\n{VALID_VRL}\n```", '.category_uid = 4'),
    (VALID_VRL, '.type_uid = 400101'),
])
def test_markdown_strip(monkeypatch, input_content, expected_contains):
    coder = VrlCoder()
    coder.enabled = True
    
    def mock_post(*args, **kwargs):
        return MockResponse(input_content)
        
    monkeypatch.setattr(requests, "post", mock_post)
    
    result = coder.generate_vrl("template", "test")
    assert expected_contains in result
