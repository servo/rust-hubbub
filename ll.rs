use libc::{c_char, c_int, c_void, size_t};

// errors.h

pub type Error = c_int;
pub const OK: c_int = 0;
pub const REPROCESS: c_int = 1;
pub const ENCODINGCHANGE: c_int = 2;
pub const PAUSED: c_int = 3;
pub const NOMEM: c_int = 5;
pub const BADPARM: c_int = 6;
pub const INVALID: c_int = 7;
pub const FILENOTFOUND: c_int = 8;
pub const NEEDDATA: c_int = 9;
pub const BADENCODING: c_int = 10;
pub const UNKNOWN: c_int = 11;

#[cfg(target_os = "macos")]
#[nolink]
#[link_args="-L../libhubbub -lhubbub -L../libparserutils -lparserutils -liconv"]
pub extern mod linking { }

#[cfg(target_os = "linux")]
#[nolink]
#[link_args="-L../libhubbub -lhubbub -L../libparserutils -lparserutils"]
pub extern mod linking { }

#[nolink]
pub extern mod error {
    pub fn hubbub_error_to_string(error: Error) -> *u8;
}

// parser.h

pub type Parser = c_void;

pub type ParserOptType = c_int;
pub const PARSER_TOKEN_HANDLER: c_int = 0;
pub const PARSER_ERROR_HANDLER: c_int = 1;
pub const PARSER_CONTENT_MODEL: c_int = 2;
pub const PARSER_TREE_HANDLER: c_int = 3;
pub const PARSER_DOCUMENT_NODE: c_int = 4;
pub const PARSER_ENABLE_SCRIPTING: c_int = 5;
pub const PARSER_PAUSE: c_int = 6;

pub struct ParserOptParamsTokenHandler {
    handler: *u8,
    pw: *c_void
}

pub struct ParserOptParamsErrorHandler {
    handler: *u8,
    pw: *c_void
}

pub struct ParserOptParamsContentModel {
    content_model: ContentModel
}

#[nolink]
pub extern mod parser {
    pub fn hubbub_parser_create(enc: *u8,
                                fix_enc: bool,
                                alloc: *u8,
                                pw: *c_void,
                                parser: **Parser)
                             -> Error;
    pub fn hubbub_parser_destroy(parser: *Parser) -> Error;
    pub fn hubbub_parser_setopt(parser: *Parser, opt_type: ParserOptType, params: *c_void)
                             -> Error;
    pub fn hubbub_parser_parse_chunk(parser: *Parser, data: *u8, len: size_t) -> Error;
    pub fn hubbub_parser_insert_chunk(parser: *Parser, data: *u8, len: size_t) -> Error;
    pub fn hubbub_parser_completed(parser: *Parser) -> Error;
    pub fn hubbub_parser_read_charset(parser: *Parser, source: *CharsetSource) -> *c_char;
}

// tree.h

pub struct TreeHandler {
    create_comment: *u8,
    create_doctype: *u8,
    create_element: *u8,
    create_text: *u8,
    ref_node: *u8,
    unref_node: *u8,
    append_child: *u8,
    insert_before: *u8,
    remove_child: *u8,
    clone_node: *u8,
    reparent_children: *u8,
    get_parent: *u8,
    has_children: *u8,
    form_associate: *u8,
    add_attributes: *u8,
    set_quirks_mode: *u8,
    encoding_change: *u8,
    complete_script: *u8,
    ctx: *c_void
}

// types.h

// Source of charset information, in order of importance.
// A client-dictated charset will override all others.
// A document-specified charset will override autodetection or the default.
pub type CharsetSource = c_int;
pub const CHARSET_UNKNOWN: c_int = 0;
pub const CHARSET_TENTATIVE: c_int = 1;
pub const CHARSET_CONFIDENT: c_int = 2;

// Content model flag
pub type ContentModel = c_int;
pub const CONTENT_MODEL_PCDATA: c_int = 0;
pub const CONTENT_MODEL_RCDATA: c_int = 1;
pub const CONTENT_MODEL_CDATA: c_int = 2;
pub const CONTENT_MODEL_PLAINTEXT: c_int = 3;

// Quirks mode flag
pub type QuirksMode = c_int;
pub const QUIRKS_MODE_NONE: c_int = 0;
pub const QUIRKS_MODE_LIMITED: c_int = 1;
pub const QUIRKS_MODE_FULL: c_int = 2;

pub type TokenType = c_int;
pub const TOKEN_DOCTYPE: c_int = 0;
pub const TOKEN_START_TAG: c_int = 1;
pub const TOKEN_END_TAG: c_int = 2;
pub const TOKEN_COMMENT: c_int = 3;
pub const TOKEN_CHARACTER: c_int = 4;
pub const TOKEN_EOF: c_int = 5;

pub type NS = c_int;
pub const NS_NULL: c_int = 0;
pub const NS_HTML: c_int = 1;
pub const NS_MATHML: c_int = 2;
pub const NS_SVG: c_int = 3;
pub const NS_XLINK: c_int = 4;
pub const NS_XML: c_int = 5;
pub const NS_XMLNS: c_int = 6;

pub struct String {
    ptr: *u8,
    len: size_t
}

pub struct Attribute {
    ns: NS,
    name: String,
    value: String,
}

pub struct Doctype {
    name: String,
    public_missing: bool,
    public_id: String,
    system_missing: bool,
    system_id: String,
    force_quirks: bool,
}

pub struct Tag {
    ns: NS,
    name: String,
    n_attributes: u32,
    attributes: *Attribute,
    self_closing: bool,
}

// Token data
pub struct Token {
    token_type: TokenType,
    data: u8,   // union: one of Doctype, Tag, Comment (string), or Character (string)
}

