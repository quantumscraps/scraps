# scraps

scraps of an operating system

## Building

Building scraps requires a Rust nightly compiler.

To build use `./x.py build <board>`.
Boards can be listed with `./x.py listboards`.
To run on a QEMU emulator, use `./x.py run <board>`.

Currently, scraps targets RISC-V and AArch64.

Current list of boards we target:
* riscvirt (RISC-V)
* raspi64 (AArch64)

You can use `./x.py help` with no arguments for more help on usage.

## todo list

Mark off stuff as it's completed here

### general
* physical page allocator
* timer
* paging

### arch-specific

#### AArch64
* ARMv8 - early paging to allow atomics to work

#### RISC-V

### board-specific