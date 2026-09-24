"""
Cluster Module: Drain3 fixed-depth tree grouping raw logs into templates.
"""
from drain3 import TemplateMiner
from drain3.template_miner_config import TemplateMinerConfig

class LogClusterer:
    def __init__(self):
        config = TemplateMinerConfig()
        config.load("") # default config
        config.profiling_enabled = False
        self.miner = TemplateMiner(config=config)

    def process_log(self, log_line: str) -> dict:
        """Process a raw log line and return its cluster template."""
        result = self.miner.add_log_message(log_line)
        return {
            "cluster_id": result["cluster_id"],
            "template": result["template_mined"],
            "size": result["cluster_size"]
        }
