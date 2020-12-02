use crate::link_var;
link_var!(__bss_start);
link_var!(__bss_size);

pub unsafe fn setup_environment(dtb_addr: *mut i8) -> ! {
    // get bss section as slice
    let mut slice = core::slice::from_raw_parts_mut(
        &__bss_start as *const _ as *mut usize,
        &__bss_size as *const _ as usize
    );

    // zero the slice
    for thing in slice.iter_mut() {
        *thing = 0;
    }

    // run kinit
    crate::kinit(dtb_addr);
}