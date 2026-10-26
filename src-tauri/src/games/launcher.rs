use std::path::PathBuf;
use std::process::Command;

use tauri::{AppHandle, Manager};

use crate::db;

use super::models::Game;
use super::repository;
use super::{normalize_existing_directory, normalize_existing_exe_path};

pub(super) fn launch_game(app: &AppHandle, id: &str) -> Result<Game, String> {
    let connection = db::open_connection(app)?;
    let id = id.trim();
    if id.is_empty() {
        return Err("游戏 ID 不能为空".to_string());
    }

    app.state::<super::runtime::RunningGames>().spawn(id, || {
        let game = repository::get_by_id(&connection, id)?
            .ok_or_else(|| "游戏不存在或已被删除".to_string())?;
        prepare_launch_command(
            &game.exe_path,
            game.work_dir.as_deref(),
            game.args.as_deref(),
        )
    })?;

    repository::get_by_id(&connection, id)?.ok_or_else(|| "游戏启动后无法读取".to_string())
}

fn prepare_launch_command(
    exe_path: &str,
    work_dir: Option<&str>,
    args: Option<&str>,
) -> Result<Command, String> {
    let exe_path = normalize_existing_exe_path(exe_path)?;
    let work_dir = match work_dir {
        Some(value) if !value.trim().is_empty() => normalize_existing_directory(value)?,
        _ => exe_path
            .parent()
            .ok_or_else(|| "无法识别游戏所在目录".to_string())?
            .to_path_buf(),
    };

    let mut command = Command::new(compatible_launch_path(exe_path)?);
    command.current_dir(compatible_launch_path(work_dir)?);
    if let Some(args) = args {
        command.args(parse_launch_args(args)?);
    }
    Ok(command)
}

fn compatible_launch_path(path: PathBuf) -> Result<PathBuf, String> {
    // canonicalize 在 Windows 上生成扩展路径。Ren'Py 等运行时不能正确处理
    // 传入的该类路径，因此只在进程启动边界转换，保留之前的文件和目录校验。
    #[cfg(windows)]
    {
        super::path_to_string(path).map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        Ok(path)
    }
}

pub(super) fn format_launch_error(error: &std::io::Error) -> String {
    match error.raw_os_error() {
        Some(193 | 216) => "该文件无法作为 Windows 程序运行，请重新选择".to_string(),
        Some(5) => "没有权限启动这个文件，请检查文件权限后重试".to_string(),
        Some(740) => "这个程序需要管理员权限，请尝试以管理员身份运行 Game Shift".to_string(),
        _ => "Windows 无法启动这个文件，请检查文件是否有效或仍可访问".to_string(),
    }
}

fn parse_launch_args(args: &str) -> Result<Vec<String>, String> {
    let mut parsed = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut argument_started = false;
    let mut characters = args.chars().peekable();

    while let Some(character) = characters.next() {
        match character {
            '"' => {
                in_quotes = !in_quotes;
                argument_started = true;
            }
            '\\' => {
                let mut backslash_count = 1;
                while characters.peek() == Some(&'\\') {
                    characters.next();
                    backslash_count += 1;
                }

                if characters.peek() == Some(&'"') {
                    current.extend(std::iter::repeat_n('\\', backslash_count / 2));
                    characters.next();
                    if backslash_count % 2 == 0 {
                        in_quotes = !in_quotes;
                    } else {
                        current.push('"');
                    }
                } else {
                    current.extend(std::iter::repeat_n('\\', backslash_count));
                }
                argument_started = true;
            }
            value if value.is_whitespace() && !in_quotes => {
                if argument_started {
                    parsed.push(std::mem::take(&mut current));
                    argument_started = false;
                }
            }
            value => {
                current.push(value);
                argument_started = true;
            }
        }
    }

    if in_quotes {
        return Err("启动参数中的引号未闭合".to_string());
    }
    if argument_started {
        parsed.push(current);
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::{format_launch_error, parse_launch_args};

    #[cfg(windows)]
    #[test]
    fn launch_paths_remove_extended_prefixes_and_preserve_unc() {
        for (input, expected) in [
            (
                r"\\?\C:\Games\中文 Game\Game.exe",
                r"C:\Games\中文 Game\Game.exe",
            ),
            (r"\\?\UNC\server\share\Game.exe", r"\\server\share\Game.exe"),
            (r"C:\Games\Game.exe", r"C:\Games\Game.exe"),
            (r"\\server\share\Games", r"\\server\share\Games"),
        ] {
            assert_eq!(
                super::compatible_launch_path(input.into()).unwrap(),
                std::path::PathBuf::from(expected)
            );
        }
    }

    #[test]
    fn launch_command_validates_paths_and_preserves_arguments() {
        // 仅构造命令、不运行游戏；临时文件用于执行真实的路径校验和 canonicalize。
        let root = std::env::temp_dir().join(format!(
            "game-shift-launch-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(root.clone());
        let game_dir = root.join("中文 Game");
        let custom_dir = root.join("Custom Working Directory");
        std::fs::create_dir_all(&game_dir).unwrap();
        std::fs::create_dir_all(&custom_dir).unwrap();
        let exe = game_dir.join("Game.exe");
        std::fs::write(&exe, []).unwrap();
        let expected_exe = super::compatible_launch_path(exe.canonicalize().unwrap()).unwrap();
        for work_dir in [None, Some(""), Some("   "), custom_dir.to_str()] {
            let command = super::prepare_launch_command(
                exe.to_str().unwrap(),
                work_dir,
                Some(r#"--label "hello world" --path "C:\Game Saves""#),
            )
            .unwrap();
            assert_eq!(command.get_program(), expected_exe.as_os_str());
            let expected_dir = if work_dir.is_some_and(|value| !value.trim().is_empty()) {
                &custom_dir
            } else {
                &game_dir
            };
            let expected_dir =
                super::compatible_launch_path(expected_dir.canonicalize().unwrap()).unwrap();
            assert_eq!(command.get_current_dir(), Some(expected_dir.as_path()));
            assert_eq!(
                command.get_args().collect::<Vec<_>>(),
                vec!["--label", "hello world", "--path", r"C:\Game Saves"]
            );
            #[cfg(windows)]
            {
                assert!(!command.get_program().to_string_lossy().starts_with(r"\\?\"));
                assert!(!command
                    .get_current_dir()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(r"\\?\"));
            }
        }
        let missing = root.join("missing.exe");
        assert_eq!(
            super::prepare_launch_command(missing.to_str().unwrap(), None, None).unwrap_err(),
            "启动程序不存在"
        );
        assert_eq!(
            super::prepare_launch_command(game_dir.to_str().unwrap(), None, None).unwrap_err(),
            "启动路径必须指向文件"
        );
        let other = root.join("file.txt");
        std::fs::write(&other, []).unwrap();
        assert_eq!(
            super::prepare_launch_command(other.to_str().unwrap(), None, None).unwrap_err(),
            "启动路径必须是 .exe 文件"
        );
        assert_eq!(
            super::prepare_launch_command(exe.to_str().unwrap(), missing.to_str(), None)
                .unwrap_err(),
            "工作目录不存在"
        );
        assert_eq!(
            super::prepare_launch_command(exe.to_str().unwrap(), exe.to_str(), None).unwrap_err(),
            "工作目录必须是文件夹"
        );
        assert!(super::prepare_launch_command(exe.to_str().unwrap(), None, Some("\"")).is_err());
    }

    #[test]
    fn launch_error_hides_windows_placeholders() {
        let error = std::io::Error::from_raw_os_error(216);
        let message = format_launch_error(&error);
        assert_eq!(message, "该文件无法作为 Windows 程序运行，请重新选择");
        assert!(!message.contains("%1"));
    }

    #[test]
    fn parses_quoted_windows_path_without_removing_backslashes() {
        let parsed =
            parse_launch_args(r#""C:\Users\lsm\AppData\Local\Temp\game shift.txt""#).unwrap();
        assert_eq!(
            parsed,
            vec![r"C:\Users\lsm\AppData\Local\Temp\game shift.txt"]
        );
    }

    #[test]
    fn parses_command_with_quoted_argument() {
        let parsed =
            parse_launch_args(r#"/c "echo Game Shift args OK> %TEMP%\game-shift-args-test.txt""#)
                .unwrap();
        assert_eq!(
            parsed,
            vec![
                "/c",
                r"echo Game Shift args OK> %TEMP%\game-shift-args-test.txt"
            ]
        );
    }

    #[test]
    fn preserves_escaped_quotes_and_empty_arguments() {
        let parsed = parse_launch_args(r#"--message "say \"hello\"" --label """#).unwrap();
        assert_eq!(parsed, vec!["--message", "say \"hello\"", "--label", ""]);
    }

    #[test]
    fn rejects_unclosed_quotes() {
        let error = parse_launch_args(r#""C:\Games\example.exe"#).unwrap_err();
        assert_eq!(error, "启动参数中的引号未闭合");
    }
}
