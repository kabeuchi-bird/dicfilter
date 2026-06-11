//! テキストフィルタのコアロジック。
//!
//! 入力ファイルから、フィルタファイルに記載された各フィルタ文字列の
//! いずれかを含む行を抽出し、CRLF 改行で出力ファイルへ書き出す。

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// 処理中に発生し得るエラー。`Display` は日本語の詳細メッセージを返す。
#[derive(Debug)]
pub enum DicError {
    /// 入力ファイルの読み込みに失敗。
    InputRead { path: PathBuf, source: std::io::Error },
    /// フィルタファイルの読み込みに失敗。
    FilterRead { path: PathBuf, source: std::io::Error },
    /// フィルタファイルに有効なフィルタが 1 つも無い。
    EmptyFilter { path: PathBuf },
    /// 出力ファイルの書き込みに失敗。
    OutputWrite { path: PathBuf, source: std::io::Error },
}

impl fmt::Display for DicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DicError::InputRead { path, source } => write!(
                f,
                "入力ファイル「{}」の読み込みに失敗しました: {}",
                path.display(),
                source
            ),
            DicError::FilterRead { path, source } => write!(
                f,
                "フィルタファイル「{}」の読み込みに失敗しました: {}",
                path.display(),
                source
            ),
            DicError::EmptyFilter { path } => write!(
                f,
                "フィルタファイル「{}」に有効なフィルタが 1 つもありません。",
                path.display()
            ),
            DicError::OutputWrite { path, source } => write!(
                f,
                "出力ファイル「{}」の書き込みに失敗しました: {}",
                path.display(),
                source
            ),
        }
    }
}

impl std::error::Error for DicError {}

/// 文字列を行に分割する。CRLF / LF の両方に対応し、各行末の `\r` を除去する。
fn split_lines(content: &str) -> Vec<&str> {
    content
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect()
}

/// フィルタファイルを読み込み、フィルタ文字列の一覧を返す。
///
/// 1 行 1 フィルタ。空行は無視する。行に複数文字があれば、その文字列
/// 全体を 1 つのフィルタ（部分文字列）として扱う。
pub fn load_filters(path: &Path) -> Result<Vec<String>, DicError> {
    let content = fs::read_to_string(path).map_err(|source| DicError::FilterRead {
        path: path.to_path_buf(),
        source,
    })?;

    let filters: Vec<String> = split_lines(&content)
        .into_iter()
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();

    if filters.is_empty() {
        return Err(DicError::EmptyFilter {
            path: path.to_path_buf(),
        });
    }

    Ok(filters)
}

/// 入力ファイルを読み込み、フィルタに一致した行のみを返す。
///
/// 各入力行について、いずれかのフィルタ文字列を部分文字列として含む場合に
/// 抽出する。
pub fn filter_lines(input_path: &Path, filters: &[String]) -> Result<Vec<String>, DicError> {
    let content = fs::read_to_string(input_path).map_err(|source| DicError::InputRead {
        path: input_path.to_path_buf(),
        source,
    })?;

    let matched: Vec<String> = split_lines(&content)
        .into_iter()
        .filter(|line| filters.iter().any(|filter| line.contains(filter)))
        .map(|line| line.to_string())
        .collect();

    Ok(matched)
}

/// 行の一覧を CRLF 改行で出力ファイルへ書き込む。
///
/// 各行末に CRLF を付与する（末尾行にも付与する）。
pub fn write_output(path: &Path, lines: &[String]) -> Result<(), DicError> {
    let capacity: usize = lines.iter().map(|line| line.len() + 2).sum();
    let mut out = String::with_capacity(capacity);
    for line in lines {
        out.push_str(line);
        out.push_str("\r\n");
    }
    fs::write(path, out).map_err(|source| DicError::OutputWrite {
        path: path.to_path_buf(),
        source,
    })
}

/// 一連の処理（フィルタ読込 → 抽出 → 出力）を実行し、抽出した行数を返す。
pub fn run(input: &Path, output: &Path, filter: &Path) -> Result<usize, DicError> {
    let filters = load_filters(filter)?;
    let matched = filter_lines(input, &filters)?;
    write_output(output, &matched)?;
    Ok(matched.len())
}

/// エラー発生時に、日本語のエラー詳細を出力ファイルへ書き込む（ベストエフォート）。
///
/// 戻り値は出力ファイルへの書き込みに成功したかどうか。
pub fn write_error_report(output: &Path, err: &DicError) -> bool {
    let body = format!(
        "エラーが発生しました。\r\n\r\n詳細: {}\r\n",
        err
    );
    fs::write(output, body).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("dicfilter_test_{}_{}", std::process::id(), name));
        p
    }

    fn write_file(path: &Path, content: &[u8]) {
        let mut f = fs::File::create(path).unwrap();
        f.write_all(content).unwrap();
    }

    #[test]
    fn split_handles_crlf_and_lf() {
        assert_eq!(split_lines("a\r\nb\nc"), vec!["a", "b", "c"]);
    }

    #[test]
    fn filters_skip_empty_lines_and_keep_multichar() {
        let fpath = temp_path("filter.txt");
        write_file(&fpath, b"ab\r\n\r\nx\n");
        let filters = load_filters(&fpath).unwrap();
        assert_eq!(filters, vec!["ab".to_string(), "x".to_string()]);
        fs::remove_file(&fpath).ok();
    }

    #[test]
    fn extracts_matching_lines_and_writes_crlf() {
        let input = temp_path("input.txt");
        let filter = temp_path("filt.txt");
        let output = temp_path("out.txt");
        // LF 入力でも処理できること
        write_file(&input, b"hello\nworld\nabc\n");
        write_file(&filter, b"ab\nwo\n");
        let n = run(&input, &output, &filter).unwrap();
        assert_eq!(n, 2);
        let written = fs::read(&output).unwrap();
        assert_eq!(written, b"world\r\nabc\r\n");
        for p in [input, filter, output] {
            fs::remove_file(&p).ok();
        }
    }

    #[test]
    fn missing_input_errors() {
        let input = temp_path("nope_input.txt");
        let filter = temp_path("filt2.txt");
        let output = temp_path("out2.txt");
        write_file(&filter, b"x\n");
        let err = run(&input, &output, &filter).unwrap_err();
        assert!(matches!(err, DicError::InputRead { .. }));
        assert!(format!("{}", err).contains("読み込みに失敗"));
        fs::remove_file(&filter).ok();
    }
}
