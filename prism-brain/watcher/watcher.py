"""
Watcher Module: File watchdog detecting entries in dlq.jsonl.
"""
import json
import os
from typing import Callable
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler

class DlqEventHandler(FileSystemEventHandler):
    def __init__(self, target_file: str, callback: Callable[[dict], None]):
        self.target_file = target_file
        self.callback = callback
        self.last_position = 0

    def on_modified(self, event):
        if os.path.basename(event.src_path) == self.target_file:
            self.process_new_lines(event.src_path)

    def process_new_lines(self, path: str) -> None:
        try:
            with open(path, 'r') as f:
                f.seek(self.last_position)
                lines = f.readlines()
                self.last_position = f.tell()
                for line in lines:
                    if line.strip():
                        try:
                            self.callback(json.loads(line))
                        except json.JSONDecodeError:
                            pass
        except FileNotFoundError:
            pass

def start_watcher(path: str, callback: Callable[[dict], None]) -> Observer:
    """Start watching the DLQ path for new logs."""
    target_dir = os.path.dirname(path)
    target_file = os.path.basename(path)
    if not os.path.exists(target_dir):
        os.makedirs(target_dir, exist_ok=True)
    
    event_handler = DlqEventHandler(target_file, callback)
    observer = Observer()
    observer.schedule(event_handler, target_dir, recursive=False)
    observer.start()
    return observer
