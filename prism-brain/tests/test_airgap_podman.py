import os
import subprocess

def test_rust_binary_builds_static():
    """Verify the prism binary can be built (prerequisite for air-gap)."""
    result = subprocess.run(
        ["cargo", "build", "--workspace"],
        capture_output=True, text=True,
        cwd=os.path.join(os.path.dirname(__file__), "../..") 
    )
    assert result.returncode == 0, f"cargo build failed: {result.stderr}"

def test_no_network_dependencies_in_core():
    """Verify core pipeline has no hardcoded external URLs."""
    import re
    core_dir = os.path.join(os.path.dirname(__file__), "../../crates")
    for root, dirs, files in os.walk(core_dir):
        for f in files:
            if f.endswith('.rs'):
                content = open(os.path.join(root, f)).read()
                # Should not contain hardcoded external URLs (localhost/internal is OK)
                external_urls = re.findall(r'https?://(?!localhost|127\.0\.0\.1|0\.0\.0\.0)[^"\s]+', content)
                assert len(external_urls) == 0, f"External URL found in {f}: {external_urls}"
