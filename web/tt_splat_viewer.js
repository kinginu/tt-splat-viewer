/**
 * Called from JS (the `b` key) to cycle the background preset.
 */
export function cycle_background() {
    wasm.cycle_background();
}

/**
 * Called from JS drag-and-drop with the target pane (0 = left/WSR, 1 = right/3DGS) and file bytes.
 * @param {number} pane
 * @param {Uint8Array} bytes
 * @returns {boolean}
 */
export function load_ply_into(pane, bytes) {
    const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.load_ply_into(pane, ptr0, len0);
    return ret !== 0;
}

/**
 * The view rotation as 9 floats (column-major): columns are the world X/Y/Z axes in camera space
 * (x right, y down, z into screen). JS uses these to draw the orientation gizmo outside the panes.
 * @returns {Float32Array}
 */
export function view_basis() {
    const ret = wasm.view_basis();
    var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
    return v1;
}

/**
 * WASM entry: `wasm_bindgen(start)` must be sync, so kick the async `run()` onto the microtask queue.
 */
export function wasm_start() {
    wasm.wasm_start();
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Window_4826de6dc685a78e: function(arg0) {
            const ret = arg0.Window;
            return ret;
        },
        __wbg_Window_b33eab7646a0a21f: function(arg0) {
            const ret = arg0.Window;
            return ret;
        },
        __wbg_WorkerGlobalScope_8b38f06124ea34df: function(arg0) {
            const ret = arg0.WorkerGlobalScope;
            return ret;
        },
        __wbg___wbindgen_debug_string_8a447059637473e2: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_is_function_acc5528be2b923f2: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_object_0beba4a1980d3eea: function(arg0) {
            const val = arg0;
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_undefined_721f8decd50c87a3: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_string_get_71bb4348194e31f0: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_ea4887a5f8f9a9db: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_33c39e13d73b25f6: function(arg0) {
            arg0._wbg_cb_unref();
        },
        __wbg_abort_6e6ea7d259504afc: function(arg0) {
            arg0.abort();
        },
        __wbg_activeElement_39e967783d67360b: function(arg0) {
            const ret = arg0.activeElement;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_addEventListener_ea90bc131475777e: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_addListener_ddaa585636995443: function() { return handleError(function (arg0, arg1) {
            arg0.addListener(arg1);
        }, arguments); },
        __wbg_altKey_7412dd64f8c921c7: function(arg0) {
            const ret = arg0.altKey;
            return ret;
        },
        __wbg_altKey_bd27feb84dabed76: function(arg0) {
            const ret = arg0.altKey;
            return ret;
        },
        __wbg_appendChild_acb7691406591783: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.appendChild(arg1);
            return ret;
        }, arguments); },
        __wbg_beginComputePass_d319df000ff35c59: function(arg0, arg1) {
            const ret = arg0.beginComputePass(arg1);
            return ret;
        },
        __wbg_beginRenderPass_9626dca71d2ae1ac: function(arg0, arg1) {
            const ret = arg0.beginRenderPass(arg1);
            return ret;
        },
        __wbg_blockSize_ec8ab846ac2b2395: function(arg0) {
            const ret = arg0.blockSize;
            return ret;
        },
        __wbg_body_c94f9310613c153b: function(arg0) {
            const ret = arg0.body;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_buffer_9e4d98d0766fb908: function(arg0) {
            const ret = arg0.buffer;
            return ret;
        },
        __wbg_button_bb3285fcdf965178: function(arg0) {
            const ret = arg0.button;
            return ret;
        },
        __wbg_buttons_c15069bb056ecbe0: function(arg0) {
            const ret = arg0.buttons;
            return ret;
        },
        __wbg_cancelAnimationFrame_32c0ef91e4077228: function() { return handleError(function (arg0, arg1) {
            arg0.cancelAnimationFrame(arg1);
        }, arguments); },
        __wbg_cancelIdleCallback_d2ed70e82f4d3a29: function(arg0, arg1) {
            arg0.cancelIdleCallback(arg1 >>> 0);
        },
        __wbg_catch_2bb4b6f2fafb00e7: function(arg0, arg1) {
            const ret = arg0.catch(arg1);
            return ret;
        },
        __wbg_clearBuffer_37b257a482c9cebe: function(arg0, arg1, arg2, arg3) {
            arg0.clearBuffer(arg1, arg2, arg3);
        },
        __wbg_clearBuffer_7cf11127ecf8378b: function(arg0, arg1, arg2) {
            arg0.clearBuffer(arg1, arg2);
        },
        __wbg_clearTimeout_ef73c3ecbb0ece69: function(arg0, arg1) {
            arg0.clearTimeout(arg1);
        },
        __wbg_close_38d22e2ee9732188: function(arg0) {
            arg0.close();
        },
        __wbg_code_87bb99fdec02d313: function(arg0, arg1) {
            const ret = arg1.code;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_configure_6e71a9640e3973e3: function(arg0, arg1) {
            arg0.configure(arg1);
        },
        __wbg_contains_aeb13d2d343b24e8: function(arg0, arg1) {
            const ret = arg0.contains(arg1);
            return ret;
        },
        __wbg_contentRect_0fb8c8900d689a0c: function(arg0) {
            const ret = arg0.contentRect;
            return ret;
        },
        __wbg_copyBufferToBuffer_54083aa13ccfdf51: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.copyBufferToBuffer(arg1, arg2, arg3, arg4, arg5);
        },
        __wbg_copyBufferToTexture_2f41b115494e0e3c: function(arg0, arg1, arg2, arg3) {
            arg0.copyBufferToTexture(arg1, arg2, arg3);
        },
        __wbg_copyExternalImageToTexture_ab34dd5dfd7c6e4c: function(arg0, arg1, arg2, arg3) {
            arg0.copyExternalImageToTexture(arg1, arg2, arg3);
        },
        __wbg_copyTextureToBuffer_63a2fea47ec5ca70: function(arg0, arg1, arg2, arg3) {
            arg0.copyTextureToBuffer(arg1, arg2, arg3);
        },
        __wbg_copyTextureToTexture_d5d6a52cdb793533: function(arg0, arg1, arg2, arg3) {
            arg0.copyTextureToTexture(arg1, arg2, arg3);
        },
        __wbg_createBindGroupLayout_325e06222c562656: function(arg0, arg1) {
            const ret = arg0.createBindGroupLayout(arg1);
            return ret;
        },
        __wbg_createBindGroup_6950418662ed4fc7: function(arg0, arg1) {
            const ret = arg0.createBindGroup(arg1);
            return ret;
        },
        __wbg_createBuffer_2800d8992e808725: function(arg0, arg1) {
            const ret = arg0.createBuffer(arg1);
            return ret;
        },
        __wbg_createCommandEncoder_02de2e4f29274ccb: function(arg0, arg1) {
            const ret = arg0.createCommandEncoder(arg1);
            return ret;
        },
        __wbg_createComputePipeline_86e7ef974737f649: function(arg0, arg1) {
            const ret = arg0.createComputePipeline(arg1);
            return ret;
        },
        __wbg_createElement_9e23ac95e40e302c: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_createPipelineLayout_1463d3f115498e26: function(arg0, arg1) {
            const ret = arg0.createPipelineLayout(arg1);
            return ret;
        },
        __wbg_createQuerySet_ca4fcb2a89b2ffa7: function(arg0, arg1) {
            const ret = arg0.createQuerySet(arg1);
            return ret;
        },
        __wbg_createRenderBundleEncoder_fa9b41d318d1003e: function(arg0, arg1) {
            const ret = arg0.createRenderBundleEncoder(arg1);
            return ret;
        },
        __wbg_createRenderPipeline_fb5527b97b1bf281: function(arg0, arg1) {
            const ret = arg0.createRenderPipeline(arg1);
            return ret;
        },
        __wbg_createSampler_ceaa520fd5c5b097: function(arg0, arg1) {
            const ret = arg0.createSampler(arg1);
            return ret;
        },
        __wbg_createShaderModule_78f1dfca66a3bc84: function(arg0, arg1) {
            const ret = arg0.createShaderModule(arg1);
            return ret;
        },
        __wbg_createTexture_e3f915f0857f7aa5: function(arg0, arg1) {
            const ret = arg0.createTexture(arg1);
            return ret;
        },
        __wbg_createView_1552f801507d195f: function(arg0, arg1) {
            const ret = arg0.createView(arg1);
            return ret;
        },
        __wbg_ctrlKey_00097886a21597a7: function(arg0) {
            const ret = arg0.ctrlKey;
            return ret;
        },
        __wbg_ctrlKey_e2bcba26214642c1: function(arg0) {
            const ret = arg0.ctrlKey;
            return ret;
        },
        __wbg_debug_efaf8c48cff2394a: function(arg0) {
            console.debug(arg0);
        },
        __wbg_deltaMode_af2c1b6d0e654eb8: function(arg0) {
            const ret = arg0.deltaMode;
            return ret;
        },
        __wbg_deltaX_002e415d6311b335: function(arg0) {
            const ret = arg0.deltaX;
            return ret;
        },
        __wbg_deltaY_5c6a1e0334a84e24: function(arg0) {
            const ret = arg0.deltaY;
            return ret;
        },
        __wbg_destroy_326ff7c333e93983: function(arg0) {
            arg0.destroy();
        },
        __wbg_destroy_8d8834f6921e3011: function(arg0) {
            arg0.destroy();
        },
        __wbg_destroy_912f88f4af8749b5: function(arg0) {
            arg0.destroy();
        },
        __wbg_devicePixelContentBoxSize_442c861e416326ba: function(arg0) {
            const ret = arg0.devicePixelContentBoxSize;
            return ret;
        },
        __wbg_devicePixelRatio_8e5ca44720abafec: function(arg0) {
            const ret = arg0.devicePixelRatio;
            return ret;
        },
        __wbg_disconnect_970650440e4877a1: function(arg0) {
            arg0.disconnect();
        },
        __wbg_disconnect_fff596d884a74ee7: function(arg0) {
            arg0.disconnect();
        },
        __wbg_dispatchWorkgroupsIndirect_ef6bc658d1693627: function(arg0, arg1, arg2) {
            arg0.dispatchWorkgroupsIndirect(arg1, arg2);
        },
        __wbg_dispatchWorkgroups_61749c845f263b1b: function(arg0, arg1, arg2, arg3) {
            arg0.dispatchWorkgroups(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0);
        },
        __wbg_document_2634180a4c694068: function(arg0) {
            const ret = arg0.document;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_drawIndexedIndirect_46d92ced872d6225: function(arg0, arg1, arg2) {
            arg0.drawIndexedIndirect(arg1, arg2);
        },
        __wbg_drawIndexedIndirect_704613509c572c9b: function(arg0, arg1, arg2) {
            arg0.drawIndexedIndirect(arg1, arg2);
        },
        __wbg_drawIndexed_f3cf21dad3761daf: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
        },
        __wbg_drawIndexed_fd8751bd851b98f6: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
        },
        __wbg_drawIndirect_4648923b5c3a62f3: function(arg0, arg1, arg2) {
            arg0.drawIndirect(arg1, arg2);
        },
        __wbg_drawIndirect_956cb7ecab3f16cc: function(arg0, arg1, arg2) {
            arg0.drawIndirect(arg1, arg2);
        },
        __wbg_draw_d074ff7953e205b6: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_draw_d802068e48a92da8: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_end_010c73cc9ea83929: function(arg0) {
            arg0.end();
        },
        __wbg_end_58ff87e41ffd623e: function(arg0) {
            arg0.end();
        },
        __wbg_error_6fe2f1675730f2bd: function(arg0, arg1) {
            console.error(arg0, arg1);
        },
        __wbg_error_933f449d72fef598: function(arg0) {
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
        __wbg_error_f617cf885723ab9a: function(arg0) {
            const ret = arg0.error;
            return ret;
        },
        __wbg_executeBundles_482ce9b46e0c2d2d: function(arg0, arg1) {
            arg0.executeBundles(arg1);
        },
        __wbg_features_03e9e972f4fc0381: function(arg0) {
            const ret = arg0.features;
            return ret;
        },
        __wbg_features_7372f297c9ce5d88: function(arg0) {
            const ret = arg0.features;
            return ret;
        },
        __wbg_finish_47376b623ccfc3ad: function(arg0, arg1) {
            const ret = arg0.finish(arg1);
            return ret;
        },
        __wbg_finish_5273dda2b06e066c: function(arg0, arg1) {
            const ret = arg0.finish(arg1);
            return ret;
        },
        __wbg_finish_b5c96c0dbd283361: function(arg0) {
            const ret = arg0.finish();
            return ret;
        },
        __wbg_finish_d25194e256d38eb5: function(arg0) {
            const ret = arg0.finish();
            return ret;
        },
        __wbg_focus_c80921c19b28520b: function() { return handleError(function (arg0) {
            arg0.focus();
        }, arguments); },
        __wbg_fullscreenElement_0961e3f245820d9a: function(arg0) {
            const ret = arg0.fullscreenElement;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_getBindGroupLayout_109c3bc088d9eef8: function(arg0, arg1) {
            const ret = arg0.getBindGroupLayout(arg1 >>> 0);
            return ret;
        },
        __wbg_getBindGroupLayout_8127da82d1305265: function(arg0, arg1) {
            const ret = arg0.getBindGroupLayout(arg1 >>> 0);
            return ret;
        },
        __wbg_getCoalescedEvents_28e845a86e8b7ce7: function(arg0) {
            const ret = arg0.getCoalescedEvents;
            return ret;
        },
        __wbg_getCoalescedEvents_b5b03d5636ebc2f4: function(arg0) {
            const ret = arg0.getCoalescedEvents();
            return ret;
        },
        __wbg_getComputedStyle_3fed4438a3d089a6: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.getComputedStyle(arg1);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getContext_486aab500e1c34c9: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getContext_70c2d1bed75d4122: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getCurrentTexture_86217983a38f2e85: function(arg0) {
            const ret = arg0.getCurrentTexture();
            return ret;
        },
        __wbg_getMappedRange_1d21a07781078b5c: function(arg0, arg1, arg2) {
            const ret = arg0.getMappedRange(arg1, arg2);
            return ret;
        },
        __wbg_getOwnPropertyDescriptor_7f06ccc7c66b4ef1: function(arg0, arg1) {
            const ret = Object.getOwnPropertyDescriptor(arg0, arg1);
            return ret;
        },
        __wbg_getPreferredCanvasFormat_28c07d9efd0d5536: function(arg0) {
            const ret = arg0.getPreferredCanvasFormat();
            return (__wbindgen_enum_GpuTextureFormat.indexOf(ret) + 1 || 96) - 1;
        },
        __wbg_getPropertyValue_25192bdc8db9053c: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg1.getPropertyValue(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_get_197a3fe98f169e38: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        },
        __wbg_get_37b48b8fa52d1f2c: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_gpu_d80e8bd525bd4ba8: function(arg0) {
            const ret = arg0.gpu;
            return ret;
        },
        __wbg_has_f1a2e53cd2b230bb: function(arg0, arg1, arg2) {
            const ret = arg0.has(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_height_e9afb506f31052a9: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_info_e2925fd3ec267c1a: function(arg0) {
            console.info(arg0);
        },
        __wbg_inlineSize_8895590ca1875f8d: function(arg0) {
            const ret = arg0.inlineSize;
            return ret;
        },
        __wbg_instanceof_GpuAdapter_756c7c5930793c56: function(arg0) {
            let result;
            try {
                result = arg0 instanceof GPUAdapter;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuCanvasContext_97084852a24cda42: function(arg0) {
            let result;
            try {
                result = arg0 instanceof GPUCanvasContext;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuDeviceLostInfo_23dc505e0f4c6916: function(arg0) {
            let result;
            try {
                result = arg0 instanceof GPUDeviceLostInfo;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuOutOfMemoryError_a10b029451767dba: function(arg0) {
            let result;
            try {
                result = arg0 instanceof GPUOutOfMemoryError;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuValidationError_2d9aaa1f6d534392: function(arg0) {
            let result;
            try {
                result = arg0 instanceof GPUValidationError;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Object_60be3eaa7a661141: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Object;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_0d356b88a2f77c42: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_isIntersecting_e02cd273824aee08: function(arg0) {
            const ret = arg0.isIntersecting;
            return ret;
        },
        __wbg_is_de5b366c746e004c: function(arg0, arg1) {
            const ret = Object.is(arg0, arg1);
            return ret;
        },
        __wbg_key_d6b3d2cce3d41872: function(arg0, arg1) {
            const ret = arg1.key;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_label_9dcdd9f98958290b: function(arg0, arg1) {
            const ret = arg1.label;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_length_589238bdcf171f0e: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_c6054974c0a6cdb9: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_limits_7b27ae8ed804308f: function(arg0) {
            const ret = arg0.limits;
            return ret;
        },
        __wbg_limits_a93eb0deb0a91c85: function(arg0) {
            const ret = arg0.limits;
            return ret;
        },
        __wbg_location_8fedbe6d61285cd1: function(arg0) {
            const ret = arg0.location;
            return ret;
        },
        __wbg_log_6b5af08dd293697f: function(arg0) {
            console.log(arg0);
        },
        __wbg_lost_7a42a99b8fce6015: function(arg0) {
            const ret = arg0.lost;
            return ret;
        },
        __wbg_mapAsync_5f6365c5e7b25039: function(arg0, arg1, arg2, arg3) {
            const ret = arg0.mapAsync(arg1 >>> 0, arg2, arg3);
            return ret;
        },
        __wbg_matchMedia_049ef5c8f8f904f2: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.matchMedia(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_matches_a4c6202857b1dac9: function(arg0) {
            const ret = arg0.matches;
            return ret;
        },
        __wbg_maxBindGroups_9d47224ed34d55ca: function(arg0) {
            const ret = arg0.maxBindGroups;
            return ret;
        },
        __wbg_maxBindingsPerBindGroup_0a6b94af0d5ad56e: function(arg0) {
            const ret = arg0.maxBindingsPerBindGroup;
            return ret;
        },
        __wbg_maxBufferSize_2dbf89ce0d1600b2: function(arg0) {
            const ret = arg0.maxBufferSize;
            return ret;
        },
        __wbg_maxColorAttachmentBytesPerSample_02271f3d221491c0: function(arg0) {
            const ret = arg0.maxColorAttachmentBytesPerSample;
            return ret;
        },
        __wbg_maxColorAttachments_9071248ced72fd33: function(arg0) {
            const ret = arg0.maxColorAttachments;
            return ret;
        },
        __wbg_maxComputeInvocationsPerWorkgroup_5bb9876b3b7942b4: function(arg0) {
            const ret = arg0.maxComputeInvocationsPerWorkgroup;
            return ret;
        },
        __wbg_maxComputeWorkgroupSizeX_660bf742859e09e2: function(arg0) {
            const ret = arg0.maxComputeWorkgroupSizeX;
            return ret;
        },
        __wbg_maxComputeWorkgroupSizeY_43fdb06f9b396f89: function(arg0) {
            const ret = arg0.maxComputeWorkgroupSizeY;
            return ret;
        },
        __wbg_maxComputeWorkgroupSizeZ_2d9b6e7006222a13: function(arg0) {
            const ret = arg0.maxComputeWorkgroupSizeZ;
            return ret;
        },
        __wbg_maxComputeWorkgroupStorageSize_f19d9f1c077a8374: function(arg0) {
            const ret = arg0.maxComputeWorkgroupStorageSize;
            return ret;
        },
        __wbg_maxComputeWorkgroupsPerDimension_139d2a325187cb44: function(arg0) {
            const ret = arg0.maxComputeWorkgroupsPerDimension;
            return ret;
        },
        __wbg_maxDynamicStorageBuffersPerPipelineLayout_99faf9f0a613dded: function(arg0) {
            const ret = arg0.maxDynamicStorageBuffersPerPipelineLayout;
            return ret;
        },
        __wbg_maxDynamicUniformBuffersPerPipelineLayout_66cd945561e2f169: function(arg0) {
            const ret = arg0.maxDynamicUniformBuffersPerPipelineLayout;
            return ret;
        },
        __wbg_maxInterStageShaderComponents_8790c8053bd90b1d: function(arg0) {
            const ret = arg0.maxInterStageShaderComponents;
            return ret;
        },
        __wbg_maxSampledTexturesPerShaderStage_089216ded99dfb40: function(arg0) {
            const ret = arg0.maxSampledTexturesPerShaderStage;
            return ret;
        },
        __wbg_maxSamplersPerShaderStage_41af2d97ab1e7a95: function(arg0) {
            const ret = arg0.maxSamplersPerShaderStage;
            return ret;
        },
        __wbg_maxStorageBufferBindingSize_06a37b936c0545b2: function(arg0) {
            const ret = arg0.maxStorageBufferBindingSize;
            return ret;
        },
        __wbg_maxStorageBuffersPerShaderStage_0ca5012237bc1ab8: function(arg0) {
            const ret = arg0.maxStorageBuffersPerShaderStage;
            return ret;
        },
        __wbg_maxStorageTexturesPerShaderStage_edf9f7e84f6cd0cb: function(arg0) {
            const ret = arg0.maxStorageTexturesPerShaderStage;
            return ret;
        },
        __wbg_maxTextureArrayLayers_2db698ce41f88a3c: function(arg0) {
            const ret = arg0.maxTextureArrayLayers;
            return ret;
        },
        __wbg_maxTextureDimension1D_a1b9fa937eb23aae: function(arg0) {
            const ret = arg0.maxTextureDimension1D;
            return ret;
        },
        __wbg_maxTextureDimension2D_dfd3b6073a25f3e8: function(arg0) {
            const ret = arg0.maxTextureDimension2D;
            return ret;
        },
        __wbg_maxTextureDimension3D_88d5567f1b02f259: function(arg0) {
            const ret = arg0.maxTextureDimension3D;
            return ret;
        },
        __wbg_maxUniformBufferBindingSize_4cefee07ce14ddce: function(arg0) {
            const ret = arg0.maxUniformBufferBindingSize;
            return ret;
        },
        __wbg_maxUniformBuffersPerShaderStage_1c67b1b249ec615a: function(arg0) {
            const ret = arg0.maxUniformBuffersPerShaderStage;
            return ret;
        },
        __wbg_maxVertexAttributes_d87e12ad9c7df794: function(arg0) {
            const ret = arg0.maxVertexAttributes;
            return ret;
        },
        __wbg_maxVertexBufferArrayStride_10724ff39058f023: function(arg0) {
            const ret = arg0.maxVertexBufferArrayStride;
            return ret;
        },
        __wbg_maxVertexBuffers_575dcefaafddda96: function(arg0) {
            const ret = arg0.maxVertexBuffers;
            return ret;
        },
        __wbg_media_405c765b5e24963e: function(arg0, arg1) {
            const ret = arg1.media;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_message_0f0738bd8fc62cf3: function(arg0, arg1) {
            const ret = arg1.message;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_message_403a148c1f7dbe1c: function(arg0, arg1) {
            const ret = arg1.message;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_metaKey_1c2847b62ad062c1: function(arg0) {
            const ret = arg0.metaKey;
            return ret;
        },
        __wbg_metaKey_e661374fe349fa2f: function(arg0) {
            const ret = arg0.metaKey;
            return ret;
        },
        __wbg_minStorageBufferOffsetAlignment_c25e683e8e3d11c2: function(arg0) {
            const ret = arg0.minStorageBufferOffsetAlignment;
            return ret;
        },
        __wbg_minUniformBufferOffsetAlignment_b2342d1fe6ae40a6: function(arg0) {
            const ret = arg0.minUniformBufferOffsetAlignment;
            return ret;
        },
        __wbg_movementX_128a28f825e5e554: function(arg0) {
            const ret = arg0.movementX;
            return ret;
        },
        __wbg_movementY_10cde9bca43c7656: function(arg0) {
            const ret = arg0.movementY;
            return ret;
        },
        __wbg_navigator_017bc45e84c473cc: function(arg0) {
            const ret = arg0.navigator;
            return ret;
        },
        __wbg_navigator_935098efd1dc7fe5: function(arg0) {
            const ret = arg0.navigator;
            return ret;
        },
        __wbg_new_1e52e3c919312525: function() { return handleError(function (arg0) {
            const ret = new ResizeObserver(arg0);
            return ret;
        }, arguments); },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_new_2e117a478906f062: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_36e147a8ced3c6e0: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_51233fa2a760b272: function() { return handleError(function () {
            const ret = new AbortController();
            return ret;
        }, arguments); },
        __wbg_new_98e81f23cc202cc4: function() { return handleError(function () {
            const ret = new MessageChannel();
            return ret;
        }, arguments); },
        __wbg_new_e469d9dc75b3bc01: function() { return handleError(function (arg0) {
            const ret = new IntersectionObserver(arg0);
            return ret;
        }, arguments); },
        __wbg_new_from_slice_543b875b27789a8f: function(arg0, arg1) {
            const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_new_with_byte_offset_and_length_f2b65504a914f37a: function(arg0, arg1, arg2) {
            const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
            return ret;
        },
        __wbg_now_cace042f68c814d8: function(arg0) {
            const ret = arg0.now();
            return ret;
        },
        __wbg_observe_592b94857eed8e27: function(arg0, arg1) {
            arg0.observe(arg1);
        },
        __wbg_observe_91d5c6a617c3d390: function(arg0, arg1, arg2) {
            arg0.observe(arg1, arg2);
        },
        __wbg_observe_a987521b8fae2aaf: function(arg0, arg1) {
            arg0.observe(arg1);
        },
        __wbg_offsetX_18bd0c96e3cc256f: function(arg0) {
            const ret = arg0.offsetX;
            return ret;
        },
        __wbg_offsetY_8ad219aaab7c8785: function(arg0) {
            const ret = arg0.offsetY;
            return ret;
        },
        __wbg_onpointerrawupdate_d1a28c472d93bae1: function(arg0) {
            const ret = arg0.onpointerrawupdate;
            return ret;
        },
        __wbg_performance_5fc5a6563dcd33de: function(arg0) {
            const ret = arg0.performance;
            return ret;
        },
        __wbg_persisted_cce9b6da86c07260: function(arg0) {
            const ret = arg0.persisted;
            return ret;
        },
        __wbg_pointerId_2c4027fd33473f83: function(arg0) {
            const ret = arg0.pointerId;
            return ret;
        },
        __wbg_pointerType_2dfba7dcd3e033b4: function(arg0, arg1) {
            const ret = arg1.pointerType;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_popErrorScope_17fe83373fbcdf57: function(arg0) {
            const ret = arg0.popErrorScope();
            return ret;
        },
        __wbg_port1_aaca04ed552c16ef: function(arg0) {
            const ret = arg0.port1;
            return ret;
        },
        __wbg_port2_baea902d7925a544: function(arg0) {
            const ret = arg0.port2;
            return ret;
        },
        __wbg_postMessage_5b1dc9f8de88488a: function() { return handleError(function (arg0, arg1) {
            arg0.postMessage(arg1);
        }, arguments); },
        __wbg_postTask_f12296fcc40cd194: function(arg0, arg1, arg2) {
            const ret = arg0.postTask(arg1, arg2);
            return ret;
        },
        __wbg_pressure_727ca025d24d77ec: function(arg0) {
            const ret = arg0.pressure;
            return ret;
        },
        __wbg_preventDefault_c047d3e292b08cc4: function(arg0) {
            arg0.preventDefault();
        },
        __wbg_prototype_18f36c19df59124c: function() {
            const ret = ResizeObserverEntry.prototype;
            return ret;
        },
        __wbg_prototypesetcall_d721637c7ca66eb8: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
        },
        __wbg_pushErrorScope_cee3368e8cfdbcd7: function(arg0, arg1) {
            arg0.pushErrorScope(__wbindgen_enum_GpuErrorFilter[arg1]);
        },
        __wbg_push_f724b5db8acf89d2: function(arg0, arg1) {
            const ret = arg0.push(arg1);
            return ret;
        },
        __wbg_querySelectorAll_ffda3c891a9eb29a: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.querySelectorAll(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_queueMicrotask_1c9b3800e321a967: function(arg0) {
            const ret = arg0.queueMicrotask;
            return ret;
        },
        __wbg_queueMicrotask_311744e534a929a3: function(arg0) {
            queueMicrotask(arg0);
        },
        __wbg_queue_6c38b51b5c1a8e25: function(arg0) {
            const ret = arg0.queue;
            return ret;
        },
        __wbg_reason_1accd231be72a82c: function(arg0) {
            const ret = arg0.reason;
            return (__wbindgen_enum_GpuDeviceLostReason.indexOf(ret) + 1 || 3) - 1;
        },
        __wbg_removeEventListener_c6782a9c30557d31: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.removeEventListener(getStringFromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_removeListener_9b7f8155dd998421: function() { return handleError(function (arg0, arg1) {
            arg0.removeListener(arg1);
        }, arguments); },
        __wbg_removeProperty_2015da6fd4077af0: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg1.removeProperty(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_repeat_46a5c90df9f6ef0b: function(arg0) {
            const ret = arg0.repeat;
            return ret;
        },
        __wbg_requestAdapter_a525195f353e3e91: function(arg0, arg1) {
            const ret = arg0.requestAdapter(arg1);
            return ret;
        },
        __wbg_requestAnimationFrame_b92ccfbc8ca777a8: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.requestAnimationFrame(arg1);
            return ret;
        }, arguments); },
        __wbg_requestDevice_de819cc45484b72f: function(arg0, arg1) {
            const ret = arg0.requestDevice(arg1);
            return ret;
        },
        __wbg_requestFullscreen_57ab68e4daed67b5: function(arg0) {
            const ret = arg0.requestFullscreen();
            return ret;
        },
        __wbg_requestFullscreen_b5f0047211bec414: function(arg0) {
            const ret = arg0.requestFullscreen;
            return ret;
        },
        __wbg_requestIdleCallback_25d3b54bee33cdc0: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.requestIdleCallback(arg1);
            return ret;
        }, arguments); },
        __wbg_requestIdleCallback_c731d5c92902dc09: function(arg0) {
            const ret = arg0.requestIdleCallback;
            return ret;
        },
        __wbg_resolveQuerySet_645f3c0bd853faf8: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.resolveQuerySet(arg1, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
        },
        __wbg_resolve_d82363d90af6928a: function(arg0) {
            const ret = Promise.resolve(arg0);
            return ret;
        },
        __wbg_scheduler_a2a5c7b99e847d21: function(arg0) {
            const ret = arg0.scheduler;
            return ret;
        },
        __wbg_scheduler_b69307a1d905d2bd: function(arg0) {
            const ret = arg0.scheduler;
            return ret;
        },
        __wbg_setAttribute_8bccfbabf2a83682: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setBindGroup_173204f2c6368121: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.setBindGroup(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
        },
        __wbg_setBindGroup_2e4ad64dde21d63b: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.setBindGroup(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
        },
        __wbg_setBindGroup_99ceaaae485fc1c0: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.setBindGroup(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
        },
        __wbg_setBindGroup_9e1e6c6245b37ba5: function(arg0, arg1, arg2) {
            arg0.setBindGroup(arg1 >>> 0, arg2);
        },
        __wbg_setBindGroup_df7e810c4ea57b8a: function(arg0, arg1, arg2) {
            arg0.setBindGroup(arg1 >>> 0, arg2);
        },
        __wbg_setBindGroup_ef16e94180da850e: function(arg0, arg1, arg2) {
            arg0.setBindGroup(arg1 >>> 0, arg2);
        },
        __wbg_setBlendConstant_9c5dbb83cac8041f: function(arg0, arg1) {
            arg0.setBlendConstant(arg1);
        },
        __wbg_setIndexBuffer_003b795df7515765: function(arg0, arg1, arg2, arg3) {
            arg0.setIndexBuffer(arg1, __wbindgen_enum_GpuIndexFormat[arg2], arg3);
        },
        __wbg_setIndexBuffer_1b00b28cf77de627: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.setIndexBuffer(arg1, __wbindgen_enum_GpuIndexFormat[arg2], arg3, arg4);
        },
        __wbg_setIndexBuffer_aa31160e9f6ac23a: function(arg0, arg1, arg2, arg3) {
            arg0.setIndexBuffer(arg1, __wbindgen_enum_GpuIndexFormat[arg2], arg3);
        },
        __wbg_setIndexBuffer_b73936c4220dd2c2: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.setIndexBuffer(arg1, __wbindgen_enum_GpuIndexFormat[arg2], arg3, arg4);
        },
        __wbg_setPipeline_261d5088606a6ff2: function(arg0, arg1) {
            arg0.setPipeline(arg1);
        },
        __wbg_setPipeline_500cd4a5dacc45a2: function(arg0, arg1) {
            arg0.setPipeline(arg1);
        },
        __wbg_setPipeline_5378bd10e1ea4828: function(arg0, arg1) {
            arg0.setPipeline(arg1);
        },
        __wbg_setPointerCapture_87f6b55ceca474a9: function() { return handleError(function (arg0, arg1) {
            arg0.setPointerCapture(arg1);
        }, arguments); },
        __wbg_setProperty_da5e4438a912e787: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setScissorRect_1943297930f44ab7: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.setScissorRect(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_setStencilReference_2fd4b8e590e52cf2: function(arg0, arg1) {
            arg0.setStencilReference(arg1 >>> 0);
        },
        __wbg_setTimeout_6e540f43d21684d9: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.setTimeout(arg1);
            return ret;
        }, arguments); },
        __wbg_setTimeout_8afa0b5ed243c77d: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.setTimeout(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_setVertexBuffer_407c7b3069f2aacf: function(arg0, arg1, arg2, arg3) {
            arg0.setVertexBuffer(arg1 >>> 0, arg2, arg3);
        },
        __wbg_setVertexBuffer_4ea910535c3ee375: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.setVertexBuffer(arg1 >>> 0, arg2, arg3, arg4);
        },
        __wbg_setVertexBuffer_9c0b0be80b1850ae: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.setVertexBuffer(arg1 >>> 0, arg2, arg3, arg4);
        },
        __wbg_setVertexBuffer_c587bc318da86ed1: function(arg0, arg1, arg2, arg3) {
            arg0.setVertexBuffer(arg1 >>> 0, arg2, arg3);
        },
        __wbg_setViewport_ab36a8de3986940f: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.setViewport(arg1, arg2, arg3, arg4, arg5, arg6);
        },
        __wbg_set_272b80acaf9a75e8: function(arg0, arg1, arg2) {
            arg0.set(arg1, arg2 >>> 0);
        },
        __wbg_set_4564f7dc44fcb0c9: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(arg0, arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_set_box_3fa4e9e6b8b96df4: function(arg0, arg1) {
            arg0.box = __wbindgen_enum_ResizeObserverBoxOptions[arg1];
        },
        __wbg_set_height_ad5056ea051acd78: function(arg0, arg1) {
            arg0.height = arg1 >>> 0;
        },
        __wbg_set_height_ef298446b359b0c5: function(arg0, arg1) {
            arg0.height = arg1 >>> 0;
        },
        __wbg_set_onmessage_8700520361fa9f48: function(arg0, arg1) {
            arg0.onmessage = arg1;
        },
        __wbg_set_onuncapturederror_89ceb004b91b7ef2: function(arg0, arg1) {
            arg0.onuncapturederror = arg1;
        },
        __wbg_set_width_031bdecd763c5855: function(arg0, arg1) {
            arg0.width = arg1 >>> 0;
        },
        __wbg_set_width_f9e631f4ee129e5c: function(arg0, arg1) {
            arg0.width = arg1 >>> 0;
        },
        __wbg_shiftKey_273ee8c901267b55: function(arg0) {
            const ret = arg0.shiftKey;
            return ret;
        },
        __wbg_shiftKey_e57eae650446e681: function(arg0) {
            const ret = arg0.shiftKey;
            return ret;
        },
        __wbg_signal_4d9d567be73ea52c: function(arg0) {
            const ret = arg0.signal;
            return ret;
        },
        __wbg_size_8dc0ed89c335d9a2: function(arg0) {
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
        __wbg_start_8ca2b69cfb7618b6: function(arg0) {
            arg0.start();
        },
        __wbg_static_accessor_GLOBAL_THIS_2fee5048bcca5938: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_ce44e66a4935da8c: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_44f6e0cb5e67cdad: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_168f178805d978fe: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_style_f00fbf31e0a68f0e: function(arg0) {
            const ret = arg0.style;
            return ret;
        },
        __wbg_submit_cd452c592163f04e: function(arg0, arg1) {
            arg0.submit(arg1);
        },
        __wbg_then_05edfc8a4fea5106: function(arg0, arg1, arg2) {
            const ret = arg0.then(arg1, arg2);
            return ret;
        },
        __wbg_then_2a84678a50976959: function(arg0, arg1, arg2) {
            const ret = arg0.then(arg1, arg2);
            return ret;
        },
        __wbg_then_591b6b3a75ee817a: function(arg0, arg1) {
            const ret = arg0.then(arg1);
            return ret;
        },
        __wbg_then_c768c7c3e60c20ef: function(arg0, arg1) {
            const ret = arg0.then(arg1);
            return ret;
        },
        __wbg_unmap_185ffe998b4a0139: function(arg0) {
            arg0.unmap();
        },
        __wbg_unobserve_e9b5d98f37ccf1ff: function(arg0, arg1) {
            arg0.unobserve(arg1);
        },
        __wbg_usage_471b278fc5e76d78: function(arg0) {
            const ret = arg0.usage;
            return ret;
        },
        __wbg_valueOf_c4f805e57755a0ee: function(arg0) {
            const ret = arg0.valueOf();
            return ret;
        },
        __wbg_visibilityState_39ac2e458f892eec: function(arg0) {
            const ret = arg0.visibilityState;
            return (__wbindgen_enum_VisibilityState.indexOf(ret) + 1 || 3) - 1;
        },
        __wbg_warn_cd671287bc02594a: function(arg0) {
            console.warn(arg0);
        },
        __wbg_webkitFullscreenElement_8fb3dc71acc1fc53: function(arg0) {
            const ret = arg0.webkitFullscreenElement;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_webkitRequestFullscreen_36311aefefbbd4f6: function(arg0) {
            arg0.webkitRequestFullscreen();
        },
        __wbg_width_08e66520b99071f4: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbg_writeBuffer_4747cd198a195bf3: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.writeBuffer(arg1, arg2, arg3, arg4, arg5);
        },
        __wbg_writeTexture_56623ca6664ce174: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.writeTexture(arg1, arg2, arg3, arg4);
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 196, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a);
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 419, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h7ab048009a7d2d79);
            return ret;
        },
        __wbindgen_cast_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 453, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__ha48b416f1a454df5);
            return ret;
        },
        __wbindgen_cast_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Array<any>"), NamedExternref("ResizeObserver")], shim_idx: 72, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__he873a1a884efa923);
            return ret;
        },
        __wbindgen_cast_0000000000000005: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Array<any>")], shim_idx: 74, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h6ebd38c657f92859);
            return ret;
        },
        __wbindgen_cast_0000000000000006: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 196, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_5);
            return ret;
        },
        __wbindgen_cast_0000000000000007: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("FocusEvent")], shim_idx: 74, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h6ebd38c657f92859_6);
            return ret;
        },
        __wbindgen_cast_0000000000000008: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("GPUUncapturedErrorEvent")], shim_idx: 419, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h7ab048009a7d2d79_7);
            return ret;
        },
        __wbindgen_cast_0000000000000009: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("KeyboardEvent")], shim_idx: 196, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_8);
            return ret;
        },
        __wbindgen_cast_000000000000000a: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("PageTransitionEvent")], shim_idx: 196, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_9);
            return ret;
        },
        __wbindgen_cast_000000000000000b: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("PointerEvent")], shim_idx: 196, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_10);
            return ret;
        },
        __wbindgen_cast_000000000000000c: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("WheelEvent")], shim_idx: 196, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_11);
            return ret;
        },
        __wbindgen_cast_000000000000000d: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 201, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h13f05b481575933d);
            return ret;
        },
        __wbindgen_cast_000000000000000e: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_000000000000000f: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
            const ret = getArrayU8FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_0000000000000010: function(arg0, arg1) {
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
        "./tt_splat_viewer_bg.js": import0,
    };
}

function wasm_bindgen__convert__closures_____invoke__h13f05b481575933d(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures_____invoke__h13f05b481575933d(arg0, arg1);
}

function wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h7ab048009a7d2d79(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h7ab048009a7d2d79(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h6ebd38c657f92859(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h6ebd38c657f92859(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_5(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_5(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h6ebd38c657f92859_6(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h6ebd38c657f92859_6(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h7ab048009a7d2d79_7(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h7ab048009a7d2d79_7(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_8(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_8(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_9(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_9(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_10(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_10(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_11(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h228c338ee4ef9c0a_11(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__ha48b416f1a454df5(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen__convert__closures_____invoke__ha48b416f1a454df5(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen__convert__closures_____invoke__he873a1a884efa923(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures_____invoke__he873a1a884efa923(arg0, arg1, arg2, arg3);
}


const __wbindgen_enum_GpuDeviceLostReason = ["unknown", "destroyed"];


const __wbindgen_enum_GpuErrorFilter = ["validation", "out-of-memory", "internal"];


const __wbindgen_enum_GpuIndexFormat = ["uint16", "uint32"];


const __wbindgen_enum_GpuTextureFormat = ["r8unorm", "r8snorm", "r8uint", "r8sint", "r16uint", "r16sint", "r16float", "rg8unorm", "rg8snorm", "rg8uint", "rg8sint", "r32uint", "r32sint", "r32float", "rg16uint", "rg16sint", "rg16float", "rgba8unorm", "rgba8unorm-srgb", "rgba8snorm", "rgba8uint", "rgba8sint", "bgra8unorm", "bgra8unorm-srgb", "rgb9e5ufloat", "rgb10a2uint", "rgb10a2unorm", "rg11b10ufloat", "rg32uint", "rg32sint", "rg32float", "rgba16uint", "rgba16sint", "rgba16float", "rgba32uint", "rgba32sint", "rgba32float", "stencil8", "depth16unorm", "depth24plus", "depth24plus-stencil8", "depth32float", "depth32float-stencil8", "bc1-rgba-unorm", "bc1-rgba-unorm-srgb", "bc2-rgba-unorm", "bc2-rgba-unorm-srgb", "bc3-rgba-unorm", "bc3-rgba-unorm-srgb", "bc4-r-unorm", "bc4-r-snorm", "bc5-rg-unorm", "bc5-rg-snorm", "bc6h-rgb-ufloat", "bc6h-rgb-float", "bc7-rgba-unorm", "bc7-rgba-unorm-srgb", "etc2-rgb8unorm", "etc2-rgb8unorm-srgb", "etc2-rgb8a1unorm", "etc2-rgb8a1unorm-srgb", "etc2-rgba8unorm", "etc2-rgba8unorm-srgb", "eac-r11unorm", "eac-r11snorm", "eac-rg11unorm", "eac-rg11snorm", "astc-4x4-unorm", "astc-4x4-unorm-srgb", "astc-5x4-unorm", "astc-5x4-unorm-srgb", "astc-5x5-unorm", "astc-5x5-unorm-srgb", "astc-6x5-unorm", "astc-6x5-unorm-srgb", "astc-6x6-unorm", "astc-6x6-unorm-srgb", "astc-8x5-unorm", "astc-8x5-unorm-srgb", "astc-8x6-unorm", "astc-8x6-unorm-srgb", "astc-8x8-unorm", "astc-8x8-unorm-srgb", "astc-10x5-unorm", "astc-10x5-unorm-srgb", "astc-10x6-unorm", "astc-10x6-unorm-srgb", "astc-10x8-unorm", "astc-10x8-unorm-srgb", "astc-10x10-unorm", "astc-10x10-unorm-srgb", "astc-12x10-unorm", "astc-12x10-unorm-srgb", "astc-12x12-unorm", "astc-12x12-unorm-srgb"];


const __wbindgen_enum_ResizeObserverBoxOptions = ["border-box", "content-box", "device-pixel-content-box"];


const __wbindgen_enum_VisibilityState = ["hidden", "visible"];

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

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
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

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
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

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
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

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
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
        module_or_path = new URL('tt_splat_viewer_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
