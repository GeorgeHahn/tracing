use tracing::subscriber::with_default;
use tracing_attributes::instrument;
use tracing_test::span::NewSpan;
use tracing_test::*;

#[instrument(fields(foo = "bar", dsa = true, num = 1))]
fn fn_no_param() {}

#[instrument(fields(foo = "bar"))]
fn fn_param(param: u32) {}

#[instrument(fields(foo = "bar", empty))]
fn fn_empty_field() {}

#[instrument(fields(len = s.len()))]
fn fn_expr_field(s: &str) {}

#[instrument(fields(s.len = s.len(), s.is_empty = s.is_empty()))]
fn fn_two_expr_fields(s: &str) {
    let _ = s;
}

#[instrument(fields(%s, s.len = s.len()))]
fn fn_clashy_expr_field(s: &str) {
    let _ = s;
}

#[instrument(fields(s = "s"))]
fn fn_clashy_expr_field2(s: &str) {
    let _ = s;
}

#[derive(Debug)]
struct HasField {
    my_field: &'static str,
}

impl HasField {
    #[instrument(fields(my_field = self.my_field), skip(self))]
    fn self_expr_field(&self) {}
}

#[test]
fn fields() {
    let span = span::new().with_field(
        field("foo")
            .with_value(&"bar")
            .and(field("dsa").with_value(&true))
            .and(field("num").with_value(&1))
            .only(),
    );
    run_test(span, || {
        fn_no_param();
    });
}

#[test]
fn expr_field() {
    let span = span::new().with_field(
        field("s")
            .with_value(&tracing::field::debug("hello world"))
            .and(field("len").with_value(&"hello world".len()))
            .only(),
    );
    run_test(span, || {
        fn_expr_field(&"hello world");
    });
}

#[test]
fn two_expr_fields() {
    let span = span::new().with_field(
        field("s")
            .with_value(&tracing::field::debug("hello world"))
            .and(field("s.len").with_value(&"hello world".len()))
            .and(field("s.is_empty").with_value(&false))
            .only(),
    );
    run_test(span, || {
        fn_two_expr_fields(&"hello world");
    });
}

#[test]
fn clashy_expr_field() {
    let span1 = span::new().with_field(
        // Overriding the `s` field should record `s` as a `Display` value,
        // rather than as a `Debug` value.
        field("s")
            .with_value(&tracing::field::display("hello world"))
            .and(field("s.len").with_value(&"hello world".len()))
            .only(),
    );
    run_test(span1, || {
        fn_clashy_expr_field(&"hello world");
    });

    let span1 = span::new().with_field(field("s").with_value(&"s").only());
    run_test(span1, || {
        fn_clashy_expr_field2(&"hello world");
    });
}

#[test]
fn self_expr_field() {
    let span = span::new().with_field(field("my_field").with_value(&"hello world").only());
    run_test(span, || {
        let has_field = HasField {
            my_field: "hello world",
        };
        has_field.self_expr_field();
    });
}

#[test]
fn parameters_with_fields() {
    let span = span::new().with_field(
        field("foo")
            .with_value(&"bar")
            .and(field("param").with_value(&format_args!("1")))
            .only(),
    );
    run_test(span, || {
        fn_param(1);
    });
}

#[test]
fn empty_field() {
    let span = span::new().with_field(field("foo").with_value(&"bar").only());
    run_test(span, || {
        fn_empty_field();
    });
}

fn run_test<F: FnOnce() -> T, T>(span1: NewSpan, fun: F) {
    let (subscriber, handle) = subscriber::expect()
        .new_span(span1)
        .enter(span::new())
        .exit(span::new())
        .done()
        .run_with_handle();

    with_default(subscriber, fun);
    handle.assert_finished();
}
