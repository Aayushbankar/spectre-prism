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

@pytest.mark.parametrize("input_content, expected", [
    ("```vrl\n.ip = \"1.2.3.4\"\n```", '.ip = "1.2.3.4"'),
    ("```\n.ip = \"1.2.3.4\"\n```", '.ip = "1.2.3.4"'),
    (".ip = \"1.2.3.4\"", '.ip = "1.2.3.4"'),
    ("```\nfirst\n```\n```\nsecond\n```", 'first\nsecond'),
])
def test_markdown_strip(monkeypatch, input_content, expected):
    coder = VrlCoder()
    coder.enabled = True
    
    def mock_post(*args, **kwargs):
        return MockResponse(input_content)
        
    monkeypatch.setattr(requests, "post", mock_post)
    
    result = coder.generate_vrl("template", "test")
    assert result == expected
