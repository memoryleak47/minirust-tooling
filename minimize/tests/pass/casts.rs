extern crate intrinsics;
use intrinsics::*;

fn black_box<T>(t: T) -> T { t }

fn main() {
    print(black_box(0u32) as u8); // 0
    print(black_box(256u32 + 42) as u8); // 42
    print(black_box(255u8) as i8); // -1
}
