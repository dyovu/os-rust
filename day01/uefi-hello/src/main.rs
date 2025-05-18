
#![no_main] 
// uefiアプリケーションでのエントリーポイントは通常のmainではなく、entryマクロで指定する

#![no_std]  
// ブートローダーを作ってるのでベアメタル環境、stdは使えない 
// coreとallocクレートは引き続き使用できます

use log::info;
use uefi::prelude::*;

// UEFIアプリケーションのmain関数は引数を取らず、Statusを返す
#[entry] 
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Hello world!");
    boot::stall(10_000_000);
    Status::SUCCESS
}
