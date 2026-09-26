"""
Gatekeeper Module: Robust HitL approval flow with state-safe atomic operations.
"""
import os
import json
import shutil
import time
import uuid
import subprocess
from pathlib import Path
from enum import Enum
from dataclasses import dataclass, asdict
from typing import Optional, List, Dict
from filelock import FileLock


class RuleState(Enum):
    PENDING = "pending"
    APPROVED = "approved"
    REJECTED = "rejected"
    DEPLOYED = "deployed"
    FAILED = "failed"


@dataclass
class RuleMetadata:
    rule_id: str
    device_type: str
    vendor_name: str
    signature: str
    state: str
    created_at: float
    updated_at: float
    vrl_path: str
    yaml_path: str
    raw_log_sample: str
    dry_run_result: Optional[str] = None
    error: Optional[str] = None


class Gatekeeper:
    def __init__(self, rules_dir: str = "/etc/prism/rules", base_dir: str = "/tmp/prism"):
        self.base_dir = Path(base_dir)
        self.rules_dir = Path(rules_dir)
        self.pending_dir = self.base_dir / "pending_rules"
        self.approved_dir = self.base_dir / "approved_rules"
        self.rejected_dir = self.base_dir / "rejected_rules"
        self.deployed_dir = self.base_dir / "deployed_rules"
        self.metadata_dir = self.base_dir / "rule_metadata"
        
        self._ensure_dirs()
        self._lock = FileLock(str(self.base_dir / ".gatekeeper.lock"))

    def _ensure_dirs(self):
        for d in [self.pending_dir, self.approved_dir, self.rejected_dir, 
                  self.deployed_dir, self.metadata_dir, self.rules_dir]:
            d.mkdir(parents=True, exist_ok=True)

    def _metadata_path(self, rule_id: str) -> Path:
        return self.metadata_dir / f"{rule_id}.json"

    def _save_metadata(self, meta: RuleMetadata):
        meta.updated_at = time.time()
        with self._lock:
            with open(self._metadata_path(meta.rule_id), 'w') as f:
                json.dump(asdict(meta), f, indent=2)

    def _load_metadata(self, rule_id: str) -> Optional[RuleMetadata]:
        path = self._metadata_path(rule_id)
        if not path.exists():
            return None
        with open(path, 'r') as f:
            data = json.load(f)
        return RuleMetadata(**data)

    def _list_rules_by_state(self, state: RuleState) -> List[RuleMetadata]:
        rules = []
        for meta_file in self.metadata_dir.glob("*.json"):
            try:
                meta = self._load_metadata(meta_file.stem)
                if meta and meta.state == state.value:
                    rules.append(meta)
            except Exception:
                pass
        return sorted(rules, key=lambda m: m.created_at, reverse=True)

    def _run_dry_run(self, vrl_path: Path, raw_log: str) -> tuple[bool, str]:
        """Run prism binary dry-run and return (success, output)."""
        try:
            prism_bin = Path("/mnt/work/projects/sih/prism/target/release/prism")
            if not prism_bin.exists():
                prism_bin = Path("prism")
            
            result = subprocess.run(
                [str(prism_bin), "--dry-run-vrl", str(vrl_path), "--payload", "-"],
                input=raw_log,
                capture_output=True,
                text=True,
                timeout=10
            )
            if result.returncode == 0:
                return True, result.stdout
            else:
                return False, result.stderr
        except FileNotFoundError:
            return False, "prism binary not found in PATH or target/release"
        except subprocess.TimeoutExpired:
            return False, "Dry-run timed out after 10 seconds"
        except Exception as e:
            return False, f"Dry-run error: {e}"

    def submit_rule(self, device_type: str, vrl_code: str, signature: str, 
                    raw_log: str = "") -> str:
        """Submit a new rule for HitL approval. Returns rule_id."""
        rule_id = f"rule_{device_type.replace(' ', '_').lower()}_{uuid.uuid4().hex[:8]}"
        vendor_name = device_type.replace(' ', '_').lower()
        
        with self._lock:
            # Write VRL to pending (atomic write via temp file)
            vrl_path = self.pending_dir / f"{vendor_name}_{rule_id}.vrl"
            tmp_vrl = vrl_path.with_suffix(".vrl.tmp")
            with open(tmp_vrl, 'w') as f:
                f.write(vrl_code)
            tmp_vrl.rename(vrl_path)
            
            # Write YAML metadata
            yaml_path = self.pending_dir / f"{vendor_name}_{rule_id}.yaml"
            yaml_content = f"signature: \"{signature}\"\nvrl_file: \"{vrl_path}\"\ndevice_type: \"{device_type}\"\n"
            tmp_yaml = yaml_path.with_suffix(".yaml.tmp")
            with open(tmp_yaml, 'w') as f:
                f.write(yaml_content)
            tmp_yaml.rename(yaml_path)
            
            # Create metadata record
            meta = RuleMetadata(
                rule_id=rule_id,
                device_type=device_type,
                vendor_name=vendor_name,
                signature=signature,
                state=RuleState.PENDING.value,
                created_at=time.time(),
                updated_at=time.time(),
                vrl_path=str(vrl_path),
                yaml_path=str(yaml_path),
                raw_log_sample=raw_log[:2000] if raw_log else ""
            )
            self._save_metadata(meta)
            
            # Run dry-run validation
            success, output = self._run_dry_run(vrl_path, raw_log)
            if not success:
                meta.state = RuleState.FAILED.value
                meta.error = output
                meta.dry_run_result = output
                self._save_metadata(meta)
                # Keep in pending dir for user review - don't move to rejected
                print(f"[Gatekeeper] Dry-run validation failed for {rule_id}: {output}")
                return rule_id
            
            meta.dry_run_result = output
            self._save_metadata(meta)
            
        return rule_id

    def _move_rule_files(self, meta: RuleMetadata, target_dir: Path):
        """Move rule files to target directory."""
        target_dir.mkdir(parents=True, exist_ok=True)
        for src_path_str in [meta.vrl_path, meta.yaml_path]:
            src = Path(src_path_str)
            if src.exists():
                dst = target_dir / src.name
                shutil.move(str(src), str(dst))
                # Update metadata paths
                if src_path_str == meta.vrl_path:
                    meta.vrl_path = str(dst)
                else:
                    meta.yaml_path = str(dst)

    def approve_rule(self, rule_id: str) -> bool:
        """Approve a pending rule and deploy to active rules directory."""
        meta = self._load_metadata(rule_id)
        if not meta:
            raise ValueError(f"Rule {rule_id} not found")
        if meta.state != RuleState.PENDING.value:
            raise ValueError(f"Rule {rule_id} is not in PENDING state (current: {meta.state})")
        
        with self._lock:
            # Move files to approved dir
            self._move_rule_files(meta, self.approved_dir)
            
            # Deploy to active rules directory (atomic copy)
            vrl_src = Path(meta.vrl_path)
            vrl_dst = self.rules_dir / vrl_src.name
            tmp_dst = vrl_dst.with_suffix(".vrl.tmp")
            shutil.copy2(vrl_src, tmp_dst)
            tmp_dst.rename(vrl_dst)
            
            # Update state
            meta.state = RuleState.APPROVED.value
            meta.vrl_path = str(vrl_dst)
            self._save_metadata(meta)
            
        return True

    def reject_rule(self, rule_id: str, reason: str = "Rejected by operator") -> bool:
        """Reject a pending rule."""
        meta = self._load_metadata(rule_id)
        if not meta:
            raise ValueError(f"Rule {rule_id} not found")
        if meta.state not in [RuleState.PENDING.value, RuleState.FAILED.value]:
            raise ValueError(f"Rule {rule_id} cannot be rejected (state: {meta.state})")
        
        with self._lock:
            self._move_rule_files(meta, self.rejected_dir)
            meta.state = RuleState.REJECTED.value
            meta.error = reason
            self._save_metadata(meta)
            
        return True

    def deploy_rule(self, rule_id: str) -> bool:
        """Deploy an approved rule to active rules (alias for approve if pending)."""
        meta = self._load_metadata(rule_id)
        if not meta:
            raise ValueError(f"Rule {rule_id} not found")
        if meta.state == RuleState.PENDING.value:
            return self.approve_rule(rule_id)
        elif meta.state == RuleState.APPROVED.value:
            # Already approved, ensure it's in rules dir
            vrl_src = Path(meta.vrl_path)
            vrl_dst = self.rules_dir / vrl_src.name
            if not vrl_dst.exists():
                shutil.copy2(vrl_src, vrl_dst)
            meta.state = RuleState.DEPLOYED.value
            self._save_metadata(meta)
            return True
        else:
            raise ValueError(f"Rule {rule_id} cannot be deployed (state: {meta.state})")

    def get_pending_rules(self) -> List[RuleMetadata]:
        return self._list_rules_by_state(RuleState.PENDING)

    def get_approved_rules(self) -> List[RuleMetadata]:
        return self._list_rules_by_state(RuleState.APPROVED)

    def get_rejected_rules(self) -> List[RuleMetadata]:
        return self._list_rules_by_state(RuleState.REJECTED)

    def get_all_rules(self) -> List[RuleMetadata]:
        rules = []
        for meta_file in self.metadata_dir.glob("*.json"):
            try:
                meta = self._load_metadata(meta_file.stem)
                if meta:
                    rules.append(meta)
            except Exception:
                pass
        return sorted(rules, key=lambda m: m.created_at, reverse=True)

    def get_rule_preview(self, rule_id: str) -> Optional[str]:
        """Get VRL content for preview."""
        meta = self._load_metadata(rule_id)
        if not meta:
            return None
        vrl_path = Path(meta.vrl_path)
        if vrl_path.exists():
            return vrl_path.read_text()
        return None

    def cleanup_old_rules(self, max_age_hours: int = 24, keep_deployed: int = 10):
        """Clean up old rule files and metadata."""
        cutoff = time.time() - (max_age_hours * 3600)
        deployed = self.get_approved_rules() + self._list_rules_by_state(RuleState.DEPLOYED)
        deployed = sorted(deployed, key=lambda m: m.updated_at, reverse=True)
        keep_ids = set(m.rule_id for m in deployed[:keep_deployed])
        
        with self._lock:
            for meta_file in self.metadata_dir.glob("*.json"):
                try:
                    meta = self._load_metadata(meta_file.stem)
                    if not meta:
                        continue
                    if meta.rule_id in keep_ids:
                        continue
                    if meta.updated_at < cutoff:
                        # Remove files
                        for p in [meta.vrl_path, meta.yaml_path]:
                            Path(p).unlink(missing_ok=True)
                        # Remove metadata
                        meta_file.unlink(missing_ok=True)
                except Exception:
                    pass


def create_gatekeeper(config: Dict = None) -> Gatekeeper:
    """Factory function for creating Gatekeeper with config."""
    config = config or {}
    rules_dir = config.get("rules_dir", "/etc/prism/rules")
    base_dir = config.get("base_dir", "/tmp/prism")
    return Gatekeeper(rules_dir=rules_dir, base_dir=base_dir)