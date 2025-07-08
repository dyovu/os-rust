
#![no_main]
#![no_std]

use core::panic::PanicInfo;

/// パニックしたらマジでやばいからloopさせる
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)] // エントリーポイントである_start()の名前を変更しないためにmangleをさせない
pub extern "C" fn _start() -> ! { // _startで始めるのはUnixの慣習でリンカの設定でそうなってる
    
    loop {}
}

