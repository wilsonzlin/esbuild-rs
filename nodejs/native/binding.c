// TODO All calls to napi_{create,throw}*_error might fail.

#include <stdlib.h>

#define NAPI_VERSION 4
#include <node_api.h>

#include "../build/libesbuild.h"

napi_threadsafe_function js_receiver;

static char const* JS_RECEIVER_DESC = "esbuild-native JavaScript receiver callback";
static char const* ERRMSG_INTERR_RELEASE_SRC_BUFFER_FAILED = "Failed to release source buffer reference";
static char const* ERRMSG_INTERR_RELEASE_RES_BUFFER_FAILED = "Failed to release result buffer reference";
static char const* ERRMSG_INTERR_CREATE_RES_BUFFER_FAILED = "Failed to create result buffer";
static char const* ERRMSG_INTERR_CREATE_JS_ID_FAILED = "Failed to create JS ID number";
static char const* ERRMSG_INTERR_CREATE_JS_MIN_LEN_FAILED = "Failed to create minified code length JS number";

struct invocation_data {
  unsigned long long id;
  napi_ref src_buffer_ref;
  napi_ref res_buffer_ref;
};

struct call_js_receiver_data {
  struct invocation_data* invocation_data;
  unsigned long long min_code_len;
};

void call_js_receiver(
  napi_env env,
  napi_value js_callback,
  void* _ctx,
  void* data_raw
) {
  napi_value undefined;
  // TODO Could this fail?
  napi_get_undefined(env, &undefined);

  napi_value error_msg = undefined;
  napi_value res_id = undefined;
  napi_value res_min_code_len = undefined;

  struct call_js_receiver_data* data = (struct call_js_receiver_data*) data_raw;
  struct invocation_data* invocation_data = (struct invocation_data*) data->invocation_data;

  // Free source buffer.
  // TODO Not sure if this is OK, as env might be different to the one ref was created with.
  if (napi_delete_reference(env, invocation_data->src_buffer_ref) != napi_ok) {
      // TODO Can't do much if this fails...
      napi_create_string_utf8(
        env,
        ERRMSG_INTERR_RELEASE_SRC_BUFFER_FAILED,
        sizeof(ERRMSG_INTERR_RELEASE_SRC_BUFFER_FAILED) - 1,
        &error_msg
      );
      goto finally;
  }

  // Decrease refcount of result buffer.
  // TODO Not sure if this is OK, as env might be different to the one ref was created with.
  if (napi_delete_reference(env, invocation_data->res_buffer_ref) != napi_ok) {
      // TODO Can't do much if this fails...
      napi_create_string_utf8(
        env,
        ERRMSG_INTERR_RELEASE_RES_BUFFER_FAILED,
        sizeof(ERRMSG_INTERR_RELEASE_RES_BUFFER_FAILED) - 1,
        &error_msg
      );
      goto finally;
  }

  if (napi_create_int64(env, (int64_t) invocation_data->id, &res_id) != napi_ok) {
      // TODO Can't do much if this fails...
      napi_create_string_utf8(
        env,
        ERRMSG_INTERR_CREATE_JS_ID_FAILED,
        sizeof(ERRMSG_INTERR_CREATE_JS_ID_FAILED) - 1,
        &error_msg
      );
      goto finally;
  }

  if (napi_create_int64(env, (int64_t) data->min_code_len, &res_min_code_len) != napi_ok) {
      // TODO Can't do much if this fails...
      napi_create_string_utf8(
        env,
        ERRMSG_INTERR_CREATE_JS_MIN_LEN_FAILED,
        sizeof(ERRMSG_INTERR_CREATE_JS_MIN_LEN_FAILED) - 1,
        &error_msg
      );
      goto finally;
  }

  napi_value error = undefined;
finally:
  if (error_msg != undefined) {
    // TODO Can't do much if this fails...
    napi_create_error(env, NULL, error_msg, &error);
  }
  napi_value call_args[3] = {error, res_id, res_min_code_len};
  napi_value call_result;
  if (napi_call_function(
    env,
    undefined,
    js_callback,
    3,
    call_args,
    &call_result
  ) != napi_ok) {
    // TODO Can't do much here...
  }

  free(invocation_data);
  free(data);
}

void minify_js_complete_handler(
  void* invocation_data,
  unsigned long long min_code_len
) {
  // TODO check for NULL
  struct call_js_receiver_data* data = malloc(sizeof(struct call_js_receiver_data));
  data->invocation_data = invocation_data;
  data->min_code_len = min_code_len;
  if (napi_call_threadsafe_function(js_receiver, (void*) data, napi_tsfn_nonblocking) != napi_ok) {
    // TODO
  }
}

// TODO Create corresponding delete method, and destroy any existing.
napi_value node_method_start_service(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value argv[1];
  napi_value _this;
  void* _data;

  napi_value undefined;
  // TODO Could this fail?
  napi_get_undefined(env, &undefined);

  napi_status get_cb_info_status =
    napi_get_cb_info(env, info, &argc, argv, &_this, &_data);
  if (get_cb_info_status != napi_ok) {
    napi_throw_error(env, "INTERR_GET_CB_INFO_FAILED", "Failed to get callback info");
    return undefined;
  }

  napi_value js_callback_arg = argv[0];
  napi_valuetype arg_type;
  napi_status get_arg_type_status =
    napi_typeof(env, js_callback_arg, &arg_type);
  if (get_arg_type_status != napi_ok) {
    // TODO
  }
  if (arg_type != napi_function) {
    napi_throw_type_error(env, "NOTAFN", "First argument is not a function");
    return undefined;
  }

  napi_value js_receiver_desc;
  if (napi_create_string_utf8(
    env,
    JS_RECEIVER_DESC,
    sizeof(JS_RECEIVER_DESC) - 1,
    &js_receiver_desc
  ) != napi_ok) {
    napi_throw_error(env, "INTERR_CREATE_JS_RECEIVER_DESC_FAILED", "Failed to create JS receiver callback description string");
    return undefined;
  }

  if (napi_create_threadsafe_function(
    env,
    // napi_value func.
    js_callback_arg,
    // napi_value async_resource.
    NULL,
    // napi_value async_resource_name.
    js_receiver_desc,
    // size_t max_queue_size.
    0,
    // size_t initial_thread_count.
    1,
    // void* thread_finalize_data.
    NULL,
    // napi_finalize thread_finalize_cb.
    NULL,
    // void* context.
    NULL,
    // napi_threadsafe_function_call_js call_js_cb.
    &call_js_receiver,
    // napi_threadsafe_function* result.
    &js_receiver
  ) != napi_ok) {
    napi_throw_error(env, "INTERR_CREATE_JS_RECEIVER_FAILED", "Failed to create JS receiver");
    return undefined;
  }

  return undefined;
}

napi_value node_method_stop_service(napi_env env, napi_callback_info info) {
  if (napi_release_threadsafe_function(js_receiver, napi_tsfn_abort) != napi_ok) {
    // TODO
  }

  napi_value undefined;
  // TODO Could this fail?
  napi_get_undefined(env, &undefined);
  return undefined;
}

napi_value node_method_minify(napi_env env, napi_callback_info info) {
  napi_value undefined;
  // TODO Could this fail?
  napi_get_undefined(env, &undefined);

  size_t argc = 2;
  napi_value argv[2];
  napi_value _this;
  void* _data;

  // Get the arguments.
  if (napi_get_cb_info(env, info, &argc, argv, &_this, &_data) != napi_ok) {
    napi_throw_error(env, "INTERR_GET_CB_INFO_FAILED", "Failed to get callback info");
    return undefined;
  }

  // Get the ID argument.
  napi_value js_id_arg = argv[0];
  int64_t sid;
  if (napi_get_value_int64(env, js_id_arg, &sid) != napi_ok || sid < 0) {
    napi_throw_error(env, "GET_ID_FAILED", "Failed to parse ID");
    return undefined;
  }
  unsigned long long id = (unsigned long long) sid;

  // Ensure buffer lives long enough until minification has finished.
  napi_value buffer_arg = argv[1];
  napi_ref buffer_arg_ref;
  if (napi_create_reference(env, buffer_arg, 1, &buffer_arg_ref) != napi_ok) {
    napi_throw_error(env, "INTERR_CREATE_SRC_BUFFER_REF", "Failed to create reference for source buffer");
    return undefined;
  }

  // Get pointer to bytes from buffer.
  void* buffer_data;
  size_t buffer_len;
  if (napi_get_buffer_info(env, buffer_arg, &buffer_data, &buffer_len) != napi_ok || buffer_len == 0 || buffer_data == NULL) {
    napi_throw_error(env, "INTERR_GET_SRC_BUFFER_INFO", "Failed to read source buffer");
    return undefined;
  }

  // Preallocate buffer so Go can directly copy to here and avoid double copying.
  napi_value res_buffer;
  void* res_buf_data;
  if (napi_create_buffer(env, buffer_len, &res_buf_data, &res_buffer) != napi_ok) {
      napi_throw_error(env, "INTERR_CREATE_RES_BUFFER", "Failed to create result buffer");
      return undefined;
  }
  napi_ref res_buffer_ref;
  if (napi_create_reference(env, res_buffer, 1, &res_buffer_ref) != napi_ok) {
    napi_throw_error(env, "INTERR_CREATE_RES_BUFFER_REF", "Failed to create reference for result buffer");
    return undefined;
  }

  GoString buffer_as_gostr = {
    .p = (char const*) buffer_data,
    .n = buffer_len,
  };

  // TODO This will be freed later in the happy path, but check error paths too (including outside this function).
  struct invocation_data* invocation_data = malloc(sizeof(struct invocation_data));
  invocation_data->id = id;
  invocation_data->src_buffer_ref = buffer_arg_ref;
  invocation_data->res_buffer_ref = res_buffer_ref;

  MinifyJs(
    buffer_as_gostr,
    res_buf_data,
    &minify_js_complete_handler,
    (void*) invocation_data
  );

  return res_buffer;
}

napi_value node_module_init(napi_env env, napi_value exports) {
  napi_status status;
  napi_property_descriptor props[] = {
    {"minify", NULL, node_method_minify, NULL, NULL, NULL, napi_default, NULL},
    {"startService", NULL, node_method_start_service, NULL, NULL, NULL, napi_default, NULL},
    {"stopService", NULL, node_method_stop_service, NULL, NULL, NULL, napi_default, NULL},
  };
  status = napi_define_properties(env, exports, 3, props);
  if (status != napi_ok) return NULL;
  return exports;
}

NAPI_MODULE(NODE_GYP_MODULE_NAME, node_module_init)
