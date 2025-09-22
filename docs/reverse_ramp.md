int CreateAndConfigureMetalTexture(int arg0, int arg1, int arg2, int arg3, int arg4, int arg5, int arg6, int arg7, int arg8) {
    var_24 = arg8;
    var_16 = arg7;
    var_8 = arg6;
    r15 = arg5;
    var_2C = arg4;
    r13 = arg3;
    r14 = arg2;
    r12 = arg1;
    rbx = arg0;
    sub_10193ff00(arg0, arg1, arg2, r13, r15, var_8, var_16, var_24);
    *rbx = 0x1023a4fe8;
    *(int32_t *)(rbx + 0x14) = 0x1;
    *(int128_t *)(rbx + 0x18) = intrinsic_movups(*(int128_t *)(rbx + 0x18), 0x0);
    rax = [MTLTextureDescriptor new];
    *(rbx + 0x20) = rax;
    if ((r13 & 0xfe) == 0xc) {
            rax = sub_10193c080();
            (*(*rax + 0x38))(rax);
            rax = [*((*(*rax + 0x30))(rax) + 0x180) isDepth24Stencil8PixelFormatSupported];
            rdx = 0xff + (rax == 0x0 ? 0x1 : 0x0) + (rax == 0x0 ? 0x1 : 0x0) * 0x4;
            rax = *(rbx + 0x20);
    }
    else {
            rdx = *(0x1020941d0 + (r13 & 0xff) * 0x8);
    }
    [rax setPixelFormat:rdx];
    if (r12 < 0x4) {
            [*(rbx + 0x20) setTextureType:*(0x102094298 + sign_extend_64(r12) * 0x8)];
    }
    [*(rbx + 0x20) setWidth:r15];
    [*(rbx + 0x20) setHeight:var_8];
    if (r12 == 0x3) {
            [*(rbx + 0x20) setDepth:0x1];
            rax = &@selector(setArrayLength:);
    }
    else {
            rax = &@selector(setDepth:);
    }
    (*_objc_msgSend)(*(rbx + 0x20), *rax);
    [*(rbx + 0x20) setResourceOptions:0x20];
    [*(rbx + 0x20) setMipmapLevelCount:var_24];
    rax = var_2C & 0xff ^ 0x5;
    rdi = *(rbx + 0x20);
    rdx = 0x1;
    if (r14 != 0x0) {
            rdx = rax;
    }
    [rdi setUsage:rdx];
    rax = sub_10193c080();
    (*(*rax + 0x38))(rax);
    rax = [*((*(*rax + 0x30))(rax) + 0x180) newTextureWithDescriptor:*(rbx + 0x20)];
    *(rbx + 0x18) = rax;
    return rax;
}

这是游戏里唯一创建 MTLTextureDescriptor 的地方

int sub_101959be0(int arg0, int arg1, int arg2, int arg3, int arg4, int arg5, int arg6) {
    var_8 = arg6;
    r14 = arg1;
    var_48 = arg0;
    rax = operator new(0x28, arg1, arg2, arg3);
    r12 = rax;
    var_40 = arg4;
    var_34 = arg3;
    var_140 = arg3;
    var_30 = arg2;
    CreateAndConfigureMetalTexture(rax, 0x0, arg5, r14, 0x0, arg2, var_140, 0x1, arg4);
    if (*(int32_t *)(var_8 + 0x8) != 0x0) {
            var_2C = r14 & 0xff;
            rdx = sub_101940000(0x0, r14 & 0xff, var_30, var_34, 0x1, var_40);
            r15 = var_48;
            rax = [*(r15 + 0x180) newBufferWithLength:rdx options:0x0];
            rbx = rax;
            r14 = [rax contents];
            rax = *(r15 + 0x910);
            if (rax == 0x0) {
                    rax = operator new(0x10, @selector(contents), rdx, 0x0);
                    *(int128_t *)rax = intrinsic_movups(*(int128_t *)rax, 0x0);
                    *(r15 + 0x910) = rax;
                    **(r15 + 0x910) = [*(r15 + 0x188) commandBuffer];
                    *(*(r15 + 0x910) + 0x8) = [**(r15 + 0x910) blitCommandEncoder];
                    [**(r15 + 0x910) retain];
                    [*(*(r15 + 0x910) + 0x8) retain];
                    rax = *(r15 + 0x910);
            }
            var_40 = rax;
            var_50 = rbx;
            var_68 = r14;
            if (*(int32_t *)(var_8 + 0x8) != 0x0) {
                    var_58 = @selector(copyFromBuffer:sourceOffset:sourceBytesPerRow:sourceBytesPerImage:sourceSize:toTexture:destinationSlice:destinationLevel:destinationOrigin:);
                    r15 = 0x0;
                    rbx = 0x0;
                    var_60 = r12;
                    do {
                            CMP(var_30 >> rbx, 0x1);
                            r14 = (var_30 >> rbx) + 0x0 + CARRY(RFLAGS(cf));
                            CMP(var_34 >> rbx, 0x1);
                            r13 = sub_101940000(0x0, var_2C, r14, (var_34 >> rbx) + 0x0 + CARRY(RFLAGS(cf)), 0x1, 0x1);
                            sub_10193ff50(var_2C, r14);
                            memcpy(var_68 + r15, *(*var_8 + rbx * 0x8), r13);
                            rdi = *(var_40 + 0x8);
                            r12 = var_60;
                            var_D0 = intrinsic_movaps(var_D0, 0x0);
                            xmm0 = intrinsic_movaps(0x0, var_D0);
                            var_110 = intrinsic_movups(var_110, xmm0);
                            var_140 = intrinsic_movups(var_140, intrinsic_movups(xmm0, r14));
                            (*_objc_msgSend)(rdi, var_58);
                            r15 = r15 + r13;
                            rbx = rbx + 0x1;
                    } while (rbx < *(int32_t *)(var_8 + 0x8));
            }
            var_B0 = 0x0;
            rdi = *var_40;
            var_F8 = *__NSConcreteStackBlock;
            [rdi addCompletedHandler:&var_F8];
            _Block_object_dispose(&var_B0, 0x8);
    }
    rax = r12;
    return rax;
}

int sub_10195a020(int arg0, int arg1, int arg2, int arg3, int arg4, int arg5, int arg6, int arg7) {
    var_16 = arg7;
    var_40 = arg4;
    r13 = arg1;
    var_50 = arg0;
    rax = operator new(0x28, arg1, arg2, arg3);
    r12 = rax;
    var_48 = arg5;
    r15 = var_40;
    var_2C = arg2;
    var_160 = r15;
    var_3C = arg3;
    CreateAndConfigureMetalTexture(rax, 0x3, arg6 & 0xff, r13, 0x0, arg3, var_160, arg2, arg5);
    if (*(int32_t *)(var_16 + 0x8) != 0x0) {
            var_54 = r13 & 0xff;
            r14 = var_3C;
            rdx = sub_101940000(0x3, r13 & 0xff, r14, r15, var_2C, var_48);
            r15 = var_50;
            rax = [*(r15 + 0x180) newBufferWithLength:rdx options:0x0];
            rbx = rax;
            var_88 = [rax contents];
            rax = *(r15 + 0x910);
            if (rax == 0x0) {
                    rax = operator new(0x10, @selector(contents), rdx, 0x0);
                    *(int128_t *)rax = intrinsic_movups(*(int128_t *)rax, 0x0);
                    *(r15 + 0x910) = rax;
                    **(r15 + 0x910) = [*(r15 + 0x188) commandBuffer];
                    *(*(r15 + 0x910) + 0x8) = [**(r15 + 0x910) blitCommandEncoder];
                    [**(r15 + 0x910) retain];
                    [*(*(r15 + 0x910) + 0x8) retain];
                    rax = *(r15 + 0x910);
            }
            var_60 = rax;
            rax = var_2C;
            if (rax != 0x0) {
                    var_70 = rax;
                    rdx = 0x0;
                    rsi = 0x0;
                    var_80 = r12;
                    var_78 = rbx;
                    do {
                            rdi = *var_16;
                            rax = rdx << 0x4;
                            var_38 = rdi;
                            var_68 = rdx;
                            if (*(int32_t *)(rdi + rax + 0x8) != 0x0) {
                                    var_38 = var_38 + rax;
                                    var_90 = @selector(copyFromBuffer:sourceOffset:sourceBytesPerRow:sourceBytesPerImage:sourceSize:toTexture:destinationSlice:destinationLevel:destinationOrigin:);
                                    r15 = 0x0;
                                    do {
                                            CMP(r14 >> r15, 0x1);
                                            rbx = (r14 >> r15) + 0x0 + CARRY(RFLAGS(cf));
                                            CMP(var_40 >> r15, 0x1);
                                            rax = sub_101940000(0x0, var_54, rbx, (var_40 >> r15) + 0x0 + CARRY(RFLAGS(cf)), 0x1, 0x1);
                                            r13 = rax;
                                            sub_10193ff50(var_54, rbx);
                                            memcpy(var_88 + rsi, *(*var_38 + r15 * 0x8), rax);
                                            rdi = *(var_60 + 0x8);
                                            r14 = var_3C;
                                            var_A8 = rbx;
                                            rbx = var_78;
                                            r12 = var_80;
                                            var_F0 = intrinsic_movaps(var_F0, 0x0);
                                            xmm0 = intrinsic_movaps(0x0, var_F0);
                                            var_130 = intrinsic_movups(var_130, xmm0);
                                            var_160 = intrinsic_movups(var_160, intrinsic_movups(xmm0, var_A8));
                                            (*_objc_msgSend)(rdi, var_90);
                                            r13 = r13 + rsi;
                                            r15 = r15 + 0x1;
                                            rsi = r13;
                                    } while (r15 < *(int32_t *)(var_38 + 0x8));
                            }
                            else {
                                    r13 = rsi;
                            }
                            rdx = var_68 + 0x1;
                            rsi = r13;
                            if (rdx == var_70) {
                                break;
                            }
                            else {
                                continue;
                            }
                    } while (true);
            }
            var_D8 = 0x0;
            rdi = *var_60;
            var_118 = *__NSConcreteStackBlock;
            [rdi addCompletedHandler:&var_118];
            _Block_object_dispose(&var_D8, 0x8);
    }
    rax = r12;
    return rax;
}

int sub_10195a460(int arg0, int arg1, int arg2, int arg3, int arg4, int arg5, int arg6, int arg7) {
    var_16 = arg7;
    var_38 = arg2;
    r12 = arg1;
    var_50 = arg0;
    rax = operator new(0x28, arg1, arg2, arg3);
    r13 = rax;
    var_40 = arg5;
    r14 = arg4;
    var_2C = arg3;
    var_140 = arg3;
    r15 = var_38;
    CreateAndConfigureMetalTexture(rax, 0x1, arg6 & 0xff, r12, 0x0, r15, var_140, arg4, arg5);
    if (*(int32_t *)(var_16 + 0x8) != 0x0) {
            var_30 = r12 & 0xff;
            var_34 = r14;
            rdx = sub_101940000(0x1, r12 & 0xff, r15, var_2C, r14, var_40);
            r14 = var_50;
            rax = [*(r14 + 0x180) newBufferWithLength:rdx options:0x0];
            r12 = rax;
            rbx = [rax contents];
            rax = *(r14 + 0x910);
            if (rax == 0x0) {
                    rax = operator new(0x10, @selector(contents), rdx, 0x0);
                    *(int128_t *)rax = intrinsic_movups(*(int128_t *)rax, 0x0);
                    *(r14 + 0x910) = rax;
                    **(r14 + 0x910) = [*(r14 + 0x188) commandBuffer];
                    *(*(r14 + 0x910) + 0x8) = [**(r14 + 0x910) blitCommandEncoder];
                    [**(r14 + 0x910) retain];
                    [*(*(r14 + 0x910) + 0x8) retain];
                    rax = *(r14 + 0x910);
            }
            var_40 = rax;
            rdx = var_34;
            rdi = var_2C;
            rsi = var_30;
            var_58 = r12;
            var_70 = rbx;
            if (*(int32_t *)(var_16 + 0x8) != 0x0) {
                    var_60 = @selector(copyFromBuffer:sourceOffset:sourceBytesPerRow:sourceBytesPerImage:sourceSize:toTexture:destinationSlice:destinationLevel:destinationOrigin:);
                    r12 = 0x0;
                    r14 = 0x0;
                    var_68 = r13;
                    do {
                            CMP(var_38 >> r14, 0x1);
                            r15 = (var_38 >> r14) + 0x0 + CARRY(RFLAGS(cf));
                            CMP(rdi >> r14, 0x1);
                            r13 = (rdi >> r14) + 0x0 + CARRY(RFLAGS(cf));
                            CMP(rdx >> r14, 0x1);
                            var_48 = sub_101940000(0x1, rsi, r15, r13, (rdx >> r14) + 0x0 + CARRY(RFLAGS(cf)), 0x1);
                            sub_10193ff50(var_30, r15);
                            memcpy(var_70 + r12, *(*var_16 + r14 * 0x8), var_48);
                            rdi = *(var_40 + 0x8);
                            r13 = var_68;
                            var_D0 = intrinsic_movaps(var_D0, 0x0);
                            xmm0 = intrinsic_movaps(0x0, var_D0);
                            var_110 = intrinsic_movups(var_110, xmm0);
                            var_140 = intrinsic_movups(var_140, intrinsic_movups(xmm0, r15));
                            (*_objc_msgSend)(rdi, var_60);
                            rsi = var_30;
                            rdi = var_2C;
                            rdx = var_34;
                            r12 = r12 + var_48;
                            r14 = r14 + 0x1;
                    } while (r14 < *(int32_t *)(var_16 + 0x8));
            }
            var_B8 = 0x0;
            rdi = *var_40;
            var_F8 = *__NSConcreteStackBlock;
            [rdi addCompletedHandler:&var_F8];
            _Block_object_dispose(&var_B8, 0x8);
    }
    rax = r13;
    return rax;
}

int sub_10195a850(int arg0, int arg1, int arg2, int arg3, int arg4, int arg5) {
    rbx = arg5;
    r15 = arg1;
    var_78 = arg0;
    rax = operator new(0x28, arg1, arg2, arg3);
    r13 = rax;
    var_50 = arg3;
    var_160 = arg2;
    var_9C = arg2;
    CreateAndConfigureMetalTexture(rax, 0x2, arg4, r15, 0x0, arg2, var_160, 0x1, arg3);
    if (*(int32_t *)(rbx + 0x8) != 0x0) {
            var_58 = rbx;
            var_90 = r13;
            rbx = r15 & 0xff;
            rdx = sub_101940000(0x2, r15 & 0xff, var_9C, var_9C, 0x1, var_50);
            r12 = var_78;
            rax = [*(r12 + 0x180) newBufferWithLength:rdx options:0x0];
            r14 = rax;
            r15 = [rax contents];
            rax = *(r12 + 0x910);
            if (rax == 0x0) {
                    rax = operator new(0x10, @selector(contents), rdx, 0x0);
                    *(int128_t *)rax = intrinsic_movups(*(int128_t *)rax, 0x0);
                    *(r12 + 0x910) = rax;
                    **(r12 + 0x910) = [*(r12 + 0x188) commandBuffer];
                    *(*(r12 + 0x910) + 0x8) = [**(r12 + 0x910) blitCommandEncoder];
                    [**(r12 + 0x910) retain];
                    [*(*(r12 + 0x910) + 0x8) retain];
                    r14 = r14;
                    rax = *(r12 + 0x910);
            }
            var_88 = rax;
            var_98 = r15;
            if (*(int32_t *)(var_58 + 0x8) != 0x0) {
                    rdx = rbx;
                    r13 = 0x0;
                    var_A8 = @selector(copyFromBuffer:sourceOffset:sourceBytesPerRow:sourceBytesPerImage:sourceSize:toTexture:destinationSlice:destinationLevel:destinationOrigin:);
                    rbx = 0x0;
                    rcx = 0x0;
                    var_50 = r14;
                    var_A0 = rdx;
                    do {
                            CMP(var_9C >> rcx, 0x1);
                            rax = sub_101940000(0x0, var_A0, (var_9C >> rcx) + 0x0 + CARRY(RFLAGS(cf)), (var_9C >> rcx) + 0x0 + CARRY(RFLAGS(cf)), 0x1, 0x1);
                            var_7C = rax;
                            r15 = rax;
                            sub_10193ff50(var_A0, (var_9C >> rcx) + 0x0 + CARRY(RFLAGS(cf)));
                            memcpy(var_98 + rbx, *(*var_58 + r13), r15);
                            rdi = *(var_88 + 0x8);
                            var_70 = intrinsic_movaps(var_70, 0x0);
                            xmm0 = intrinsic_movaps(0x0, var_70);
                            var_130 = intrinsic_movups(var_130, xmm0);
                            var_160 = intrinsic_movups(var_160, intrinsic_movups(xmm0, (var_9C >> rcx) + 0x0 + CARRY(RFLAGS(cf))));
                            _objc_msgSend_10228c418(rdi, var_A8);
                            memcpy(var_98 + rbx + var_7C, *(*var_58 + r13 + 0x8), r15);
                            rdi = *(var_88 + 0x8);
                            var_70 = intrinsic_movaps(var_70, 0x0);
                            xmm0 = intrinsic_movaps(0x0, var_70);
                            var_130 = intrinsic_movups(var_130, xmm0);
                            var_160 = intrinsic_movups(var_160, intrinsic_movups(xmm0, (var_9C >> rcx) + 0x0 + CARRY(RFLAGS(cf))));
                            (*_objc_msgSend)(rdi, var_A8);
                            memcpy(var_98 + var_7C + rbx + var_7C, *(*var_58 + r13 + 0x10), r15);
                            rdi = *(var_88 + 0x8);
                            var_70 = intrinsic_movaps(var_70, 0x0);
                            xmm0 = intrinsic_movaps(0x0, var_70);
                            var_130 = intrinsic_movups(var_130, xmm0);
                            var_160 = intrinsic_movups(var_160, intrinsic_movups(xmm0, (var_9C >> rcx) + 0x0 + CARRY(RFLAGS(cf))));
                            (*_objc_msgSend)(rdi, var_A8);
                            memcpy(var_98 + var_7C + var_7C + rbx + var_7C, *(*var_58 + r13 + 0x18), r15);
                            rdi = *(var_88 + 0x8);
                            var_70 = intrinsic_movaps(var_70, 0x0);
                            xmm0 = intrinsic_movaps(0x0, var_70);
                            var_130 = intrinsic_movups(var_130, xmm0);
                            var_160 = intrinsic_movups(var_160, intrinsic_movups(xmm0, (var_9C >> rcx) + 0x0 + CARRY(RFLAGS(cf))));
                            _objc_msgSend_10228c418(rdi, var_A8);
                            memcpy(var_98 + var_7C + var_7C + var_7C + rbx + var_7C, *(*var_58 + r13 + 0x20), r15);
                            rdi = *(var_88 + 0x8);
                            var_70 = intrinsic_movaps(var_70, 0x0);
                            xmm0 = intrinsic_movaps(0x0, var_70);
                            var_130 = intrinsic_movups(var_130, xmm0);
                            var_160 = intrinsic_movups(var_160, intrinsic_movups(xmm0, (var_9C >> rcx) + 0x0 + CARRY(RFLAGS(cf))));
                            (*_objc_msgSend)(rdi, var_A8);
                            memcpy(var_98 + var_7C + var_7C + var_7C + var_7C + rbx + var_7C, *(*var_58 + r13 + 0x28), r15);
                            rdi = *(var_88 + 0x8);
                            r14 = var_50;
                            var_70 = intrinsic_movaps(var_70, 0x0);
                            xmm0 = intrinsic_movaps(0x0, var_70);
                            var_130 = intrinsic_movups(var_130, xmm0);
                            var_160 = intrinsic_movups(var_160, intrinsic_movups(xmm0, (var_9C >> rcx) + 0x0 + CARRY(RFLAGS(cf))));
                            (*_objc_msgSend)(rdi, var_A8);
                            rbx = var_7C + var_7C + var_7C + var_7C + var_7C + rbx + var_7C;
                            rcx = rcx + 0x1;
                            r13 = r13 + 0x30;
                    } while (rcx < *(int32_t *)(var_58 + 0x8));
            }
            var_E8 = 0x0;
            rdi = *var_88;
            var_110 = *__NSConcreteStackBlock;
            [rdi addCompletedHandler:&var_110];
            _Block_object_dispose(&var_E8, 0x8);
            r13 = var_90;
    }
    rax = r13;
    return rax;
}


int sub_10195b8b0(int arg0, int arg1, int arg2, int arg3) {
    rcx = arg3;
    rdx = arg2;
    rsi = arg1;
    if (rsi > 0xc) {
            r13 = 0x0;
    }
    else {
            r13 = sub_10193f730(rsi);
            rax = operator new(0x28, rsi, rdx, rcx);
            rbx = rax;
            CreateAndConfigureMetalTexture(rax, 0x0, 0x1, r13 & 0xff, 0x1, rdx, rcx, 0x1, 0x1);
            rax = operator new(0x28, 0x0, 0x1, r13 & 0xff);
            r13 = rax;
            sub_10193f6c0(rax, rsi, rdx, rcx, 0x4, rdx);
            *r13 = 0x1023a4e38;
            *(r13 + 0x20) = rbx;
            *(int32_t *)(rbx + 0x14) = *(int32_t *)(rbx + 0x14) + 0x1;
    }
    rax = r13;
    return rax;
}

这是所有用到 CreateAndConfigureMetalTexture 的函数，哪一个更可能是程序生成的 ramp 作为 sampler呢