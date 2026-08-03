use std::path::{Path, PathBuf};

use super::{
    assess_candidate, candidate_group_key, contains_game_data_files,
    contains_alicesoft_system_runtime, contains_multiple_numbered_pack_files,
    contains_nwjs_runtime_markers, contains_numbered_pack_runtime_markers,
    contains_renpy_runtime, contains_rpg_maker_web_app, identifiers_match, infer_game_name,
    infer_game_root, is_auxiliary_directory_name,
    reconcile_nested_candidate_groups, select_recommendations, should_exclude_exe,
    should_skip_directory, CandidateDraft, CandidateEvidence, ExecutableMetadata, ScanCandidate,
    DIRECT_GAME_ROOT_BONUS, RECOMMEND_THRESHOLD,
};

#[test]
fn excludes_only_clear_non_game_executables() {
    for path in [
        r"C:\Games\Example\unins000.exe",
        r"C:\Games\Example\UE4PrereqSetup_x64.exe",
        r"C:\Games\Example\UnityCrashHandler64.exe",
        r"C:\Games\Example\EasyAntiCheat_EOS_Setup.exe",
        r"C:\Games\Example\7za.exe",
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
fn recognizes_legacy_game_data_next_to_the_main_executable() {
    let assessment = assess_candidate(
        Path::new(r"C:\Games\Visual Novel\Game.exe"),
        &CandidateEvidence {
            file_size: 4_000_000,
            directly_in_game_root: true,
            has_game_data_files: true,
            ..CandidateEvidence::default()
        },
    );

    assert!(assessment.score >= RECOMMEND_THRESHOLD);
    assert!(assessment.has_strong_game_signal);
    assert!(contains_game_data_files(
        &["game.exe".to_string(), "data.xp3".to_string()]
            .into_iter()
            .collect()
    ));
}

#[test]
fn recognizes_a_strict_renpy_distribution_structure() {
    let names = ["game", "lib", "renpy", "eternum.py", "eternum.exe"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert!(contains_renpy_runtime(&names, "eternum"));
    assert!(contains_renpy_runtime(&names, "eternum-32"));

    let assessment = assess_candidate(
        Path::new(r"C:\Games\Eternum-0.9.5-pc\Eternum.exe"),
        &CandidateEvidence {
            file_size: 273_920,
            has_renpy_runtime: true,
            directly_in_game_root: true,
            ..CandidateEvidence::default()
        },
    );
    assert!(assessment.score >= RECOMMEND_THRESHOLD);
    assert!(assessment.has_strong_game_signal);
}

#[test]
fn does_not_treat_partial_python_directories_as_renpy() {
    let missing_runtime = ["game", "lib", "example.py"]
        .into_iter()
        .map(str::to_string)
        .collect();
    let missing_matching_script = ["game", "lib", "renpy", "other.py"]
        .into_iter()
        .map(str::to_string)
        .collect();

    assert!(!contains_renpy_runtime(&missing_runtime, "example"));
    assert!(!contains_renpy_runtime(
        &missing_matching_script,
        "example"
    ));
}

#[test]
fn lowers_the_32_bit_renpy_variant_when_a_default_executable_exists() {
    let regular = assess_candidate(
        Path::new(r"C:\Games\Eternum\Eternum.exe"),
        &CandidateEvidence {
            has_renpy_runtime: true,
            directly_in_game_root: true,
            ..CandidateEvidence::default()
        },
    );
    let legacy_32_bit = assess_candidate(
        Path::new(r"C:\Games\Eternum\Eternum-32.exe"),
        &CandidateEvidence {
            has_renpy_runtime: true,
            directly_in_game_root: true,
            is_32_bit_variant_with_default: true,
            ..CandidateEvidence::default()
        },
    );

    assert!(regular.score > legacy_32_bit.score);
}

#[test]
fn recognizes_a_strict_rpg_maker_nwjs_distribution_structure() {
    let runtime_names = [
        "resources.pak",
        "icudtl.dat",
        "locales",
        "chrome_100_percent.pak",
        "v8_context_snapshot.bin",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    let app_names = ["package.json", "index.html", "data", "js", "img", "audio"]
        .into_iter()
        .map(str::to_string)
        .collect();

    assert!(contains_nwjs_runtime_markers(&runtime_names));
    assert!(contains_rpg_maker_web_app(&app_names));

    let assessment = assess_candidate(
        Path::new(r"C:\Games\MANARI\Game.exe"),
        &CandidateEvidence {
            file_size: 176_723_456,
            has_rpg_maker_nwjs_runtime: true,
            directly_in_game_root: true,
            ..CandidateEvidence::default()
        },
    );
    assert!(assessment.score >= RECOMMEND_THRESHOLD);
    assert!(assessment.has_strong_game_signal);
}

#[test]
fn does_not_treat_an_ordinary_chromium_app_as_an_rpg_maker_game() {
    let partial_runtime = ["resources.pak", "icudtl.dat", "locales"]
        .into_iter()
        .map(str::to_string)
        .collect();
    let ordinary_web_app = ["package.json", "index.html", "js"]
        .into_iter()
        .map(str::to_string)
        .collect();

    assert!(!contains_nwjs_runtime_markers(&partial_runtime));
    assert!(!contains_rpg_maker_web_app(&ordinary_web_app));
}

#[test]
fn recognizes_a_strict_numbered_pack_game_distribution_structure() {
    let root_names = [
        "dll",
        "gamedata",
        "savedata",
        "enginesetting.exe",
        "engine_gui.u.txt",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    let game_data_names = ["system", "data0.pack", "data1.pack", "data2.pack"]
        .into_iter()
        .map(str::to_string)
        .collect();

    assert!(contains_numbered_pack_runtime_markers(&root_names));
    assert!(contains_multiple_numbered_pack_files(&game_data_names));

    let localized_root_names = ["dll", "gamedata", "エンジン設定.exe"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert!(contains_numbered_pack_runtime_markers(
        &localized_root_names
    ));

    let assessment = assess_candidate(
        Path::new(r"C:\Games\Bimanibun\startup.exe"),
        &CandidateEvidence {
            file_size: 32_269_840,
            has_numbered_pack_game_runtime: true,
            directly_in_game_root: true,
            ..CandidateEvidence::default()
        },
    );
    assert!(assessment.score >= RECOMMEND_THRESHOLD);
    assert!(assessment.has_strong_game_signal);

    let localized_settings = assess_candidate(
        Path::new(r"C:\Games\Bimanibun\エンジン設定.exe"),
        &CandidateEvidence {
            file_size: 646_656,
            has_numbered_pack_game_runtime: true,
            directly_in_game_root: true,
            ..CandidateEvidence::default()
        },
    );
    assert!(assessment.score > localized_settings.score);
    assert!(localized_settings
        .reasons
        .iter()
        .any(|reason| reason.contains("配置程序")));
}

#[test]
fn does_not_treat_a_single_pack_file_as_a_numbered_pack_game() {
    let partial_root = ["dll", "gamedata"]
        .into_iter()
        .map(str::to_string)
        .collect();
    let single_pack = ["data0.pack", "assets.pack", "database.pack"]
        .into_iter()
        .map(str::to_string)
        .collect();

    assert!(!contains_numbered_pack_runtime_markers(&partial_root));
    assert!(!contains_multiple_numbered_pack_files(&single_pack));
}

#[test]
fn recognizes_alicesoft_system_generations_from_strict_layouts() {
    let alice_start = ["alicestart.ini", "rance9.ain", "rance9cg.afa"]
        .into_iter()
        .map(str::to_string)
        .collect();
    let system_40 = [
        "system40.exe",
        "system40.ini",
        "rance6.ain",
        "rance6ba.ald",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    let xsystem_35 = [".xsys35rc", "system39.ain", "rance4ga.ald"]
        .into_iter()
        .map(str::to_string)
        .collect();
    let system_3 = [
        "system3.exe",
        "system3.ini",
        "adisk.dat",
        "acg.dat",
        "bcg.dat",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert!(contains_alicesoft_system_runtime(&alice_start));
    assert!(contains_alicesoft_system_runtime(&system_40));
    assert!(contains_alicesoft_system_runtime(&xsystem_35));
    assert!(contains_alicesoft_system_runtime(&system_3));

    let assessment = assess_candidate(
        Path::new(r"C:\Games\Rance 8\RanceQuest.exe"),
        &CandidateEvidence {
            file_size: 3_284_480,
            has_alicesoft_system_runtime: true,
            directly_in_game_root: true,
            ..CandidateEvidence::default()
        },
    );
    assert!(assessment.score >= RECOMMEND_THRESHOLD);
    assert!(assessment.has_strong_game_signal);
}

#[test]
fn rejects_partial_alicesoft_markers_and_penalizes_known_tools() {
    let partial = ["game.ain", "assets.ald", "system40.exe"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert!(!contains_alicesoft_system_runtime(&partial));

    for tool in ["OpenSaveFolder.exe", "arc_conv.exe"] {
        let assessment = assess_candidate(
            Path::new(tool),
            &CandidateEvidence {
                has_alicesoft_system_runtime: true,
                ..CandidateEvidence::default()
            },
        );
        assert!(
            assessment
                .reasons
                .iter()
                .any(|reason| reason.contains("工具")),
            "{tool}"
        );
    }
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
fn recommends_a_clearly_leading_strong_candidate_just_below_the_standard_threshold() {
    let group = PathBuf::from(r"C:\Games\Example");
    let mut drafts = vec![
        draft("NestedGame.exe", 52, true, &group),
        draft("RootStarter.exe", 18, false, &group),
    ];

    select_recommendations(&mut drafts);

    assert!(drafts[0].candidate.recommended);
    assert!(!drafts[1].candidate.recommended);
}

#[test]
fn keeps_close_strong_candidates_below_the_standard_threshold_for_confirmation() {
    let group = PathBuf::from(r"C:\Games\Example");
    let mut drafts = vec![
        draft("ExampleA.exe", 52, true, &group),
        draft("ExampleB.exe", 49, false, &group),
    ];

    select_recommendations(&mut drafts);

    assert!(drafts.iter().all(|draft| !draft.candidate.recommended));
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
fn collapses_backup_and_patch_directories_into_the_game_group() {
    let scan_root = Path::new(r"C:\Games\Publisher");
    let main = Path::new(r"C:\Games\Publisher\RIDDLE JOKER[官中]\RiddleJoker.exe");
    let backup = Path::new(r"C:\Games\Publisher\RIDDLE JOKER[官中]\原版备份\RiddleJoker.exe");
    let patch = Path::new(r"C:\Games\Publisher\RIDDLE JOKER[官中]\补丁\patch.exe");
    let expected = PathBuf::from(r"C:\Games\Publisher\RIDDLE JOKER[官中]");

    assert_eq!(infer_game_root(main, scan_root), expected);
    assert_eq!(infer_game_root(backup, scan_root), expected);
    assert_eq!(infer_game_root(patch, scan_root), expected);
    assert!(is_auxiliary_directory_name("原版备份"));
    assert!(is_auxiliary_directory_name("crack"));
}

#[test]
fn matches_executable_names_to_decorated_game_directories() {
    assert!(identifiers_match("RiddleJoker.exe", "RIDDLE JOKER[官中]"));
    assert!(identifiers_match(
        "Example.exe",
        "Example（简体中文汉化版）"
    ));
    assert!(identifiers_match(
        "美少女万华镜4.exe",
        "美少女万华镜4-罪与罚的少女"
    ));
    assert!(!identifiers_match("Game.exe", "My Favorite Game"));
}

#[test]
fn prefers_the_main_executable_over_backup_and_crack_copies() {
    let group = PathBuf::from(r"C:\Games\RIDDLE JOKER[官中]");
    let mut drafts = vec![
        draft_with_path("RiddleJoker.exe", 63, false, &group, &group),
        draft_with_path(
            "RiddleJoker.exe",
            18,
            false,
            &group.join("原版备份"),
            &group,
        ),
        draft_with_path(
            "RiddleJoker_crack.exe",
            4,
            false,
            &group.join("crack"),
            &group,
        ),
    ];

    select_recommendations(&mut drafts);

    assert!(drafts[0].candidate.recommended);
    assert!(!drafts[1].candidate.recommended);
    assert!(!drafts[2].candidate.recommended);
}

#[test]
fn makes_nested_executables_compete_with_a_direct_parent_executable() {
    // Windows canonicalize 可能产生 `\\?\` 前缀，而展示路径会移除它；分组不能依赖展示字符串。
    let scan_root = Path::new(r"\\?\C:\Games");
    let game_root = PathBuf::from(r"\\?\C:\Games\Trade Master");
    let nested_root = game_root.join("data");
    let mut drafts = vec![
        draft_with_path("YARISUTEMESUBUTA.exe", 63, false, &game_root, &game_root),
        draft_with_path("YM.exe", 63, false, &nested_root, &nested_root),
    ];
    for draft in &mut drafts {
        draft.candidate.exe_path = draft.candidate.exe_path.trim_start_matches(r"\\?\").to_string();
    }

    reconcile_nested_candidate_groups(scan_root, &mut drafts);
    select_recommendations(&mut drafts);

    assert_eq!(drafts[0].group_key, game_root);
    assert_eq!(drafts[1].group_key, game_root);
    assert_eq!(drafts[1].score, 63 - DIRECT_GAME_ROOT_BONUS);
    assert_eq!(drafts[1].candidate.name, "Trade Master");
    assert!(drafts[0].candidate.recommended);
    assert!(!drafts[1].candidate.recommended);
}

#[test]
fn keeps_searching_below_a_directory_without_a_direct_executable() {
    let scan_root = Path::new(r"C:\Games");
    let nested_root = PathBuf::from(r"C:\Games\Publisher\Trade Master");
    let mut drafts = vec![draft_with_path(
        "YARISUTEMESUBUTA.exe",
        63,
        false,
        &nested_root,
        &nested_root,
    )];

    reconcile_nested_candidate_groups(scan_root, &mut drafts);

    assert_eq!(drafts[0].group_key, nested_root);
    assert_eq!(drafts[0].score, 63);
    assert_eq!(drafts[0].candidate.name, "YARISUTEMESUBUTA");
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
    draft_with_path(name, score, strong, group, group)
}

fn draft_with_path(
    name: &str,
    score: i32,
    strong: bool,
    directory: &Path,
    group: &Path,
) -> CandidateDraft {
    CandidateDraft {
        candidate: ScanCandidate {
            name: name.trim_end_matches(".exe").to_string(),
            exe_path: directory.join(name).to_string_lossy().into_owned(),
            folder_path: directory.to_string_lossy().into_owned(),
            exe_file_name: name.to_string(),
            exists: false,
            recommended: false,
            confidence: score.clamp(0, 100) as u8,
            reasons: Vec::new(),
        },
        executable_directory: directory.to_path_buf(),
        group_key: group.to_path_buf(),
        score,
        has_strong_game_signal: strong,
    }
}
