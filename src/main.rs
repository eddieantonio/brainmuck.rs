use mmap_utils::{MappedRegion, WritableRegion};

fn main() -> Result<(), &'static str> {
    let my_page = MappedRegion::allocate(4096)?;

    println!(
        "my page is at {:0X} and has size {}",
        my_page.addr() as usize,
        my_page.len()
    );
    let mut my_page = WritableRegion::from(my_page)?;
    assemble(&mut my_page[..]);
    for i in 0..8 {
        println!("{:2}: {:02X}", i, my_page[i]);
    }

    Ok(())
}

fn assemble(p: &mut [u8]) {
    // mov x0, #42
    // s op          hw imm16                Rd
    // 1 10 1|0010|1 00 0|0000|0000|0101|010 0|0000
    // D      2    8      0    0    5    4     0
    p[3] = 0xd2;
    p[2] = 0x80;
    p[1] = 0x05;
    p[0] = 0x40;

    // NB: x30 is the link register
    // ret x30
    //          opc   op2     op3     Rn     op4
    // 1101|011 0|100 1|1111 |0000|00 11|110 0|0000
    // D    6     9     F     0    3     C     0
    p[7] = 0xd6;
    p[6] = 0x5f;
    p[5] = 0x03;
    p[4] = 0xC0;
}
