//! WP-0130: the prompt catalog (ADR-0011). Every platform prompt is a
//! versioned file under `prompts/`, checked here the way ADR-0011 §4 asks:
//! the files and the compiled registry agree, front matter is complete,
//! placeholders match the declared variables, versions are contiguous with
//! one active version, every prompt has an evaluation suite that passes, the
//! wording follows the style rules, a released prompt is never edited in
//! place, and no service keeps a system prompt of its own.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use lanesra_core::prompts;
use sha2::{Digest, Sha256};

fn core_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn files_under(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files_under(&path, ext, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(path);
        }
    }
}

#[test]
fn every_prompt_file_is_compiled_in_and_parses() {
    // Covers: REQ-LOOP-016 (WP-0130)
    let root = core_dir().join("prompts");
    let mut on_disk = Vec::new();
    files_under(&root, "md", &mut on_disk);
    let on_disk: BTreeSet<String> = on_disk
        .iter()
        .map(|p| {
            p.strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    let registered: BTreeSet<String> = prompts::FILES.iter().map(|(p, _)| p.to_string()).collect();
    assert_eq!(on_disk, registered, "prompts/ and prompts::FILES differ");
    for (path, text) in prompts::FILES {
        prompts::front_matter(text).unwrap_or_else(|e| panic!("{path}: {e}"));
    }
    assert_eq!(prompts::all().len(), prompts::FILES.len());
}

#[test]
fn front_matter_is_complete_and_matches_the_template() {
    // Covers: REQ-LOOP-016 (WP-0130)
    for p in prompts::all() {
        let f = &p.front;
        let expected_path = format!("{}.v{}.md", f.id, f.version);
        assert_eq!(
            p.path, expected_path,
            "{}: id and version must match the file name",
            p.path
        );
        assert!(
            ["draft", "active", "retired"].contains(&f.status.as_str()),
            "{}: unknown status {}",
            p.path,
            f.status
        );
        assert!(
            f.work_package.starts_with("WP-"),
            "{}: work_package",
            p.path
        );
        assert!(
            !f.output_contract.trim().is_empty(),
            "{}: output_contract",
            p.path
        );
        assert!(!f.changelog.trim().is_empty(), "{}: changelog", p.path);
        for (name, spec) in &f.variables {
            assert_eq!(spec.kind, "text", "{}: variable {name} type", p.path);
        }
        let used: BTreeSet<String> = prompts::placeholders(p.template()).into_iter().collect();
        let declared: BTreeSet<String> = f.variables.keys().cloned().collect();
        assert_eq!(
            used, declared,
            "{}: every placeholder must be declared, and every declared variable used",
            p.path
        );
    }
}

#[test]
fn versions_are_contiguous_with_exactly_one_active() {
    // Covers: REQ-LOOP-016 (WP-0130)
    let mut by_id: BTreeMap<&str, Vec<(u32, &str)>> = BTreeMap::new();
    for p in prompts::all() {
        by_id
            .entry(p.id)
            .or_default()
            .push((p.version, p.front.status.as_str()));
    }
    for (id, mut versions) in by_id {
        versions.sort();
        let numbers: Vec<u32> = versions.iter().map(|(v, _)| *v).collect();
        let expected: Vec<u32> = (1..=numbers.len() as u32).collect();
        assert_eq!(numbers, expected, "{id}: versions must run 1, 2, 3 ...");
        let active = versions.iter().filter(|(_, s)| *s == "active").count();
        assert_eq!(active, 1, "{id}: exactly one version must be active");
    }
}

#[test]
fn every_prompt_passes_its_evaluation_suite() {
    // Covers: REQ-LOOP-016 (WP-0130)
    for p in prompts::all() {
        let suite_path = core_dir().join(&p.front.eval_suite);
        let suite: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&suite_path)
                .unwrap_or_else(|e| panic!("{}: suite {}: {e}", p.path, suite_path.display())),
        )
        .unwrap_or_else(|e| panic!("{}: suite is not JSON: {e}", p.path));
        assert_eq!(
            suite["prompt"], p.id,
            "{}: suite is for another prompt",
            p.path
        );
        if p.front.status != "active" {
            continue;
        }
        let cases = suite["cases"].as_array().expect("cases");
        assert!(!cases.is_empty(), "{}: the suite has no cases", p.path);
        for case in cases {
            let name = case["name"].as_str().unwrap_or("?");
            let vars: Vec<(String, String)> = case["variables"]
                .as_object()
                .map(|m| {
                    m.iter()
                        .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let borrowed: Vec<(&str, &str)> =
                vars.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            let (text, used) = prompts::render_active(p.id, &borrowed)
                .unwrap_or_else(|e| panic!("{} / {name}: {e}", p.id));
            assert_eq!(used.version, p.version);
            for want in case["expect_contains"].as_array().into_iter().flatten() {
                let want = want.as_str().unwrap();
                assert!(text.contains(want), "{} / {name}: missing {want:?}", p.id);
            }
            for unwanted in case["expect_not_contains"].as_array().into_iter().flatten() {
                let unwanted = unwanted.as_str().unwrap();
                assert!(
                    !text.contains(unwanted),
                    "{} / {name}: has {unwanted:?}",
                    p.id
                );
            }
            if let Some(start) = case["expect_starts_with"].as_str() {
                assert!(
                    text.starts_with(start),
                    "{} / {name}: must start {start:?}",
                    p.id
                );
            }
        }
    }
}

/// The `BANNED` list in `program/tools/check_vocabulary.py` (REQ-CLEAN-003).
fn clean_room_banned_words() -> Vec<String> {
    let path = core_dir().join("../../program/tools/check_vocabulary.py");
    let source = std::fs::read_to_string(path).unwrap();
    let line = source
        .lines()
        .find(|l| l.starts_with("BANNED = ["))
        .expect("BANNED list");
    line.split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

#[test]
fn active_prompts_follow_the_style_rules() {
    // Covers: REQ-LOOP-016 (WP-0130): calm wording, no pressure language,
    // no shouting, no clean-room-banned vocabulary (ADR-0011 §4).
    const ALLOWED_CAPS: &[&str] = &["PASS", "FAIL", "REWRITE", "CLARIFY", "JSON"];
    const BANNED: &[&str] = &[
        "your job depends",
        "or else",
        "you will be penalized",
        "you will be punished",
        "this is very important",
        "this is critical",
        "under no circumstances",
    ];
    // The clean-room vocabulary is defined once, in the program's own check.
    let clean_room = clean_room_banned_words();
    assert!(
        !clean_room.is_empty(),
        "could not read the clean-room word list"
    );
    for p in prompts::all().iter().filter(|p| p.front.status == "active") {
        let text = format!("{}\n{}", p.front.output_contract, p.template());
        let lower = text.to_lowercase();
        for phrase in BANNED {
            assert!(!lower.contains(phrase), "{}: contains {phrase:?}", p.path);
        }
        for word in &clean_room {
            assert!(
                !lower.contains(&word.to_lowercase()),
                "{}: uses clean-room-banned vocabulary {word:?}",
                p.path
            );
        }
        for word in text.split(|c: char| !c.is_ascii_alphabetic()) {
            if word.len() >= 4 && word.chars().all(|c| c.is_ascii_uppercase()) {
                assert!(ALLOWED_CAPS.contains(&word), "{}: shouts {word:?}", p.path);
            }
        }
        assert!(!text.contains('!'), "{}: no exclamation marks", p.path);
    }
}

#[test]
fn a_released_prompt_is_never_edited_in_place() {
    // Covers: REQ-LOOP-016 (WP-0130 AC-2): changing a released prompt means
    // a new version file, so the change shows as a diff between two files.
    let lock = std::fs::read_to_string(core_dir().join("prompts/catalog.lock")).unwrap();
    let pinned: BTreeMap<(String, u32), String> = lock
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            (
                (parts[0].to_string(), parts[1].parse().unwrap()),
                parts[2].to_string(),
            )
        })
        .collect();
    let mut missing = Vec::new();
    for p in prompts::all() {
        let hash = hex::encode(Sha256::digest(p.template().as_bytes()));
        match pinned.get(&(p.id.to_string(), p.version)) {
            Some(expected) => assert_eq!(
                &hash,
                expected,
                "{} v{} was edited in place; add v{} instead and leave v{} as it was",
                p.id,
                p.version,
                p.version + 1,
                p.version
            ),
            None if p.front.status != "draft" => {
                missing.push(format!("{} {} {hash}", p.id, p.version))
            }
            None => {}
        }
    }
    assert!(
        missing.is_empty(),
        "add these released versions to prompts/catalog.lock:\n{}",
        missing.join("\n")
    );
}

#[test]
fn no_service_keeps_a_system_prompt_of_its_own() {
    // Covers: REQ-LOOP-016 (WP-0130 AC-1): instructions to a model live in
    // the catalog. Text a tool returns as data (for example the platform
    // overview) is not a prompt and is not matched by these openings.
    const OPENINGS: &[&str] = &[
        "You are ",
        "You translate",
        "You help ",
        "You will receive",
        "Respond with ONLY",
        "Respond with only",
        "Answer with only",
        "Output nothing else",
        "Your last response",
        "Respond again",
        "Send your answer",
    ];
    let mut sources = Vec::new();
    files_under(&core_dir().join("src/services"), "rs", &mut sources);
    let mut found = Vec::new();
    for path in sources {
        let text = std::fs::read_to_string(&path).unwrap();
        for (n, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            for opening in OPENINGS {
                if code.contains(&format!("\"{opening}")) || code.contains(&format!("\\n{opening}"))
                {
                    found.push(format!("{}:{}: {}", path.display(), n + 1, code.trim()));
                }
            }
        }
    }
    assert!(
        found.is_empty(),
        "system prompt text outside the catalog:\n{}",
        found.join("\n")
    );
}
