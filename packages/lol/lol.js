/* @ts-self-types="./lol.d.ts" */
import * as import1 from "./snippets/basisu_c_sys-273c0aeb51b100af/wasm/basisu_transcoder.js"


/**
 * Format that each sample has. Usually, this corresponds to the sampling
 * depth of the audio source. For example, 16 bit quantized samples can be
 * encoded in `i16` or `u16`. Note that the quantized sampling depth is not
 * directly visible for formats where [`is_float`] is true.
 *
 * Also note that the backend must support the encoding of the quantized
 * samples in the given format, as there is no generic transformation from one
 * format into the other done inside the frontend-library code. You can query
 * the supported formats by using [`supported_input_configs`].
 *
 * A good rule of thumb is to use [`SampleFormat::I16`] as this covers typical
 * music (WAV, MP3) as well as typical audio input devices on most platforms,
 *
 * [`is_float`]: SampleFormat::is_float
 * [`supported_input_configs`]: crate::traits::DeviceTrait::supported_input_configs
 * @enum {0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14}
 */
export const SampleFormat = Object.freeze({
    /**
     * `i8` with a valid range of `i8::MIN..=i8::MAX` with `0` being the origin.
     */
    I8: 0, "0": "I8",
    /**
     * `i16` with a valid range of `i16::MIN..=i16::MAX` with `0` being the origin.
     */
    I16: 1, "1": "I16",
    /**
     * `I24` with a valid range of `-(1 << 23)..=((1 << 23) - 1)` with `0` being the origin.
     *
     * This format uses 4 bytes of storage but only 24 bits are significant.
     */
    I24: 2, "2": "I24",
    /**
     * `i32` with a valid range of `i32::MIN..=i32::MAX` with `0` being the origin.
     */
    I32: 3, "3": "I32",
    /**
     * `i64` with a valid range of `i64::MIN..=i64::MAX` with `0` being the origin.
     */
    I64: 4, "4": "I64",
    /**
     * `u8` with a valid range of `u8::MIN..=u8::MAX` with `1 << 7 == 128` being the origin.
     */
    U8: 5, "5": "U8",
    /**
     * `u16` with a valid range of `u16::MIN..=u16::MAX` with `1 << 15 == 32768` being the origin.
     */
    U16: 6, "6": "U16",
    /**
     * `U24` with a valid range of `0..=((1 << 24) - 1)` with `1 << 23 == 8388608` being the origin.
     *
     * This format uses 4 bytes of storage but only 24 bits are significant.
     */
    U24: 7, "7": "U24",
    /**
     * `u32` with a valid range of `u32::MIN..=u32::MAX` with `1 << 31` being the origin.
     */
    U32: 8, "8": "U32",
    /**
     * `U48` with a valid range of '0..(1 << 48)' with `1 << 47` being the origin
     * `u64` with a valid range of `u64::MIN..=u64::MAX` with `1 << 63` being the origin.
     */
    U64: 9, "9": "U64",
    /**
     * `f32` with a valid range of `-1.0..=1.0` with `0.0` being the origin.
     */
    F32: 10, "10": "F32",
    /**
     * `f64` with a valid range of `-1.0..=1.0` with `0.0` being the origin.
     */
    F64: 11, "11": "F64",
    /**
     * DSD 1-bit stream in u8 container (8 bits = 8 DSD samples) with 0x69 being the silence byte pattern.
     */
    DsdU8: 12, "12": "DsdU8",
    /**
     * DSD 1-bit stream in u16 container (16 bits = 16 DSD samples) with 0x69 being the silence byte pattern.
     */
    DsdU16: 13, "13": "DsdU16",
    /**
     * DSD 1-bit stream in u32 container (32 bits = 32 DSD samples) with 0x69 being the silence byte pattern.
     */
    DsdU32: 14, "14": "DsdU32",
});

/**
 * The set of parameters used to describe how to open a stream.
 *
 * The sample format is omitted in favour of using a sample type.
 *
 * See also [`BufferSize`] for details on buffer size behavior and latency considerations.
 */
export class StreamConfig {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StreamConfigFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_streamconfig_free(ptr, 0);
    }
    /**
     * @returns {number | undefined}
     */
    get buffer_size() {
        const ret = wasm.__wbg_get_streamconfig_buffer_size(this.__wbg_ptr);
        return ret === 0x100000001 ? undefined : ret;
    }
    /**
     * @returns {number}
     */
    get channels() {
        const ret = wasm.__wbg_get_streamconfig_channels(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get sample_rate() {
        const ret = wasm.__wbg_get_streamconfig_sample_rate(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number | null} [arg0]
     */
    set buffer_size(arg0) {
        wasm.__wbg_set_streamconfig_buffer_size(this.__wbg_ptr, isLikeNone(arg0) ? 0x100000001 : (arg0) >>> 0);
    }
    /**
     * @param {number} arg0
     */
    set channels(arg0) {
        wasm.__wbg_set_streamconfig_channels(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set sample_rate(arg0) {
        wasm.__wbg_set_streamconfig_sample_rate(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) StreamConfig.prototype[Symbol.dispose] = StreamConfig.prototype.free;
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_HEAPU8_153c54682533edcc: function(arg0) {
            const ret = arg0.HEAPU8;
            return ret;
        },
        __wbg_Window_5bac5165340af82e: function(arg0) {
            const ret = arg0.Window;
            return ret;
        },
        __wbg_Window_6faec6a6bf671114: function(arg0) {
            const ret = arg0.Window;
            return ret;
        },
        __wbg_Window_c7f91e3f80ae0a0e: function(arg0) {
            const ret = arg0.Window;
            return ret;
        },
        __wbg_WorkerGlobalScope_20a029ca55594726: function(arg0) {
            const ret = arg0.WorkerGlobalScope;
            return ret;
        },
        __wbg_WorkerGlobalScope_d0d150069210a6e8: function(arg0) {
            const ret = arg0.WorkerGlobalScope;
            return ret;
        },
        __wbg___wbindgen_boolean_get_6ea149f0a8dcc5ff: function(arg0) {
            const v = arg0;
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_ab4b34d23d6778bd: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_is_falsy_c07bb72123e65555: function(arg0) {
            const ret = !arg0;
            return ret;
        },
        __wbg___wbindgen_is_function_3baa9db1a987f47d: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_null_52ff4ec04186736f: function(arg0) {
            const ret = arg0 === null;
            return ret;
        },
        __wbg___wbindgen_is_object_63322ec0cd6ea4ef: function(arg0) {
            const val = arg0;
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_undefined_29a43b4d42920abd: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_number_get_c7f42aed0525c451: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_7ed5322991caaec5: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_6b64449b9b9ed33c: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__bt_alloc_996c13d90dbf94e7: function(arg0, arg1) {
            const ret = arg0._bt_alloc(BigInt.asUintN(64, arg1));
            return ret;
        },
        __wbg__bt_basis_compute_transcoded_image_size_in_bytes_b7f2fe186a6e8349: function(arg0, arg1, arg2, arg3) {
            const ret = arg0._bt_basis_compute_transcoded_image_size_in_bytes(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0);
            return ret;
        },
        __wbg__bt_basis_get_bytes_per_block_or_pixel_55cb380affe87d99: function(arg0, arg1) {
            const ret = arg0._bt_basis_get_bytes_per_block_or_pixel(arg1 >>> 0);
            return ret;
        },
        __wbg__bt_basis_get_transcoder_texture_format_from_basis_tex_format_2c70bc6f430a1004: function(arg0, arg1) {
            const ret = arg0._bt_basis_get_transcoder_texture_format_from_basis_tex_format(arg1 >>> 0);
            return ret;
        },
        __wbg__bt_basis_is_format_supported_c74cc682a278257e: function(arg0, arg1, arg2) {
            const ret = arg0._bt_basis_is_format_supported(arg1 >>> 0, arg2 >>> 0);
            return ret;
        },
        __wbg__bt_free_4a75f61f6d796adf: function(arg0, arg1) {
            arg0._bt_free(BigInt.asUintN(64, arg1));
        },
        __wbg__bt_init_bf4d6cc29d0f4ba7: function(arg0) {
            arg0._bt_init();
        },
        __wbg__bt_ktx2_close_5f0389f38446b6d9: function(arg0, arg1) {
            arg0._bt_ktx2_close(BigInt.asUintN(64, arg1));
        },
        __wbg__bt_ktx2_get_basis_tex_format_ab69de6ee998f96d: function(arg0, arg1) {
            const ret = arg0._bt_ktx2_get_basis_tex_format(BigInt.asUintN(64, arg1));
            return ret;
        },
        __wbg__bt_ktx2_get_dfd_channel_id0_5dd1372ddb8cef7d: function(arg0, arg1) {
            const ret = arg0._bt_ktx2_get_dfd_channel_id0(BigInt.asUintN(64, arg1));
            return ret;
        },
        __wbg__bt_ktx2_get_dfd_channel_id1_a682b3ee1fed94cd: function(arg0, arg1) {
            const ret = arg0._bt_ktx2_get_dfd_channel_id1(BigInt.asUintN(64, arg1));
            return ret;
        },
        __wbg__bt_ktx2_get_faces_c423051cb7beb87a: function(arg0, arg1) {
            const ret = arg0._bt_ktx2_get_faces(BigInt.asUintN(64, arg1));
            return ret;
        },
        __wbg__bt_ktx2_get_height_9e89d57dd5609461: function(arg0, arg1) {
            const ret = arg0._bt_ktx2_get_height(BigInt.asUintN(64, arg1));
            return ret;
        },
        __wbg__bt_ktx2_get_layers_94a48ed62481e108: function(arg0, arg1) {
            const ret = arg0._bt_ktx2_get_layers(BigInt.asUintN(64, arg1));
            return ret;
        },
        __wbg__bt_ktx2_get_level_orig_height_3001a458ae0eb144: function(arg0, arg1, arg2, arg3, arg4) {
            const ret = arg0._bt_ktx2_get_level_orig_height(BigInt.asUintN(64, arg1), arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
            return ret;
        },
        __wbg__bt_ktx2_get_level_orig_width_6923a3c13180ccdd: function(arg0, arg1, arg2, arg3, arg4) {
            const ret = arg0._bt_ktx2_get_level_orig_width(BigInt.asUintN(64, arg1), arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
            return ret;
        },
        __wbg__bt_ktx2_get_levels_f54e8014ec4aac29: function(arg0, arg1) {
            const ret = arg0._bt_ktx2_get_levels(BigInt.asUintN(64, arg1));
            return ret;
        },
        __wbg__bt_ktx2_get_width_95780eb40045edd9: function(arg0, arg1) {
            const ret = arg0._bt_ktx2_get_width(BigInt.asUintN(64, arg1));
            return ret;
        },
        __wbg__bt_ktx2_is_srgb_068d265d611c5a3d: function(arg0, arg1) {
            const ret = arg0._bt_ktx2_is_srgb(BigInt.asUintN(64, arg1));
            return ret;
        },
        __wbg__bt_ktx2_is_uastc_ldr_4x4_eb6c97921bdb1e31: function(arg0, arg1) {
            const ret = arg0._bt_ktx2_is_uastc_ldr_4x4(BigInt.asUintN(64, arg1));
            return ret;
        },
        __wbg__bt_ktx2_open_4921a842ef17b5bd: function(arg0, arg1, arg2) {
            const ret = arg0._bt_ktx2_open(BigInt.asUintN(64, arg1), arg2 >>> 0);
            return ret;
        },
        __wbg__bt_ktx2_start_transcoding_1299f54a65ef35ca: function(arg0, arg1) {
            const ret = arg0._bt_ktx2_start_transcoding(BigInt.asUintN(64, arg1));
            return ret;
        },
        __wbg__bt_ktx2_transcode_image_level_5b2d35b01da6e6cc: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11, arg12, arg13) {
            const ret = arg0._bt_ktx2_transcode_image_level(BigInt.asUintN(64, arg1), arg2 >>> 0, arg3 >>> 0, arg4 >>> 0, BigInt.asUintN(64, arg5), arg6 >>> 0, arg7 >>> 0, arg8 >>> 0, arg9 >>> 0, arg10 >>> 0, arg11, arg12, BigInt.asUintN(64, arg13));
            return ret;
        },
        __wbg__wbg_cb_unref_b46c9b5a9f08ec37: function(arg0) {
            arg0._wbg_cb_unref();
        },
        __wbg_abort_4ce5b484434ef6fd: function(arg0) {
            arg0.abort();
        },
        __wbg_activeElement_737cd2e5ce862ac0: function(arg0) {
            const ret = arg0.activeElement;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_activeTexture_3df5a43f55a69a6c: function(arg0, arg1) {
            arg0.activeTexture(arg1 >>> 0);
        },
        __wbg_activeTexture_546afc38eb98df71: function(arg0, arg1) {
            arg0.activeTexture(arg1 >>> 0);
        },
        __wbg_addEventListener_8176dab41b09531c: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_addListener_7202e355ec2df79d: function() { return handleError(function (arg0, arg1) {
            arg0.addListener(arg1);
        }, arguments); },
        __wbg_altKey_3116112ec764f316: function(arg0) {
            const ret = arg0.altKey;
            return ret;
        },
        __wbg_altKey_c4f26b40f1b826b4: function(arg0) {
            const ret = arg0.altKey;
            return ret;
        },
        __wbg_animate_8f41e2f47c7d04ab: function(arg0, arg1, arg2) {
            const ret = arg0.animate(arg1, arg2);
            return ret;
        },
        __wbg_appendChild_e95c8b3b936d250a: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.appendChild(arg1);
            return ret;
        }, arguments); },
        __wbg_arrayBuffer_848c392b70c67d3d: function() { return handleError(function (arg0) {
            const ret = arg0.arrayBuffer();
            return ret;
        }, arguments); },
        __wbg_attachShader_1eec3a0d2bfe6f83: function(arg0, arg1, arg2) {
            arg0.attachShader(arg1, arg2);
        },
        __wbg_attachShader_e1c4cb1f00f167df: function(arg0, arg1, arg2) {
            arg0.attachShader(arg1, arg2);
        },
        __wbg_axes_6832257759ff899c: function(arg0) {
            const ret = arg0.axes;
            return ret;
        },
        __wbg_beginComputePass_8409c5720049090e: function(arg0, arg1) {
            const ret = arg0.beginComputePass(arg1);
            return ret;
        },
        __wbg_beginQuery_330ed668ec983f20: function(arg0, arg1, arg2) {
            arg0.beginQuery(arg1 >>> 0, arg2);
        },
        __wbg_beginRenderPass_a19cc6156a7858b4: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.beginRenderPass(arg1);
            return ret;
        }, arguments); },
        __wbg_bindAttribLocation_3d20dac16f0dd5d2: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.bindAttribLocation(arg1, arg2 >>> 0, getStringFromWasm0(arg3, arg4));
        },
        __wbg_bindAttribLocation_ba0ee411099ec472: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.bindAttribLocation(arg1, arg2 >>> 0, getStringFromWasm0(arg3, arg4));
        },
        __wbg_bindBufferRange_538b702311d21a3a: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.bindBufferRange(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5);
        },
        __wbg_bindBuffer_710a611286e86fe9: function(arg0, arg1, arg2) {
            arg0.bindBuffer(arg1 >>> 0, arg2);
        },
        __wbg_bindBuffer_b193f35215c88d5d: function(arg0, arg1, arg2) {
            arg0.bindBuffer(arg1 >>> 0, arg2);
        },
        __wbg_bindFramebuffer_8d7b9da43a5c1c2b: function(arg0, arg1, arg2) {
            arg0.bindFramebuffer(arg1 >>> 0, arg2);
        },
        __wbg_bindFramebuffer_fab857ccf69f3da9: function(arg0, arg1, arg2) {
            arg0.bindFramebuffer(arg1 >>> 0, arg2);
        },
        __wbg_bindRenderbuffer_21cf597fc94be1c7: function(arg0, arg1, arg2) {
            arg0.bindRenderbuffer(arg1 >>> 0, arg2);
        },
        __wbg_bindRenderbuffer_30a3bb44ea0e058f: function(arg0, arg1, arg2) {
            arg0.bindRenderbuffer(arg1 >>> 0, arg2);
        },
        __wbg_bindSampler_2179eb28a43d1075: function(arg0, arg1, arg2) {
            arg0.bindSampler(arg1 >>> 0, arg2);
        },
        __wbg_bindTexture_a87fb41b3319bcb9: function(arg0, arg1, arg2) {
            arg0.bindTexture(arg1 >>> 0, arg2);
        },
        __wbg_bindTexture_c3fcb7dc0c448083: function(arg0, arg1, arg2) {
            arg0.bindTexture(arg1 >>> 0, arg2);
        },
        __wbg_bindVertexArrayOES_b0e8a5a6c8a88c84: function(arg0, arg1) {
            arg0.bindVertexArrayOES(arg1);
        },
        __wbg_bindVertexArray_ea785b5f2238eb93: function(arg0, arg1) {
            arg0.bindVertexArray(arg1);
        },
        __wbg_blendColor_ca4a431a958a4984: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.blendColor(arg1, arg2, arg3, arg4);
        },
        __wbg_blendColor_fd6d1cb4a2be4efd: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.blendColor(arg1, arg2, arg3, arg4);
        },
        __wbg_blendEquationSeparate_1dedaa54091b78a5: function(arg0, arg1, arg2) {
            arg0.blendEquationSeparate(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_blendEquationSeparate_8a6f5cdd3d6af806: function(arg0, arg1, arg2) {
            arg0.blendEquationSeparate(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_blendEquation_0abbff18abcf6c63: function(arg0, arg1) {
            arg0.blendEquation(arg1 >>> 0);
        },
        __wbg_blendEquation_f0b8a2ea6cfe3a8a: function(arg0, arg1) {
            arg0.blendEquation(arg1 >>> 0);
        },
        __wbg_blendFuncSeparate_a1f8e0d6a1fa6fa6: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.blendFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_blendFuncSeparate_d3b4bffd37fd37de: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.blendFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_blendFunc_73401f287153631f: function(arg0, arg1, arg2) {
            arg0.blendFunc(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_blendFunc_9c1ee0744b7da386: function(arg0, arg1, arg2) {
            arg0.blendFunc(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_blitFramebuffer_796256485239eebc: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
            arg0.blitFramebuffer(arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0);
        },
        __wbg_blockSize_9bfce6be11544dd1: function(arg0) {
            const ret = arg0.blockSize;
            return ret;
        },
        __wbg_body_c7b35a55457167ba: function(arg0) {
            const ret = arg0.body;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_brand_3bc196a43eceb8af: function(arg0, arg1) {
            const ret = arg1.brand;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_brands_b7dcf262485c3e7c: function(arg0) {
            const ret = arg0.brands;
            return ret;
        },
        __wbg_bufferData_5788346a959129ab: function(arg0, arg1, arg2, arg3) {
            arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
        },
        __wbg_bufferData_6669d1a205932a9c: function(arg0, arg1, arg2, arg3) {
            arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
        },
        __wbg_bufferData_f267cdc80efbd6a0: function(arg0, arg1, arg2, arg3) {
            arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
        },
        __wbg_bufferData_f401229c915b8028: function(arg0, arg1, arg2, arg3) {
            arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
        },
        __wbg_bufferSubData_3708c0445a03981a: function(arg0, arg1, arg2, arg3) {
            arg0.bufferSubData(arg1 >>> 0, arg2, arg3);
        },
        __wbg_bufferSubData_ade66d88865db9fc: function(arg0, arg1, arg2, arg3) {
            arg0.bufferSubData(arg1 >>> 0, arg2, arg3);
        },
        __wbg_button_c794bf4b1dcd7c4c: function(arg0) {
            const ret = arg0.button;
            return ret;
        },
        __wbg_buttons_1f20131afa235956: function(arg0) {
            const ret = arg0.buttons;
            return ret;
        },
        __wbg_buttons_9b45c5f89c8d91db: function(arg0) {
            const ret = arg0.buttons;
            return ret;
        },
        __wbg_call_a24592a6f349a97e: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_cancelAnimationFrame_3fe3db137219c343: function() { return handleError(function (arg0, arg1) {
            arg0.cancelAnimationFrame(arg1);
        }, arguments); },
        __wbg_cancelIdleCallback_cc76338bb44d0b0a: function(arg0, arg1) {
            arg0.cancelIdleCallback(arg1 >>> 0);
        },
        __wbg_cancel_65f38182e2eeac5c: function(arg0) {
            arg0.cancel();
        },
        __wbg_catch_e9362815fd0b24cf: function(arg0, arg1) {
            const ret = arg0.catch(arg1);
            return ret;
        },
        __wbg_clearBuffer_45f598320c588982: function(arg0, arg1, arg2, arg3) {
            arg0.clearBuffer(arg1, arg2, arg3);
        },
        __wbg_clearBuffer_c43149005da4f328: function(arg0, arg1, arg2) {
            arg0.clearBuffer(arg1, arg2);
        },
        __wbg_clearBufferfv_715310a7cc30715d: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.clearBufferfv(arg1 >>> 0, arg2, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_clearBufferiv_a086fd83c8c5759c: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.clearBufferiv(arg1 >>> 0, arg2, getArrayI32FromWasm0(arg3, arg4));
        },
        __wbg_clearBufferuiv_8260c12b38743ac5: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.clearBufferuiv(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4));
        },
        __wbg_clearDepth_6a75b6bfa763b04c: function(arg0, arg1) {
            arg0.clearDepth(arg1);
        },
        __wbg_clearDepth_e602528ddc745c62: function(arg0, arg1) {
            arg0.clearDepth(arg1);
        },
        __wbg_clearStencil_584a4c6144f1164d: function(arg0, arg1) {
            arg0.clearStencil(arg1);
        },
        __wbg_clearStencil_8a4463aa6ab4f980: function(arg0, arg1) {
            arg0.clearStencil(arg1);
        },
        __wbg_clearTimeout_1a62f3563b1611b3: function(arg0, arg1) {
            arg0.clearTimeout(arg1);
        },
        __wbg_clear_d82c0c485d1af30e: function(arg0, arg1) {
            arg0.clear(arg1 >>> 0);
        },
        __wbg_clear_e39cde04b063e709: function(arg0, arg1) {
            arg0.clear(arg1 >>> 0);
        },
        __wbg_clientWaitSync_ce22c6bcd7b1c355: function(arg0, arg1, arg2, arg3) {
            const ret = arg0.clientWaitSync(arg1, arg2 >>> 0, arg3 >>> 0);
            return ret;
        },
        __wbg_clipboard_a08ffae077ba7949: function(arg0) {
            const ret = arg0.clipboard;
            return ret;
        },
        __wbg_close_7e700111d27bdd8c: function(arg0) {
            arg0.close();
        },
        __wbg_close_cc91f5b0a1697e4f: function() { return handleError(function (arg0) {
            const ret = arg0.close();
            return ret;
        }, arguments); },
        __wbg_code_09d0c59f9029dd28: function(arg0, arg1) {
            const ret = arg1.code;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_colorMask_5e1ce60e460bf963: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.colorMask(arg1 !== 0, arg2 !== 0, arg3 !== 0, arg4 !== 0);
        },
        __wbg_colorMask_71190391f59922fe: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.colorMask(arg1 !== 0, arg2 !== 0, arg3 !== 0, arg4 !== 0);
        },
        __wbg_compileShader_b39b7d5caca97c9d: function(arg0, arg1) {
            arg0.compileShader(arg1);
        },
        __wbg_compileShader_fc084de511370bc0: function(arg0, arg1) {
            arg0.compileShader(arg1);
        },
        __wbg_compressedTexSubImage2D_846de8fb28dec7a9: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8, arg9);
        },
        __wbg_compressedTexSubImage2D_8b3ae69927d595ce: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
            arg0.compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8);
        },
        __wbg_compressedTexSubImage2D_b2126dd76e4f45d9: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
            arg0.compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8);
        },
        __wbg_compressedTexSubImage3D_08741720ce7a1a6b: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
            arg0.compressedTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10);
        },
        __wbg_compressedTexSubImage3D_6d41d588f1b1fa8a: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.compressedTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10, arg11);
        },
        __wbg_configure_16541864db644c70: function() { return handleError(function (arg0, arg1) {
            arg0.configure(arg1);
        }, arguments); },
        __wbg_connect_301bfaee317657e7: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.connect(arg1);
            return ret;
        }, arguments); },
        __wbg_connected_86aef276927b774b: function(arg0) {
            const ret = arg0.connected;
            return ret;
        },
        __wbg_contains_495334b455843d23: function(arg0, arg1) {
            const ret = arg0.contains(arg1);
            return ret;
        },
        __wbg_contentRect_e3958925fadb3298: function(arg0) {
            const ret = arg0.contentRect;
            return ret;
        },
        __wbg_copyBufferSubData_8646c023657c626d: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.copyBufferSubData(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5);
        },
        __wbg_copyBufferToBuffer_ab55c9b7610f063b: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.copyBufferToBuffer(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_copyBufferToBuffer_b9173642596dac6a: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.copyBufferToBuffer(arg1, arg2, arg3, arg4);
        }, arguments); },
        __wbg_copyExternalImageToTexture_6d56ad685a99824d: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.copyExternalImageToTexture(arg1, arg2, arg3);
        }, arguments); },
        __wbg_copyTexSubImage2D_30dff1abdab02706: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
            arg0.copyTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
        },
        __wbg_copyTexSubImage2D_a2aa1d558b98374c: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
            arg0.copyTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
        },
        __wbg_copyTexSubImage3D_182ddcace125b399: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.copyTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9);
        },
        __wbg_copyTextureToBuffer_c6a46adf0738476b: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.copyTextureToBuffer(arg1, arg2, arg3);
        }, arguments); },
        __wbg_copyTextureToTexture_9e4d8e43703037c7: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.copyTextureToTexture(arg1, arg2, arg3);
        }, arguments); },
        __wbg_copyToChannel_346ff43ca43e57d1: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.copyToChannel(getArrayF32FromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_createBindGroupLayout_adb8337a6808ae24: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.createBindGroupLayout(arg1);
            return ret;
        }, arguments); },
        __wbg_createBindGroup_91159ca759115307: function(arg0, arg1) {
            const ret = arg0.createBindGroup(arg1);
            return ret;
        },
        __wbg_createBufferSource_08e05c18c42b3726: function() { return handleError(function (arg0) {
            const ret = arg0.createBufferSource();
            return ret;
        }, arguments); },
        __wbg_createBuffer_59de141e89014140: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.createBuffer(arg1);
            return ret;
        }, arguments); },
        __wbg_createBuffer_6ad9886c8fed1a21: function(arg0) {
            const ret = arg0.createBuffer();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createBuffer_f68202a47c36c3d6: function(arg0) {
            const ret = arg0.createBuffer();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createBuffer_fad6af7eb9950da0: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg0.createBuffer(arg1 >>> 0, arg2 >>> 0, arg3);
            return ret;
        }, arguments); },
        __wbg_createCommandEncoder_dc2b2ca6f09bd4c3: function(arg0, arg1) {
            const ret = arg0.createCommandEncoder(arg1);
            return ret;
        },
        __wbg_createComputePipeline_d9b5cceb5f793856: function(arg0, arg1) {
            const ret = arg0.createComputePipeline(arg1);
            return ret;
        },
        __wbg_createElement_bbd4c90086fe6f7b: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_createFramebuffer_03fa5aab12587b89: function(arg0) {
            const ret = arg0.createFramebuffer();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createFramebuffer_211e9c2acecac22f: function(arg0) {
            const ret = arg0.createFramebuffer();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createImageBitmap_d68820ff1d8f4591: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.createImageBitmap(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_createObjectURL_46e1b0c55389893b: function() { return handleError(function (arg0, arg1) {
            const ret = URL.createObjectURL(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_createPipelineLayout_a5290f84492f8b1e: function(arg0, arg1) {
            const ret = arg0.createPipelineLayout(arg1);
            return ret;
        },
        __wbg_createProgram_635f6f85c5f3c83d: function(arg0) {
            const ret = arg0.createProgram();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createProgram_bedc70c0d16e41df: function(arg0) {
            const ret = arg0.createProgram();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createQuery_276833144fe200cc: function(arg0) {
            const ret = arg0.createQuery();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createRenderPipeline_f7aca470ad8ce865: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.createRenderPipeline(arg1);
            return ret;
        }, arguments); },
        __wbg_createRenderbuffer_35e6bde126d973f9: function(arg0) {
            const ret = arg0.createRenderbuffer();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createRenderbuffer_cb16d578fe2964c3: function(arg0) {
            const ret = arg0.createRenderbuffer();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createSampler_6b972cd00bcc5dfb: function(arg0, arg1) {
            const ret = arg0.createSampler(arg1);
            return ret;
        },
        __wbg_createSampler_7a0e4bf12aad2139: function(arg0) {
            const ret = arg0.createSampler();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createShaderModule_bbe0476992dd060e: function(arg0, arg1) {
            const ret = arg0.createShaderModule(arg1);
            return ret;
        },
        __wbg_createShader_2c8d8c9f17967efe: function(arg0, arg1) {
            const ret = arg0.createShader(arg1 >>> 0);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createShader_5484e429d7514a9d: function(arg0, arg1) {
            const ret = arg0.createShader(arg1 >>> 0);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createTexture_011d4b0badf853e3: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.createTexture(arg1);
            return ret;
        }, arguments); },
        __wbg_createTexture_caeb4349ae5c7a83: function(arg0) {
            const ret = arg0.createTexture();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createTexture_f9850d55f04c7883: function(arg0) {
            const ret = arg0.createTexture();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createVertexArrayOES_25823ca742b59551: function(arg0) {
            const ret = arg0.createVertexArrayOES();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createVertexArray_a8c3e6799bdb5af8: function(arg0) {
            const ret = arg0.createVertexArray();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createView_1ef8f1ddc16facb0: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.createView(arg1);
            return ret;
        }, arguments); },
        __wbg_ctrlKey_31968cccd46bdef6: function(arg0) {
            const ret = arg0.ctrlKey;
            return ret;
        },
        __wbg_ctrlKey_a49693667722b909: function(arg0) {
            const ret = arg0.ctrlKey;
            return ret;
        },
        __wbg_cullFace_87cf8b47e8d3edd2: function(arg0, arg1) {
            arg0.cullFace(arg1 >>> 0);
        },
        __wbg_cullFace_c35bb54d07e68290: function(arg0, arg1) {
            arg0.cullFace(arg1 >>> 0);
        },
        __wbg_currentTime_8e9bfa251075a7d7: function(arg0) {
            const ret = arg0.currentTime;
            return ret;
        },
        __wbg_decode_4120b4ae5587bf11: function(arg0) {
            const ret = arg0.decode();
            return ret;
        },
        __wbg_deleteBuffer_521c77539f9941c1: function(arg0, arg1) {
            arg0.deleteBuffer(arg1);
        },
        __wbg_deleteBuffer_558c85bd550b15df: function(arg0, arg1) {
            arg0.deleteBuffer(arg1);
        },
        __wbg_deleteFramebuffer_2fb61c893f34a853: function(arg0, arg1) {
            arg0.deleteFramebuffer(arg1);
        },
        __wbg_deleteFramebuffer_a0b3386cb631ca92: function(arg0, arg1) {
            arg0.deleteFramebuffer(arg1);
        },
        __wbg_deleteProgram_6d3a2bdf7fc6d658: function(arg0, arg1) {
            arg0.deleteProgram(arg1);
        },
        __wbg_deleteProgram_8175823e816f19ed: function(arg0, arg1) {
            arg0.deleteProgram(arg1);
        },
        __wbg_deleteQuery_774f28829e3eb161: function(arg0, arg1) {
            arg0.deleteQuery(arg1);
        },
        __wbg_deleteRenderbuffer_826cb5d271ce4663: function(arg0, arg1) {
            arg0.deleteRenderbuffer(arg1);
        },
        __wbg_deleteRenderbuffer_fe0ab39c4929254b: function(arg0, arg1) {
            arg0.deleteRenderbuffer(arg1);
        },
        __wbg_deleteSampler_34704dd7176c6cb6: function(arg0, arg1) {
            arg0.deleteSampler(arg1);
        },
        __wbg_deleteShader_379785984071d8af: function(arg0, arg1) {
            arg0.deleteShader(arg1);
        },
        __wbg_deleteShader_460e3d0b80ea3790: function(arg0, arg1) {
            arg0.deleteShader(arg1);
        },
        __wbg_deleteSync_bdc4a0bdb747530d: function(arg0, arg1) {
            arg0.deleteSync(arg1);
        },
        __wbg_deleteTexture_6de16581bf7e5e00: function(arg0, arg1) {
            arg0.deleteTexture(arg1);
        },
        __wbg_deleteTexture_8714aac647598458: function(arg0, arg1) {
            arg0.deleteTexture(arg1);
        },
        __wbg_deleteVertexArrayOES_6bac63f2a6cf4257: function(arg0, arg1) {
            arg0.deleteVertexArrayOES(arg1);
        },
        __wbg_deleteVertexArray_c0c2dbbda37e677b: function(arg0, arg1) {
            arg0.deleteVertexArray(arg1);
        },
        __wbg_deltaMode_e3330902f10b9218: function(arg0) {
            const ret = arg0.deltaMode;
            return ret;
        },
        __wbg_deltaX_7f421a85054bc57c: function(arg0) {
            const ret = arg0.deltaX;
            return ret;
        },
        __wbg_deltaY_ca7744a8772482e1: function(arg0) {
            const ret = arg0.deltaY;
            return ret;
        },
        __wbg_depthFunc_2dcf4f6cd1ae352f: function(arg0, arg1) {
            arg0.depthFunc(arg1 >>> 0);
        },
        __wbg_depthFunc_477c738f0d31fb27: function(arg0, arg1) {
            arg0.depthFunc(arg1 >>> 0);
        },
        __wbg_depthMask_0212eafbadf5c510: function(arg0, arg1) {
            arg0.depthMask(arg1 !== 0);
        },
        __wbg_depthMask_79ce0d02cd6be571: function(arg0, arg1) {
            arg0.depthMask(arg1 !== 0);
        },
        __wbg_depthRange_9e176d2eecf5817d: function(arg0, arg1, arg2) {
            arg0.depthRange(arg1, arg2);
        },
        __wbg_depthRange_aa0a658c759230f3: function(arg0, arg1, arg2) {
            arg0.depthRange(arg1, arg2);
        },
        __wbg_description_972ee565dde8fe3f: function(arg0, arg1) {
            const ret = arg1.description;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_destination_7aa167ec1225162d: function(arg0) {
            const ret = arg0.destination;
            return ret;
        },
        __wbg_destroy_479a1ccb4eb28cd7: function(arg0) {
            arg0.destroy();
        },
        __wbg_devicePixelContentBoxSize_c1a8da18615df561: function(arg0) {
            const ret = arg0.devicePixelContentBoxSize;
            return ret;
        },
        __wbg_devicePixelRatio_18e6533e6d7f4088: function(arg0) {
            const ret = arg0.devicePixelRatio;
            return ret;
        },
        __wbg_disableVertexAttribArray_c56221197975648d: function(arg0, arg1) {
            arg0.disableVertexAttribArray(arg1 >>> 0);
        },
        __wbg_disableVertexAttribArray_dbf84d5ba8f67bad: function(arg0, arg1) {
            arg0.disableVertexAttribArray(arg1 >>> 0);
        },
        __wbg_disable_c83e7f9d8a8660e6: function(arg0, arg1) {
            arg0.disable(arg1 >>> 0);
        },
        __wbg_disable_d115c77f70b6b789: function(arg0, arg1) {
            arg0.disable(arg1 >>> 0);
        },
        __wbg_disconnect_b688a8dfdd1f8a2e: function(arg0) {
            arg0.disconnect();
        },
        __wbg_disconnect_d173374266b80cfa: function(arg0) {
            arg0.disconnect();
        },
        __wbg_dispatchWorkgroupsIndirect_c588855da1b3b4ea: function(arg0, arg1, arg2) {
            arg0.dispatchWorkgroupsIndirect(arg1, arg2);
        },
        __wbg_dispatchWorkgroups_863a24e86337dc58: function(arg0, arg1, arg2, arg3) {
            arg0.dispatchWorkgroups(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0);
        },
        __wbg_document_7a41071f2f439323: function(arg0) {
            const ret = arg0.document;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_drawArraysInstancedANGLE_f6a18e87c8a056c3: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.drawArraysInstancedANGLE(arg1 >>> 0, arg2, arg3, arg4);
        },
        __wbg_drawArraysInstanced_43319f9817fad285: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.drawArraysInstanced(arg1 >>> 0, arg2, arg3, arg4);
        },
        __wbg_drawArrays_058a7d3434327b6d: function(arg0, arg1, arg2, arg3) {
            arg0.drawArrays(arg1 >>> 0, arg2, arg3);
        },
        __wbg_drawArrays_0b6495544ecb3b5e: function(arg0, arg1, arg2, arg3) {
            arg0.drawArrays(arg1 >>> 0, arg2, arg3);
        },
        __wbg_drawBuffersWEBGL_674a96484245cee8: function(arg0, arg1) {
            arg0.drawBuffersWEBGL(arg1);
        },
        __wbg_drawBuffers_0808a2009fb32b11: function(arg0, arg1) {
            arg0.drawBuffers(arg1);
        },
        __wbg_drawElementsInstancedANGLE_01b7fe3dcfda1f57: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.drawElementsInstancedANGLE(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
        },
        __wbg_drawElementsInstanced_9cdd75777f6fe52e: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.drawElementsInstanced(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
        },
        __wbg_drawIndexedIndirect_99c3a8847840c757: function(arg0, arg1, arg2) {
            arg0.drawIndexedIndirect(arg1, arg2);
        },
        __wbg_drawIndexed_c5e4a5b9b73cf1a9: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
        },
        __wbg_drawIndirect_b633c26f6078437e: function(arg0, arg1, arg2) {
            arg0.drawIndirect(arg1, arg2);
        },
        __wbg_draw_9a35daa0096c6f2c: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_enableVertexAttribArray_44d2f9d5bd7d4773: function(arg0, arg1) {
            arg0.enableVertexAttribArray(arg1 >>> 0);
        },
        __wbg_enableVertexAttribArray_a6fb4500c619f67f: function(arg0, arg1) {
            arg0.enableVertexAttribArray(arg1 >>> 0);
        },
        __wbg_enable_aafffd647725f82c: function(arg0, arg1) {
            arg0.enable(arg1 >>> 0);
        },
        __wbg_enable_e9e223bf04c318ac: function(arg0, arg1) {
            arg0.enable(arg1 >>> 0);
        },
        __wbg_endQuery_35c4e07c06fe3d01: function(arg0, arg1) {
            arg0.endQuery(arg1 >>> 0);
        },
        __wbg_end_176171edf82ded94: function(arg0) {
            arg0.end();
        },
        __wbg_end_1db12af2e0ff1235: function(arg0) {
            arg0.end();
        },
        __wbg_error_2001591ad2463697: function(arg0) {
            console.error(arg0);
        },
        __wbg_error_a6fa202b58aa1cd3: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_error_c9510133954a995f: function(arg0) {
            const ret = arg0.error;
            return ret;
        },
        __wbg_error_f536c7930d1c5c8d: function(arg0, arg1) {
            console.error(arg0, arg1);
        },
        __wbg_eval_0f5002e126d86aff: function() { return handleError(function (arg0, arg1) {
            const ret = eval(getStringFromWasm0(arg0, arg1));
            return ret;
        }, arguments); },
        __wbg_exec_819aa537d4f2cfc2: function(arg0, arg1, arg2) {
            const ret = arg0.exec(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_exitFullscreen_a1251cd38cfea434: function(arg0) {
            arg0.exitFullscreen();
        },
        __wbg_exitPointerLock_60ba0cb435ab9b15: function(arg0) {
            arg0.exitPointerLock();
        },
        __wbg_features_67a150044f55609a: function(arg0) {
            const ret = arg0.features;
            return ret;
        },
        __wbg_features_8db042ef0fb9a3f9: function(arg0) {
            const ret = arg0.features;
            return ret;
        },
        __wbg_fenceSync_a6b717c8aa19605f: function(arg0, arg1, arg2) {
            const ret = arg0.fenceSync(arg1 >>> 0, arg2 >>> 0);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_fetch_5ccc4e4f205384ba: function(arg0, arg1, arg2) {
            const ret = arg0.fetch(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_fetch_bc4036ca04c5b635: function(arg0, arg1, arg2) {
            const ret = arg0.fetch(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_finish_48a7b6da7b76999e: function(arg0) {
            const ret = arg0.finish();
            return ret;
        },
        __wbg_finish_68d7c5925d3fa394: function(arg0, arg1) {
            const ret = arg0.finish(arg1);
            return ret;
        },
        __wbg_flush_83c9379ecf842793: function(arg0) {
            arg0.flush();
        },
        __wbg_flush_919dd5bcf0622389: function(arg0) {
            arg0.flush();
        },
        __wbg_focus_089295847acbfa20: function() { return handleError(function (arg0) {
            arg0.focus();
        }, arguments); },
        __wbg_framebufferRenderbuffer_45ca809f8ae492b2: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.framebufferRenderbuffer(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4);
        },
        __wbg_framebufferRenderbuffer_fb45867659c40998: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.framebufferRenderbuffer(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4);
        },
        __wbg_framebufferTexture2D_44e56e9e14542bb5: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5);
        },
        __wbg_framebufferTexture2D_f54db6e0dc9fac5e: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5);
        },
        __wbg_framebufferTextureLayer_9304ec957d83d40c: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.framebufferTextureLayer(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5);
        },
        __wbg_framebufferTextureMultiviewOVR_da9e610e7905c5d6: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.framebufferTextureMultiviewOVR(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5, arg6);
        },
        __wbg_frontFace_8355322e74d5ff2c: function(arg0, arg1) {
            arg0.frontFace(arg1 >>> 0);
        },
        __wbg_frontFace_fc39e9727cc574df: function(arg0, arg1) {
            arg0.frontFace(arg1 >>> 0);
        },
        __wbg_fullscreenElement_2eed7fc26f0751e2: function(arg0) {
            const ret = arg0.fullscreenElement;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_getBoundingClientRect_ddac06b2c6b52b98: function(arg0) {
            const ret = arg0.getBoundingClientRect();
            return ret;
        },
        __wbg_getBufferSubData_008fe53c81fd2c77: function(arg0, arg1, arg2, arg3) {
            arg0.getBufferSubData(arg1 >>> 0, arg2, arg3);
        },
        __wbg_getCoalescedEvents_08ae0f67553c536f: function(arg0) {
            const ret = arg0.getCoalescedEvents();
            return ret;
        },
        __wbg_getCoalescedEvents_3e003f63d9ebbc05: function(arg0) {
            const ret = arg0.getCoalescedEvents;
            return ret;
        },
        __wbg_getComputedStyle_a23c121719ab715c: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.getComputedStyle(arg1);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getContext_367a8d870ace1970: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2), arg3);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getContext_69ddc504535a2e7b: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getContext_7721c879aeeb69f4: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2), arg3);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getContext_fc146f8ec021d074: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getCurrentTexture_9b00da7f6bc38606: function() { return handleError(function (arg0) {
            const ret = arg0.getCurrentTexture();
            return ret;
        }, arguments); },
        __wbg_getExtension_5228364a0715c7db: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getExtension(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getGamepads_10336aefc9bdd388: function() { return handleError(function (arg0) {
            const ret = arg0.getGamepads();
            return ret;
        }, arguments); },
        __wbg_getIndexedParameter_03d7b63e08249113: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getIndexedParameter(arg1 >>> 0, arg2 >>> 0);
            return ret;
        }, arguments); },
        __wbg_getMappedRange_4a3dc3f452433b71: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getMappedRange(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_getOwnPropertyDescriptor_131bd582a45a6f5d: function(arg0, arg1) {
            const ret = Object.getOwnPropertyDescriptor(arg0, arg1);
            return ret;
        },
        __wbg_getParameter_594f21b1d26abeed: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.getParameter(arg1 >>> 0);
            return ret;
        }, arguments); },
        __wbg_getParameter_e1c6e394a2959d43: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.getParameter(arg1 >>> 0);
            return ret;
        }, arguments); },
        __wbg_getPreferredCanvasFormat_54381f1ef7aec03d: function(arg0) {
            const ret = arg0.getPreferredCanvasFormat();
            return (__wbindgen_enum_GpuTextureFormat.indexOf(ret) + 1 || 96) - 1;
        },
        __wbg_getProgramInfoLog_00af0d3e29c73293: function(arg0, arg1, arg2) {
            const ret = arg1.getProgramInfoLog(arg2);
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getProgramInfoLog_612d2724e854e752: function(arg0, arg1, arg2) {
            const ret = arg1.getProgramInfoLog(arg2);
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getProgramParameter_6aa39c38709e0d9d: function(arg0, arg1, arg2) {
            const ret = arg0.getProgramParameter(arg1, arg2 >>> 0);
            return ret;
        },
        __wbg_getProgramParameter_d18275e84d037799: function(arg0, arg1, arg2) {
            const ret = arg0.getProgramParameter(arg1, arg2 >>> 0);
            return ret;
        },
        __wbg_getPropertyValue_0bc8c6164d246228: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg1.getPropertyValue(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_getQueryParameter_9a1b5a99c953b0de: function(arg0, arg1, arg2) {
            const ret = arg0.getQueryParameter(arg1, arg2 >>> 0);
            return ret;
        },
        __wbg_getRandomValues_3f44b700395062e5: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_getRandomValues_ef12552bf5acd2fe: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_getShaderInfoLog_57fd85336a768aa9: function(arg0, arg1, arg2) {
            const ret = arg1.getShaderInfoLog(arg2);
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getShaderInfoLog_ef603aa10b52d639: function(arg0, arg1, arg2) {
            const ret = arg1.getShaderInfoLog(arg2);
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getShaderParameter_4676ea57a8db83ec: function(arg0, arg1, arg2) {
            const ret = arg0.getShaderParameter(arg1, arg2 >>> 0);
            return ret;
        },
        __wbg_getShaderParameter_f1ed538581985875: function(arg0, arg1, arg2) {
            const ret = arg0.getShaderParameter(arg1, arg2 >>> 0);
            return ret;
        },
        __wbg_getSupportedExtensions_a6b7a4d43810c644: function(arg0) {
            const ret = arg0.getSupportedExtensions();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_getSupportedProfiles_07d149b0534e6e7d: function(arg0) {
            const ret = arg0.getSupportedProfiles();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_getSyncParameter_eca2fe84d5f74227: function(arg0, arg1, arg2) {
            const ret = arg0.getSyncParameter(arg1, arg2 >>> 0);
            return ret;
        },
        __wbg_getUniformBlockIndex_79370b4799b9dd60: function(arg0, arg1, arg2, arg3) {
            const ret = arg0.getUniformBlockIndex(arg1, getStringFromWasm0(arg2, arg3));
            return ret;
        },
        __wbg_getUniformLocation_084155a4348002df: function(arg0, arg1, arg2, arg3) {
            const ret = arg0.getUniformLocation(arg1, getStringFromWasm0(arg2, arg3));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_getUniformLocation_91e9e13f695e50c5: function(arg0, arg1, arg2, arg3) {
            const ret = arg0.getUniformLocation(arg1, getStringFromWasm0(arg2, arg3));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_get_0cfbe604d86bac03: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_get_6011fa3a58f61074: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_get_8360291721e2339f: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        },
        __wbg_get_unchecked_17f53dad852b9588: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        },
        __wbg_gpu_3f9d7df9a18237f8: function(arg0) {
            const ret = arg0.gpu;
            return ret;
        },
        __wbg_has_72be553580021507: function(arg0, arg1, arg2) {
            const ret = arg0.has(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_has_880f1d472f7cecba: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.has(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_height_1bd361dd29823921: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_height_528848d067cc2221: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_height_686fe15182fb5d4e: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_height_e9bd2453b0432ed7: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_height_f8efae863e677d02: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_height_fc2f1def9f6e7730: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_id_8fa3216e70a8c29c: function(arg0, arg1) {
            const ret = arg1.id;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_includes_591176a7a8b346e9: function(arg0, arg1, arg2) {
            const ret = arg0.includes(arg1, arg2);
            return ret;
        },
        __wbg_index_6a20cd83a75cf3bb: function(arg0) {
            const ret = arg0.index;
            return ret;
        },
        __wbg_info_46732e46da34944d: function(arg0) {
            const ret = arg0.info;
            return ret;
        },
        __wbg_inlineSize_ade7bedbe596e98c: function(arg0) {
            const ret = arg0.inlineSize;
            return ret;
        },
        __wbg_instanceof_DomException_2ce86bc775566ba2: function(arg0) {
            let result;
            try {
                result = arg0 instanceof DOMException;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuAdapter_dc7e13c1676da9bd: function(arg0) {
            let result;
            try {
                result = arg0 instanceof GPUAdapter;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuCanvasContext_c2609c698a76a6b6: function(arg0) {
            let result;
            try {
                result = arg0 instanceof GPUCanvasContext;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuDeviceLostInfo_2f41e62007ddfa63: function(arg0) {
            let result;
            try {
                result = arg0 instanceof GPUDeviceLostInfo;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuOutOfMemoryError_71010a519adf2661: function(arg0) {
            let result;
            try {
                result = arg0 instanceof GPUOutOfMemoryError;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuValidationError_a2f8dbe5d7c71adf: function(arg0) {
            let result;
            try {
                result = arg0 instanceof GPUValidationError;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlCanvasElement_ea4dfc3bb77c734b: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLCanvasElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Object_7c99480a1cdfb911: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Object;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Promise_78658358a9b27cd4: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Promise;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Response_9b2d111407865ff2: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Response;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Uint8Array_152ba1f289edcf3f: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Uint8Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_WebGl2RenderingContext_23f2da2f294d4c8e: function(arg0) {
            let result;
            try {
                result = arg0 instanceof WebGL2RenderingContext;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_cc64c86c8ef9e02b: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_invalidateFramebuffer_c731d4aba0d9ae7a: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.invalidateFramebuffer(arg1 >>> 0, arg2);
        }, arguments); },
        __wbg_isArray_c3109d14ffc06469: function(arg0) {
            const ret = Array.isArray(arg0);
            return ret;
        },
        __wbg_isIntersecting_10f717a22304a79d: function(arg0) {
            const ret = arg0.isIntersecting;
            return ret;
        },
        __wbg_isSecureContext_3d130f603c477578: function(arg0) {
            const ret = arg0.isSecureContext;
            return ret;
        },
        __wbg_is_8f7ba86b7f249abd: function(arg0, arg1) {
            const ret = Object.is(arg0, arg1);
            return ret;
        },
        __wbg_key_2cbc38fa83cdb336: function(arg0, arg1) {
            const ret = arg1.key;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_label_18cae34ff19933d7: function(arg0, arg1) {
            const ret = arg1.label;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_length_3d4ecd04bd8d22f1: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_9f1775224cf1d815: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_limits_220da60782102dd7: function(arg0) {
            const ret = arg0.limits;
            return ret;
        },
        __wbg_limits_8837ca9ac1296563: function(arg0) {
            const ret = arg0.limits;
            return ret;
        },
        __wbg_linkProgram_0f095b446d393a30: function(arg0, arg1) {
            arg0.linkProgram(arg1);
        },
        __wbg_linkProgram_aa5b01ff0fcf3a80: function(arg0, arg1) {
            arg0.linkProgram(arg1);
        },
        __wbg_location_8f2306ac5789eb87: function(arg0) {
            const ret = arg0.location;
            return ret;
        },
        __wbg_log_0c201ade58bb55e1: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), getStringFromWasm0(arg6, arg7));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_log_ce2c4456b290c5e7: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_lost_75246ae62f6970bc: function(arg0) {
            const ret = arg0.lost;
            return ret;
        },
        __wbg_mapAsync_288e2fddbc3f7f7b: function(arg0, arg1, arg2, arg3) {
            const ret = arg0.mapAsync(arg1 >>> 0, arg2, arg3);
            return ret;
        },
        __wbg_mapping_5ea59bfa93022a11: function(arg0) {
            const ret = arg0.mapping;
            return (__wbindgen_enum_GamepadMappingType.indexOf(ret) + 1 || 3) - 1;
        },
        __wbg_mark_b4d943f3bc2d2404: function(arg0, arg1) {
            performance.mark(getStringFromWasm0(arg0, arg1));
        },
        __wbg_matchMedia_ce9949babceac178: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.matchMedia(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_matches_60339f60d9118f30: function(arg0) {
            const ret = arg0.matches;
            return ret;
        },
        __wbg_maxBindGroups_3e48365ce9cb69b2: function(arg0) {
            const ret = arg0.maxBindGroups;
            return ret;
        },
        __wbg_maxBindingsPerBindGroup_19eab6283879be75: function(arg0) {
            const ret = arg0.maxBindingsPerBindGroup;
            return ret;
        },
        __wbg_maxBufferSize_8086300d000af7cb: function(arg0) {
            const ret = arg0.maxBufferSize;
            return ret;
        },
        __wbg_maxChannelCount_c6d456df80fd882c: function(arg0) {
            const ret = arg0.maxChannelCount;
            return ret;
        },
        __wbg_maxColorAttachmentBytesPerSample_ee822e1793bea12f: function(arg0) {
            const ret = arg0.maxColorAttachmentBytesPerSample;
            return ret;
        },
        __wbg_maxColorAttachments_3110f22e4c5e3621: function(arg0) {
            const ret = arg0.maxColorAttachments;
            return ret;
        },
        __wbg_maxComputeInvocationsPerWorkgroup_e1b61d9c74f79e81: function(arg0) {
            const ret = arg0.maxComputeInvocationsPerWorkgroup;
            return ret;
        },
        __wbg_maxComputeWorkgroupSizeX_202ebe3252c09676: function(arg0) {
            const ret = arg0.maxComputeWorkgroupSizeX;
            return ret;
        },
        __wbg_maxComputeWorkgroupSizeY_4f66f59c2daaa8f1: function(arg0) {
            const ret = arg0.maxComputeWorkgroupSizeY;
            return ret;
        },
        __wbg_maxComputeWorkgroupSizeZ_eadb1eb36902e045: function(arg0) {
            const ret = arg0.maxComputeWorkgroupSizeZ;
            return ret;
        },
        __wbg_maxComputeWorkgroupStorageSize_05e0131572ec6c1e: function(arg0) {
            const ret = arg0.maxComputeWorkgroupStorageSize;
            return ret;
        },
        __wbg_maxComputeWorkgroupsPerDimension_47cd4aa37eba4a57: function(arg0) {
            const ret = arg0.maxComputeWorkgroupsPerDimension;
            return ret;
        },
        __wbg_maxDynamicStorageBuffersPerPipelineLayout_122112462e514d25: function(arg0) {
            const ret = arg0.maxDynamicStorageBuffersPerPipelineLayout;
            return ret;
        },
        __wbg_maxDynamicUniformBuffersPerPipelineLayout_4c57dbd81a8d1c49: function(arg0) {
            const ret = arg0.maxDynamicUniformBuffersPerPipelineLayout;
            return ret;
        },
        __wbg_maxInterStageShaderVariables_5bb90c2a06f1e9ce: function(arg0) {
            const ret = arg0.maxInterStageShaderVariables;
            return ret;
        },
        __wbg_maxSampledTexturesPerShaderStage_cea16550f969bbdc: function(arg0) {
            const ret = arg0.maxSampledTexturesPerShaderStage;
            return ret;
        },
        __wbg_maxSamplersPerShaderStage_1cbd8dba92d87dd9: function(arg0) {
            const ret = arg0.maxSamplersPerShaderStage;
            return ret;
        },
        __wbg_maxStorageBufferBindingSize_ff2e77e686018944: function(arg0) {
            const ret = arg0.maxStorageBufferBindingSize;
            return ret;
        },
        __wbg_maxStorageBuffersPerShaderStage_e496ad22f8b97f12: function(arg0) {
            const ret = arg0.maxStorageBuffersPerShaderStage;
            return ret;
        },
        __wbg_maxStorageTexturesPerShaderStage_258aab0d332d9efe: function(arg0) {
            const ret = arg0.maxStorageTexturesPerShaderStage;
            return ret;
        },
        __wbg_maxTextureArrayLayers_6fffbda0cd6f3036: function(arg0) {
            const ret = arg0.maxTextureArrayLayers;
            return ret;
        },
        __wbg_maxTextureDimension1D_53d154cf8f16d439: function(arg0) {
            const ret = arg0.maxTextureDimension1D;
            return ret;
        },
        __wbg_maxTextureDimension2D_578c2c471b73bb60: function(arg0) {
            const ret = arg0.maxTextureDimension2D;
            return ret;
        },
        __wbg_maxTextureDimension3D_3532b309b08a5ddf: function(arg0) {
            const ret = arg0.maxTextureDimension3D;
            return ret;
        },
        __wbg_maxUniformBufferBindingSize_6c3b6b8424799146: function(arg0) {
            const ret = arg0.maxUniformBufferBindingSize;
            return ret;
        },
        __wbg_maxUniformBuffersPerShaderStage_911223507ba8d12a: function(arg0) {
            const ret = arg0.maxUniformBuffersPerShaderStage;
            return ret;
        },
        __wbg_maxVertexAttributes_399d9b947e980d08: function(arg0) {
            const ret = arg0.maxVertexAttributes;
            return ret;
        },
        __wbg_maxVertexBufferArrayStride_b5550ff3b3aa4a9e: function(arg0) {
            const ret = arg0.maxVertexBufferArrayStride;
            return ret;
        },
        __wbg_maxVertexBuffers_15be37c3f8fbfe0a: function(arg0) {
            const ret = arg0.maxVertexBuffers;
            return ret;
        },
        __wbg_measure_84362959e621a2c1: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            let deferred0_0;
            let deferred0_1;
            let deferred1_0;
            let deferred1_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                deferred1_0 = arg2;
                deferred1_1 = arg3;
                performance.measure(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
                wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
            }
        }, arguments); },
        __wbg_media_e755b0c3bda4816a: function(arg0, arg1) {
            const ret = arg1.media;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_message_c152a993ca3c8fa8: function(arg0, arg1) {
            const ret = arg1.message;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_message_e59c1a10ce6fce88: function(arg0, arg1) {
            const ret = arg1.message;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_message_ec476bcf269dd7c4: function(arg0, arg1) {
            const ret = arg1.message;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_metaKey_665498d01ebfd062: function(arg0) {
            const ret = arg0.metaKey;
            return ret;
        },
        __wbg_metaKey_f8f3c1d2a5b88850: function(arg0) {
            const ret = arg0.metaKey;
            return ret;
        },
        __wbg_minStorageBufferOffsetAlignment_5c389200e0be5fe1: function(arg0) {
            const ret = arg0.minStorageBufferOffsetAlignment;
            return ret;
        },
        __wbg_minUniformBufferOffsetAlignment_b9d974e659cd3e20: function(arg0) {
            const ret = arg0.minUniformBufferOffsetAlignment;
            return ret;
        },
        __wbg_movementX_cacb769bb4f92fb5: function(arg0) {
            const ret = arg0.movementX;
            return ret;
        },
        __wbg_movementY_6a643f8b43d5a15b: function(arg0) {
            const ret = arg0.movementY;
            return ret;
        },
        __wbg_navigator_353318de944ca7f6: function(arg0) {
            const ret = arg0.navigator;
            return ret;
        },
        __wbg_navigator_bc077756492232c5: function(arg0) {
            const ret = arg0.navigator;
            return ret;
        },
        __wbg_new_0c7403db6e782f19: function(arg0) {
            const ret = new Uint8Array(arg0);
            return ret;
        },
        __wbg_new_1b792d90f7c7a3b4: function() { return handleError(function () {
            const ret = new MessageChannel();
            return ret;
        }, arguments); },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_new_682678e2f47e32bc: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_98c22165a42231aa: function() { return handleError(function () {
            const ret = new AbortController();
            return ret;
        }, arguments); },
        __wbg_new_a73bd86e73440d2f: function() { return handleError(function (arg0) {
            const ret = new IntersectionObserver(arg0);
            return ret;
        }, arguments); },
        __wbg_new_aa8d0fa9762c29bd: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_ad8d9a2aa2624a65: function() { return handleError(function (arg0) {
            const ret = new ResizeObserver(arg0);
            return ret;
        }, arguments); },
        __wbg_new_ca878e5fdbbbf099: function() { return handleError(function () {
            const ret = new Image();
            return ret;
        }, arguments); },
        __wbg_new_d9e8ade8a7fba252: function() { return handleError(function (arg0, arg1) {
            const ret = new Worker(getStringFromWasm0(arg0, arg1));
            return ret;
        }, arguments); },
        __wbg_new_feb5a86b8a237921: function(arg0, arg1, arg2, arg3) {
            const ret = new RegExp(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            return ret;
        },
        __wbg_new_from_slice_b5ea43e23f6008c0: function(arg0, arg1) {
            const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_new_with_byte_offset_and_length_01848e8d6a3d49ad: function(arg0, arg1, arg2) {
            const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
            return ret;
        },
        __wbg_new_with_context_options_f909c1402cb9fe8c: function() { return handleError(function (arg0) {
            const ret = new lAudioContext(arg0);
            return ret;
        }, arguments); },
        __wbg_new_with_str_sequence_and_options_2cfc7ae8f9435aa4: function() { return handleError(function (arg0, arg1) {
            const ret = new Blob(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_new_with_u8_clamped_array_c33a2d80c19b3dc7: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = new ImageData(getClampedArrayU8FromWasm0(arg0, arg1), arg2 >>> 0);
            return ret;
        }, arguments); },
        __wbg_now_a9b7df1cbee90986: function() {
            const ret = Date.now();
            return ret;
        },
        __wbg_now_e7c6795a7f81e10f: function(arg0) {
            const ret = arg0.now();
            return ret;
        },
        __wbg_observe_59e08e55b0dd238f: function(arg0, arg1) {
            arg0.observe(arg1);
        },
        __wbg_observe_5ea88d68554155e1: function(arg0, arg1, arg2) {
            arg0.observe(arg1, arg2);
        },
        __wbg_observe_c79fbdfb1452af30: function(arg0, arg1) {
            arg0.observe(arg1);
        },
        __wbg_of_07054ba808010e4f: function(arg0) {
            const ret = Array.of(arg0);
            return ret;
        },
        __wbg_of_7532e43da680ecb3: function(arg0, arg1) {
            const ret = Array.of(arg0, arg1);
            return ret;
        },
        __wbg_offsetX_a9bf2ea7f0575ac9: function(arg0) {
            const ret = arg0.offsetX;
            return ret;
        },
        __wbg_offsetY_10e5433a1bbd4c01: function(arg0) {
            const ret = arg0.offsetY;
            return ret;
        },
        __wbg_onSubmittedWorkDone_81e152567230130a: function(arg0) {
            const ret = arg0.onSubmittedWorkDone();
            return ret;
        },
        __wbg_performance_3fcf6e32a7e1ed0a: function(arg0) {
            const ret = arg0.performance;
            return ret;
        },
        __wbg_persisted_bfebef6179ea1e1a: function(arg0) {
            const ret = arg0.persisted;
            return ret;
        },
        __wbg_pixelStorei_0da594e7ec84d2ef: function(arg0, arg1, arg2) {
            arg0.pixelStorei(arg1 >>> 0, arg2);
        },
        __wbg_pixelStorei_6f7ca5f58231a418: function(arg0, arg1, arg2) {
            arg0.pixelStorei(arg1 >>> 0, arg2);
        },
        __wbg_play_3997a1be51d27925: function(arg0) {
            arg0.play();
        },
        __wbg_pointerId_b99c11e1f5e3731d: function(arg0) {
            const ret = arg0.pointerId;
            return ret;
        },
        __wbg_pointerType_5c8062de6087884a: function(arg0, arg1) {
            const ret = arg1.pointerType;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_polygonOffset_82e7973a0d5f6313: function(arg0, arg1, arg2) {
            arg0.polygonOffset(arg1, arg2);
        },
        __wbg_polygonOffset_f1e208e8df962cd4: function(arg0, arg1, arg2) {
            arg0.polygonOffset(arg1, arg2);
        },
        __wbg_popDebugGroup_59f9ef930b2c6c68: function(arg0) {
            arg0.popDebugGroup();
        },
        __wbg_popErrorScope_bb2c94b6b3fbd6ba: function(arg0) {
            const ret = arg0.popErrorScope();
            return ret;
        },
        __wbg_port1_8267146008301e78: function(arg0) {
            const ret = arg0.port1;
            return ret;
        },
        __wbg_port2_1742efa161730e58: function(arg0) {
            const ret = arg0.port2;
            return ret;
        },
        __wbg_postMessage_1f50a9885ee45fb0: function() { return handleError(function (arg0, arg1) {
            arg0.postMessage(arg1);
        }, arguments); },
        __wbg_postMessage_2e8ce5e10ce05091: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.postMessage(arg1, arg2);
        }, arguments); },
        __wbg_postTask_e2439afddcdfbb55: function(arg0, arg1, arg2) {
            const ret = arg0.postTask(arg1, arg2);
            return ret;
        },
        __wbg_pressed_5885d40140802fd3: function(arg0) {
            const ret = arg0.pressed;
            return ret;
        },
        __wbg_pressure_f5789eab65b5c2ae: function(arg0) {
            const ret = arg0.pressure;
            return ret;
        },
        __wbg_preventDefault_f55c01cb5fd2bcc0: function(arg0) {
            arg0.preventDefault();
        },
        __wbg_prototype_0d5bb2023db3bcfc: function() {
            const ret = ResizeObserverEntry.prototype;
            return ret;
        },
        __wbg_prototypesetcall_a6b02eb00b0f4ce2: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
        },
        __wbg_pushDebugGroup_18ca02af72bc23b8: function(arg0, arg1, arg2) {
            arg0.pushDebugGroup(getStringFromWasm0(arg1, arg2));
        },
        __wbg_pushErrorScope_1fa39584d72a12a8: function(arg0, arg1) {
            arg0.pushErrorScope(__wbindgen_enum_GpuErrorFilter[arg1]);
        },
        __wbg_push_471a5b068a5295f6: function(arg0, arg1) {
            const ret = arg0.push(arg1);
            return ret;
        },
        __wbg_queryCounterEXT_08e9bfae0dff258b: function(arg0, arg1, arg2) {
            arg0.queryCounterEXT(arg1, arg2 >>> 0);
        },
        __wbg_querySelectorAll_e9e3fbd41310476e: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.querySelectorAll(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_querySelector_8d395ebd237ebd46: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.querySelector(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_queueMicrotask_0aed009ff060f723: function(arg0, arg1) {
            arg0.queueMicrotask(arg1);
        },
        __wbg_queueMicrotask_5d15a957e6aa920e: function(arg0) {
            queueMicrotask(arg0);
        },
        __wbg_queueMicrotask_6913321b637d352e: function(arg0) {
            queueMicrotask(arg0);
        },
        __wbg_queueMicrotask_f8819e5ffc402f36: function(arg0) {
            const ret = arg0.queueMicrotask;
            return ret;
        },
        __wbg_queue_81f5d725809ccd54: function(arg0) {
            const ret = arg0.queue;
            return ret;
        },
        __wbg_readBuffer_a2d26b22c3faabd0: function(arg0, arg1) {
            arg0.readBuffer(arg1 >>> 0);
        },
        __wbg_readPixels_a78444c3ffa2ad18: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            arg0.readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, arg7);
        }, arguments); },
        __wbg_readPixels_bfac0d542650a07a: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            arg0.readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, arg7);
        }, arguments); },
        __wbg_readPixels_dd7e621f7a36e2ac: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            arg0.readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, arg7);
        }, arguments); },
        __wbg_readText_fe784da4f0e0cd03: function(arg0) {
            const ret = arg0.readText();
            return ret;
        },
        __wbg_reason_032ebc72b13c977e: function(arg0) {
            const ret = arg0.reason;
            return (__wbindgen_enum_GpuDeviceLostReason.indexOf(ret) + 1 || 3) - 1;
        },
        __wbg_removeEventListener_7bdf07404d9b24bd: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.removeEventListener(getStringFromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_removeListener_dcb0b2ae1124b401: function() { return handleError(function (arg0, arg1) {
            arg0.removeListener(arg1);
        }, arguments); },
        __wbg_removeProperty_af5e61d737797fcc: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg1.removeProperty(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_renderbufferStorageMultisample_363ea6ea6644c47f: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.renderbufferStorageMultisample(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
        },
        __wbg_renderbufferStorage_0745213c8a3edba7: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.renderbufferStorage(arg1 >>> 0, arg2 >>> 0, arg3, arg4);
        },
        __wbg_renderbufferStorage_838d8e6ca86ee3ca: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.renderbufferStorage(arg1 >>> 0, arg2 >>> 0, arg3, arg4);
        },
        __wbg_repeat_f5ff89c357b71af1: function(arg0) {
            const ret = arg0.repeat;
            return ret;
        },
        __wbg_requestAdapter_90f7496e67f82c21: function(arg0, arg1) {
            const ret = arg0.requestAdapter(arg1);
            return ret;
        },
        __wbg_requestAnimationFrame_6f039d778639cc28: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.requestAnimationFrame(arg1);
            return ret;
        }, arguments); },
        __wbg_requestDevice_5c307ce72228d3f7: function(arg0, arg1) {
            const ret = arg0.requestDevice(arg1);
            return ret;
        },
        __wbg_requestFullscreen_3f16e43f398ce624: function(arg0) {
            const ret = arg0.requestFullscreen();
            return ret;
        },
        __wbg_requestFullscreen_b977a3a0697e883c: function(arg0) {
            const ret = arg0.requestFullscreen;
            return ret;
        },
        __wbg_requestIdleCallback_3689e3e38f6cfc02: function(arg0) {
            const ret = arg0.requestIdleCallback;
            return ret;
        },
        __wbg_requestIdleCallback_fd04869e36d71d03: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.requestIdleCallback(arg1);
            return ret;
        }, arguments); },
        __wbg_requestPointerLock_9c4d4cabc91a29c5: function(arg0) {
            arg0.requestPointerLock();
        },
        __wbg_resolve_e6c466bc1052f16c: function(arg0) {
            const ret = Promise.resolve(arg0);
            return ret;
        },
        __wbg_resume_b12ef3046a46b3f7: function() { return handleError(function (arg0) {
            const ret = arg0.resume();
            return ret;
        }, arguments); },
        __wbg_revokeObjectURL_1d23b31dc4ef5f52: function() { return handleError(function (arg0, arg1) {
            URL.revokeObjectURL(getStringFromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_samplerParameterf_974a275475147bd9: function(arg0, arg1, arg2, arg3) {
            arg0.samplerParameterf(arg1, arg2 >>> 0, arg3);
        },
        __wbg_samplerParameteri_8a634d1b1b1e79ad: function(arg0, arg1, arg2, arg3) {
            arg0.samplerParameteri(arg1, arg2 >>> 0, arg3);
        },
        __wbg_scheduler_a17d41c9c822fc26: function(arg0) {
            const ret = arg0.scheduler;
            return ret;
        },
        __wbg_scheduler_b35fe73ba70e89cc: function(arg0) {
            const ret = arg0.scheduler;
            return ret;
        },
        __wbg_scissor_a52de5e62ebadc16: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.scissor(arg1, arg2, arg3, arg4);
        },
        __wbg_scissor_b71fb7e05633cf3d: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.scissor(arg1, arg2, arg3, arg4);
        },
        __wbg_setAttribute_6fde4098d274155c: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setBindGroup_2b53fd8bfb43674d: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.setBindGroup(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
        }, arguments); },
        __wbg_setBindGroup_58960c4b1bcdd182: function(arg0, arg1, arg2) {
            arg0.setBindGroup(arg1 >>> 0, arg2);
        },
        __wbg_setBindGroup_5d9cc0ccc87c9640: function(arg0, arg1, arg2) {
            arg0.setBindGroup(arg1 >>> 0, arg2);
        },
        __wbg_setBindGroup_a62f9de1cb2449b2: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.setBindGroup(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
        }, arguments); },
        __wbg_setBlendConstant_00efcd0411cbd141: function() { return handleError(function (arg0, arg1) {
            arg0.setBlendConstant(arg1);
        }, arguments); },
        __wbg_setIndexBuffer_b94e5d57d9f987b1: function(arg0, arg1, arg2, arg3) {
            arg0.setIndexBuffer(arg1, __wbindgen_enum_GpuIndexFormat[arg2], arg3);
        },
        __wbg_setIndexBuffer_fe1825c2b9e2d364: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.setIndexBuffer(arg1, __wbindgen_enum_GpuIndexFormat[arg2], arg3, arg4);
        },
        __wbg_setPipeline_1fc98ad28be0d04a: function(arg0, arg1) {
            arg0.setPipeline(arg1);
        },
        __wbg_setPipeline_9f6b0a3c5901572d: function(arg0, arg1) {
            arg0.setPipeline(arg1);
        },
        __wbg_setPointerCapture_0ade0346ebef3bfc: function() { return handleError(function (arg0, arg1) {
            arg0.setPointerCapture(arg1);
        }, arguments); },
        __wbg_setProperty_0d903d23a71dfe70: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setScissorRect_98e8337e62425096: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.setScissorRect(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_setStencilReference_0a822a2ae19699a2: function(arg0, arg1) {
            arg0.setStencilReference(arg1 >>> 0);
        },
        __wbg_setTimeout_5a5ca8752c41f8ad: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.setTimeout(arg1);
            return ret;
        }, arguments); },
        __wbg_setTimeout_d8786dd31f90da0f: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.setTimeout(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_setVertexBuffer_c3bb3670263af952: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.setVertexBuffer(arg1 >>> 0, arg2, arg3, arg4);
        },
        __wbg_setVertexBuffer_c3c88170005afc1b: function(arg0, arg1, arg2, arg3) {
            arg0.setVertexBuffer(arg1 >>> 0, arg2, arg3);
        },
        __wbg_setViewport_007a2c7160c6bedb: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.setViewport(arg1, arg2, arg3, arg4, arg5, arg6);
        },
        __wbg_set_022bee52d0b05b19: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(arg0, arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_set_a_2f4495829c853bba: function(arg0, arg1) {
            arg0.a = arg1;
        },
        __wbg_set_access_802ef755476d4064: function(arg0, arg1) {
            arg0.access = __wbindgen_enum_GpuStorageTextureAccess[arg1];
        },
        __wbg_set_address_mode_u_c13cdf94d097b16d: function(arg0, arg1) {
            arg0.addressModeU = __wbindgen_enum_GpuAddressMode[arg1];
        },
        __wbg_set_address_mode_v_c09db9861cd052a6: function(arg0, arg1) {
            arg0.addressModeV = __wbindgen_enum_GpuAddressMode[arg1];
        },
        __wbg_set_address_mode_w_0b49c35f3d4322bf: function(arg0, arg1) {
            arg0.addressModeW = __wbindgen_enum_GpuAddressMode[arg1];
        },
        __wbg_set_alpha_29642d2219224544: function(arg0, arg1) {
            arg0.alpha = arg1;
        },
        __wbg_set_alpha_mode_65ba0adaef90e1f3: function(arg0, arg1) {
            arg0.alphaMode = __wbindgen_enum_GpuCanvasAlphaMode[arg1];
        },
        __wbg_set_alpha_to_coverage_enabled_ab6a22e18e338493: function(arg0, arg1) {
            arg0.alphaToCoverageEnabled = arg1 !== 0;
        },
        __wbg_set_array_layer_count_de83f575c3f6d15e: function(arg0, arg1) {
            arg0.arrayLayerCount = arg1 >>> 0;
        },
        __wbg_set_array_stride_2033aeb8a42130f9: function(arg0, arg1) {
            arg0.arrayStride = arg1;
        },
        __wbg_set_aspect_4c0237c8f21de349: function(arg0, arg1) {
            arg0.aspect = __wbindgen_enum_GpuTextureAspect[arg1];
        },
        __wbg_set_aspect_adde591ce42eb208: function(arg0, arg1) {
            arg0.aspect = __wbindgen_enum_GpuTextureAspect[arg1];
        },
        __wbg_set_aspect_feb0fac859e82372: function(arg0, arg1) {
            arg0.aspect = __wbindgen_enum_GpuTextureAspect[arg1];
        },
        __wbg_set_attributes_39e5a71bf05309a6: function(arg0, arg1) {
            arg0.attributes = arg1;
        },
        __wbg_set_b_7081554879455e65: function(arg0, arg1) {
            arg0.b = arg1;
        },
        __wbg_set_base_array_layer_ab196aad24c8fac6: function(arg0, arg1) {
            arg0.baseArrayLayer = arg1 >>> 0;
        },
        __wbg_set_base_mip_level_15d29fc182e25a82: function(arg0, arg1) {
            arg0.baseMipLevel = arg1 >>> 0;
        },
        __wbg_set_beginning_of_pass_write_index_0cb71e33ea66ffc8: function(arg0, arg1) {
            arg0.beginningOfPassWriteIndex = arg1 >>> 0;
        },
        __wbg_set_beginning_of_pass_write_index_c2f97408798615ca: function(arg0, arg1) {
            arg0.beginningOfPassWriteIndex = arg1 >>> 0;
        },
        __wbg_set_bind_group_layouts_5c298441f47e30a1: function(arg0, arg1) {
            arg0.bindGroupLayouts = arg1;
        },
        __wbg_set_binding_234b4c508d19a0a8: function(arg0, arg1) {
            arg0.binding = arg1 >>> 0;
        },
        __wbg_set_binding_fd933455b600a07f: function(arg0, arg1) {
            arg0.binding = arg1 >>> 0;
        },
        __wbg_set_blend_1dbdd086fc4fdebf: function(arg0, arg1) {
            arg0.blend = arg1;
        },
        __wbg_set_box_e76b1c9ae3cbed18: function(arg0, arg1) {
            arg0.box = __wbindgen_enum_ResizeObserverBoxOptions[arg1];
        },
        __wbg_set_buffer_8f0ef5be1b92d605: function(arg0, arg1) {
            arg0.buffer = arg1;
        },
        __wbg_set_buffer_a58d247ab5b5f5b8: function(arg0, arg1) {
            arg0.buffer = arg1;
        },
        __wbg_set_buffer_b04e4d70b1eb4630: function(arg0, arg1) {
            arg0.buffer = arg1;
        },
        __wbg_set_buffer_c0b9567219746d80: function(arg0, arg1) {
            arg0.buffer = arg1;
        },
        __wbg_set_buffers_3f9c487ea01dddcf: function(arg0, arg1) {
            arg0.buffers = arg1;
        },
        __wbg_set_bytes_per_row_39bcca8e0c25e0ee: function(arg0, arg1) {
            arg0.bytesPerRow = arg1 >>> 0;
        },
        __wbg_set_bytes_per_row_b8d0d0a0847ff2ea: function(arg0, arg1) {
            arg0.bytesPerRow = arg1 >>> 0;
        },
        __wbg_set_channelCount_82c04409caddcf6a: function(arg0, arg1) {
            arg0.channelCount = arg1 >>> 0;
        },
        __wbg_set_clear_value_1663cbe7da00e7e4: function(arg0, arg1) {
            arg0.clearValue = arg1;
        },
        __wbg_set_code_3bb44fc02aa17153: function(arg0, arg1, arg2) {
            arg0.code = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_color_attachments_b740d060dacde5c0: function(arg0, arg1) {
            arg0.colorAttachments = arg1;
        },
        __wbg_set_color_d0208d092af4f2e6: function(arg0, arg1) {
            arg0.color = arg1;
        },
        __wbg_set_compare_00dc33383c873ad5: function(arg0, arg1) {
            arg0.compare = __wbindgen_enum_GpuCompareFunction[arg1];
        },
        __wbg_set_compare_11834994f7d75687: function(arg0, arg1) {
            arg0.compare = __wbindgen_enum_GpuCompareFunction[arg1];
        },
        __wbg_set_compute_6c1b9ba0e3041f2c: function(arg0, arg1) {
            arg0.compute = arg1;
        },
        __wbg_set_count_ab42cbc78635ed91: function(arg0, arg1) {
            arg0.count = arg1 >>> 0;
        },
        __wbg_set_cull_mode_c4f1ef740bd14c40: function(arg0, arg1) {
            arg0.cullMode = __wbindgen_enum_GpuCullMode[arg1];
        },
        __wbg_set_cursor_8d686ff9dd99a325: function(arg0, arg1, arg2) {
            arg0.cursor = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_d8f1efe557b9e7e1: function(arg0, arg1, arg2) {
            arg0.set(arg1, arg2 >>> 0);
        },
        __wbg_set_depth_bias_clamp_f573c2dda55692a6: function(arg0, arg1) {
            arg0.depthBiasClamp = arg1;
        },
        __wbg_set_depth_bias_ebe05aecbb98e11f: function(arg0, arg1) {
            arg0.depthBias = arg1;
        },
        __wbg_set_depth_bias_slope_scale_27c8208740c46086: function(arg0, arg1) {
            arg0.depthBiasSlopeScale = arg1;
        },
        __wbg_set_depth_clear_value_57c2283d39fbb181: function(arg0, arg1) {
            arg0.depthClearValue = arg1;
        },
        __wbg_set_depth_compare_a9c538cec0e01535: function(arg0, arg1) {
            arg0.depthCompare = __wbindgen_enum_GpuCompareFunction[arg1];
        },
        __wbg_set_depth_fail_op_42b9d46a7c67baae: function(arg0, arg1) {
            arg0.depthFailOp = __wbindgen_enum_GpuStencilOperation[arg1];
        },
        __wbg_set_depth_load_op_f95fdb158b819261: function(arg0, arg1) {
            arg0.depthLoadOp = __wbindgen_enum_GpuLoadOp[arg1];
        },
        __wbg_set_depth_or_array_layers_7335d3fc04cd5ade: function(arg0, arg1) {
            arg0.depthOrArrayLayers = arg1 >>> 0;
        },
        __wbg_set_depth_read_only_878b741b02a4dd71: function(arg0, arg1) {
            arg0.depthReadOnly = arg1 !== 0;
        },
        __wbg_set_depth_stencil_1c7bed669574dd1e: function(arg0, arg1) {
            arg0.depthStencil = arg1;
        },
        __wbg_set_depth_stencil_attachment_82ce8924f4e0e79b: function(arg0, arg1) {
            arg0.depthStencilAttachment = arg1;
        },
        __wbg_set_depth_store_op_4c56ab1d005c7bf6: function(arg0, arg1) {
            arg0.depthStoreOp = __wbindgen_enum_GpuStoreOp[arg1];
        },
        __wbg_set_depth_write_enabled_f726d4f27a24ff7e: function(arg0, arg1) {
            arg0.depthWriteEnabled = arg1 !== 0;
        },
        __wbg_set_device_f991f8a955db69f7: function(arg0, arg1) {
            arg0.device = arg1;
        },
        __wbg_set_dimension_7ca3d24380d365e4: function(arg0, arg1) {
            arg0.dimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
        },
        __wbg_set_dimension_87dd70a08e54ea98: function(arg0, arg1) {
            arg0.dimension = __wbindgen_enum_GpuTextureDimension[arg1];
        },
        __wbg_set_dst_factor_1382684d97e2aec4: function(arg0, arg1) {
            arg0.dstFactor = __wbindgen_enum_GpuBlendFactor[arg1];
        },
        __wbg_set_duration_bfef0b021dc8fd5b: function(arg0, arg1) {
            arg0.duration = arg1;
        },
        __wbg_set_end_of_pass_write_index_3476a9a4411846af: function(arg0, arg1) {
            arg0.endOfPassWriteIndex = arg1 >>> 0;
        },
        __wbg_set_end_of_pass_write_index_5e969b5aa2f94e75: function(arg0, arg1) {
            arg0.endOfPassWriteIndex = arg1 >>> 0;
        },
        __wbg_set_entries_44ee8dc60918063d: function(arg0, arg1) {
            arg0.entries = arg1;
        },
        __wbg_set_entries_803b89386febf57c: function(arg0, arg1) {
            arg0.entries = arg1;
        },
        __wbg_set_entry_point_418e5aecbf7f95b4: function(arg0, arg1, arg2) {
            arg0.entryPoint = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_entry_point_a84dd78ae4a97c6d: function(arg0, arg1, arg2) {
            arg0.entryPoint = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_entry_point_ac45ddee35909233: function(arg0, arg1, arg2) {
            arg0.entryPoint = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_external_texture_73d5e5303574a1e8: function(arg0, arg1) {
            arg0.externalTexture = arg1;
        },
        __wbg_set_fail_op_6f4612035f584d02: function(arg0, arg1) {
            arg0.failOp = __wbindgen_enum_GpuStencilOperation[arg1];
        },
        __wbg_set_flip_y_21c0cdab245f4d89: function(arg0, arg1) {
            arg0.flipY = arg1 !== 0;
        },
        __wbg_set_format_2bd90cb220cc6884: function(arg0, arg1) {
            arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_3cc5d6ead9a8cce0: function(arg0, arg1) {
            arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_40d793124494a9df: function(arg0, arg1) {
            arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_723d6bb38a9e71d3: function(arg0, arg1) {
            arg0.format = __wbindgen_enum_GpuVertexFormat[arg1];
        },
        __wbg_set_format_c23f7c142762c3a7: function(arg0, arg1) {
            arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_e0af83ab86ee58dc: function(arg0, arg1) {
            arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_fcbaa54d6b5c186a: function(arg0, arg1) {
            arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_fragment_9b5673b1b740fe0e: function(arg0, arg1) {
            arg0.fragment = arg1;
        },
        __wbg_set_front_face_bb590812353fd2e0: function(arg0, arg1) {
            arg0.frontFace = __wbindgen_enum_GpuFrontFace[arg1];
        },
        __wbg_set_g_aa23517844bd7f61: function(arg0, arg1) {
            arg0.g = arg1;
        },
        __wbg_set_has_dynamic_offset_ea1fb6bd94b0c904: function(arg0, arg1) {
            arg0.hasDynamicOffset = arg1 !== 0;
        },
        __wbg_set_height_24d07d982f176ac6: function(arg0, arg1) {
            arg0.height = arg1 >>> 0;
        },
        __wbg_set_height_66583e77881d3a51: function(arg0, arg1) {
            arg0.height = arg1 >>> 0;
        },
        __wbg_set_height_be9b2b920bd68401: function(arg0, arg1) {
            arg0.height = arg1 >>> 0;
        },
        __wbg_set_iterations_b84d4d3302a291a0: function(arg0, arg1) {
            arg0.iterations = arg1;
        },
        __wbg_set_label_08e9f27a97fdc9f7: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_0e9f90ea4e961823: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_1736939dde71ec96: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_280bd57b618e4cf6: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_34d2766c2203f76a: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_4bf9f5458cdc0a68: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_797345a8c9c86146: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_8fdd5f28eea3ca08: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_a4be4acc3510c62f: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_bb92451e0d92abf4: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_c3405868bd8f6ab5: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_d5519c3081c41e5a: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_d73358f96a62d3bc: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_f00eb249a34df7db: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_f571593aaa82f18b: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_layout_9590b02a1d72ac45: function(arg0, arg1) {
            arg0.layout = arg1;
        },
        __wbg_set_layout_a065a939d1d05a2d: function(arg0, arg1) {
            arg0.layout = arg1;
        },
        __wbg_set_layout_d008ec94bedc0844: function(arg0, arg1) {
            arg0.layout = arg1;
        },
        __wbg_set_load_op_07c59d4ab60a3a01: function(arg0, arg1) {
            arg0.loadOp = __wbindgen_enum_GpuLoadOp[arg1];
        },
        __wbg_set_lod_max_clamp_fd1548dc78538913: function(arg0, arg1) {
            arg0.lodMaxClamp = arg1;
        },
        __wbg_set_lod_min_clamp_b489016289e378d2: function(arg0, arg1) {
            arg0.lodMinClamp = arg1;
        },
        __wbg_set_mag_filter_b4e8d7f2fa665d2e: function(arg0, arg1) {
            arg0.magFilter = __wbindgen_enum_GpuFilterMode[arg1];
        },
        __wbg_set_mapped_at_creation_c78869832c67816c: function(arg0, arg1) {
            arg0.mappedAtCreation = arg1 !== 0;
        },
        __wbg_set_mask_cee9de29cbe61459: function(arg0, arg1) {
            arg0.mask = arg1 >>> 0;
        },
        __wbg_set_max_anisotropy_a019fd38d9ba634e: function(arg0, arg1) {
            arg0.maxAnisotropy = arg1;
        },
        __wbg_set_min_binding_size_26f877007450686c: function(arg0, arg1) {
            arg0.minBindingSize = arg1;
        },
        __wbg_set_min_filter_cd8cf3dcdeebaa5b: function(arg0, arg1) {
            arg0.minFilter = __wbindgen_enum_GpuFilterMode[arg1];
        },
        __wbg_set_mip_level_161666aedb691ca3: function(arg0, arg1) {
            arg0.mipLevel = arg1 >>> 0;
        },
        __wbg_set_mip_level_count_1993f039035d2469: function(arg0, arg1) {
            arg0.mipLevelCount = arg1 >>> 0;
        },
        __wbg_set_mip_level_count_9a86e098393fe360: function(arg0, arg1) {
            arg0.mipLevelCount = arg1 >>> 0;
        },
        __wbg_set_mip_level_e61d3964c419f64b: function(arg0, arg1) {
            arg0.mipLevel = arg1 >>> 0;
        },
        __wbg_set_mipmap_filter_a436d61249cfa785: function(arg0, arg1) {
            arg0.mipmapFilter = __wbindgen_enum_GpuMipmapFilterMode[arg1];
        },
        __wbg_set_module_77c9a4994de5185d: function(arg0, arg1) {
            arg0.module = arg1;
        },
        __wbg_set_module_951f2b6e5477a260: function(arg0, arg1) {
            arg0.module = arg1;
        },
        __wbg_set_module_a7b3448454ca8879: function(arg0, arg1) {
            arg0.module = arg1;
        },
        __wbg_set_multisample_bb6537e862d91237: function(arg0, arg1) {
            arg0.multisample = arg1;
        },
        __wbg_set_multisampled_9642e942e4d9d3ee: function(arg0, arg1) {
            arg0.multisampled = arg1 !== 0;
        },
        __wbg_set_offset_3e55dd16ffd7aac5: function(arg0, arg1) {
            arg0.offset = arg1;
        },
        __wbg_set_offset_5c23fa7eb774d62b: function(arg0, arg1) {
            arg0.offset = arg1;
        },
        __wbg_set_offset_a3a60cec10207186: function(arg0, arg1) {
            arg0.offset = arg1;
        },
        __wbg_set_offset_debfe602a5fbf272: function(arg0, arg1) {
            arg0.offset = arg1;
        },
        __wbg_set_onmessage_f25fb55032dd93eb: function(arg0, arg1) {
            arg0.onmessage = arg1;
        },
        __wbg_set_onuncapturederror_8f485d34a545d58b: function(arg0, arg1) {
            arg0.onuncapturederror = arg1;
        },
        __wbg_set_operation_74a529d361734388: function(arg0, arg1) {
            arg0.operation = __wbindgen_enum_GpuBlendOperation[arg1];
        },
        __wbg_set_origin_42cf0cf261f50d63: function(arg0, arg1) {
            arg0.origin = arg1;
        },
        __wbg_set_origin_d09654f499e9edb8: function(arg0, arg1) {
            arg0.origin = arg1;
        },
        __wbg_set_origin_f7a8894367b28556: function(arg0, arg1) {
            arg0.origin = arg1;
        },
        __wbg_set_pass_op_8abd39478c76666a: function(arg0, arg1) {
            arg0.passOp = __wbindgen_enum_GpuStencilOperation[arg1];
        },
        __wbg_set_power_preference_b8b4ea5da6674cf7: function(arg0, arg1) {
            arg0.powerPreference = __wbindgen_enum_GpuPowerPreference[arg1];
        },
        __wbg_set_premultiplied_alpha_dde44b27abcf88fc: function(arg0, arg1) {
            arg0.premultipliedAlpha = arg1 !== 0;
        },
        __wbg_set_premultiply_alpha_1419f115d34d09fa: function(arg0, arg1) {
            arg0.premultiplyAlpha = __wbindgen_enum_PremultiplyAlpha[arg1];
        },
        __wbg_set_primitive_f189fcdcb22d09e0: function(arg0, arg1) {
            arg0.primitive = arg1;
        },
        __wbg_set_query_set_2862e48f0ed8ffe8: function(arg0, arg1) {
            arg0.querySet = arg1;
        },
        __wbg_set_query_set_dcf406a51ece8f85: function(arg0, arg1) {
            arg0.querySet = arg1;
        },
        __wbg_set_r_8961014434a7656e: function(arg0, arg1) {
            arg0.r = arg1;
        },
        __wbg_set_required_features_ec67124fd26c4d29: function(arg0, arg1) {
            arg0.requiredFeatures = arg1;
        },
        __wbg_set_required_limits_c9ee7006f1d1f2ab: function(arg0, arg1) {
            arg0.requiredLimits = arg1;
        },
        __wbg_set_resolve_target_cc7a6f0d2973ea34: function(arg0, arg1) {
            arg0.resolveTarget = arg1;
        },
        __wbg_set_resource_86645e7515651c0e: function(arg0, arg1) {
            arg0.resource = arg1;
        },
        __wbg_set_rows_per_image_0cc67df1420cf33b: function(arg0, arg1) {
            arg0.rowsPerImage = arg1 >>> 0;
        },
        __wbg_set_rows_per_image_7203b6e2d244a111: function(arg0, arg1) {
            arg0.rowsPerImage = arg1 >>> 0;
        },
        __wbg_set_sample_count_4d7160817d98838f: function(arg0, arg1) {
            arg0.sampleCount = arg1 >>> 0;
        },
        __wbg_set_sample_rate_c698733b6083a69f: function(arg0, arg1) {
            arg0.sampleRate = arg1;
        },
        __wbg_set_sample_type_8d4d5b141ce0f724: function(arg0, arg1) {
            arg0.sampleType = __wbindgen_enum_GpuTextureSampleType[arg1];
        },
        __wbg_set_sampler_35bcbac78bd4356f: function(arg0, arg1) {
            arg0.sampler = arg1;
        },
        __wbg_set_shader_location_3ce5152f6d464a63: function(arg0, arg1) {
            arg0.shaderLocation = arg1 >>> 0;
        },
        __wbg_set_size_81a77f7f4f34fbed: function(arg0, arg1) {
            arg0.size = arg1;
        },
        __wbg_set_size_85cb1c2c4c3ea73a: function(arg0, arg1) {
            arg0.size = arg1;
        },
        __wbg_set_size_981550e5d7941340: function(arg0, arg1) {
            arg0.size = arg1;
        },
        __wbg_set_source_51577a2cebeadf81: function(arg0, arg1) {
            arg0.source = arg1;
        },
        __wbg_set_src_59f1be1c833b4918: function(arg0, arg1, arg2) {
            arg0.src = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_src_factor_9a8e0943a05c9174: function(arg0, arg1) {
            arg0.srcFactor = __wbindgen_enum_GpuBlendFactor[arg1];
        },
        __wbg_set_stencil_back_596ea9628419413d: function(arg0, arg1) {
            arg0.stencilBack = arg1;
        },
        __wbg_set_stencil_clear_value_15afeb03c22cd51d: function(arg0, arg1) {
            arg0.stencilClearValue = arg1 >>> 0;
        },
        __wbg_set_stencil_front_31be994e05be5aaa: function(arg0, arg1) {
            arg0.stencilFront = arg1;
        },
        __wbg_set_stencil_load_op_1cd94e9e8c54f611: function(arg0, arg1) {
            arg0.stencilLoadOp = __wbindgen_enum_GpuLoadOp[arg1];
        },
        __wbg_set_stencil_read_mask_1635f30a0e6539e3: function(arg0, arg1) {
            arg0.stencilReadMask = arg1 >>> 0;
        },
        __wbg_set_stencil_read_only_f071431988182ad8: function(arg0, arg1) {
            arg0.stencilReadOnly = arg1 !== 0;
        },
        __wbg_set_stencil_store_op_a244d5347f386c8c: function(arg0, arg1) {
            arg0.stencilStoreOp = __wbindgen_enum_GpuStoreOp[arg1];
        },
        __wbg_set_stencil_write_mask_7809f82a1debe58f: function(arg0, arg1) {
            arg0.stencilWriteMask = arg1 >>> 0;
        },
        __wbg_set_step_mode_eb762c8c4264418f: function(arg0, arg1) {
            arg0.stepMode = __wbindgen_enum_GpuVertexStepMode[arg1];
        },
        __wbg_set_storage_texture_22f78b5171d1195a: function(arg0, arg1) {
            arg0.storageTexture = arg1;
        },
        __wbg_set_store_op_386596acc7bf2c16: function(arg0, arg1) {
            arg0.storeOp = __wbindgen_enum_GpuStoreOp[arg1];
        },
        __wbg_set_strip_index_format_e76748cd840ab562: function(arg0, arg1) {
            arg0.stripIndexFormat = __wbindgen_enum_GpuIndexFormat[arg1];
        },
        __wbg_set_targets_22473476afe0dabd: function(arg0, arg1) {
            arg0.targets = arg1;
        },
        __wbg_set_texture_2c34d28ab9666948: function(arg0, arg1) {
            arg0.texture = arg1;
        },
        __wbg_set_texture_ac9a46252c0cb532: function(arg0, arg1) {
            arg0.texture = arg1;
        },
        __wbg_set_texture_aeea930400349204: function(arg0, arg1) {
            arg0.texture = arg1;
        },
        __wbg_set_timestamp_writes_0236dfc7ae2b1a03: function(arg0, arg1) {
            arg0.timestampWrites = arg1;
        },
        __wbg_set_timestamp_writes_d1259248cc80f658: function(arg0, arg1) {
            arg0.timestampWrites = arg1;
        },
        __wbg_set_topology_e18a15a717ebc912: function(arg0, arg1) {
            arg0.topology = __wbindgen_enum_GpuPrimitiveTopology[arg1];
        },
        __wbg_set_type_31b1662dd5a6144d: function(arg0, arg1) {
            arg0.type = __wbindgen_enum_GpuSamplerBindingType[arg1];
        },
        __wbg_set_type_719f40cf36d314f1: function(arg0, arg1) {
            arg0.type = __wbindgen_enum_GpuBufferBindingType[arg1];
        },
        __wbg_set_type_8b2743f6b4de4035: function(arg0, arg1, arg2) {
            arg0.type = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_unclipped_depth_0f5d142d317e3a7c: function(arg0, arg1) {
            arg0.unclippedDepth = arg1 !== 0;
        },
        __wbg_set_usage_26861a639595cd45: function(arg0, arg1) {
            arg0.usage = arg1 >>> 0;
        },
        __wbg_set_usage_7b79a227ada2f5cc: function(arg0, arg1) {
            arg0.usage = arg1 >>> 0;
        },
        __wbg_set_usage_d9ff4b7757fac246: function(arg0, arg1) {
            arg0.usage = arg1 >>> 0;
        },
        __wbg_set_usage_e8d45decd5c483b3: function(arg0, arg1) {
            arg0.usage = arg1 >>> 0;
        },
        __wbg_set_vertex_b95705590b782671: function(arg0, arg1) {
            arg0.vertex = arg1;
        },
        __wbg_set_view_6ff951d6e3f9e337: function(arg0, arg1) {
            arg0.view = arg1;
        },
        __wbg_set_view_cf298e1e7b6ef38a: function(arg0, arg1) {
            arg0.view = arg1;
        },
        __wbg_set_view_dimension_87c95b0d987a14cd: function(arg0, arg1) {
            arg0.viewDimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
        },
        __wbg_set_view_dimension_e99ec138da7b8f83: function(arg0, arg1) {
            arg0.viewDimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
        },
        __wbg_set_view_formats_733fb624c2f2ef6b: function(arg0, arg1) {
            arg0.viewFormats = arg1;
        },
        __wbg_set_view_formats_c2b27891ca5d2740: function(arg0, arg1) {
            arg0.viewFormats = arg1;
        },
        __wbg_set_visibility_315bcac6427d0ba0: function(arg0, arg1) {
            arg0.visibility = arg1 >>> 0;
        },
        __wbg_set_width_5cda41d4d06a14dd: function(arg0, arg1) {
            arg0.width = arg1 >>> 0;
        },
        __wbg_set_width_63034f88f9905ea3: function(arg0, arg1) {
            arg0.width = arg1 >>> 0;
        },
        __wbg_set_width_adc925bca9c5351a: function(arg0, arg1) {
            arg0.width = arg1 >>> 0;
        },
        __wbg_set_write_mask_0b6ca0cb1b797997: function(arg0, arg1) {
            arg0.writeMask = arg1 >>> 0;
        },
        __wbg_set_x_0b48c73e72f71653: function(arg0, arg1) {
            arg0.x = arg1 >>> 0;
        },
        __wbg_set_x_ffcb360b171098d5: function(arg0, arg1) {
            arg0.x = arg1 >>> 0;
        },
        __wbg_set_y_046a6a6e9b0ccbc6: function(arg0, arg1) {
            arg0.y = arg1 >>> 0;
        },
        __wbg_set_y_db82e366feb18537: function(arg0, arg1) {
            arg0.y = arg1 >>> 0;
        },
        __wbg_set_z_cec02b76fd208d0e: function(arg0, arg1) {
            arg0.z = arg1 >>> 0;
        },
        __wbg_shaderSource_084cd6ed337b36be: function(arg0, arg1, arg2, arg3) {
            arg0.shaderSource(arg1, getStringFromWasm0(arg2, arg3));
        },
        __wbg_shaderSource_9b5906e1f027a314: function(arg0, arg1, arg2, arg3) {
            arg0.shaderSource(arg1, getStringFromWasm0(arg2, arg3));
        },
        __wbg_shiftKey_dcf8ee699c273ed2: function(arg0) {
            const ret = arg0.shiftKey;
            return ret;
        },
        __wbg_shiftKey_e483c13c966878f6: function(arg0) {
            const ret = arg0.shiftKey;
            return ret;
        },
        __wbg_signal_fdc54643b47bf85b: function(arg0) {
            const ret = arg0.signal;
            return ret;
        },
        __wbg_size_c0f3d571b8977d71: function(arg0) {
            const ret = arg0.size;
            return ret;
        },
        __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_start_344d0f6ecabcdcb7: function() { return handleError(function (arg0, arg1) {
            arg0.start(arg1);
        }, arguments); },
        __wbg_start_fe881c7e1e08aeef: function(arg0) {
            arg0.start();
        },
        __wbg_static_accessor_GLOBAL_8cfadc87a297ca02: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_602256ae5c8f42cf: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_e445c1c7484aecc3: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_f20e8576ef1e0f17: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_status_43e0d2f15b22d69f: function(arg0) {
            const ret = arg0.status;
            return ret;
        },
        __wbg_stencilFuncSeparate_6793932aaaa884c6: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.stencilFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3, arg4 >>> 0);
        },
        __wbg_stencilFuncSeparate_fcdf02a803d479c6: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.stencilFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3, arg4 >>> 0);
        },
        __wbg_stencilMaskSeparate_29ca6c0767ad75fd: function(arg0, arg1, arg2) {
            arg0.stencilMaskSeparate(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_stencilMaskSeparate_5aca8b40d26d13be: function(arg0, arg1, arg2) {
            arg0.stencilMaskSeparate(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_stencilMask_38acb5180bfdee01: function(arg0, arg1) {
            arg0.stencilMask(arg1 >>> 0);
        },
        __wbg_stencilMask_7f6b699426cca747: function(arg0, arg1) {
            arg0.stencilMask(arg1 >>> 0);
        },
        __wbg_stencilOpSeparate_1a6bcbdb8de495b9: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.stencilOpSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_stencilOpSeparate_774288d006ebab65: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.stencilOpSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_stringify_91082ed7a5a5769e: function() { return handleError(function (arg0) {
            const ret = JSON.stringify(arg0);
            return ret;
        }, arguments); },
        __wbg_style_c331a9f6564f8f62: function(arg0) {
            const ret = arg0.style;
            return ret;
        },
        __wbg_subarray_f8ca46a25b1f5e0d: function(arg0, arg1, arg2) {
            const ret = arg0.subarray(arg1 >>> 0, arg2 >>> 0);
            return ret;
        },
        __wbg_submit_f39583470d95df20: function(arg0, arg1) {
            arg0.submit(arg1);
        },
        __wbg_texImage2D_b17c7723201a6d5e: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texImage2D_bd0466091ed50f83: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texImage2D_f110542c571d15a4: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texImage3D_4bd56d113304ee34: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
            arg0.texImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8 >>> 0, arg9 >>> 0, arg10);
        }, arguments); },
        __wbg_texImage3D_b197787478b2ebe9: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
            arg0.texImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8 >>> 0, arg9 >>> 0, arg10);
        }, arguments); },
        __wbg_texParameteri_83c7801427720baa: function(arg0, arg1, arg2, arg3) {
            arg0.texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
        },
        __wbg_texParameteri_bc24667dff936ebd: function(arg0, arg1, arg2, arg3) {
            arg0.texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
        },
        __wbg_texStorage2D_cd0049448f436f56: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.texStorage2D(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
        },
        __wbg_texStorage3D_645d2a06d38f0291: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.texStorage3D(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5, arg6);
        },
        __wbg_texSubImage2D_5d41ae5586dadcb3: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_6993404ab54773b7: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_7e472dfbf112e954: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_9c0f642762c6c35b: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_ba83ad5c3053f8d4: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_bbeaf09a21601313: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_ed6bface246ddd63: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_f3956198cbc38810: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage3D_5e37ae4a691b540a: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_texSubImage3D_8a2ec691664d8372: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_texSubImage3D_8c8ff30abc439d92: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_texSubImage3D_9bfaeed7c21e9f3b: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_texSubImage3D_c572dcb916b31c0d: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_texSubImage3D_e513c1602d1fe843: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_texSubImage3D_ff660c5e8fde34e3: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_then_6701bb8428537e07: function(arg0, arg1) {
            const ret = arg0.then(arg1);
            return ret;
        },
        __wbg_then_792e0c862b060889: function(arg0, arg1, arg2) {
            const ret = arg0.then(arg1, arg2);
            return ret;
        },
        __wbg_then_8e16ee11f05e4827: function(arg0, arg1) {
            const ret = arg0.then(arg1);
            return ret;
        },
        __wbg_then_a50dc2689b076063: function(arg0, arg1, arg2) {
            const ret = arg0.then(arg1, arg2);
            return ret;
        },
        __wbg_toBlob_1678e1c85166e598: function() { return handleError(function (arg0, arg1) {
            arg0.toBlob(arg1);
        }, arguments); },
        __wbg_transferFromImageBitmap_faa4df0ef84c8b27: function(arg0, arg1) {
            arg0.transferFromImageBitmap(arg1);
        },
        __wbg_uniform1f_e5a0491ecd710bbc: function(arg0, arg1, arg2) {
            arg0.uniform1f(arg1, arg2);
        },
        __wbg_uniform1f_f3284bea42055704: function(arg0, arg1, arg2) {
            arg0.uniform1f(arg1, arg2);
        },
        __wbg_uniform1i_bde3c7d92bc444b2: function(arg0, arg1, arg2) {
            arg0.uniform1i(arg1, arg2);
        },
        __wbg_uniform1i_cfd4726efd9d58b4: function(arg0, arg1, arg2) {
            arg0.uniform1i(arg1, arg2);
        },
        __wbg_uniform1ui_7cea83045fb3528f: function(arg0, arg1, arg2) {
            arg0.uniform1ui(arg1, arg2 >>> 0);
        },
        __wbg_uniform2fv_0cc1bfc11f911dda: function(arg0, arg1, arg2, arg3) {
            arg0.uniform2fv(arg1, getArrayF32FromWasm0(arg2, arg3));
        },
        __wbg_uniform2fv_811d97ec656282f5: function(arg0, arg1, arg2, arg3) {
            arg0.uniform2fv(arg1, getArrayF32FromWasm0(arg2, arg3));
        },
        __wbg_uniform2iv_4a92fbe600a6fb88: function(arg0, arg1, arg2, arg3) {
            arg0.uniform2iv(arg1, getArrayI32FromWasm0(arg2, arg3));
        },
        __wbg_uniform2iv_6ee824c63294e364: function(arg0, arg1, arg2, arg3) {
            arg0.uniform2iv(arg1, getArrayI32FromWasm0(arg2, arg3));
        },
        __wbg_uniform2uiv_922e0d0ff07dacda: function(arg0, arg1, arg2, arg3) {
            arg0.uniform2uiv(arg1, getArrayU32FromWasm0(arg2, arg3));
        },
        __wbg_uniform3fv_8aba848c825c4dcc: function(arg0, arg1, arg2, arg3) {
            arg0.uniform3fv(arg1, getArrayF32FromWasm0(arg2, arg3));
        },
        __wbg_uniform3fv_ff2fddc612532e5f: function(arg0, arg1, arg2, arg3) {
            arg0.uniform3fv(arg1, getArrayF32FromWasm0(arg2, arg3));
        },
        __wbg_uniform3iv_32b9d4b3b46fc3fb: function(arg0, arg1, arg2, arg3) {
            arg0.uniform3iv(arg1, getArrayI32FromWasm0(arg2, arg3));
        },
        __wbg_uniform3iv_cb33dfef7e21e4cf: function(arg0, arg1, arg2, arg3) {
            arg0.uniform3iv(arg1, getArrayI32FromWasm0(arg2, arg3));
        },
        __wbg_uniform3uiv_4619b614eed3b03b: function(arg0, arg1, arg2, arg3) {
            arg0.uniform3uiv(arg1, getArrayU32FromWasm0(arg2, arg3));
        },
        __wbg_uniform4f_0fdc74603a961cb5: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.uniform4f(arg1, arg2, arg3, arg4, arg5);
        },
        __wbg_uniform4f_ff633df35b5cc94e: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.uniform4f(arg1, arg2, arg3, arg4, arg5);
        },
        __wbg_uniform4fv_68ac9dbef7f9bdf8: function(arg0, arg1, arg2, arg3) {
            arg0.uniform4fv(arg1, getArrayF32FromWasm0(arg2, arg3));
        },
        __wbg_uniform4fv_7376535df870aea3: function(arg0, arg1, arg2, arg3) {
            arg0.uniform4fv(arg1, getArrayF32FromWasm0(arg2, arg3));
        },
        __wbg_uniform4iv_ce9132ad6ecb1d68: function(arg0, arg1, arg2, arg3) {
            arg0.uniform4iv(arg1, getArrayI32FromWasm0(arg2, arg3));
        },
        __wbg_uniform4iv_f7af2ec6948943d4: function(arg0, arg1, arg2, arg3) {
            arg0.uniform4iv(arg1, getArrayI32FromWasm0(arg2, arg3));
        },
        __wbg_uniform4uiv_4292a20d0e4d3596: function(arg0, arg1, arg2, arg3) {
            arg0.uniform4uiv(arg1, getArrayU32FromWasm0(arg2, arg3));
        },
        __wbg_uniformBlockBinding_937f5d284b5d4fca: function(arg0, arg1, arg2, arg3) {
            arg0.uniformBlockBinding(arg1, arg2 >>> 0, arg3 >>> 0);
        },
        __wbg_uniformMatrix2fv_16670171c53575fa: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix2fv_cc942086aed65fce: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix2x3fv_af1aab8077d29a47: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix2x3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix2x4fv_81d033dbc8213f56: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix2x4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix3fv_677ba74bf760a105: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix3fv_d36ebef53bf8ae93: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix3x2fv_acd7964b2aef846e: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix3x2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix3x4fv_b2df592396e29a0a: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix3x4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix4fv_65df27ae81aac4a7: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix4fv_ad33dd8ac90a1166: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix4x2fv_d791ce5f7982d137: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix4x2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix4x3fv_190833de504c7482: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix4x3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_unmap_9455a68932e9b935: function(arg0) {
            arg0.unmap();
        },
        __wbg_unobserve_fba3a73b4a61a859: function(arg0, arg1) {
            arg0.unobserve(arg1);
        },
        __wbg_usage_609dddbf539baf5f: function(arg0) {
            const ret = arg0.usage;
            return ret;
        },
        __wbg_useProgram_6403314e6307ff8f: function(arg0, arg1) {
            arg0.useProgram(arg1);
        },
        __wbg_useProgram_b0607e62e147410b: function(arg0, arg1) {
            arg0.useProgram(arg1);
        },
        __wbg_userAgentData_31b8f893e8977e94: function(arg0) {
            const ret = arg0.userAgentData;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_userAgent_609f939440dc6b62: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.userAgent;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_valueOf_9760e35383abbb01: function(arg0) {
            const ret = arg0.valueOf();
            return ret;
        },
        __wbg_value_437a9fa72203180a: function(arg0) {
            const ret = arg0.value;
            return ret;
        },
        __wbg_vertexAttribDivisorANGLE_49500429f99e1d27: function(arg0, arg1, arg2) {
            arg0.vertexAttribDivisorANGLE(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_vertexAttribDivisor_406c4f2dab66050b: function(arg0, arg1, arg2) {
            arg0.vertexAttribDivisor(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_vertexAttribIPointer_a64fdd378b987c16: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.vertexAttribIPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
        },
        __wbg_vertexAttribPointer_89754c61239e5837: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
        },
        __wbg_vertexAttribPointer_dfec25e05e323ba4: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
        },
        __wbg_videoHeight_c6473e0876716097: function(arg0) {
            const ret = arg0.videoHeight;
            return ret;
        },
        __wbg_videoWidth_e8cbf52940ff039d: function(arg0) {
            const ret = arg0.videoWidth;
            return ret;
        },
        __wbg_viewport_325ef6f6b074c24f: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.viewport(arg1, arg2, arg3, arg4);
        },
        __wbg_viewport_b1858453ab05f289: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.viewport(arg1, arg2, arg3, arg4);
        },
        __wbg_visibilityState_cbab2cc123aa13ec: function(arg0) {
            const ret = arg0.visibilityState;
            return (__wbindgen_enum_VisibilityState.indexOf(ret) + 1 || 3) - 1;
        },
        __wbg_webkitExitFullscreen_f487871f11a8185e: function(arg0) {
            arg0.webkitExitFullscreen();
        },
        __wbg_webkitFullscreenElement_4055d847f8ff064e: function(arg0) {
            const ret = arg0.webkitFullscreenElement;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_webkitRequestFullscreen_c4ec4df7be373ffd: function(arg0) {
            arg0.webkitRequestFullscreen();
        },
        __wbg_width_0b85c08524c203ea: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbg_width_3aacf063073c2757: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbg_width_5adcb07d04d08bdf: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbg_width_8d02fb9e26a75e0d: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbg_width_c8191b3a9df04090: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbg_width_ddbe321b233b5921: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbg_writeBuffer_2384abff9a0faef7: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.writeBuffer(arg1, arg2, getArrayU8FromWasm0(arg3, arg4), arg5, arg6);
        }, arguments); },
        __wbg_writeText_41e0b9b209591a06: function(arg0, arg1, arg2) {
            const ret = arg0.writeText(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_writeTexture_d42ce6ec94b2c6ca: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.writeTexture(arg1, getArrayU8FromWasm0(arg2, arg3), arg4, arg5);
        }, arguments); },
        __wbg_x_0083194d4284e4b7: function(arg0) {
            const ret = arg0.x;
            return ret;
        },
        __wbg_y_749e1551b16245f8: function(arg0) {
            const ret = arg0.y;
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 134350, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__1_);
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 178064, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___basisu_c_sys_8319643fc44298e3___web__binding__Basisu__core_c0933739ca4b745e___result__Result_____wasm_bindgen_5c215077723ed606___JsError___true_);
            return ret;
        },
        __wbindgen_cast_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 216038, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__2_);
            return ret;
        },
        __wbindgen_cast_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 225150, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue__core_c0933739ca4b745e___result__Result_____wasm_bindgen_5c215077723ed606___JsError___true_);
            return ret;
        },
        __wbindgen_cast_0000000000000005: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 24799, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true_);
            return ret;
        },
        __wbindgen_cast_0000000000000006: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Array<any>"), NamedExternref("ResizeObserver")], shim_idx: 24802, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___js_sys_22231fab66d6c031___Array__web_sys_289a6338de913506___features__gen_ResizeObserver__ResizeObserver______true_);
            return ret;
        },
        __wbindgen_cast_0000000000000007: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Array<any>")], shim_idx: 24799, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__6);
            return ret;
        },
        __wbindgen_cast_0000000000000008: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 24799, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__7);
            return ret;
        },
        __wbindgen_cast_0000000000000009: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("FocusEvent")], shim_idx: 24799, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__8);
            return ret;
        },
        __wbindgen_cast_000000000000000a: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("GPUUncapturedErrorEvent")], shim_idx: 134350, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__1__9);
            return ret;
        },
        __wbindgen_cast_000000000000000b: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("KeyboardEvent")], shim_idx: 24799, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__10);
            return ret;
        },
        __wbindgen_cast_000000000000000c: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("PageTransitionEvent")], shim_idx: 24799, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__11);
            return ret;
        },
        __wbindgen_cast_000000000000000d: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("PointerEvent")], shim_idx: 24799, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__12);
            return ret;
        },
        __wbindgen_cast_000000000000000e: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("WheelEvent")], shim_idx: 24799, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__13);
            return ret;
        },
        __wbindgen_cast_000000000000000f: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Option(NamedExternref("Blob"))], shim_idx: 24797, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke___core_c0933739ca4b745e___option__Option_web_sys_289a6338de913506___features__gen_Blob__Blob_______true_);
            return ret;
        },
        __wbindgen_cast_0000000000000010: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 106620, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke_______true__1_);
            return ret;
        },
        __wbindgen_cast_0000000000000011: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 24810, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen_5c215077723ed606___convert__closures_____invoke_______true_);
            return ret;
        },
        __wbindgen_cast_0000000000000012: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_0000000000000013: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(F32)) -> NamedExternref("Float32Array")`.
            const ret = getArrayF32FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_0000000000000014: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(I16)) -> NamedExternref("Int16Array")`.
            const ret = getArrayI16FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_0000000000000015: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(I32)) -> NamedExternref("Int32Array")`.
            const ret = getArrayI32FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_0000000000000016: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(I8)) -> NamedExternref("Int8Array")`.
            const ret = getArrayI8FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_0000000000000017: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U16)) -> NamedExternref("Uint16Array")`.
            const ret = getArrayU16FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_0000000000000018: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U32)) -> NamedExternref("Uint32Array")`.
            const ret = getArrayU32FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_0000000000000019: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
            const ret = getArrayU8FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_000000000000001a: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./lol_bg.js": import0,
        "./snippets/basisu_c_sys-273c0aeb51b100af/wasm/basisu_transcoder.js": import1,
    };
}

const lAudioContext = (typeof AudioContext !== 'undefined' ? AudioContext : (typeof webkitAudioContext !== 'undefined' ? webkitAudioContext : undefined));
function wasm_bindgen_5c215077723ed606___convert__closures_____invoke_______true__1_(arg0, arg1) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke_______true__1_(arg0, arg1);
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke_______true_(arg0, arg1) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke_______true_(arg0, arg1);
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__1_(arg0, arg1, arg2) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__1_(arg0, arg1, arg2);
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__2_(arg0, arg1, arg2) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__2_(arg0, arg1, arg2);
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true_(arg0, arg1, arg2) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true_(arg0, arg1, arg2);
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__6(arg0, arg1, arg2) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__6(arg0, arg1, arg2);
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__7(arg0, arg1, arg2) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__7(arg0, arg1, arg2);
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__8(arg0, arg1, arg2) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__8(arg0, arg1, arg2);
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__1__9(arg0, arg1, arg2) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__1__9(arg0, arg1, arg2);
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__10(arg0, arg1, arg2) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__10(arg0, arg1, arg2);
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__11(arg0, arg1, arg2) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__11(arg0, arg1, arg2);
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__12(arg0, arg1, arg2) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__12(arg0, arg1, arg2);
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__13(arg0, arg1, arg2) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue______true__13(arg0, arg1, arg2);
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___basisu_c_sys_8319643fc44298e3___web__binding__Basisu__core_c0933739ca4b745e___result__Result_____wasm_bindgen_5c215077723ed606___JsError___true_(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___basisu_c_sys_8319643fc44298e3___web__binding__Basisu__core_c0933739ca4b745e___result__Result_____wasm_bindgen_5c215077723ed606___JsError___true_(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue__core_c0933739ca4b745e___result__Result_____wasm_bindgen_5c215077723ed606___JsError___true_(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___wasm_bindgen_5c215077723ed606___JsValue__core_c0933739ca4b745e___result__Result_____wasm_bindgen_5c215077723ed606___JsError___true_(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___js_sys_22231fab66d6c031___Array__web_sys_289a6338de913506___features__gen_ResizeObserver__ResizeObserver______true_(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___js_sys_22231fab66d6c031___Array__web_sys_289a6338de913506___features__gen_ResizeObserver__ResizeObserver______true_(arg0, arg1, arg2, arg3);
}

function wasm_bindgen_5c215077723ed606___convert__closures_____invoke___core_c0933739ca4b745e___option__Option_web_sys_289a6338de913506___features__gen_Blob__Blob_______true_(arg0, arg1, arg2) {
    wasm.wasm_bindgen_5c215077723ed606___convert__closures_____invoke___core_c0933739ca4b745e___option__Option_web_sys_289a6338de913506___features__gen_Blob__Blob_______true_(arg0, arg1, isLikeNone(arg2) ? 0 : addToExternrefTable0(arg2));
}


const __wbindgen_enum_GamepadMappingType = ["", "standard"];


const __wbindgen_enum_GpuAddressMode = ["clamp-to-edge", "repeat", "mirror-repeat"];


const __wbindgen_enum_GpuBlendFactor = ["zero", "one", "src", "one-minus-src", "src-alpha", "one-minus-src-alpha", "dst", "one-minus-dst", "dst-alpha", "one-minus-dst-alpha", "src-alpha-saturated", "constant", "one-minus-constant", "src1", "one-minus-src1", "src1-alpha", "one-minus-src1-alpha"];


const __wbindgen_enum_GpuBlendOperation = ["add", "subtract", "reverse-subtract", "min", "max"];


const __wbindgen_enum_GpuBufferBindingType = ["uniform", "storage", "read-only-storage"];


const __wbindgen_enum_GpuCanvasAlphaMode = ["opaque", "premultiplied"];


const __wbindgen_enum_GpuCompareFunction = ["never", "less", "equal", "less-equal", "greater", "not-equal", "greater-equal", "always"];


const __wbindgen_enum_GpuCullMode = ["none", "front", "back"];


const __wbindgen_enum_GpuDeviceLostReason = ["unknown", "destroyed"];


const __wbindgen_enum_GpuErrorFilter = ["validation", "out-of-memory", "internal"];


const __wbindgen_enum_GpuFilterMode = ["nearest", "linear"];


const __wbindgen_enum_GpuFrontFace = ["ccw", "cw"];


const __wbindgen_enum_GpuIndexFormat = ["uint16", "uint32"];


const __wbindgen_enum_GpuLoadOp = ["load", "clear"];


const __wbindgen_enum_GpuMipmapFilterMode = ["nearest", "linear"];


const __wbindgen_enum_GpuPowerPreference = ["low-power", "high-performance"];


const __wbindgen_enum_GpuPrimitiveTopology = ["point-list", "line-list", "line-strip", "triangle-list", "triangle-strip"];


const __wbindgen_enum_GpuSamplerBindingType = ["filtering", "non-filtering", "comparison"];


const __wbindgen_enum_GpuStencilOperation = ["keep", "zero", "replace", "invert", "increment-clamp", "decrement-clamp", "increment-wrap", "decrement-wrap"];


const __wbindgen_enum_GpuStorageTextureAccess = ["write-only", "read-only", "read-write"];


const __wbindgen_enum_GpuStoreOp = ["store", "discard"];


const __wbindgen_enum_GpuTextureAspect = ["all", "stencil-only", "depth-only"];


const __wbindgen_enum_GpuTextureDimension = ["1d", "2d", "3d"];


const __wbindgen_enum_GpuTextureFormat = ["r8unorm", "r8snorm", "r8uint", "r8sint", "r16uint", "r16sint", "r16float", "rg8unorm", "rg8snorm", "rg8uint", "rg8sint", "r32uint", "r32sint", "r32float", "rg16uint", "rg16sint", "rg16float", "rgba8unorm", "rgba8unorm-srgb", "rgba8snorm", "rgba8uint", "rgba8sint", "bgra8unorm", "bgra8unorm-srgb", "rgb9e5ufloat", "rgb10a2uint", "rgb10a2unorm", "rg11b10ufloat", "rg32uint", "rg32sint", "rg32float", "rgba16uint", "rgba16sint", "rgba16float", "rgba32uint", "rgba32sint", "rgba32float", "stencil8", "depth16unorm", "depth24plus", "depth24plus-stencil8", "depth32float", "depth32float-stencil8", "bc1-rgba-unorm", "bc1-rgba-unorm-srgb", "bc2-rgba-unorm", "bc2-rgba-unorm-srgb", "bc3-rgba-unorm", "bc3-rgba-unorm-srgb", "bc4-r-unorm", "bc4-r-snorm", "bc5-rg-unorm", "bc5-rg-snorm", "bc6h-rgb-ufloat", "bc6h-rgb-float", "bc7-rgba-unorm", "bc7-rgba-unorm-srgb", "etc2-rgb8unorm", "etc2-rgb8unorm-srgb", "etc2-rgb8a1unorm", "etc2-rgb8a1unorm-srgb", "etc2-rgba8unorm", "etc2-rgba8unorm-srgb", "eac-r11unorm", "eac-r11snorm", "eac-rg11unorm", "eac-rg11snorm", "astc-4x4-unorm", "astc-4x4-unorm-srgb", "astc-5x4-unorm", "astc-5x4-unorm-srgb", "astc-5x5-unorm", "astc-5x5-unorm-srgb", "astc-6x5-unorm", "astc-6x5-unorm-srgb", "astc-6x6-unorm", "astc-6x6-unorm-srgb", "astc-8x5-unorm", "astc-8x5-unorm-srgb", "astc-8x6-unorm", "astc-8x6-unorm-srgb", "astc-8x8-unorm", "astc-8x8-unorm-srgb", "astc-10x5-unorm", "astc-10x5-unorm-srgb", "astc-10x6-unorm", "astc-10x6-unorm-srgb", "astc-10x8-unorm", "astc-10x8-unorm-srgb", "astc-10x10-unorm", "astc-10x10-unorm-srgb", "astc-12x10-unorm", "astc-12x10-unorm-srgb", "astc-12x12-unorm", "astc-12x12-unorm-srgb"];


const __wbindgen_enum_GpuTextureSampleType = ["float", "unfilterable-float", "depth", "sint", "uint"];


const __wbindgen_enum_GpuTextureViewDimension = ["1d", "2d", "2d-array", "cube", "cube-array", "3d"];


const __wbindgen_enum_GpuVertexFormat = ["uint8", "uint8x2", "uint8x4", "sint8", "sint8x2", "sint8x4", "unorm8", "unorm8x2", "unorm8x4", "snorm8", "snorm8x2", "snorm8x4", "uint16", "uint16x2", "uint16x4", "sint16", "sint16x2", "sint16x4", "unorm16", "unorm16x2", "unorm16x4", "snorm16", "snorm16x2", "snorm16x4", "float16", "float16x2", "float16x4", "float32", "float32x2", "float32x3", "float32x4", "uint32", "uint32x2", "uint32x3", "uint32x4", "sint32", "sint32x2", "sint32x3", "sint32x4", "unorm10-10-10-2", "unorm8x4-bgra"];


const __wbindgen_enum_GpuVertexStepMode = ["vertex", "instance"];


const __wbindgen_enum_PremultiplyAlpha = ["none", "premultiply", "default"];


const __wbindgen_enum_ResizeObserverBoxOptions = ["border-box", "content-box", "device-pixel-content-box"];


const __wbindgen_enum_VisibilityState = ["hidden", "visible"];
const StreamConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_streamconfig_free(ptr >>> 0, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => wasm.__wbindgen_destroy_closure(state.a, state.b));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayI16FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt16ArrayMemory0().subarray(ptr / 2, ptr / 2 + len);
}

function getArrayI32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayI8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getArrayU16FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint16ArrayMemory0().subarray(ptr / 2, ptr / 2 + len);
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getClampedArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ClampedArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat32ArrayMemory0 = null;
function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

let cachedInt16ArrayMemory0 = null;
function getInt16ArrayMemory0() {
    if (cachedInt16ArrayMemory0 === null || cachedInt16ArrayMemory0.byteLength === 0) {
        cachedInt16ArrayMemory0 = new Int16Array(wasm.memory.buffer);
    }
    return cachedInt16ArrayMemory0;
}

let cachedInt32ArrayMemory0 = null;
function getInt32ArrayMemory0() {
    if (cachedInt32ArrayMemory0 === null || cachedInt32ArrayMemory0.byteLength === 0) {
        cachedInt32ArrayMemory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32ArrayMemory0;
}

let cachedInt8ArrayMemory0 = null;
function getInt8ArrayMemory0() {
    if (cachedInt8ArrayMemory0 === null || cachedInt8ArrayMemory0.byteLength === 0) {
        cachedInt8ArrayMemory0 = new Int8Array(wasm.memory.buffer);
    }
    return cachedInt8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint16ArrayMemory0 = null;
function getUint16ArrayMemory0() {
    if (cachedUint16ArrayMemory0 === null || cachedUint16ArrayMemory0.byteLength === 0) {
        cachedUint16ArrayMemory0 = new Uint16Array(wasm.memory.buffer);
    }
    return cachedUint16ArrayMemory0;
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedUint8ClampedArrayMemory0 = null;
function getUint8ClampedArrayMemory0() {
    if (cachedUint8ClampedArrayMemory0 === null || cachedUint8ClampedArrayMemory0.byteLength === 0) {
        cachedUint8ClampedArrayMemory0 = new Uint8ClampedArray(wasm.memory.buffer);
    }
    return cachedUint8ClampedArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeMutClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_destroy_closure(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedInt16ArrayMemory0 = null;
    cachedInt32ArrayMemory0 = null;
    cachedInt8ArrayMemory0 = null;
    cachedUint16ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    cachedUint8ClampedArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('lol_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
