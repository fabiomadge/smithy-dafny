// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
impl super::Client {
    /// Constructs a fluent builder for the [`GetStringUTF8`](crate::operation::get_string_utf8::builders::GetStringUTF8FluentBuilder) operation.
    ///
    /// - The fluent builder is configurable:
    ///   - [`value(impl Into<Option<String>>)`](crate::operation::get_string_utf8::builders::GetStringUTF8FluentBuilder::name) / [`set_name(Option<String>)`](crate::operation::get_string_utf8::builders::GetStringUTF8FluentBuilder::set_name):(undocumented)<br>
    /// - On success, responds with [`GetStringUTF8Output`](crate::operation::get_string_utf8::GetStringUTF8Output) with field(s):
    ///   - [`value(Option<String>)`](crate::operation::get_string_utf8::GetStringUTF8Output::value): (undocumented)
    /// - On failure, responds with [`SdkError<GetStringUTF8Error>`](crate::operation::get_string_utf8::GetStringUTF8Error)
    pub fn get_string_utf8(
        &self,
    ) -> crate::operation::get_string_utf8::builders::GetStringUTF8FluentBuilder {
        crate::operation::get_string_utf8::builders::GetStringUTF8FluentBuilder::new(
            self.handle.clone(),
        )
    }
}
