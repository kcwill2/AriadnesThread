use super::*;
use tempfile::tempdir;

struct FlagCase {
    label: &'static str,
    command: Option<&'static str>,
    args: &'static [&'static str],
    compiler: &'static str,
    defines: &'static [&'static str],
    include_dirs: &'static [&'static str],
    system_include_dirs: &'static [&'static str],
    forced_includes: &'static [&'static str],
    lang_std: Option<&'static str>,
    arch: Option<&'static str>,
    raw_flags: &'static [&'static str],
}

fn flag_case(label: &'static str, args: &'static [&'static str]) -> FlagCase {
    FlagCase {
        label,
        command: None,
        args,
        compiler: "cl.exe",
        defines: &[],
        include_dirs: &[],
        system_include_dirs: &[],
        forced_includes: &[],
        lang_std: None,
        arch: None,
        raw_flags: &[],
    }
}

impl FlagCase {
    fn with_command(mut self, command: &'static str) -> Self {
        self.command = Some(command);
        self
    }

    fn with_compiler(mut self, compiler: &'static str) -> Self {
        self.compiler = compiler;
        self
    }

    fn with_defines(mut self, defines: &'static [&'static str]) -> Self {
        self.defines = defines;
        self
    }

    fn with_includes(mut self, include_dirs: &'static [&'static str]) -> Self {
        self.include_dirs = include_dirs;
        self
    }

    fn with_system_includes(mut self, system_include_dirs: &'static [&'static str]) -> Self {
        self.system_include_dirs = system_include_dirs;
        self
    }

    fn with_forced(mut self, forced_includes: &'static [&'static str]) -> Self {
        self.forced_includes = forced_includes;
        self
    }

    fn with_lang_std(mut self, lang_std: &'static str) -> Self {
        self.lang_std = Some(lang_std);
        self
    }

    fn with_arch(mut self, arch: &'static str) -> Self {
        self.arch = Some(arch);
        self
    }

    fn with_raw_flags(mut self, raw_flags: &'static [&'static str]) -> Self {
        self.raw_flags = raw_flags;
        self
    }
}

fn paths_of(items: &[&str]) -> Vec<PathBuf> {
    items.iter().copied().map(PathBuf::from).collect()
}

fn check_flag_case(case: &FlagCase) {
    let command = case.command.map(str::to_string);
    let args: Vec<String> = case.args.iter().map(|arg| arg.to_string()).collect();
    let (
        compiler,
        defines,
        include_dirs,
        system_include_dirs,
        forced_includes,
        lang_std,
        arch,
        raw_flags,
    ) = parse_args(&command, &args);

    assert_eq!(compiler, case.compiler, "[{}]", case.label);
    assert_eq!(defines, case.defines, "[{}]", case.label);
    assert_eq!(include_dirs, paths_of(case.include_dirs), "[{}]", case.label);
    assert_eq!(
        system_include_dirs,
        paths_of(case.system_include_dirs),
        "[{}]",
        case.label
    );
    assert_eq!(
        forced_includes,
        paths_of(case.forced_includes),
        "[{}]",
        case.label
    );
    assert_eq!(lang_std.as_deref(), case.lang_std, "[{}]", case.label);
    assert_eq!(arch.as_deref(), case.arch, "[{}]", case.label);
    assert_eq!(raw_flags, case.raw_flags, "[{}]", case.label);
}

fn write_compile_commands(root: &Path, entries: serde_json::Value) {
    fs::write(root.join("compile_commands.json"), entries.to_string()).unwrap();
}

#[test]
fn parse_args_defines_lock_current_behavior() {
    let cases = vec![
        flag_case("joined -DFOO", &["cl.exe", "-DFOO"]).with_defines(&["FOO"]),
        flag_case("joined -DFOO=bar", &["cl.exe", "-DFOO=bar"]).with_defines(&["FOO=bar"]),
        flag_case("spaced -D FOO", &["cl.exe", "-D", "FOO"]).with_defines(&["FOO"]),
        flag_case("msvc spaced /D FOO", &["cl.exe", "/D", "FOO"]).with_defines(&["FOO"]),
        // QUIRK: trailing standalone -D yields empty value, silently dropped.
        flag_case("trailing -D drops", &["cl.exe", "-D"]),
        // QUIRK: -D= records the bare "=" as a define value.
        flag_case("-D= records bare equals", &["cl.exe", "-D="]).with_defines(&["="]),
        // QUIRK: standalone -D blindly consumes the next token even when it
        // is another flag, so -DNDEBUG is recorded as define "-DNDEBUG"
        // and the NDEBUG define is lost.
        flag_case("-D swallows following flag", &["cl.exe", "-D", "-DNDEBUG"])
            .with_defines(&["-DNDEBUG"]),
        // QUIRK: standalone -D consumes the next token even when empty.
        flag_case("-D swallows empty token", &["cl.exe", "-D", "", "extra"]),
    ];
    for case in cases {
        check_flag_case(&case);
    }
}

#[test]
fn parse_args_include_flags_lock_current_behavior() {
    let cases = vec![
        flag_case("joined -Iinc", &["cl.exe", "-Iinc"]).with_includes(&["inc"]),
        flag_case("spaced -I inc", &["cl.exe", "-I", "inc"]).with_includes(&["inc"]),
        flag_case("joined /Iinc", &["cl.exe", "/Iinc"]).with_includes(&["inc"]),
        flag_case("spaced /I inc", &["cl.exe", "/I", "inc"]).with_includes(&["inc"]),
        // insertion order is preserved at parse level (normalize() sorts later)
        flag_case("include order preserved", &["cc", "-Ib", "-Ia"])
            .with_compiler("cc")
            .with_includes(&["b", "a"]),
        // QUIRK: standalone -I blindly consumes the next token even when it
        // is another flag, so -Werror becomes an include dir and is lost
        // from raw_flags.
        flag_case("-I swallows following flag", &["cl.exe", "-I", "-Werror"])
            .with_includes(&["-Werror"]),
        // QUIRK: trailing standalone -I yields empty value, silently dropped.
        flag_case("trailing -I drops", &["cl.exe", "-I"]),
        flag_case("joined -isystempath", &["cc", "-isystemsys"])
            .with_compiler("cc")
            .with_system_includes(&["sys"]),
        flag_case("spaced -isystem path", &["cc", "-isystem", "sys"])
            .with_compiler("cc")
            .with_system_includes(&["sys"]),
        flag_case("joined /external:Iext", &["cc", "/external:Iext"])
            .with_compiler("cc")
            .with_system_includes(&["ext"]),
        // QUIRK: the space-separated MSVC /external:I form is not supported:
        // the flag contributes nothing AND the following path token is
        // silently dropped (never even reaching raw_flags).
        flag_case("/external:I space form drops both", &["cc", "/external:I", "ext"])
            .with_compiler("cc"),
        flag_case("joined -FIprefix.h", &["cl.exe", "-FIprefix.h"])
            .with_forced(&["prefix.h"]),
        flag_case("spaced /FI stdafx.h", &["cl.exe", "/FI", "stdafx.h"])
            .with_forced(&["stdafx.h"]),
        // QUIRK: trailing standalone -FI yields empty value, silently dropped.
        flag_case("trailing -FI drops", &["cl.exe", "-FI"]),
    ];
    for case in cases {
        check_flag_case(&case);
    }
}

#[test]
fn parse_args_standard_and_arch_lock_current_behavior() {
    let cases = vec![
        flag_case("-std=c++17", &["cc", "-std=c++17"])
            .with_compiler("cc")
            .with_lang_std("c++17"),
        flag_case("/std:c++latest", &["cl.exe", "/std:c++latest"])
            .with_lang_std("c++latest"),
        // QUIRK: -std= with empty value records Some("").
        flag_case("-std= empty records empty", &["cc", "-std="])
            .with_compiler("cc")
            .with_lang_std(""),
        // QUIRK: later -std wins within one entry (overwrite, not append).
        flag_case("duplicate -std last wins", &["cc", "-std=c++11", "-std=c++20"])
            .with_compiler("cc")
            .with_lang_std("c++20"),
        // QUIRK: /std= (wrong separator for MSVC) is unrecognized and lands
        // in raw_flags instead of setting the standard.
        flag_case("/std= unrecognized falls through", &["cl.exe", "/std=c++17"])
            .with_raw_flags(&["/std=c++17"]),
        flag_case("/arch:AVX2", &["cl.exe", "/arch:AVX2"]).with_arch("AVX2"),
        // QUIRK: /arch: with empty value records Some("").
        flag_case("/arch: empty records empty", &["cl.exe", "/arch:"]).with_arch(""),
        // QUIRK: -arch: (dash form) is unrecognized and lands in raw_flags.
        flag_case("-arch: unrecognized falls through", &["cc", "-arch:AVX"])
            .with_compiler("cc")
            .with_raw_flags(&["-arch:AVX"]),
    ];
    for case in cases {
        check_flag_case(&case);
    }
}

#[test]
fn parse_args_unknown_and_dropped_lock_current_behavior() {
    let cases = vec![
        flag_case("empty arg list keeps defaults", &[]),
        flag_case("unknown flags accumulate in raw_flags", &["cl.exe", "/c", "/O2"])
            .with_raw_flags(&["/c", "/O2"]),
        // QUIRK: non-flag positional tokens (e.g. source files) are
        // silently dropped, never recorded anywhere.
        flag_case("source file dropped", &["cl.exe", "/c", "main.cpp"])
            .with_raw_flags(&["/c"]),
        // QUIRK: a leading flag is not consumed as the compiler, so the
        // default "cl.exe" is kept even for a gcc-style invocation whose
        // first token is a flag.
        flag_case("leading flag keeps default compiler", &["-Werror", "main.c"])
            .with_raw_flags(&["-Werror"]),
        // QUIRK: a mid-list unix-style path is not considered a flag (the
        // is_flag heuristic excludes "usr"/"bin" to protect unix compilers
        // as first arg), so it is silently dropped mid-list.
        flag_case("mid-list /usr path dropped", &["cc", "/usr/include/x"])
            .with_compiler("cc"),
        // QUIRK: when both command and arguments are present, the command
        // string is ignored entirely by arg parsing.
        flag_case("arguments beat command string", &["clang", "-DBAR"])
            .with_command("cl.exe /DFOO")
            .with_compiler("clang")
            .with_defines(&["BAR"]),
    ];
    for case in cases {
        check_flag_case(&case);
    }
}

#[test]
fn parse_args_response_files_lock_current_behavior() {
    // Current code has NO response-file support; lock that fact.
    let cases = vec![
        // QUIRK: an @response token mid-list is a non-flag positional and
        // is silently dropped (not passed through, not in raw_flags).
        flag_case("@rsp mid-list dropped", &["cl.exe", "@rsp.rsp", "main.c"]),
        // QUIRK: an @response token as the FIRST arg is consumed as the
        // compiler name.
        flag_case("@rsp as first arg becomes compiler", &["@rsp.rsp"])
            .with_compiler("@rsp.rsp"),
        // QUIRK: same drop behavior via the command-string path.
        flag_case("@rsp in command string dropped", &[])
            .with_command("cl.exe @rsp.rsp /c")
            .with_raw_flags(&["/c"]),
    ];
    for case in cases {
        check_flag_case(&case);
    }
}

#[test]
fn parse_args_command_string_split_lock_current_behavior() {
    let cases = vec![
        flag_case("whitespace runs collapse", &[])
            .with_command("cc    -D FOO   -c")
            .with_compiler("cc")
            .with_defines(&["FOO"])
            .with_raw_flags(&["-c"]),
        // attached double-quoted path (no space after -I) becomes one token
        flag_case("attached quoted include path", &[])
            .with_command("cc -I\"C:/My Dir/inc\" -c main.c")
            .with_compiler("cc")
            .with_includes(&["C:/My Dir/inc"])
            .with_raw_flags(&["-c"]),
        // QUIRK: an empty quoted token materializes as NO token, so a
        // standalone -D reaches past it and consumes the next real token.
        flag_case("empty quoted token vanishes", &[])
            .with_command("cc -D \"\" x")
            .with_compiler("cc")
            .with_defines(&["x"]),
        // QUIRK: quotes are a plain toggle; unbalanced/mid-arg quotes glue
        // tokens together and keep interior spaces.
        flag_case("quote toggle glues fragments", &[])
            .with_command("cc -Da\"b c\"d")
            .with_compiler("cc")
            .with_defines(&["ab cd"]),
    ];
    for case in cases {
        check_flag_case(&case);
    }
}

#[test]
fn parse_compile_commands_json_command_form_lock_current_behavior() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write_compile_commands(
        root,
        serde_json::json!([{
            "directory": root,
            "command": "cl.exe -I \"C:/My Dir/inc\" /DFOO src/a.cpp",
            "file": "src/a.cpp",
        }]),
    );

    let result = BuildDiscoverer::new(root).discover().unwrap();

    assert_eq!(result.contexts.len(), 1);
    let context = &result.contexts[0];
    assert_eq!(context.name, "Default|x64");
    assert_eq!(context.compiler, "cl.exe");
    assert_eq!(context.defines, vec!["FOO"]);
    assert_eq!(context.include_dirs, paths_of(&["C:/My Dir/inc"]));
    assert!(context.raw_flags.is_empty());

    // QUIRK: the recorded TU arguments use split_whitespace() (quotes NOT
    // honored), diverging from the quote-aware splitter parse_args uses for
    // flag extraction: the quoted include path fragments into two args.
    assert_eq!(
        result.translation_units[0].compile_command.arguments,
        vec!["cl.exe", "-I", "\"C:/My", "Dir/inc\"", "/DFOO", "src/a.cpp"]
    );
    assert_eq!(result.translation_units[0].file_id, ariadne_repo::FileId::from_relative_path("src/a.cpp"));
}

#[test]
fn parse_compile_commands_json_path_resolution_lock_current_behavior() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Relative directory + relative file: joined verbatim, NOT canonicalized.
    write_compile_commands(
        root,
        serde_json::json!([{
            "directory": "build",
            "command": "cl.exe /c src/a.cpp",
            "file": "src/a.cpp",
        }]),
    );
    let result = BuildDiscoverer::new(root).discover().unwrap();
    let tu = &result.translation_units[0];
    assert_eq!(tu.source_path, PathBuf::from("build").join("src/a.cpp"));
    assert_eq!(tu.compile_command.directory, PathBuf::from("build"));
    assert_eq!(tu.file_id, ariadne_repo::FileId::from_relative_path("src/a.cpp"));

    // Absolute file under the repo root maps back to a relative FileId
    // (exercising the strip_prefix / verbatim-path fallback branch).
    let absolute_file = root.join("src/a.cpp");
    write_compile_commands(
        root,
        serde_json::json!([{
            "directory": root,
            "command": "cl.exe /c",
            "file": absolute_file,
        }]),
    );
    let result = BuildDiscoverer::new(root).discover().unwrap();
    let tu = &result.translation_units[0];
    assert_eq!(tu.source_path, absolute_file);
    assert_eq!(tu.file_id, ariadne_repo::FileId::from_relative_path("src/a.cpp"));
}

#[test]
fn parse_compile_commands_json_defaults_lock_current_behavior() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_compile_commands(root, serde_json::json!([]));
    let empty_result = BuildDiscoverer::new(root).discover().unwrap();
    assert!(empty_result.contexts.is_empty());
    assert!(empty_result.translation_units.is_empty());

    // QUIRK: an entry with neither command nor arguments is accepted and
    // defaults to compiler "cl.exe" / context Default|x64 / no recorded args.
    write_compile_commands(
        root,
        serde_json::json!([{
            "directory": root,
            "file": "src/a.cpp",
        }]),
    );
    let result = BuildDiscoverer::new(root).discover().unwrap();
    assert_eq!(result.contexts.len(), 1);
    assert_eq!(result.contexts[0].name, "Default|x64");
    assert_eq!(result.contexts[0].compiler, "cl.exe");
    let tu = &result.translation_units[0];
    assert!(tu.compile_command.command.is_none());
    assert!(tu.compile_command.arguments.is_empty());
    assert!(tu.compile_command.output.is_none());
}

#[test]
fn parse_compile_commands_json_rejects_malformed_lock_current_behavior() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join("compile_commands.json"), "[ {\"directory\":").unwrap();
    let broken = BuildDiscoverer::new(root).discover().unwrap_err();
    let broken_chain = format!("{:#}", broken);
    assert!(
        broken_chain.contains("Failed to parse compile_commands.json"),
        "error chain should name the parse failure: {broken_chain}"
    );

    fs::write(
        root.join("compile_commands.json"),
        serde_json::json!([{ "directory": root, "command": "cl.exe" }]).to_string(),
    )
    .unwrap();
    let missing_file = BuildDiscoverer::new(root).discover().unwrap_err();
    assert!(
        format!("{:#}", missing_file).contains("missing field `file`"),
        "directory-only entry must fail with a missing-field error"
    );

    fs::write(
        root.join("compile_commands.json"),
        serde_json::json!([{ "file": "src/a.cpp", "command": "cl.exe" }]).to_string(),
    )
    .unwrap();
    let missing_dir = BuildDiscoverer::new(root).discover().unwrap_err();
    assert!(
        format!("{:#}", missing_dir).contains("missing field `directory`"),
        "file-only entry must fail with a missing-field error"
    );
}

#[test]
fn parse_compile_commands_json_context_merge_lock_current_behavior() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write_compile_commands(
        root,
        serde_json::json!([
            {
                "directory": "b1",
                "command": "cl.exe /c /DDEBUG /arch:AVX /std:c++11 /Ia src/one.cpp",
                "file": "src/one.cpp",
            },
            {
                "directory": "b2",
                "command": "cl.exe /c /DDEBUG /arch:SSE2 /std:c++17 /Ib src/two.cpp",
                "file": "src/two.cpp",
            },
        ]),
    );

    let result = BuildDiscoverer::new(root).discover().unwrap();

    // Same context name merges entries into one CompilationContext.
    assert_eq!(result.contexts.len(), 1);
    let context = &result.contexts[0];
    assert_eq!(context.name, "Debug|x64");
    // QUIRK: arch / standard are first-wins per context (is_none guard);
    // the second entry's SSE2 / c++17 are silently ignored.
    assert_eq!(context.target_arch.as_deref(), Some("AVX"));
    assert_eq!(context.language_standard.as_deref(), Some("c++11"));
    assert_eq!(context.defines, vec!["DEBUG"]);
    assert_eq!(context.include_dirs, paths_of(&["a", "b"]));
    assert_eq!(context.raw_flags, vec!["/c"]);
    assert_eq!(result.translation_units.len(), 2);
    assert!(result.translation_units.iter().all(|unit| unit.context_name == "Debug|x64"));
}
