use std::path::{Path, PathBuf};

use super::{
    assess_candidate, candidate_group_key, identifiers_match, infer_game_name, infer_game_root,
    select_recommendations, should_exclude_exe, should_skip_directory, CandidateDraft,
    CandidateEvidence, ExecutableMetadata, ScanCandidate, RECOMMEND_THRESHOLD,
};

#[test]
fn excludes_only_clear_non_game_executables() {
    for path in [
        r"C:\Games\Example\unins000.exe",
        r"C:\Games\Example\UE4PrereqSetup_x64.exe",
        r"C:\Games\Example\UnityCrashHandler64.exe",
        r"C:\Games\Example\EasyAntiCheat_EOS_Setup.exe",
    ] {
        assert!(should_exclude_exe(Path::new(path)), "{path}");
    }

    assert!(!should_exclude_exe(Path::new(
        r"C:\Games\Example\ExampleLauncher.exe"
    )));
    assert!(!should_exclude_exe(Path::new(
        r"C:\Games\Example\Binaries\Win64\Example-Win64-Shipping.exe"
    )));
}

#[test]
fn skips_dependency_and_development_directories_but_not_game_binary_directories() {
    assert!(should_skip_directory(Path::new(r"C:\Game\_CommonRedist")));
    assert!(should_skip_directory(Path::new(r"C:\Project\node_modules")));
    assert!(!should_skip_directory(Path::new(r"C:\Game\Binaries")));
    assert!(!should_skip_directory(Path::new(r"C:\Game\Win64")));
}

#[test]
fn recognizes_unity_and_unreal_main_program_evidence() {
    let unity = assess_candidate(
        Path::new(r"C:\Games\Hollow Knight\hollow_knight.exe"),
        &CandidateEvidence {
            file_size: 800_000,
            has_unity_data_directory: true,
            has_unity_runtime: true,
            filename_matches_game_root: true,
            directly_in_game_root: true,
            ..CandidateEvidence::default()
        },
    );
    assert!(unity.score >= RECOMMEND_THRESHOLD);
    assert!(unity.has_strong_game_signal);

    let unreal = assess_candidate(
        Path::new(r"C:\Games\Example\Binaries\Win64\Example-Win64-Shipping.exe"),
        &CandidateEvidence {
            file_size: 40_000_000,
            is_unreal_shipping_binary: true,
            filename_matches_game_root: true,
            prefers_64_bit: true,
            ..CandidateEvidence::default()
        },
    );
    assert!(unreal.score >= RECOMMEND_THRESHOLD);
    assert!(unreal.has_strong_game_signal);
}

#[test]
fn keeps_launchers_as_candidates_without_recommending_them_from_name_alone() {
    let launcher = assess_candidate(
        Path::new(r"C:\Games\Example\ExampleLauncher.exe"),
        &CandidateEvidence {
            file_size: 4_000_000,
            directly_in_game_root: true,
            ..CandidateEvidence::default()
        },
    );

    assert!(launcher.score < RECOMMEND_THRESHOLD);
    assert!(launcher
        .reasons
        .iter()
        .any(|reason| reason.contains("启动器")));
}

#[test]
fn recommends_only_the_strongest_executable_in_a_game_directory() {
    let group = PathBuf::from(r"C:\Games\Example");
    let mut drafts = vec![
        draft("Example.exe", 82, true, &group),
        draft("ExampleLauncher.exe", 28, false, &group),
        draft("ExampleServer.exe", 5, false, &group),
    ];

    select_recommendations(&mut drafts);

    assert!(drafts[0].candidate.recommended);
    assert!(!drafts[1].candidate.recommended);
    assert!(!drafts[2].candidate.recommended);
}

#[test]
fn keeps_existing_state_independent_from_main_executable_classification() {
    let group = PathBuf::from(r"C:\Games\Example");
    let mut existing_main = draft("Example.exe", 82, true, &group);
    existing_main.candidate.exists = true;
    let mut drafts = vec![
        existing_main,
        draft("ExampleLauncher.exe", 28, false, &group),
    ];

    select_recommendations(&mut drafts);

    assert!(drafts[0].candidate.exists);
    assert!(drafts[0].candidate.recommended);
    assert!(!drafts[1].candidate.exists);
    assert!(!drafts[1].candidate.recommended);
}

#[test]
fn leaves_close_candidates_for_manual_confirmation() {
    let group = PathBuf::from(r"C:\Games\Example");
    let mut drafts = vec![
        draft("ExampleA.exe", 64, false, &group),
        draft("ExampleB.exe", 60, false, &group),
    ];

    select_recommendations(&mut drafts);

    assert!(drafts.iter().all(|draft| !draft.candidate.recommended));
    assert!(drafts.iter().all(|draft| draft
        .candidate
        .reasons
        .iter()
        .any(|reason| reason.contains("证据接近"))));
}

#[test]
fn resolves_generic_binary_directories_to_the_game_root() {
    let scan_root = Path::new(r"C:\Games");
    let exe = Path::new(r"C:\Games\Example\Game\Binaries\Win64\Example-Win64-Shipping.exe");
    let game_root = infer_game_root(exe, scan_root);

    assert_eq!(game_root, PathBuf::from(r"C:\Games\Example"));
    assert_eq!(
        candidate_group_key(exe, scan_root, &game_root),
        PathBuf::from(r"C:\Games\Example")
    );
}

#[test]
fn avoids_generic_directory_names_when_inferring_the_game_name() {
    let scan_root = Path::new(r"C:\Games");
    let exe = Path::new(r"C:\Games\Example Title\Binaries\Win64\Example-Win64-Shipping.exe");
    let game_root = infer_game_root(exe, scan_root);
    let name = infer_game_name(exe, scan_root, &game_root, &ExecutableMetadata::default());

    assert_eq!(name, "Example Title");
    assert!(identifiers_match("Example-Win64-Shipping.exe", "Example"));
}

#[test]
fn uses_pe_product_name_for_loose_executables_in_a_library_root() {
    let scan_root = Path::new(r"C:\Games");
    let exe = Path::new(r"C:\Games\mystery.exe");
    let metadata = ExecutableMetadata {
        product_name: Some("Mystery Adventure".to_string()),
        ..ExecutableMetadata::default()
    };

    assert_eq!(
        infer_game_name(exe, scan_root, scan_root, &metadata),
        "Mystery Adventure"
    );
}

fn draft(name: &str, score: i32, strong: bool, group: &Path) -> CandidateDraft {
    CandidateDraft {
        candidate: ScanCandidate {
            name: name.trim_end_matches(".exe").to_string(),
            exe_path: group.join(name).to_string_lossy().into_owned(),
            folder_path: group.to_string_lossy().into_owned(),
            exe_file_name: name.to_string(),
            exists: false,
            recommended: false,
            confidence: score.clamp(0, 100) as u8,
            reasons: Vec::new(),
        },
        group_key: group.to_path_buf(),
        score,
        has_strong_game_signal: strong,
    }
}
