use crate::link_var;
link_var!(__bss_start);
link_var!(__bss_size);

const fn usize_subdivide(size: usize) -> usize {
    assert!(
        size % core::mem::size_of::<usize>() == 0,
        "bss size must be a multiple of sizeof(usize)"
    );
    size / core::mem::size_of::<usize>()
}
pub unsafe fn setup_environment(dtb_addr: *mut i8) -> ! {
    // get bss section as slice
    let mut slice = core::slice::from_raw_parts_mut(
        &__bss_start as *const _ as *mut usize,
        usize_subdivide(&__bss_size as *const _ as usize)
    );

    // zero the slice
    for thing in slice.iter_mut() {
        *thing = 0;
    }

    // run kinit
    crate::kinit(dtb_addr);
}