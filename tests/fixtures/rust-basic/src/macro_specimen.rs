// rust-language R-4 fixture: the honest D-2 caveat specimen. The
// `mod generated_by_macro` below exists only after macro expansion; syn
// reads written code, so the module it generates stays invisible to the
// analyzer. This is the specimen the analysis_caveat names ("macro
// expansion is out of scope"), kept tiny and ASCII - the fixture is never
// compiled by any suite.

macro_rules! declare_hidden_module {
    () => {
        mod generated_by_macro {
            pub fn invisible_without_expansion() -> u32 {
                1
            }
        }
    };
}

declare_hidden_module!();

pub fn written_code_only() -> u32 {
    2
}
