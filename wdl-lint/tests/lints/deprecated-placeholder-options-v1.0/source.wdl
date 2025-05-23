## This is a test of the `DeprecatedPlaceholder` lint.

version 1.0

# None of these lints should trigger as the version is WDL v1.0 (prior to
# placeholder options being deprecated).
task a_task {
    #@ except: MetaDescription
    meta {}

    Array[String] numbers = ["1", "2", "3"]
    Boolean allow_foo = true
    String bar = "bar"

    String bad_sep_option = "~{sep="," numbers}"
    String bad_true_false_option = "~{true="--enable-foo" false="" allow_foo}"
    String bad_default_option = "~{default="false" bar}"

    #@ except: ShellCheck
    command <<<
        python script.py ~{sep=" " numbers}
        example-command ~{true="--enable-foo" false="" allow_foo}
        another-command ~{default="foobar" bar}
    >>>

    output {}

    #@ except: ExpectedRuntimeKeys
    runtime {}
}
