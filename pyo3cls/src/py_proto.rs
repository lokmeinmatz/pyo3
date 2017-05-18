// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::{Tokens, ToTokens};

use py_method;
use func::{MethodProto, impl_method_proto};


struct Methods {
    methods: &'static [&'static str],
}

struct Proto {
    name: &'static str,
    //py_methods: &'static [&'static str],
    methods: &'static [MethodProto],
}

static DEFAULT_METHODS: Methods = Methods {
    methods: &[],
};

static CONTEXT_METHODS: Methods = Methods {
    methods: &["__enter__", "__exit__"],
};

static DESCR_METHODS: Methods = Methods {
    methods: &["__delete__", "__set_name__"],
};

static NUM_METHODS: Methods = Methods {
    methods: &[
        "__radd__", "__rsub__", "__rmul__", "__rmatmul__", "__rtruediv__",
        "__rfloordiv__", "__rmod__", "__rdivmod__", "__rpow__", "__rlshift__",
        "__rrshift__", "__rand__", "__rxor__", "__ror__", "__complex__",
        "__round__"
    ],
};

static ASYNC: Proto = Proto {
    name: "Async",
    //py_methods: &[],
    methods: &[
        MethodProto::Unary{
            name: "__await__",
            proto: "_pyo3::class::async::PyAsyncAwaitProtocol"},
        MethodProto::Unary{
            name: "__aiter__",
            proto: "_pyo3::class::async::PyAsyncAiterProtocol"},
        MethodProto::Unary{
            name: "__anext__",
            proto: "_pyo3::class::async::PyAsyncAnextProtocol"},
    ],
};

static ITER: Proto = Proto {
    name: "Iter",
    //py_methods: &[],
    methods: &[
        MethodProto::Unary{
            name: "__iter__",
            proto: "_pyo3::class::iter::PyIterIterProtocol"},
        MethodProto::Unary{
            name: "__next__",
            proto: "_pyo3::class::iter::PyIterNextProtocol"},
    ],
};


static MAPPING: Proto = Proto {
    name: "Mapping",
    //py_methods: &[],
    methods: &[
        MethodProto::Len{
            name: "__len__",
            proto: "_pyo3::class::mapping::PyMappingLenProtocol"},
        MethodProto::Binary{
            name: "__getitem__",
            arg: "Key",
            proto: "_pyo3::class::mapping::PyMappingGetItemProtocol"},
        MethodProto::Ternary{
            name: "__setitem__",
            arg1: "Key",
            arg2: "Value",
            proto: "_pyo3::class::mapping::PyMappingSetItemProtocol"},
        MethodProto::Binary{
            name: "__delitem__",
            arg: "Key",
            proto: "_pyo3::class::mapping::PyMappingDelItemProtocol"},
    ],
};


pub fn build_py_proto(ast: &mut syn::Item) -> Tokens {
    match ast.node {
        syn::ItemKind::Impl(_, _, _, ref path, ref ty, ref mut impl_items) => {
            if let &Some(ref path) = path {
                if let Some(segment) = path.segments.last() {
                    match segment.ident.as_ref() {
                        "PyObjectProtocol" =>
                            impl_protocol("_pyo3::class::basic::PyObjectProtocolImpl",
                                          path.clone(), ty, impl_items, &DEFAULT_METHODS),
                        "PyAsyncProtocol" =>
                            impl_proto_impl(ty, impl_items, &ASYNC),
                        "PyMappingProtocol" =>
                            impl_proto_impl(ty, impl_items, &MAPPING),
                        "PyIterProtocol" =>
                            impl_proto_impl(ty, impl_items, &ITER),
                        "PyBufferProtocol" =>
                            impl_protocol("_pyo3::class::buffer::PyBufferProtocolImpl",
                                          path.clone(), ty, impl_items, &DEFAULT_METHODS),
                        "PyContextProtocol" =>
                            impl_protocol("_pyo3::class::context::PyContextProtocolImpl",
                                          path.clone(), ty, impl_items, &CONTEXT_METHODS),
                        "PyDescrProtocol" =>
                            impl_protocol("_pyo3::class::descr::PyDescrProtocolImpl",
                                          path.clone(), ty, impl_items, &DESCR_METHODS),
                        "PyGCProtocol" =>
                            impl_protocol("_pyo3::class::gc::PyGCProtocolImpl",
                                          path.clone(), ty, impl_items, &DEFAULT_METHODS),
                        "PySequenceProtocol" =>
                            impl_protocol("_pyo3::class::mapping::PySequenceProtocolImpl",
                                          path.clone(), ty, impl_items, &DEFAULT_METHODS),
                        "PyNumberProtocol" =>
                            impl_protocol("_pyo3::class::number::PyNumberProtocolImpl",
                                          path.clone(), ty, impl_items, &NUM_METHODS),
                        _ => panic!("#[proto] can not be used with this block"),
                    }
                } else {
                    panic!("#[proto] can only be used with protocol trait implementations")
                }
            } else {
                panic!("#[proto] can only be used with protocol trait implementations")
            }
        },
        _ => panic!("#[proto] can only be used with Impl blocks"),
    }
}

fn impl_proto_impl(ty: &Box<syn::Ty>, impls: &mut Vec<syn::ImplItem>, proto: &Proto) -> Tokens {
    let mut tokens = Tokens::new();

    for iimpl in impls.iter_mut() {
        match iimpl.node {
            syn::ImplItemKind::Method(ref mut sig, _) => {
                for m in proto.methods {
                    if m.eq(iimpl.ident.as_ref()) {
                        impl_method_proto(ty, sig, m).to_tokens(&mut tokens);
                    }
                }
            },
            _ => (),
        }
    }

    // unique mod name
    let p = proto.name;
    let n = match ty.as_ref() {
        &syn::Ty::Path(_, ref p) => {
        p.segments.last().as_ref().unwrap().ident.as_ref()
    }
    _ => "PROTO_METHODS"
    };

    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_{}_{}", n, p));
    quote! {
        #[feature(specialization)]
        #[allow(non_upper_case_globals, unused_attributes,
                unused_qualifications, unused_variables)]
        const #dummy_const: () = {
            extern crate pyo3 as _pyo3;

            #tokens
        };
    }
}

fn impl_protocol(name: &'static str,
                 path: syn::Path, ty: &Box<syn::Ty>,
                 impls: &mut Vec<syn::ImplItem>, methods: &Methods) -> Tokens {
    let mut py_methods = Vec::new();

    // get method names in impl block
    let mut meth = Vec::new();
    for iimpl in impls.iter_mut() {
        match iimpl.node {
            syn::ImplItemKind::Method(ref mut sig, ref mut block) => {
                if methods.methods.contains(&iimpl.ident.as_ref()) {
                    py_methods.push(py_method::gen_py_method(
                        ty, &iimpl.ident, sig, block, &mut iimpl.attrs));
                } else {
                    meth.push(String::from(iimpl.ident.as_ref()));
                }
            },
            _ => (),
        }
    }

    // set trait name
    let mut path = path;
    {
        let mut last = path.segments.last_mut().unwrap();
        last.ident = syn::Ident::from(name);
    }

    let i = syn::Ident::from(name);
    let tokens = if py_methods.is_empty() {
        quote! {
            impl #i for #ty {
                fn methods() -> &'static [&'static str] {
                    static METHODS: &'static [&'static str] = &[#(#meth),*];
                    METHODS
                }
            }
        }
    } else {
        quote! {
            impl #i for #ty {
                fn methods() -> &'static [&'static str] {
                    static METHODS: &'static [&'static str] = &[#(#meth,),*];
                    METHODS
                }

                fn py_methods() -> &'static [pyo3::class::PyMethodDefType] {
                    static METHODS: &'static [pyo3::class::PyMethodDefType] = &[
                        #(#py_methods),*
                    ];
                    METHODS
                }
            }
        }
    };
    let name = name.split("::").last().unwrap();
    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_{}", name));
    quote! {
        #[feature(specialization)]
        #[allow(non_upper_case_globals, unused_attributes,
                unused_qualifications, unused_variables)]
        const #dummy_const: () = {
            extern crate pyo3 as _pyo3;

            #tokens
        };
    }
}
