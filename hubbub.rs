// High-level bindings to the Hubbub HTML5 parser.

use libc::{c_char, c_void, size_t};
use ptr::{addr_of, null, offset};
use unsafe::reinterpret_cast;

enum QuirksMode {
    NoQuirks,
    LimitedQuirks,
    FullQuirks
}

enum Ns {
    NullNs,
    HtmlNs,
    MathMlNs,
    SvgNs,
    XLinkNs,
    XmlNs,
    XmlNsNs
}

struct Doctype {
    name: &str;
    public_id: option<&str>;
    system_id: option<&str>;
    force_quirks: bool;
}

struct Attribute {
    ns: Ns;
    name: &str;
    value: &str;
}

struct Tag {
    ns: Ns;
    name: &str;
    attributes: ~[Attribute];
    self_closing: bool;
}

// FIXME: This is terribly type-unsafe. But we don't have working generic extern functions yet...
type Node = uint;

struct TreeHandler {
    create_comment: @fn(data: &str) -> Node;
    create_doctype: @fn(doctype: &Doctype) -> Node;
    create_element: @fn(tag: &Tag) -> Node;
    create_text: @fn(data: &str) -> Node;
    ref_node: @fn(node: Node);
    unref_node: @fn(node: Node);
    append_child: @fn(parent: Node, child: Node) -> Node;
    insert_before: @fn(parent: Node, child: Node) -> Node;
    remove_child: @fn(parent: Node, child: Node) -> Node;
    clone_node: @fn(node: Node, deep: bool) -> Node;
    reparent_children: @fn(node: Node, new_parent: Node) -> Node;
    get_parent: @fn(node: Node, element_only: bool) -> Node;
    has_children: @fn(node: Node) -> bool;
    form_associate: @fn(form: Node, node: Node);
    add_attributes: @fn(node: Node, attribute: &[Attribute]);
    set_quirks_mode: @fn(mode: QuirksMode);
    encoding_change: @fn(encname: &str);
    complete_script: @fn(script: Node);
}

struct TreeHandlerPair {
    tree_handler: @TreeHandler;
    ll_tree_handler: ll::TreeHandler;
}

struct Parser {
    hubbub_parser: *ll::Parser;
    mut tree_handler: option<TreeHandlerPair>;

    drop {
        ll::parser::hubbub_parser_destroy(self.hubbub_parser);
    }
}

fn Parser(encoding: &str, fix_encoding: bool) -> Parser unsafe {
    let hubbub_parser = null();
    let hubbub_error = do str::as_c_str(encoding) |encoding_c| {
        ll::parser::hubbub_parser_create(reinterpret_cast(encoding_c), fix_encoding, allocator,
                                         null(), addr_of(hubbub_parser))
    };
    assert hubbub_error == ll::OK;
    return Parser {
        hubbub_parser: hubbub_parser,
        tree_handler: none
    };
}

impl Parser {
    fn set_tree_handler(&self, tree_handler: @TreeHandler) unsafe {
        self.tree_handler = some(TreeHandlerPair {
            tree_handler: tree_handler,
            ll_tree_handler: ll::TreeHandler {
                create_comment: tree_callbacks::create_comment,
                create_doctype: tree_callbacks::create_doctype,
                create_element: tree_callbacks::create_element,
                create_text: tree_callbacks::create_text,
                ref_node: tree_callbacks::ref_node,
                unref_node: tree_callbacks::unref_node,
                append_child: tree_callbacks::append_child,
                insert_before: tree_callbacks::insert_before,
                remove_child: tree_callbacks::remove_child,
                clone_node: tree_callbacks::clone_node,
                reparent_children: tree_callbacks::reparent_children,
                get_parent: tree_callbacks::get_parent,
                has_children: tree_callbacks::has_children,
                form_associate: tree_callbacks::form_associate,
                add_attributes: tree_callbacks::add_attributes,
                set_quirks_mode: tree_callbacks::set_quirks_mode,
                encoding_change: tree_callbacks::encoding_change,
                complete_script: tree_callbacks::complete_script,
                ctx: reinterpret_cast(addr_of(self.tree_handler))
            }
        });

        let ptr: *ll::TreeHandler;
        match self.tree_handler {
            none =>
                fail ~"not possible",
            some(ref tree_handler_pair) =>
                ptr = reinterpret_cast(&tree_handler_pair.ll_tree_handler)
        }

        debug!("setting tree handler");
        let hubbub_error = ll::parser::hubbub_parser_setopt(self.hubbub_parser,
                                                            ll::PARSER_TREE_HANDLER,
                                                            reinterpret_cast(&ptr));
        assert hubbub_error == ll::OK;
    }

    fn set_document_node(&self, node: Node) unsafe {
        let hubbub_error = ll::parser::hubbub_parser_setopt(self.hubbub_parser,
                                                            ll::PARSER_DOCUMENT_NODE,
                                                            reinterpret_cast(&node));
        assert hubbub_error == ll::OK;
    }

    fn enable_scripting(&self, enable: bool) unsafe {
        let hubbub_error = ll::parser::hubbub_parser_setopt(self.hubbub_parser,
                                                            ll::PARSER_ENABLE_SCRIPTING,
                                                            reinterpret_cast(&enable));
        assert hubbub_error == ll::OK;
    }

    fn parse_chunk(&self, data: &[u8]) unsafe {
        let ptr = vec::unsafe::to_ptr_slice(data);
        let hubbub_error = ll::parser::hubbub_parser_parse_chunk(self.hubbub_parser, ptr,
                                                                 data.len() as size_t);
        assert hubbub_error == ll::OK;
    }

    fn insert_chunk(&self, data: &[u8]) unsafe {
        let ptr = vec::unsafe::to_ptr_slice(data);
        let hubbub_error = ll::parser::hubbub_parser_insert_chunk(self.hubbub_parser, ptr,
                                                                  data.len() as size_t);
        assert hubbub_error == ll::OK;
    }

    fn completed(&self) {
        let hubbub_error = ll::parser::hubbub_parser_completed(self.hubbub_parser);
        assert hubbub_error == ll::OK;
    }
}

mod tree_callbacks {

    // Data conversions

    fn from_hubbub_node(node: *c_void) -> Node unsafe {
        return reinterpret_cast(node);
    }

    fn from_hubbub_string(string: &a/ll::String) -> &a/str unsafe {
        return str::unsafe::from_buf_len_nocopy(&(*string).ptr, (*string).len as uint);
    }

    fn from_hubbub_ns(ns: ll::NS) -> Ns {
        match ns {
            0 => NullNs,
            1 => HtmlNs,
            2 => MathMlNs,
            3 => SvgNs,
            4 => XLinkNs,
            5 => XmlNs,
            6 => XmlNsNs,
            _ => fail ~"unknown namespace"
        }
    }

    fn from_hubbub_quirks_mode(mode: ll::QuirksMode) -> QuirksMode {
        match mode {
            0 => NoQuirks,
            1 => LimitedQuirks,
            2 => FullQuirks,
            _ => fail ~"unknown quirks mode"
        }
    }

    fn from_hubbub_attributes(attributes: *ll::Attribute, n_attributes: u32)
                           -> ~[Attribute] unsafe {
        do vec::from_fn(n_attributes as uint) |i| {
            let attribute = offset(attributes, i);
            Attribute {
                ns: from_hubbub_ns((*attribute).ns),
                name: from_hubbub_string(&(*attribute).name),
                value: from_hubbub_string(&(*attribute).value)
            }
        }
    }

    fn from_hubbub_tag(tag: &a/ll::Tag) -> Tag/&a unsafe {
        Tag {
            ns: from_hubbub_ns((*tag).ns),
            name: from_hubbub_string(&(*tag).name),
            attributes: from_hubbub_attributes((*tag).attributes, (*tag).n_attributes),
            self_closing: (*tag).self_closing
        }
    }

    fn from_hubbub_doctype(doctype: &a/ll::Doctype) -> Doctype/&a unsafe {
        Doctype {
            name: from_hubbub_string(&doctype.name),
            public_id:
                if doctype.public_missing {
                    none
                } else {
                    some(from_hubbub_string(&doctype.public_id))
                },
            system_id:
                if doctype.system_missing {
                    none
                } else {
                    some(from_hubbub_string(&doctype.system_id))
                },
            force_quirks: doctype.force_quirks
        }
    }

    fn to_hubbub_node(node: Node) -> *c_void unsafe {
        return reinterpret_cast(node);
    }

    // Callbacks

    extern fn create_comment(ctx: *c_void, data: *ll::String, result: *mut *c_void)
                          -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        let data = reinterpret_cast(data);
        *result = to_hubbub_node(self.tree_handler.create_comment(from_hubbub_string(data)));
        return ll::OK;
    }

    extern fn create_doctype(ctx: *c_void, doctype: *ll::Doctype, result: *mut *c_void)
                          -> ll::Error unsafe {
        debug!("ll create doctype");
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        let doctype: &ll::Doctype = reinterpret_cast(doctype);
        *result = to_hubbub_node(self.tree_handler.create_doctype(&from_hubbub_doctype(doctype)));
        return ll::OK;
    }

    extern fn create_element(ctx: *c_void, tag: *ll::Tag, result: *mut *c_void)
                          -> ll::Error unsafe {
        debug!("ll create element");
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        let tag: &ll::Tag = reinterpret_cast(tag);
        *result = to_hubbub_node(self.tree_handler.create_element(&from_hubbub_tag(tag)));
        return ll::OK;
    }

    extern fn create_text(ctx: *c_void, data: *ll::String, result: *mut *c_void)
                       -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        let data = reinterpret_cast(data);
        *result = to_hubbub_node(self.tree_handler.create_text(from_hubbub_string(data)));
        return ll::OK;
    }

    extern fn ref_node(ctx: *c_void, node: *c_void) -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        self.tree_handler.ref_node(from_hubbub_node(node));
        return ll::OK;
    }

    extern fn unref_node(ctx: *c_void, node: *c_void) -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        self.tree_handler.unref_node(from_hubbub_node(node));
        return ll::OK;
    }

    extern fn append_child(ctx: *c_void, parent: *c_void, child: *c_void, result: *mut *c_void)
                        -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        *result = to_hubbub_node(self.tree_handler.append_child(from_hubbub_node(parent),
                                                                from_hubbub_node(child)));
        return ll::OK;
    }

    extern fn insert_before(ctx: *c_void, parent: *c_void, child: *c_void, result: *mut *c_void)
                        -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        *result = to_hubbub_node(self.tree_handler.insert_before(from_hubbub_node(parent),
                                                                 from_hubbub_node(child)));
        return ll::OK;
    }

    extern fn remove_child(ctx: *c_void, parent: *c_void, child: *c_void, result: *mut *c_void)
                        -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        *result = to_hubbub_node(self.tree_handler.remove_child(from_hubbub_node(parent),
                                                                from_hubbub_node(child)));
        return ll::OK;
    }

    extern fn clone_node(ctx: *c_void, node: *c_void, deep: bool, result: *mut *c_void)
                      -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        *result = to_hubbub_node(self.tree_handler.clone_node(from_hubbub_node(node), deep));
        return ll::OK;
    }

    extern fn reparent_children(ctx: *c_void, node: *c_void, new_parent: *c_void)
                             -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        self.tree_handler.reparent_children(from_hubbub_node(node), from_hubbub_node(new_parent));
        return ll::OK;
    }

    extern fn get_parent(ctx: *c_void, node: *c_void, element_only: bool, result: *mut *c_void)
                      -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        *result = to_hubbub_node(self.tree_handler.get_parent(from_hubbub_node(node),
                                                              element_only));
        return ll::OK;
    }

    extern fn has_children(ctx: *c_void, node: *c_void, result: *mut bool) -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        *result = self.tree_handler.has_children(from_hubbub_node(node));
        return ll::OK;
    }

    extern fn form_associate(ctx: *c_void, form: *c_void, node: *c_void) -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        self.tree_handler.form_associate(from_hubbub_node(form), from_hubbub_node(node));
        return ll::OK;
    }

    extern fn add_attributes(ctx: *c_void,
                             node: *c_void,
                             attributes: *ll::Attribute,
                             n_attributes: u32)
                          -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        self.tree_handler.add_attributes(from_hubbub_node(node),
                                         from_hubbub_attributes(attributes, n_attributes));
        return ll::OK;
    }

    extern fn set_quirks_mode(ctx: *c_void, mode: ll::QuirksMode) -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        self.tree_handler.set_quirks_mode(from_hubbub_quirks_mode(mode));
        return ll::OK;
    }

    extern fn encoding_change(ctx: *c_void, encname: *c_char) -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        self.tree_handler.encoding_change(str::unsafe::from_c_str(encname));
        return ll::OK;
    }

    extern fn complete_script(ctx: *c_void, script: *c_void) -> ll::Error unsafe {
        let self_opt: &option<TreeHandlerPair> = reinterpret_cast(ctx);
        let self = self_opt.get();
        self.tree_handler.complete_script(from_hubbub_node(script));
        return ll::OK;
    }
}

extern fn allocator(ptr: *c_void, len: size_t, _pw: *c_void) -> *c_void {
    return libc::realloc(ptr, len);
}

