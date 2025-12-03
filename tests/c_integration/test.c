#include <glib.h>
#include "calculator.h"

static void
test_constructor (void)
{
    Calculator *calc = calculator_new ();
    g_assert_nonnull (calc);
    g_object_unref (calc);
}

static void
test_primitive_parameters_and_return (void)
{
    Calculator *calc = calculator_new ();
    gint32 sum = calculator_add (calc, 5, 3);
    g_assert_cmpint (sum, ==, 8);
    g_object_unref (calc);
}

static void
test_boolean_return (void)
{
    Calculator *calc = calculator_new ();

    g_assert_true (calculator_is_positive (calc, 10));
    g_assert_false (calculator_is_positive (calc, -5));
    g_assert_false (calculator_is_positive (calc, 0));

    g_object_unref (calc);
}

static void
test_string_return (void)
{
    Calculator *calc = calculator_new ();
    gchar *msg = calculator_get_message (calc);

    g_assert_nonnull (msg);
    g_assert_cmpstr (msg, ==, "Hello from Rust!");

    g_free (msg);
    g_object_unref (calc);
}

static void
test_fallible_method_success (void)
{
    Calculator *calc = calculator_new ();
    GError *error = NULL;

    gint32 result = calculator_divide (calc, 10, 2, &error);
    g_assert_no_error (error);
    g_assert_cmpint (result, ==, 5);

    g_object_unref (calc);
}

static void
test_fallible_method_error (void)
{
    Calculator *calc = calculator_new ();
    GError *error = NULL;

    calculator_divide (calc, 10, 0, &error);
    g_assert_error (error, G_FILE_ERROR, G_FILE_ERROR_FAILED);

    g_error_free (error);
    g_object_unref (calc);
}

static void
test_optional_primitive_parameter (void)
{
    Calculator *calc = calculator_new ();

    g_assert_cmpint (calculator_add_optional (calc, 5, 3), ==, 8);
    g_assert_cmpint (calculator_add_optional (calc, 5, 0), ==, 5);

    g_object_unref (calc);
}

static void
test_out_parameter (void)
{
    Calculator *calc = calculator_new ();
    gint32 product = 0;

    gint32 sum = calculator_compute_sum_and_product (calc, 4, 5, &product);
    g_assert_cmpint (sum, ==, 9);
    g_assert_cmpint (product, ==, 20);

    g_object_unref (calc);
}

typedef struct {
    GMainLoop *loop;
    guint64 result;
    gboolean done;
} AsyncData;

static void
on_factorial_ready (GObject      *source,
                    GAsyncResult *res,
                    gpointer      user_data)
{
    AsyncData *data = (AsyncData *)user_data;

    data->result = calculator_compute_factorial_finish ((Calculator *)source, res);
    data->done = TRUE;

    g_main_loop_quit (data->loop);
}

static void
test_async_method (void)
{
    Calculator *calc = calculator_new ();
    GMainContext *context = g_main_context_default ();
    GMainLoop *loop = g_main_loop_new (context, FALSE);

    AsyncData data = { .loop = loop, .result = 0, .done = FALSE };
    calculator_compute_factorial (calc, 5, NULL, on_factorial_ready, &data);

    g_main_loop_run (loop);
    g_assert_true (data.done);
    g_assert_cmpuint (data.result, ==, 120);

    g_main_loop_unref (loop);
    g_object_unref (calc);
}

static void
test_async_sync_wrapper (void)
{
    Calculator *calc = calculator_new ();

    guint64 result = calculator_compute_factorial_sync (calc, 6, NULL);
    g_assert_cmpuint (result, ==, 720);

    g_object_unref (calc);
}

typedef struct {
    GMainLoop *loop;
    gint32 result;
    GError *error;
    gboolean done;
} AsyncErrorData;

static void
on_safe_divide_success_ready (GObject      *source,
                              GAsyncResult *res,
                              gpointer      user_data)
{
    AsyncErrorData *data = (AsyncErrorData *)user_data;

    data->result = calculator_safe_divide_finish ((Calculator *)source, res, &data->error);
    data->done = TRUE;

    g_main_loop_quit (data->loop);
}

static void
test_fallible_async_method_success (void)
{
    Calculator *calc = calculator_new ();
    GMainContext *context = g_main_context_default ();
    GMainLoop *loop = g_main_loop_new (context, FALSE);

    AsyncErrorData data = { .loop = loop, .result = 0, .error = NULL, .done = FALSE };
    calculator_safe_divide (calc, 20, 4, NULL, on_safe_divide_success_ready, &data);

    g_main_loop_run (loop);
    g_assert_true (data.done);
    g_assert_no_error (data.error);
    g_assert_cmpint (data.result, ==, 5);

    g_main_loop_unref (loop);
    g_object_unref (calc);
}

static void
on_safe_divide_error_ready (GObject      *source,
                             GAsyncResult *res,
                             gpointer      user_data)
{
    AsyncErrorData *data = (AsyncErrorData *)user_data;

    data->result = calculator_safe_divide_finish ((Calculator *)source, res, &data->error);
    data->done = TRUE;

    g_main_loop_quit (data->loop);
}

static void
test_fallible_async_method_error (void)
{
    Calculator *calc = calculator_new ();
    GMainContext *context = g_main_context_default ();
    GMainLoop *loop = g_main_loop_new (context, FALSE);

    AsyncErrorData data = { .loop = loop, .result = 0, .error = NULL, .done = FALSE };
    calculator_safe_divide (calc, 10, 0, NULL, on_safe_divide_error_ready, &data);

    g_main_loop_run (loop);
    g_assert_true (data.done);
    g_assert_error (data.error, G_FILE_ERROR, G_FILE_ERROR_FAILED);

    g_error_free (data.error);
    g_main_loop_unref (loop);
    g_object_unref (calc);
}

static void
test_fallible_async_sync_wrapper (void)
{
    Calculator *calc = calculator_new ();
    GError *error = NULL;

    calculator_safe_divide_sync (calc, 15, 0, NULL, &error);
    g_assert_error (error, G_FILE_ERROR, G_FILE_ERROR_FAILED);

    g_error_free (error);
    g_object_unref (calc);
}

int
main (int   argc,
      char *argv[])
{
    g_test_init (&argc, &argv, NULL);

    g_test_add_func ("/ffi/constructor", test_constructor);
    g_test_add_func ("/ffi/primitive_parameters_and_return", test_primitive_parameters_and_return);
    g_test_add_func ("/ffi/boolean_return", test_boolean_return);
    g_test_add_func ("/ffi/string_return", test_string_return);
    g_test_add_func ("/ffi/fallible_method/success", test_fallible_method_success);
    g_test_add_func ("/ffi/fallible_method/error", test_fallible_method_error);
    g_test_add_func ("/ffi/optional_primitive_parameter", test_optional_primitive_parameter);
    g_test_add_func ("/ffi/out_parameter", test_out_parameter);
    g_test_add_func ("/ffi/async_method", test_async_method);
    g_test_add_func ("/ffi/async_sync_wrapper", test_async_sync_wrapper);
    g_test_add_func ("/ffi/fallible_async_method/success", test_fallible_async_method_success);
    g_test_add_func ("/ffi/fallible_async_method/error", test_fallible_async_method_error);
    g_test_add_func ("/ffi/fallible_async_sync_wrapper", test_fallible_async_sync_wrapper);

    return g_test_run ();
}
