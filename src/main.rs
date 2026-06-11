//! dicfilter — フィルタ文字列を含む行を抽出するツール。
//!
//! コマンドライン引数がある場合は CLI モード、無い場合は GUI モードで起動する。

mod cli;
mod filter;
mod gui;

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    // 引数（プログラム名以外）が無ければ GUI を起動する。
    if std::env::args_os().count() <= 1 {
        match gui::run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("[エラー] GUI の起動に失敗しました: {}", err);
                ExitCode::FAILURE
            }
        }
    } else {
        let args = cli::CliArgs::parse();
        cli::run(args)
    }
}
