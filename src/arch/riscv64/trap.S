.section .text.trap
.global asm_trap_vector
.option norvc # Important for vectored trap vectors

asm_trap_vector:
    # Mostly taken from osblog
    # Loops unrolled because idk how to make macros work
    csrrw t6, sscratch, t6

    # general purpose
    sd x0,  0(t6)
    sd x1,  8(t6)
    sd x2,  16(t6)
    sd x3,  24(t6)
    sd x4,  32(t6)
    sd x5,  40(t6)
    sd x6,  48(t6)
    sd x7,  56(t6)
    sd x8,  64(t6)
    sd x9,  72(t6)
    sd x10, 80(t6)
    sd x11, 88(t6)
    sd x12, 96(t6)
    sd x13, 104(t6)
    sd x14, 112(t6)
    sd x15, 120(t6)
    sd x16, 128(t6)
    sd x17, 136(t6)
    sd x18, 144(t6)
    sd x19, 152(t6)
    sd x20, 160(t6)
    sd x21, 168(t6)
    sd x22, 176(t6)
    sd x23, 184(t6)
    sd x24, 192(t6)
    sd x25, 200(t6)
    sd x26, 208(t6)
    sd x27, 216(t6)
    sd x28, 224(t6)
    sd x29, 232(t6)
    sd x30, 240(t6)
    # x31 (aka t6) is done at end
    # ends at 256

    # floating point
    # commented since GC doesn't include D
    # fsd f0,  256(t6)
    # fsd f1,  264(t6)
    # fsd f2,  272(t6)
    # fsd f3,  280(t6)
    # fsd f4,  288(t6)
    # fsd f5,  296(t6)
    # fsd f6,  304(t6)
    # fsd f7,  312(t6)
    # fsd f8,  320(t6)
    # fsd f9,  328(t6)
    # fsd f10, 336(t6)
    # fsd f11, 344(t6)
    # fsd f12, 352(t6)
    # fsd f13, 360(t6)
    # fsd f14, 368(t6)
    # fsd f15, 376(t6)
    # fsd f16, 384(t6)
    # fsd f17, 392(t6)
    # fsd f18, 400(t6)
    # fsd f19, 408(t6)
    # fsd f20, 416(t6)
    # fsd f21, 424(t6)
    # fsd f22, 432(t6)
    # fsd f23, 440(t6)
    # fsd f24, 448(t6)
    # fsd f25, 456(t6)
    # fsd f26, 464(t6)
    # fsd f27, 472(t6)
    # fsd f28, 480(t6)
    # fsd f29, 488(t6)
    # fsd f30, 496(t6)
    # fsd f31, 504(t6)
    # ends at 512

    # store x31
    mv t5, t6
    csrr t6, sscratch
    sd x31, 248(t5)

    # Restore kernel trap frame
    csrw sscratch, t5

    # Setup parameters
    csrr a0, sepc
    csrr a1, stval
    csrr a2, scause
    # csrr a3, mhartid
    mv a3, zero
    csrr a4, sstatus
    mv a5, t5
    ld sp, 520(a5)
    csrr a5, sscratch
    call trap_vector

    # Restore registers
    csrw sepc, a0
    csrr t6, sscratch
    
    # general purpose
    ld x0,  0(t6)
    ld x1,  8(t6)
    ld x2,  16(t6)
    ld x3,  24(t6)
    ld x4,  32(t6)
    ld x5,  40(t6)
    ld x6,  48(t6)
    ld x7,  56(t6)
    ld x8,  64(t6)
    ld x9,  72(t6)
    ld x10, 80(t6)
    ld x11, 88(t6)
    ld x12, 96(t6)
    ld x13, 104(t6)
    ld x14, 112(t6)
    ld x15, 120(t6)
    ld x16, 128(t6)
    ld x17, 136(t6)
    ld x18, 144(t6)
    ld x19, 152(t6)
    ld x20, 160(t6)
    ld x21, 168(t6)
    ld x22, 176(t6)
    ld x23, 184(t6)
    ld x24, 192(t6)
    ld x25, 200(t6)
    ld x26, 208(t6)
    ld x27, 216(t6)
    ld x28, 224(t6)
    ld x29, 232(t6)
    ld x30, 240(t6)
    # x31 (aka t6) is done at end
    # ends at 256

    # floating point
    # fld f0,  256(t6)
    # fld f1,  264(t6)
    # fld f2,  272(t6)
    # fld f3,  280(t6)
    # fld f4,  288(t6)
    # fld f5,  296(t6)
    # fld f6,  304(t6)
    # fld f7,  312(t6)
    # fld f8,  320(t6)
    # fld f9,  328(t6)
    # fld f10, 336(t6)
    # fld f11, 344(t6)
    # fld f12, 352(t6)
    # fld f13, 360(t6)
    # fld f14, 368(t6)
    # fld f15, 376(t6)
    # fld f16, 384(t6)
    # fld f17, 392(t6)
    # fld f18, 400(t6)
    # fld f19, 408(t6)
    # fld f20, 416(t6)
    # fld f21, 424(t6)
    # fld f22, 432(t6)
    # fld f23, 440(t6)
    # fld f24, 448(t6)
    # fld f25, 456(t6)
    # fld f26, 464(t6)
    # fld f27, 472(t6)
    # fld f28, 480(t6)
    # fld f29, 488(t6)
    # fld f30, 496(t6)
    # fld f31, 504(t6)
    # ends at 512

    # load x31
    ld x31, 248(t6)

    sret