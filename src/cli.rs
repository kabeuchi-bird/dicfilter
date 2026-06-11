//! CLI モード。`dicfilter -i in.txt -o out.txt -f filter.txt`

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use crate::filter;

#[derive(Parser, Debug)]
#[command(
    name = "dicfilter",
    about = "フィルタファイルに記載された文字列のいずれかを含む行を抽出します。",
    long_about = "入力テキストファイルから、フィルタファイルに記載された各フィルタ文字列の\n\
                  いずれかを部分文字列として含む行を抽出し、出力ファイル（CRLF 改行）へ書き出します。\n\
                  引数を指定せずに起動すると GUI が立ち上がります。"
)]
pub struct CliArgs {
    /// 入力テキストファイル
    #[arg(short = 'i', long = "input", value_name = "FILE")]
    pub input: PathBuf,

    /// 出力ファイル
    #[arg(short = 'o', long = "output", value_name = "FILE", default_value = "output.txt")]
    pub output: PathBuf,

    /// フィルタファイル
    #[arg(short = 'f', long = "filter", value_name = "FILE", default_value = "filter.txt")]
    pub filter: PathBuf,
}

/// CLI を実行する。
pub fn run(args: CliArgs) -> ExitCode {
    match filter::run(&args.input, &args.output, &args.filter) {
        Ok(count) => {
            println!(
                "{} 行を抽出し、{} に出力しました。",
                count,
                args.output.display()
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            // エラーログを標準エラー出力へ
            eprintln!("[エラー] {}", err);
            // 日本語のエラー詳細を出力ファイルへ記載（ベストエフォート）
            if filter::write_error_report(&args.output, &err) {
                eprintln!(
                    "エラー詳細を {} に書き込みました。",
                    args.output.display()
                );
            } else {
                eprintln!(
                    "出力ファイル {} へのエラー詳細の書き込みにも失敗しました。",
                    args.output.display()
                );
            }
            ExitCode::FAILURE
        }
    }
}
