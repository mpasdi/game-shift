use std::process::Command;

use tauri::AppHandle;

use crate::db;

use super::models::Game;
use super::repository;
use super::{current_timestamp_millis, normalize_existing_directory, normalize_existing_exe_path};

pub(super) fn launch_game(app: &AppHandle, id: &str) -> Result<Game, String> {
    let connection = db::open_connection(app)?;
    let id = id.trim();
    if id.is_empty() {
        return Err("游戏 ID 不能为空".to_string());
    }

    let game = repository::get_by_id(&connection, id)?
        .ok_or_else(|| "游戏不存在或已被删除".to_string())?;
    let exe_path = normalize_existing_exe_path(&game.exe_path)?;
    let work_dir = match game.work_dir.as_deref() {
        Some(value) if !value.trim().is_empty() => normalize_existing_directory(value)?,
        _ => exe_path
            .parent()
            .ok_or_else(|| "无法识别游戏所在目录".to_string())?
            .to_path_buf(),
    };

    let mut command = Command::new(&exe_path);
    command.current_dir(work_dir);
    if let Some(args) = game.args.as_deref() {
        command.args(parse_launch_args(args)?);
    }
    command
        .spawn()
        .map_err(|error| format_launch_error(&error))?;

    let now = current_timestamp_millis()?;
    repository::record_launch(&connection, id, now)?;
    repository::get_by_id(&connection, id)?.ok_or_else(|| "游戏启动后无法读取".to_string())
}

fn format_launch_error(error: &std::io::Error) -> String {
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
