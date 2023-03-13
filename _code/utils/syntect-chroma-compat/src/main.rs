// FIXME: Scrapping this for now. Not entirely happy with the granularity of syntect or chroma.
// Maybe the html classes are more limited in some way. Will have to look into later :/

use lol_html::{element, rewrite_str, RewriteStrSettings};
use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

#[derive(Clone, Copy)]
enum ChromaClass {
    Comment,
    CommentHashband,
    CommentMultiline,
    CommentPreproc,
    CommentPreprocessorFile,
    CommentSingle,
    CommentSpecial,
    Error,
    Generic,
    GenericDeleted,
    GenericEmph,
    GenericError,
    GenericHeading,
    GenericInserted,
    GenericOutput,
    GenericPrompt,
    GenericStrong,
    GenericSubheading,
    GenericTraceback,
    GenericUnderline,
    Keyword,
    KeywordConstant,
    KeywordDeclaration,
    KeywordNamespace,
    KeywordPseudo,
    KeywordReserved,
    KeywordType,
    Literal,
    LiteralDate,
    LiteralNumber,
    LiteralNumberBin,
    LiteralNumberFloat,
    LiteralNumberHex,
    LiteralNumberInteger,
    LiteralNumberIntegerLong,
    LiteralNumberOct,
    LiteralString,
    LiteralStringAffix,
    LiteralStringBacktick,
    LiteralStringChar,
    LiteralStringDelimiter,
    LiteralStringDoc,
    LiteralStringDouble,
    LiteralStringEscape,
    LiteralStringHeredoc,
    LiteralStringInterpol,
    LiteralStringOther,
    LiteralStringRegex,
    LiteralStringSingle,
    LiteralStringSymbol,
    Name,
    NameAttribute,
    NameBuiltin,
    NameBuiltinPseudo,
    NameClass,
    NameConstant,
    NameDecorator,
    NameEntity,
    NameException,
    NameFunction,
    NameFunctionMagic,
    NameLabel,
    NameNamespace,
    NameOther,
    NameProperty,
    NameTag,
    NameVariable,
    NameVariableClass,
    NameVariableGlobal,
    NameVariableInstance,
    NameVariableMagic,
    Operator,
    OperatorWord,
    Other,
    Punctuation,
}

impl ChromaClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::Comment => "c",
            Self::CommentHashband => "ch",
            Self::CommentMultiline => "cm",
            Self::CommentPreproc => "cp",
            Self::CommentPreprocessorFile => "cpf",
            Self::CommentSingle => "c1",
            Self::CommentSpecial => "sc",
            Self::Error => "err",
            Self::Generic => "g",
            Self::GenericDeleted => "gd",
            Self::GenericEmph => "ge",
            Self::GenericError => "gr",
            Self::GenericHeading => "gh",
            Self::GenericInserted => "gi",
            Self::GenericOutput => "go",
            Self::GenericPrompt => "gp",
            Self::GenericStrong => "gs",
            Self::GenericSubheading => "gu",
            Self::GenericTraceback => "gt",
            Self::GenericUnderline => "gl",
            Self::Keyword => "k",
            Self::KeywordConstant => "kc",
            Self::KeywordDeclaration => "kd",
            Self::KeywordNamespace => "kn",
            Self::KeywordPseudo => "kp",
            Self::KeywordReserved => "kr",
            Self::KeywordType => "kt",
            Self::Literal => "l",
            Self::LiteralDate => "ld",
            Self::LiteralNumber => "m",
            Self::LiteralNumberBin => "mb",
            Self::LiteralNumberFloat => "mf",
            Self::LiteralNumberHex => "mh",
            Self::LiteralNumberInteger => "mi",
            Self::LiteralNumberIntegerLong => "il",
            Self::LiteralNumberOct => "mo",
            Self::LiteralString => "s",
            Self::LiteralStringAffix => "sa",
            Self::LiteralStringBacktick => "sb",
            Self::LiteralStringChar => "sc",
            Self::LiteralStringDelimiter => "dl",
            Self::LiteralStringDoc => "sd",
            Self::LiteralStringDouble => "s2",
            Self::LiteralStringEscape => "se",
            Self::LiteralStringHeredoc => "sh",
            Self::LiteralStringInterpol => "si",
            Self::LiteralStringOther => "sx",
            Self::LiteralStringRegex => "sr",
            Self::LiteralStringSingle => "s1",
            Self::LiteralStringSymbol => "ss",
            Self::Name => "n",
            Self::NameAttribute => "na",
            Self::NameBuiltin => "nb",
            Self::NameBuiltinPseudo => "bp",
            Self::NameClass => "nc",
            Self::NameConstant => "no",
            Self::NameDecorator => "nd",
            Self::NameEntity => "ni",
            Self::NameException => "ne",
            Self::NameFunction => "nf",
            Self::NameFunctionMagic => "fm",
            Self::NameLabel => "nl",
            Self::NameNamespace => "nn",
            Self::NameOther => "nx",
            Self::NameProperty => "py",
            Self::NameTag => "nt",
            Self::NameVariable => "nv",
            Self::NameVariableClass => "vc",
            Self::NameVariableGlobal => "vg",
            Self::NameVariableInstance => "vi",
            Self::NameVariableMagic => "vm",
            Self::Operator => "o",
            Self::OperatorWord => "ow",
            Self::Other => "x",
            Self::Punctuation => "p",
        }
    }
}

// Compare to bat syntax highlighting to see how things should be grouped
const CLASS_PREFIX_REMAP: &[(&str, Option<ChromaClass>)] = &[
    ("source", None),
    ("punctuation definition deleted", None),
    ("punctuation definition inserted", None),
    ("punctuation definition string", None),
    ("punctuation definition comment", None),
    ("meta separator diff", None),
    ("source", None),
    ("meta path rust", None),
    ("meta block rust", None),
    ("meta struct rust", None),
    ("meta generic rust", None),
    (
        "comment line double-slash rust",
        Some(ChromaClass::CommentSingle),
    ),
    (
        "comment line documentation rust",
        Some(ChromaClass::LiteralStringDoc),
    ),
    ("meta diff range", Some(ChromaClass::GenericSubheading)),
    ("markup deleted diff", Some(ChromaClass::GenericDeleted)),
    (
        "punctuation definition separator diff",
        Some(ChromaClass::GenericStrong),
    ),
    ("punctuation", Some(ChromaClass::Punctuation)),
    ("markup inserted diff", Some(ChromaClass::GenericInserted)),
    ("keyword other rust", Some(ChromaClass::Keyword)),
    ("storage type rust", Some(ChromaClass::KeywordType)),
    ("support type rust", Some(ChromaClass::KeywordType)),
    ("support type struct rust", Some(ChromaClass::KeywordType)),
    ("storage type struct rust", Some(ChromaClass::Keyword)),
    ("storage type function rust", Some(ChromaClass::Keyword)),
    ("entity name struct rust", Some(ChromaClass::NameClass)),
    (
        "string quoted double rust",
        // TODO: LiteralStringdouble
        Some(ChromaClass::LiteralString),
    ),
    ("keyword operator rust", Some(ChromaClass::Operator)),
    ("support function rust", Some(ChromaClass::NameProperty)),
    // TODO: OperatorWord
    ("storage modifier rust", Some(ChromaClass::Operator)),
    ("entity name function rust", Some(ChromaClass::NameFunction)),
    // -- vv Finish classifying below vv --
    ("variable other member rust", None),
    ("meta function rust", None),
    ("entity name function rust", None),
    ("meta function parameters rust", None),
    ("variable parameter rust", None),
    ("keyword operator rust", None),
    ("storage modifier rust", None),
    ("meta function return-type rust", None),
    ("meta group rust", None),
    ("support function rust", None),
    ("comment line double-slash rust", None),
    ("meta annotation rust", None),
    ("variable annotation rust", None),
    ("meta annotation parameters rust", None),
    ("meta impl rust", None),
    ("storage type impl rust", None),
    ("entity name impl rust", None),
    ("comment line documentation rust", None),
    ("storage modifier lifetime rust", None),
    ("storage type type rust", None),
    ("entity name type rust", None),
    ("keyword control rust", None),
];

fn main() {
    let syn_set = SyntaxSet::load_defaults_newlines();
    let syn_ref = syn_set.find_syntax_by_extension("rs").unwrap();
    let mut html_generator =
        ClassedHTMLGenerator::new_with_class_style(syn_ref, &syn_set, ClassStyle::Spaced);
    for line in LinesWithEndings::from(r###"println!("Hi {there}");"###) {
        html_generator
            .parse_html_for_line_which_includes_newline(line)
            .unwrap();
    }
    let html = html_generator.finalize();

    println!("{html}\n\n");

    let html = rewrite_str(
        &html,
        RewriteStrSettings {
            element_content_handlers: vec![element!("span[class]", |el| {
                if let Some(cls) = el.get_attribute("class") {
                    let mut has_matched = false;
                    for (prefix_from, to) in CLASS_PREFIX_REMAP {
                        if cls.starts_with(prefix_from) {
                            match to {
                                Some(cls) => {
                                    el.set_attribute("class", cls.as_str()).unwrap();
                                }
                                None => el.remove_and_keep_content(),
                            }

                            has_matched = true;

                            break;
                        }
                    }

                    assert!(
                        has_matched || el.attributes().is_empty(),
                        "No match on {:?}",
                        el.attributes()
                    );
                }

                Ok(())
            })],
            ..RewriteStrSettings::default()
        },
    )
    .unwrap();

    // println!("<pre class=\"chroma\"><code>{}</code></pre>", html);
}
