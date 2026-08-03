//! 本地 EXE 扫描与游戏主程序推荐。
//!
//! 处理流程：
//! 1. 递归查找 `.exe`，跳过明确无效的目录和程序；
//! 2. 为每个保留下来的 EXE 收集路径、同目录文件和 PE 版本信息；
//! 3. 根据游戏特征加分、辅助程序特征减分；
//! 4. 将同一游戏目录中的 EXE 放在一起比较，只推荐证据明显更强的一个；
//! 5. 返回推荐项、其他候选和已存在状态，最终是否导入仍由用户决定。

use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::db;

use super::models::ScanCandidate;
use super::repository;
use super::{has_exe_extension, normalize_existing_directory, path_to_string};

// 单个候选达到该分数后，才有资格进入“推荐游戏”。
// 该分数是规则评分，不是统计概率；对外展示时限制在 0-100。
const RECOMMEND_THRESHOLD: i32 = 55;
// 有强游戏证据且在同组明显领先时，可使用稍低门槛，兼容主程序位于资源子目录的游戏。
const STRONG_SIGNAL_RECOMMEND_THRESHOLD: i32 = 50;
// 同目录前两名都没有强游戏特征时，第一名至少领先该分数才会被推荐。
// 这样可以避免在两个很相似的 EXE 之间武断选择。
const MIN_RECOMMENDATION_MARGIN: i32 = 8;
// 游戏根目录直属程序相对子目录程序获得的结构性优势。
const DIRECT_GAME_ROOT_BONUS: i32 = 8;

/// Windows PE 版本资源中可用于识别程序身份的文本字段。
///
/// 很多游戏不会完整填写这些字段，所以它们只能作为辅助证据，不能单独证明某个 EXE 是游戏。
#[derive(Debug, Clone, Default)]
struct ExecutableMetadata {
    product_name: Option<String>,
    file_description: Option<String>,
    original_filename: Option<String>,
    company_name: Option<String>,
}

/// 从文件本身及周边目录采集到的原始证据。
///
/// 这里不做“是否推荐”的决定，只记录事实，方便评分规则与文件遍历解耦。
#[derive(Debug, Clone, Default)]
struct CandidateEvidence {
    file_size: u64,
    metadata: ExecutableMetadata,
    has_unity_data_directory: bool,
    has_unity_runtime: bool,
    has_game_platform_runtime: bool,
    has_renpy_runtime: bool,
    has_rpg_maker_nwjs_runtime: bool,
    has_numbered_pack_game_runtime: bool,
    has_alicesoft_system_runtime: bool,
    is_unreal_shipping_binary: bool,
    filename_matches_game_root: bool,
    metadata_matches_identity: bool,
    directly_in_game_root: bool,
    prefers_64_bit: bool,
    has_game_data_files: bool,
    is_32_bit_variant_with_default: bool,
    is_in_auxiliary_path: bool,
}

/// 单个 EXE 独立评分后的结果。
#[derive(Debug, Clone)]
struct CandidateAssessment {
    score: i32,
    reasons: Vec<String>,
    has_strong_game_signal: bool,
}

/// 扫描阶段的内部候选。
///
/// `ScanCandidate` 是返回给前端的数据；其余字段只用于同一游戏目录内的横向比较。
#[derive(Debug)]
struct CandidateDraft {
    candidate: ScanCandidate,
    executable_directory: PathBuf,
    group_key: PathBuf,
    score: i32,
    has_strong_game_signal: bool,
}

// -----------------------------------------------------------------------------
// 扫描主流程
// -----------------------------------------------------------------------------

/// 扫描入口：加载已入库路径、递归采集候选、同目录择优，最后按展示顺序排序。
pub(super) fn scan_games(app: &AppHandle, directory: &str) -> Result<Vec<ScanCandidate>, String> {
    let connection = db::open_connection(app)?;
    let root = normalize_existing_directory(directory)?;
    let existing_exe_paths = repository::query_existing_exe_paths(&connection)?;
    let mut drafts = Vec::new();
    let mut visited_directories = HashSet::new();
    let mut sibling_name_cache = HashMap::new();

    scan_directory(
        &root,
        &root,
        &existing_exe_paths,
        &mut visited_directories,
        &mut sibling_name_cache,
        &mut drafts,
    )?;
    reconcile_nested_candidate_groups(&root, &mut drafts);
    select_recommendations(&mut drafts);

    let mut candidates: Vec<_> = drafts.into_iter().map(|draft| draft.candidate).collect();
    // 展示顺序：未入库优先、推荐优先、分数高的优先，最后按名称稳定排序。
    candidates.sort_by(|left, right| {
        left.exists
            .cmp(&right.exists)
            .then_with(|| right.recommended.cmp(&left.recommended))
            .then_with(|| right.confidence.cmp(&left.confidence))
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    Ok(candidates)
}

/// 递归遍历目录并生成尚未决定 `recommended` 的候选草稿。
///
/// 无权访问、无法规范化或包含无效 Unicode 的单个目录/文件会被跳过，不中断整次扫描。
fn scan_directory(
    root: &Path,
    directory: &Path,
    existing_exe_paths: &HashSet<String>,
    visited_directories: &mut HashSet<PathBuf>,
    sibling_name_cache: &mut HashMap<PathBuf, HashSet<String>>,
    drafts: &mut Vec<CandidateDraft>,
) -> Result<(), String> {
    // Windows 目录联接和符号链接可能形成环；使用规范路径确保同一目录只扫描一次。
    let canonical_directory = match directory.canonicalize() {
        Ok(path) => path,
        Err(_) => return Ok(()),
    };
    if !visited_directories.insert(canonical_directory) {
        return Ok(());
    }

    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.is_dir() {
            if should_skip_directory(&path) {
                continue;
            }
            scan_directory(
                root,
                &path,
                existing_exe_paths,
                visited_directories,
                sibling_name_cache,
                drafts,
            )?;
            continue;
        }

        // 只有“明确无效”的 EXE 才在此处彻底丢弃。不确定的程序必须继续进入评分流程。
        if !path.is_file() || !has_exe_extension(&path) || should_exclude_exe(&path) {
            continue;
        }

        let canonical_path = match path.canonicalize() {
            Ok(path) => path,
            Err(_) => continue,
        };
        let folder_path = match canonical_path.parent() {
            Some(path) => path.to_path_buf(),
            None => continue,
        };
        let exe_file_name = match canonical_path.file_name().and_then(|value| value.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };
        let exe_path = match path_to_string(canonical_path.clone()) {
            Ok(path) => path,
            Err(_) => continue,
        };
        let folder_path_string = match path_to_string(folder_path.clone()) {
            Ok(path) => path,
            Err(_) => continue,
        };

        // 先推断游戏根目录，后续的名称匹配、候选分组和评分都会依赖它。
        let game_root = infer_game_root(&canonical_path, root);
        let evidence = collect_candidate_evidence(&canonical_path, &game_root, sibling_name_cache);
        let assessment = assess_candidate(&canonical_path, &evidence);
        let name = infer_game_name(&canonical_path, root, &game_root, &evidence.metadata);
        let group_key = candidate_group_key(&canonical_path, root, &game_root);

        drafts.push(CandidateDraft {
            candidate: ScanCandidate {
                name,
                exists: existing_exe_paths.contains(&exe_path),
                exe_path,
                folder_path: folder_path_string,
                exe_file_name,
                recommended: false,
                // 内部允许负分，返回前端时压缩到 0-100，便于排序和调试。
                confidence: assessment.score.clamp(0, 100) as u8,
                reasons: assessment.reasons,
            },
            executable_directory: folder_path,
            group_key,
            score: assessment.score,
            has_strong_game_signal: assessment.has_strong_game_signal,
        });
    }

    Ok(())
}

// -----------------------------------------------------------------------------
// 明确排除规则
// -----------------------------------------------------------------------------

/// 判断整个目录是否可以安全跳过。
///
/// 这里只放回收站、开发依赖、运行库安装包等明确不是游戏主体的目录。
/// `bin`、`Binaries`、`Win64` 等目录可能包含真正主程序，绝不能在这里排除。
fn should_skip_directory(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| {
            matches!(
                name.to_ascii_lowercase().as_str(),
                "$recycle.bin"
                    | "system volume information"
                    | ".git"
                    | ".svn"
                    | ".idea"
                    | ".vscode"
                    | "node_modules"
                    | "__pycache__"
                    | "redist"
                    | "redistributable"
                    | "redistributables"
                    | "_commonredist"
                    | "directx"
                    | "vcredist"
                    | "prereq"
                    | "prereqs"
                    | "prerequisites"
            )
        })
}

/// 判断 EXE 是否明确不应作为候选返回。
///
/// 启动器、服务器、编辑器等仍可能是用户需要的入口，因此不会在这里删除，只在评分时减分。
fn should_exclude_exe(path: &Path) -> bool {
    let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
        return true;
    };
    let name = compact_identifier(stem);

    name.starts_with("unins")
        || name.starts_with("uninstall")
        || name.starts_with("vcredist")
        || name.starts_with("dotnet")
        || name.starts_with("dxsetup")
        || name.starts_with("ue4prereqsetup")
        || name.starts_with("ue5prereqsetup")
        || name.starts_with("unitycrashhandler")
        || name.starts_with("crashreportclient")
        || name.starts_with("crashpadhandler")
        || name.starts_with("steamerrorreporter")
        || name.starts_with("easyanticheat")
        || name.starts_with("eacsetup")
        || name.starts_with("beservice")
        || matches!(
            name.as_str(),
            "setup"
                | "installer"
                | "install"
                | "dotnetfx"
                | "werfault"
                | "7z"
                | "7za"
                | "7zr"
                | "unrar"
        )
}

// -----------------------------------------------------------------------------
// 证据采集与评分
// -----------------------------------------------------------------------------

/// 收集候选证据，不在这里决定是否推荐。
///
/// 同目录文件名会按目录缓存，避免一个目录存在多个 EXE 时反复读取磁盘。
fn collect_candidate_evidence(
    path: &Path,
    game_root: &Path,
    sibling_name_cache: &mut HashMap<PathBuf, HashSet<String>>,
) -> CandidateEvidence {
    let parent = path.parent().unwrap_or(game_root);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let stem_lower = stem.to_ascii_lowercase();
    let has_rpg_maker_nwjs_runtime =
        detect_rpg_maker_nwjs_runtime(parent, sibling_name_cache);
    let has_numbered_pack_game_runtime =
        detect_numbered_pack_game_runtime(parent, sibling_name_cache);
    let sibling_names = sibling_name_cache
        .entry(parent.to_path_buf())
        .or_insert_with(|| read_sibling_names(parent));
    let unity_data_directory = format!("{}_data", stem_lower);
    let root_name = game_root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let metadata = read_executable_metadata(path);

    CandidateEvidence {
        file_size: fs::metadata(path).map_or(0, |metadata| metadata.len()),
        metadata_matches_identity: metadata_matches_candidate(&metadata, stem, root_name),
        metadata,
        has_unity_data_directory: sibling_names.contains(&unity_data_directory),
        has_unity_runtime: sibling_names.contains("unityplayer.dll")
            || sibling_names.contains("gameassembly.dll"),
        has_game_platform_runtime: sibling_names.contains("steam_api.dll")
            || sibling_names.contains("steam_api64.dll")
            || sibling_names.contains("galaxycsharpglue.dll")
            || sibling_names
                .iter()
                .any(|name| name.starts_with("eossdk-") && name.ends_with("-shipping.dll")),
        has_renpy_runtime: contains_renpy_runtime(sibling_names, &stem_lower),
        has_rpg_maker_nwjs_runtime,
        has_numbered_pack_game_runtime,
        has_alicesoft_system_runtime: contains_alicesoft_system_runtime(sibling_names),
        is_unreal_shipping_binary: stem_lower.ends_with("-win64-shipping")
            || stem_lower.ends_with("-win32-shipping")
            || stem_lower.ends_with("-shipping"),
        filename_matches_game_root: identifiers_match(stem, root_name),
        directly_in_game_root: parent == game_root,
        prefers_64_bit: path.components().any(|component| {
            component.as_os_str().to_str().is_some_and(|value| {
                matches!(
                    value.to_ascii_lowercase().as_str(),
                    "win64" | "x64" | "amd64"
                )
            })
        }) || stem_lower.contains("win64")
            || stem_lower.ends_with("x64"),
        has_game_data_files: contains_game_data_files(sibling_names),
        is_32_bit_variant_with_default: stem_lower.strip_suffix("-32").is_some_and(|base| {
            sibling_names.contains(&format!("{base}.exe"))
        }),
        // 只检查游戏根目录之下的相对路径，避免用户把整个游戏库放在 Backup 盘时全部减分。
        is_in_auxiliary_path: parent.strip_prefix(game_root).ok().is_some_and(|relative| {
            relative.components().any(|component| {
                component
                    .as_os_str()
                    .to_str()
                    .is_some_and(is_auxiliary_directory_name)
            })
        }),
    }
}

/// 将原始证据换算成规则分数和可解释的原因。
///
/// 分数越高表示“更像游戏主程序”，但不表示真实概率。规则分为：
/// - 强游戏证据：Unity 同名数据目录、Unreal Shipping 结构；
/// - 一般正向证据：平台运行库、名称匹配、PE 信息、文件大小；
/// - 负向证据：工具目录以及启动器、服务器、更新器等程序角色。
fn assess_candidate(path: &Path, evidence: &CandidateEvidence) -> CandidateAssessment {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let mut score = 0;
    let mut reasons = Vec::new();
    let mut has_strong_game_signal = false;

    // 强游戏证据：可以显著提高推荐可信度，并允许在同目录候选分数接近时择优。
    if evidence.has_unity_data_directory {
        score += 65;
        has_strong_game_signal = true;
        reasons.push("存在与程序同名的 Unity 数据目录".to_string());
    }
    if evidence.has_unity_runtime {
        score += 20;
        reasons.push("同目录包含 Unity 游戏运行库".to_string());
    }
    if evidence.is_unreal_shipping_binary {
        score += 65;
        has_strong_game_signal = true;
        reasons.push("符合 Unreal Shipping 主程序结构".to_string());
    }
    // 一般游戏证据：只能组合使用，单项不应被理解为“确定是游戏”。
    if evidence.has_game_platform_runtime {
        score += 15;
        reasons.push("同目录包含游戏平台运行库".to_string());
    }
    if evidence.has_renpy_runtime {
        score += 55;
        has_strong_game_signal = true;
        reasons.push("符合 Ren'Py 游戏发行结构".to_string());
    }
    if evidence.has_rpg_maker_nwjs_runtime {
        score += 55;
        has_strong_game_signal = true;
        reasons.push("符合 RPG Maker（NW.js）游戏发行结构".to_string());
    }
    if evidence.has_numbered_pack_game_runtime {
        score += 55;
        has_strong_game_signal = true;
        reasons.push("符合编号资源包游戏发行结构".to_string());
    }
    if evidence.has_alicesoft_system_runtime {
        score += 55;
        has_strong_game_signal = true;
        reasons.push("符合 AliceSoft System 游戏发行结构".to_string());
    }
    if evidence.filename_matches_game_root {
        score += 35;
        reasons.push("程序名与游戏目录名一致".to_string());
    }
    if evidence.directly_in_game_root {
        score += DIRECT_GAME_ROOT_BONUS;
        reasons.push("程序位于游戏根目录".to_string());
    }
    if evidence.prefers_64_bit {
        score += 8;
        reasons.push("优先选择 64 位程序".to_string());
    }
    // 老视觉小说和独立游戏往往没有可靠 PE 信息，但会在主程序旁放置引擎数据文件。
    if evidence.has_game_data_files {
        score += 40;
        has_strong_game_signal = true;
        reasons.push("同目录包含常见游戏引擎或资源数据".to_string());
    }

    // PE 元数据有助于确认程序身份，但普通桌面软件也具备这些字段，所以权重较低。
    if let Some(product_name) = evidence
        .metadata
        .product_name
        .as_deref()
        .filter(|value| is_meaningful_name(value))
    {
        score += 10;
        reasons.push(format!("PE 产品名：{product_name}"));
    }
    if let Some(company_name) = evidence
        .metadata
        .company_name
        .as_deref()
        .filter(|value| is_meaningful_name(value))
    {
        reasons.push(format!("PE 发布者：{company_name}"));
    }
    if evidence.metadata_matches_identity {
        score += 20;
        reasons.push("PE 版本信息与程序或目录名称一致".to_string());
    }

    // 文件大小只用于区分很小的辅助程序，不能单独作为游戏判断依据。
    match evidence.file_size {
        20_000_000.. => {
            score += 12;
            reasons.push("程序体积符合常见游戏主程序特征".to_string());
        }
        2_000_000.. => {
            score += 7;
            reasons.push("程序体积不像小型辅助工具".to_string());
        }
        256_000.. => score += 2,
        0..128_000 => {
            score -= 12;
            reasons.push("程序体积很小，可能是启动器或辅助工具".to_string());
        }
        _ => {}
    }

    // 路径和程序角色提供负向证据；候选仍会保留在“其他可执行文件”中。
    if evidence.is_in_auxiliary_path {
        score -= 25;
        reasons.push("程序位于工具或辅助目录".to_string());
    }
    if evidence.is_32_bit_variant_with_default {
        score -= 12;
        reasons.push("同目录存在默认版本，降低 32 位兼容程序优先级".to_string());
    }

    let role_text = [
        Some(stem),
        evidence.metadata.product_name.as_deref(),
        evidence.metadata.file_description.as_deref(),
        evidence.metadata.original_filename.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ")
    .to_ascii_lowercase();
    if let Some((penalty, reason)) = auxiliary_role_penalty(&role_text) {
        score -= penalty;
        reasons.push(reason.to_string());
    }

    CandidateAssessment {
        score: score.clamp(-100, 100),
        reasons,
        has_strong_game_signal,
    }
}

/// 根据文件名和 PE 文本判断程序更像哪类辅助角色，并返回减分值及解释。
///
/// 该函数只减分、不直接排除，避免误伤必须经 Launcher 启动的游戏。
fn auxiliary_role_penalty(value: &str) -> Option<(i32, &'static str)> {
    let compact = compact_identifier(value);
    if compact.contains("dedicatedserver") || compact.contains("server") {
        return Some((55, "名称表明它更可能是服务器程序"));
    }
    if compact.contains("crash") || compact.contains("errorreport") {
        return Some((60, "名称表明它更可能是崩溃或错误上报程序"));
    }
    if compact.contains("updater") || compact.contains("patcher") {
        return Some((45, "名称表明它更可能是更新或补丁程序"));
    }
    if compact.contains("editor") {
        return Some((50, "名称表明它更可能是编辑器"));
    }
    if compact.contains("helper")
        || compact.contains("webview")
        || compact.contains("webhelper")
        || compact.contains("service")
    {
        return Some((45, "名称表明它更可能是辅助或服务程序"));
    }
    if compact.contains("opensavefolder") {
        return Some((50, "名称表明它只是存档目录工具"));
    }
    if compact.contains("benchmark") {
        return Some((40, "名称表明它更可能是性能测试程序"));
    }
    if compact.contains("config")
        || compact.contains("settings")
        || compact.contains("エンジン設定")
    {
        return Some((35, "名称表明它更可能是配置程序"));
    }
    if compact.contains("launcher") || compact.contains("bootstrap") {
        return Some((30, "名称表明它可能只是启动器"));
    }
    if compact.contains("tool") {
        return Some((25, "名称表明它更可能是工具程序"));
    }
    if compact.contains("arcconv") {
        return Some((35, "名称表明它更可能是资源转换工具"));
    }
    None
}

// -----------------------------------------------------------------------------
// 同一游戏目录内择优
// -----------------------------------------------------------------------------

/// 在每个推断出的游戏目录中选择至多一个推荐 EXE。
///
/// 推荐必须同时满足：
/// - 第一名达到推荐阈值；
/// - 第一名有强游戏证据，或比第二名领先足够多。
///
/// `exists` 不参与评分，它只表示数据库中是否已有该路径，与识别结果保持独立。
fn select_recommendations(drafts: &mut [CandidateDraft]) {
    let mut groups: HashMap<PathBuf, Vec<usize>> = HashMap::new();
    for (index, draft) in drafts.iter().enumerate() {
        groups
            .entry(draft.group_key.clone())
            .or_default()
            .push(index);
    }

    for indices in groups.values_mut() {
        indices.sort_by_key(|index| {
            (
                Reverse(drafts[*index].score),
                drafts[*index].candidate.exe_path.to_ascii_lowercase(),
            )
        });
        let best_index = indices[0];
        let best_score = drafts[best_index].score;
        let best_has_strong_signal = drafts[best_index].has_strong_game_signal;
        let reaches_standard_threshold = best_score >= RECOMMEND_THRESHOLD;
        let reaches_relaxed_strong_threshold = best_has_strong_signal
            && best_score >= STRONG_SIGNAL_RECOMMEND_THRESHOLD;
        if !reaches_standard_threshold && !reaches_relaxed_strong_threshold {
            continue;
        }

        let runner_up_score = indices.get(1).map(|index| drafts[*index].score);
        let clearly_leads_group = runner_up_score
            .is_none_or(|runner_up| best_score - runner_up >= MIN_RECOMMENDATION_MARGIN);
        // 达到标准门槛时，强游戏证据可以打破接近分数；使用较低门槛时仍必须明显领先。
        let has_clear_winner = if reaches_standard_threshold {
            best_has_strong_signal || clearly_leads_group
        } else {
            clearly_leads_group
        };
        if has_clear_winner {
            drafts[best_index].candidate.recommended = true;
            drafts[best_index]
                .candidate
                .reasons
                .push("同一游戏目录中综合证据最强".to_string());
        } else {
            for index in indices.iter().take(2) {
                drafts[*index]
                    .candidate
                    .reasons
                    .push("同目录候选证据接近，保留待确认".to_string());
            }
        }
    }
}

// -----------------------------------------------------------------------------
// 游戏根目录、分组和名称推断
// -----------------------------------------------------------------------------

/// 根据全部扫描结果校正嵌套目录的游戏边界。
///
/// 如果 `A` 目录本身已有直属 EXE，那么 `A/B` 中独立成组的候选应回到 `A` 组竞争；
/// 如果 `A` 没有直属 EXE，则继续保留 `B` 自己推断出的游戏边界。已被 Unity、Unreal、
/// 备份目录等结构规则归到同一组的候选不会重复扣分。
fn reconcile_nested_candidate_groups(scan_root: &Path, drafts: &mut [CandidateDraft]) {
    let direct_candidate_directories: HashSet<PathBuf> = drafts
        .iter()
        .map(|draft| draft.executable_directory.clone())
        .collect();
    let root_is_library = scan_root
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(is_library_container_name);

    for draft in drafts {
        let original_group = draft.group_key.clone();
        let mut current = original_group.parent();
        let mut containing_game_root = None;

        while let Some(directory) = current {
            if !directory.starts_with(scan_root) {
                break;
            }
            if direct_candidate_directories.contains(directory)
                && !(directory == scan_root && root_is_library)
            {
                // 继续向上检查，使 A、A/B、A/B/C 都有 EXE 时统一由最外层 A 作为游戏边界。
                containing_game_root = Some(directory.to_path_buf());
            }
            if directory == scan_root {
                break;
            }
            current = directory.parent();
        }

        let Some(containing_game_root) = containing_game_root else {
            continue;
        };
        if containing_game_root == original_group {
            continue;
        }

        draft.group_key = containing_game_root.clone();
        // 该候选此前被当作自己目录的直属程序加过分；归入父游戏目录后应撤销这项优势。
        draft.score -= DIRECT_GAME_ROOT_BONUS;
        draft.candidate.confidence = draft.score.clamp(0, 100) as u8;
        draft
            .candidate
            .reasons
            .retain(|reason| reason != "程序位于游戏根目录");
        draft
            .candidate
            .reasons
            .push("上级目录存在直属程序，归入同一游戏目录比较".to_string());

        if let Some(name) = containing_game_root
            .file_name()
            .and_then(|value| value.to_str())
            .filter(|name| is_meaningful_name(name))
        {
            draft.candidate.name = name.to_string();
        }
    }
}

/// 从 EXE 所在位置向上跳过通用二进制目录，推断游戏安装根目录。
///
/// 例如 `Example/Game/Binaries/Win64/Example-Win64-Shipping.exe` 会回溯到 `Example`。
fn infer_game_root(path: &Path, scan_root: &Path) -> PathBuf {
    let mut current = path.parent().unwrap_or(scan_root);
    let mut ascended = false;

    while current != scan_root {
        let Some(name) = current.file_name().and_then(|value| value.to_str()) else {
            break;
        };
        if !(is_generic_executable_directory(name)
            || is_auxiliary_directory_name(name)
            || ascended && name.eq_ignore_ascii_case("game"))
        {
            break;
        }
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
        ascended = true;
    }

    current.to_path_buf()
}

/// 生成同目录比较所使用的分组键。
///
/// 如果用户直接扫描 `Games` 这类库目录，散落在根目录的多个 EXE 不应被当成同一个游戏，
/// 因此这些文件分别成组；正常子目录中的候选仍按推断出的游戏根目录分组。
fn candidate_group_key(path: &Path, scan_root: &Path, game_root: &Path) -> PathBuf {
    let root_is_library = scan_root
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(is_library_container_name);
    if game_root == scan_root && path.parent() == Some(scan_root) && root_is_library {
        return path.to_path_buf();
    }
    game_root.to_path_buf()
}

/// 推断默认游戏名称。
///
/// 优先级：有效的游戏根目录名 -> PE 产品名 -> EXE 文件名。
/// 通用库目录、纯数字目录和 `Win64` 等无意义名称不会直接成为游戏名。
fn infer_game_name(
    path: &Path,
    scan_root: &Path,
    game_root: &Path,
    metadata: &ExecutableMetadata,
) -> String {
    let game_root_name = game_root.file_name().and_then(|value| value.to_str());
    let root_is_library = scan_root
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(is_library_container_name);

    if let Some(name) = game_root_name
        .filter(|name| is_meaningful_name(name) && (game_root != scan_root || !root_is_library))
    {
        return name.trim().to_string();
    }
    if let Some(name) = metadata
        .product_name
        .as_deref()
        .filter(|value| is_meaningful_name(value))
    {
        return name.trim().to_string();
    }
    path.file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Game")
        .to_string()
}

/// 判断 PE 产品名、文件描述或原始文件名是否与 EXE/游戏根目录名称一致。
fn metadata_matches_candidate(metadata: &ExecutableMetadata, stem: &str, root_name: &str) -> bool {
    [
        metadata.product_name.as_deref(),
        metadata.file_description.as_deref(),
        metadata.original_filename.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|value| identifiers_match(value, stem) || identifiers_match(value, root_name))
}

/// 读取同目录全部文件名并统一为小写，用于寻找引擎和平台运行库特征。
fn read_sibling_names(directory: &Path) -> HashSet<String> {
    let Ok(entries) = fs::read_dir(directory) else {
        return HashSet::new();
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .map(|name| name.to_ascii_lowercase())
        .collect()
}

/// 真正主程序经常位于这些通用目录中，推断根目录时需要继续向上回溯。
fn is_generic_executable_directory(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "bin"
            | "binary"
            | "binaries"
            | "win32"
            | "win64"
            | "x86"
            | "x64"
            | "amd64"
            | "retail"
            | "release"
            | "shipping"
    )
}

/// 常见的多游戏库目录名，用于区分“库根目录”和“单个游戏根目录”。
fn is_library_container_name(name: &str) -> bool {
    matches!(
        name.trim().to_ascii_lowercase().as_str(),
        "game"
            | "games"
            | "library"
            | "libraries"
            | "apps"
            | "common"
            | "steamapps"
            | "gog games"
            | "epic games"
    )
}

/// 过滤过短、纯数字、占位符和通用目录名，避免生成无意义的默认游戏名。
fn is_meaningful_name(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.len() < 2 || trimmed.chars().all(|character| character.is_ascii_digit()) {
        return false;
    }
    !matches!(
        compact_identifier(trimmed).as_str(),
        "game"
            | "application"
            | "launcher"
            | "program"
            | "productname"
            | "defaultcompanyname"
            | "todo"
            | "win32"
            | "win64"
            | "x86"
            | "x64"
            | "bin"
            | "binary"
            | "binaries"
    )
}

/// 忽略大小写、标点、`.exe` 和架构/Shipping 后缀后比较两个名称。
fn identifiers_match(left: &str, right: &str) -> bool {
    let left = normalized_game_identifier(left);
    let right = normalized_game_identifier(right);
    if left.len() < 3 || right.len() < 3 {
        return false;
    }
    if left == right {
        return true;
    }

    let (shorter, longer) = if left.len() <= right.len() {
        (left.as_str(), right.as_str())
    } else {
        (right.as_str(), left.as_str())
    };
    // 支持 `Title.exe` 对应 `Title - Complete Edition`，但拒绝过短的 Game/App 等泛化命中。
    shorter.len() >= 5
        && !matches!(shorter, "game" | "games" | "application" | "launcher")
        && longer.contains(shorter)
}

/// 将游戏标识归一化，便于比较 `Example` 与 `Example-Win64-Shipping.exe`。
fn normalized_game_identifier(value: &str) -> String {
    let mut value = compact_identifier(&strip_bracketed_annotations(value));
    if value.len() > 5 && value.ends_with("exe") {
        value.truncate(value.len() - 3);
    }
    for suffix in [
        "win64shipping",
        "win32shipping",
        "shipping",
        "win64",
        "win32",
        "amd64",
        "x64",
        "x86",
    ] {
        if value.len() > suffix.len() + 2 && value.ends_with(suffix) {
            value.truncate(value.len() - suffix.len());
            break;
        }
    }
    value
}

/// 去除目录名中的语言、汉化组和版本注记，例如 `RIDDLE JOKER[官中]`。
fn strip_bracketed_annotations(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut depth = 0_u32;
    for character in value.chars() {
        match character {
            '[' | '【' | '(' | '（' => depth = depth.saturating_add(1),
            ']' | '】' | ')' | '）' if depth > 0 => depth -= 1,
            _ if depth == 0 => result.push(character),
            _ => {}
        }
    }
    result
}

/// 备份、破解、补丁和工具目录仍会被扫描，但其中的 EXE 不应作为独立游戏优先推荐。
fn is_auxiliary_directory_name(value: &str) -> bool {
    matches!(
        compact_identifier(value).as_str(),
        "tool"
            | "tools"
            | "sdk"
            | "support"
            | "extras"
            | "thirdparty"
            | "installer"
            | "installers"
            | "benchmark"
            | "backup"
            | "backups"
            | "originalbackup"
            | "originalfiles"
            | "原版备份"
            | "原始备份"
            | "备份"
            | "crack"
            | "cracks"
            | "破解"
            | "patch"
            | "patches"
            | "补丁"
            | "汉化补丁"
            | "mod"
            | "mods"
            | "修改器"
            | "trainer"
            | "download"
            | "downloads"
    )
}

/// 识别老游戏常见的数据文件；这些证据只在 EXE 同目录内生效。
fn contains_game_data_files(names: &HashSet<String>) -> bool {
    names.iter().any(|name| {
        name.ends_with(".xp3")
            || name.ends_with(".rpa")
            || name.ends_with(".pfs")
            || name.ends_with(".wolf")
            || matches!(
                name.as_str(),
                "game.ini" | "data.arc" | "script.arc" | "data.pck" | "game.pak"
            )
    }) || names.contains("nw.dll") && names.contains("www")
}

/// Ren'Py 的 Windows 发行包通常同时包含三个运行目录，以及与 EXE 同名的 Python 启动脚本。
/// `Game-32.exe` 会复用 `Game.py`，因此识别脚本名时允许移除明确的 `-32` 后缀。
fn contains_renpy_runtime(names: &HashSet<String>, executable_stem: &str) -> bool {
    if !(names.contains("game") && names.contains("lib") && names.contains("renpy")) {
        return false;
    }

    let script_stem = executable_stem
        .strip_suffix("-32")
        .unwrap_or(executable_stem);
    names.contains(&format!("{script_stem}.py"))
}

/// 检查候选目录及常见网页资源位置，识别通过 NW.js/Chromium 发布的 RPG Maker 游戏。
fn detect_rpg_maker_nwjs_runtime(
    executable_directory: &Path,
    sibling_name_cache: &mut HashMap<PathBuf, HashSet<String>>,
) -> bool {
    let has_chromium_runtime = {
        let runtime_names = sibling_name_cache
            .entry(executable_directory.to_path_buf())
            .or_insert_with(|| read_sibling_names(executable_directory));
        contains_nwjs_runtime_markers(runtime_names)
    };
    if !has_chromium_runtime {
        return false;
    }

    [
        executable_directory.join("resources").join("App"),
        executable_directory.join("www"),
    ]
    .into_iter()
    .any(|app_directory| {
        let app_names = sibling_name_cache
            .entry(app_directory.clone())
            .or_insert_with(|| read_sibling_names(&app_directory));
        contains_rpg_maker_web_app(app_names)
    })
}

/// Chromium 文件只能证明存在网页运行容器，必须继续结合 RPG Maker 资源结构判断。
fn contains_nwjs_runtime_markers(names: &HashSet<String>) -> bool {
    names.contains("resources.pak")
        && names.contains("icudtl.dat")
        && names.contains("locales")
        && (names.contains("chrome_100_percent.pak")
            || names.contains("chrome_200_percent.pak")
            || names.contains("v8_context_snapshot.bin")
            || names.contains("nw.dll"))
}

/// RPG Maker MV/MZ 的网页游戏目录包含入口清单、页面和固定的资源目录组合。
fn contains_rpg_maker_web_app(names: &HashSet<String>) -> bool {
    names.contains("package.json")
        && names.contains("index.html")
        && names.contains("data")
        && names.contains("js")
        && (names.contains("img") || names.contains("audio"))
}

/// 检查部分商业视觉小说使用的编号资源包发行结构。
fn detect_numbered_pack_game_runtime(
    executable_directory: &Path,
    sibling_name_cache: &mut HashMap<PathBuf, HashSet<String>>,
) -> bool {
    let has_runtime_layout = {
        let root_names = sibling_name_cache
            .entry(executable_directory.to_path_buf())
            .or_insert_with(|| read_sibling_names(executable_directory));
        contains_numbered_pack_runtime_markers(root_names)
    };
    if !has_runtime_layout {
        return false;
    }

    let game_data_directory = executable_directory.join("GameData");
    let game_data_names = sibling_name_cache
        .entry(game_data_directory.clone())
        .or_insert_with(|| read_sibling_names(&game_data_directory));
    contains_multiple_numbered_pack_files(game_data_names)
}

/// 单个 `GameData` 或 `.pack` 文件过于常见，必须结合运行库和引擎配置文件判断。
fn contains_numbered_pack_runtime_markers(names: &HashSet<String>) -> bool {
    names.contains("gamedata")
        && names.contains("dll")
        && (names.contains("enginesetting.exe")
            || names.contains("エンジン設定.exe")
            || names.contains("engine_gui.u.txt") && names.contains("engine_message.u.txt"))
}

fn contains_multiple_numbered_pack_files(names: &HashSet<String>) -> bool {
    names
        .iter()
        .filter(|name| {
            name.strip_prefix("data")
                .and_then(|value| value.strip_suffix(".pack"))
                .is_some_and(|index| !index.is_empty() && index.chars().all(|character| character.is_ascii_digit()))
        })
        .take(2)
        .count()
        >= 2
}

/// AliceSoft 多代 System 引擎具有稳定但不同年代的发行结构。
/// 单独的 `.ain`、`.ald` 或通用 `system*.exe` 均不足以作为游戏证据。
fn contains_alicesoft_system_runtime(names: &HashSet<String>) -> bool {
    let has_ain = names.iter().any(|name| name.ends_with(".ain"));
    let has_alice_archive = names
        .iter()
        .any(|name| name.ends_with(".afa") || name.ends_with(".ald"));

    let alice_start_layout = names.contains("alicestart.ini") && has_ain && has_alice_archive;
    let system_39_or_40_layout = has_ain
        && has_alice_archive
        && ((names.contains("system39.exe") && names.contains("system39.ini"))
            || ((names.contains("system40.exe") || names.contains("system40_chs.exe"))
                && names.contains("system40.ini")));
    let xsystem_35_layout = names.contains(".xsys35rc") && has_ain && has_alice_archive;
    let system_3_data_layout = names.contains("system3.exe")
        && names.contains("system3.ini")
        && names.contains("adisk.dat")
        && ["acg.dat", "bcg.dat", "ccg.dat"]
            .into_iter()
            .filter(|name| names.contains(*name))
            .take(2)
            .count()
            >= 2;

    alice_start_layout || system_39_or_40_layout || xsystem_35_layout || system_3_data_layout
}

/// 去除非字母数字字符并转为小写，用于不区分格式的规则匹配。
fn compact_identifier(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

// -----------------------------------------------------------------------------
// Windows PE 版本信息
// -----------------------------------------------------------------------------

/// 从 Windows 版本资源读取 ProductName、FileDescription 等文本。
///
/// 读取失败或 EXE 没有版本资源时返回空元数据，扫描过程不会因此失败。
#[cfg(windows)]
fn read_executable_metadata(path: &Path) -> ExecutableMetadata {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use std::slice;

    use windows_sys::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
    };

    let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut ignored_handle = 0;
    // SAFETY: `path_wide` is NUL terminated and remains alive for the duration of the call.
    let size = unsafe { GetFileVersionInfoSizeW(path_wide.as_ptr(), &mut ignored_handle) };
    if size == 0 {
        return ExecutableMetadata::default();
    }

    let mut version_data = vec![0_u8; size as usize];
    // SAFETY: the destination buffer has exactly the size requested by Windows.
    let loaded = unsafe {
        GetFileVersionInfoW(
            path_wide.as_ptr(),
            0,
            size,
            version_data.as_mut_ptr().cast::<c_void>(),
        )
    };
    if loaded == 0 {
        return ExecutableMetadata::default();
    }

    // 查询版本资源中的原始指针；返回值只在 `version_data` 存活期间有效。
    fn query_raw(version_data: &[u8], key: &str) -> Option<(*mut c_void, u32)> {
        let key_wide: Vec<u16> = key.encode_utf16().chain(Some(0)).collect();
        let mut value = ptr::null_mut();
        let mut length = 0;
        // SAFETY: Windows owns the returned pointer inside `version_data`; all pointers live
        // through this call and the caller consumes them before `version_data` is dropped.
        let found = unsafe {
            VerQueryValueW(
                version_data.as_ptr().cast::<c_void>(),
                key_wide.as_ptr(),
                &mut value,
                &mut length,
            )
        };
        (found != 0 && !value.is_null() && length > 0).then_some((value, length))
    }

    // 将 StringFileInfo 中的 UTF-16 文本转换为 Rust String。
    fn query_string(version_data: &[u8], key: &str) -> Option<String> {
        let (value, length) = query_raw(version_data, key)?;
        // SAFETY: for StringFileInfo keys Windows returns `length` UTF-16 code units.
        let value = unsafe { slice::from_raw_parts(value.cast::<u16>(), length as usize) };
        let value = String::from_utf16_lossy(value)
            .trim_matches('\0')
            .trim()
            .to_string();
        (!value.is_empty()).then_some(value)
    }

    // 优先使用 EXE 自己声明的语言/代码页，再补充常见的英文和简体中文组合。
    let mut translations = Vec::new();
    if let Some((value, length)) = query_raw(&version_data, r"\VarFileInfo\Translation") {
        // SAFETY: Translation is documented as an array of language/code-page u16 pairs.
        let bytes = unsafe { slice::from_raw_parts(value.cast::<u8>(), length as usize) };
        for translation in bytes.chunks_exact(4) {
            translations.push((
                u16::from_le_bytes([translation[0], translation[1]]),
                u16::from_le_bytes([translation[2], translation[3]]),
            ));
        }
    }
    for fallback in [(0x0409, 1200), (0x0409, 1252), (0x0804, 1200)] {
        if !translations.contains(&fallback) {
            translations.push(fallback);
        }
    }

    let query_field = |field: &str| {
        translations.iter().find_map(|(language, code_page)| {
            query_string(
                &version_data,
                &format!(r"\StringFileInfo\{language:04x}{code_page:04x}\{field}"),
            )
        })
    };

    ExecutableMetadata {
        product_name: query_field("ProductName"),
        file_description: query_field("FileDescription"),
        original_filename: query_field("OriginalFilename"),
        company_name: query_field("CompanyName"),
    }
}

/// 非 Windows 平台没有 PE 版本资源，返回空值以保持核心评分逻辑可测试、可编译。
#[cfg(not(windows))]
fn read_executable_metadata(_path: &Path) -> ExecutableMetadata {
    ExecutableMetadata::default()
}

#[cfg(test)]
#[path = "scanner_tests.rs"]
mod tests;
