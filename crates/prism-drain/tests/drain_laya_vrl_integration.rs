use prism_drain::Drain;
use prism_vrl_generator::{VrlGenerator, compile_vrl, run_dry_run};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[tokio::test]
async fn test_real_logs_drain_laya_vrl_pipeline() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to locate workspace root");

    let iptables_path = workspace_root.join("data/real_corpora/iptables.log");
    let snort_path = workspace_root.join("data/real_corpora/snort.log");

    if !iptables_path.exists() || !snort_path.exists() {
        eprintln!("Real corpora logs not found, skipping integration test");
        return;
    }

    // Step 1: Drain clustering on iptables logs
    let mut iptables_drain = Drain::with_capacity(100);
    let file = File::open(&iptables_path).expect("Failed to open iptables.log");
    let reader = BufReader::new(file);

    for line in reader.lines().take(50).flatten() {
        if !line.trim().is_empty() {
            iptables_drain.process(&line);
        }
    }

    assert!(!iptables_drain.is_empty(), "Drain should have formed clusters for iptables");
    let ranked = iptables_drain.ranked_templates();
    assert!(!ranked.is_empty(), "Ranked templates should not be empty");
    assert!(ranked[0].count > 1, "Top cluster should have multiple hits");

    // Step 2: Enrich with Laya
    let enriched = iptables_drain
        .enrich_with_laya()
        .await
        .expect("Laya enrichment should succeed");

    assert!(!enriched.is_empty(), "Enriched templates should not be empty");
    let top = &enriched[0];
    assert_eq!(top.class_uid, 4001, "iptables should be classified as Network Activity (4001)");
    assert_eq!(top.category_uid, 4, "category_uid should be 4");
    assert_eq!(top.vendor, "linux", "iptables vendor should be linux");
    assert!(top.confidence > 0.5, "Confidence should be high for iptables");
    assert!(!top.detector_terms.is_empty(), "Detector terms should be extracted");

    // Step 3: Feed enriched template to prism-vrl-generator
    let generator = VrlGenerator::heuristic();
    let vrl = generator
        .generate_from_template(top)
        .await
        .expect("VRL generation should succeed");

    assert!(vrl.validation_passed, "Generated VRL must pass validation");
    assert!(vrl.content.contains(".class_uid = 4001"), "VRL should have class_uid 4001");
    assert!(vrl.content.contains(".category_uid = 4"), "VRL should have category_uid 4");
    assert!(vrl.content.contains(".src_endpoint.ip"), "VRL should extract source endpoint IP");
    assert!(!vrl.content.contains("parse_syslog"), "VRL must NOT contain parse_syslog");
    assert!(!vrl.content.contains("```"), "VRL must NOT contain markdown code fences");

    // Verify VRL compiles and runs dry-run with real VRL engine
    compile_vrl(&vrl.content).expect("Generated VRL must compile with vrl crate");
    let dry_run_res = run_dry_run(&vrl.content, &top.samples[0]).expect("VRL dry-run must succeed");
    assert_eq!(dry_run_res["class_uid"], 4001);

    // Step 4: Test Snort clustering + Laya + VRL
    let mut snort_drain = Drain::with_capacity(100);
    let snort_file = File::open(&snort_path).expect("Failed to open snort.log");
    let snort_reader = BufReader::new(snort_file);

    for line in snort_reader.lines().take(50).flatten() {
        if !line.trim().is_empty() {
            snort_drain.process(&line);
        }
    }

    assert!(!snort_drain.is_empty(), "Drain should have formed clusters for snort");
    let snort_enriched = snort_drain
        .enrich_with_laya()
        .await
        .expect("Snort Laya enrichment should succeed");

    assert!(!snort_enriched.is_empty(), "Snort enriched templates should not be empty");
    let snort_top = &snort_enriched[0];
    assert_eq!(snort_top.vendor, "snort", "Snort vendor should be detected as snort");
    assert!(snort_top.confidence > 0.5, "Confidence should be high for snort");
    assert!(!snort_top.detector_terms.is_empty(), "Snort detector terms should be extracted");

    let snort_vrl = generator
        .generate_from_template(snort_top)
        .await
        .expect("Snort VRL generation should succeed");

    assert!(snort_vrl.validation_passed, "Snort VRL must pass validation");
    assert!(!snort_vrl.content.contains("parse_syslog"), "Snort VRL must NOT contain parse_syslog");
    compile_vrl(&snort_vrl.content).expect("Snort VRL must compile with vrl crate");

    // Step 5: Test OpenSSH clustering + Laya + VRL
    let openssh_path = workspace_root.join("data/real_corpora/OpenSSH.full.log");
    if openssh_path.exists() {
        let mut ssh_drain = Drain::with_capacity(100);
        let ssh_file = File::open(&openssh_path).expect("Failed to open OpenSSH.full.log");
        let ssh_reader = BufReader::new(ssh_file);

        for line in ssh_reader.lines().take(50).flatten() {
            if !line.trim().is_empty() {
                ssh_drain.process(&line);
            }
        }

        assert!(!ssh_drain.is_empty(), "Drain should have formed clusters for OpenSSH");
        let ssh_enriched = ssh_drain
            .enrich_with_laya()
            .await
            .expect("OpenSSH Laya enrichment should succeed");

        assert!(!ssh_enriched.is_empty(), "OpenSSH enriched templates should not be empty");
        let auth_template = ssh_enriched
            .iter()
            .find(|t| t.vendor == "openssh")
            .unwrap_or(&ssh_enriched[0]);

        assert_eq!(auth_template.vendor, "openssh", "OpenSSH vendor should be detected as openssh");
        assert!(auth_template.confidence > 0.5, "Confidence should be high for OpenSSH");
        assert!(!auth_template.detector_terms.is_empty(), "OpenSSH detector terms should be extracted");

        let ssh_vrl = generator
            .generate_from_template(auth_template)
            .await
            .expect("OpenSSH VRL generation should succeed");

        assert!(ssh_vrl.validation_passed, "OpenSSH VRL must pass validation");
        assert!(!ssh_vrl.content.contains("parse_syslog"), "OpenSSH VRL must NOT contain parse_syslog");
        assert!(ssh_vrl.content.contains(&format!(".class_uid = {}", auth_template.class_uid)));
        compile_vrl(&ssh_vrl.content).expect("OpenSSH VRL must compile with vrl crate");
    }
}
