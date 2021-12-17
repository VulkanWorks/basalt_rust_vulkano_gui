use proc_macro2::TokenTree;
use proc_macro2::LineColumn;
use proc_macro2::Delimiter;
use proc_macro2::Ident;
use proc_macro2::Group;
use proc_macro2::Span;
use proc_macro2::Punct;
use proc_macro2::Spacing;
use proc_macro2::Literal;
use std::iter::once;

#[derive(Debug)]
struct KeyValue {
    line: usize,
    groups: Vec<String>,
    key: String,
    value: String,
    unit: Option<String>,
}

fn parse_group(input: proc_macro2::TokenStream, groups: Vec<String>) -> Vec<KeyValue> {
    let mut out = Vec::new();
    let mut line = 0;
    let mut key: Option<String> = None;
    let mut val: Option<String> = None;
    let mut unit: Option<String> = None;

    // 0: Read Key
    // 1: Read Colon
    // 2: Read Val
    // 3: Read Comma or Unit
    // 4: Read Comma if Unit present
    // 5: Read Comma after Group

    let mut state: u8 = 0;

    for tree in input {
        match state {
            0 => match tree {
                TokenTree::Ident(ref ident) => {
                    key = Some(format!("{}", ident));
                    line = tree.span().start().line;
                    state += 1;
                    continue;
                },
                _ => {
                    let LineColumn { line, column } = tree.span().start();
                    panic!("Expected key at [{}:{}]", line, column);
                }
            },
            1 => match tree {
                TokenTree::Punct(ref punct) => match punct.as_char() {
                    ':' => {
                        state += 1;
                        continue;
                    },
                    c => {
                        let LineColumn { line, column } = tree.span().start();
                        panic!("Expected ':' instead of '{}' at [{}:{}]", c, line, column);
                    }
                },
                _ => {
                    let LineColumn { line, column } = tree.span().start();
                    panic!("Expected ':' at [{}:{}]", line, column);
                }
            },
            2 => match tree {
                TokenTree::Literal(ref literal) => {
                    val = Some(format!("{}", literal));
                    state += 1;
                    continue;
                },
                TokenTree::Ident(ref ident) => {
                    val = Some(format!("{}", ident));
                    state += 1;
                    continue;
                },
                TokenTree::Group(ref group) => match group.delimiter() {
                    Delimiter::Brace => {
                        let mut sub_groups = groups.clone();
                        sub_groups.push(key.as_ref().unwrap().clone());
                        out.append(&mut parse_group(group.stream(), sub_groups));
                        state = 5;
                        continue;
                    },
                    delimiter => {
                        let disp = match delimiter {        
                            Delimiter::Parenthesis => "(",
                            Delimiter::Bracket => "[",
                            Delimiter::None => "Ø",
                            _ => unreachable!()
                        };

                        let LineColumn { line, column } = tree.span().start();
                        panic!("Expected value or '{{' instead of '{}' at [{}:{}]", disp, line, column);
                    }
                },
                _ => {
                    let LineColumn { line, column } = tree.span().start();
                    panic!("Expected value or '{{' at [{}:{}]", line, column);
                }
            },
            3 => match tree {
                TokenTree::Punct(ref punct) => match punct.as_char() {
                    ',' => {
                        out.push(KeyValue {
                            line,
                            key: key.take().unwrap(),
                            value: val.take().unwrap(),
                            groups: groups.clone(),
                            unit: unit.take(),
                        });

                        state = 0;
                        continue;
                    },
                    c => {
                        let LineColumn { line, column } = tree.span().start();
                        panic!("Expected ',' instead of '{}' at [{}:{}]", c, line, column);
                    }
                }
                TokenTree::Ident(ref ident) => {
                    unit = Some(format!("{}", ident));
                    state += 1;
                    continue;
                },
                _ => {
                    let LineColumn { line, column } = tree.span().start();
                    panic!("Expected ',' or unit (e.g. px, pct) at [{}:{}]", line, column);
                }
            },
            4 | 5 => match tree {
                TokenTree::Punct(ref punct) => match punct.as_char() {
                    ',' => {
                        if state == 4 {
                            out.push(KeyValue {
                                line,
                                key: key.take().unwrap(),
                                value: val.take().unwrap(),
                                groups: groups.clone(),
                                unit: unit.take(),
                            });
                        }

                        state = 0;
                        continue;
                    },
                    c => {
                        let LineColumn { line, column } = tree.span().start();
                        panic!("Expected ',' instead of '{}' at [{}:{}]", c, line, column);
                    }
                },
                _ => {
                    let LineColumn { line, column } = tree.span().start();
                    panic!("Expected ',' at [{}, {}]", line, column);
                }
            },
            _ => unreachable!()
        }
    }

    out
}

fn kv_trees<K: AsRef<str>>(key: K, value: Vec<TokenTree>) -> Vec<TokenTree> {
    let mut out: Vec<TokenTree> = Vec::new();
    out.push(Ident::new(key.as_ref(), Span::call_site()).into());
    out.push(Punct::new(':', Spacing::Alone).into());
    out.push(Ident::new("Some", Span::call_site()).into());
    out.push(Group::new(
        Delimiter::Parenthesis,
        proc_macro2::TokenStream::from_iter(value.into_iter())
    ).into());
    out.push(Punct::new(',', Spacing::Alone).into());
    out
}

fn enum_trees<E: AsRef<str>, V: AsRef<str>>(enu: E, var: V) -> Vec<TokenTree> {
    let mut out: Vec<TokenTree> = Vec::new();
    out.push(Ident::new(enu.as_ref(), Span::call_site()).into());
    out.push(Punct::new(':', Spacing::Joint).into());
    out.push(Punct::new(':', Spacing::Alone).into());
    out.push(Ident::new(var.as_ref(), Span::call_site()).into());
    out
}

fn px_or_pct_trees<S: AsRef<str>, D: AsRef<str>>(src_field: S, dst_field: D, kv: KeyValue) -> Vec<TokenTree> {
    let mut value_trees: Vec<TokenTree> = Vec::new();
    let is_pct = match kv.unit {
        Some(some) => match some.as_str() {
            "px" => false,
            "pct" => true,
            _ => panic!("Unknown unit type '{}' for '{}'. Expected either 'px' or 'pct' on line {}.",
                some,
                src_field.as_ref(),
                kv.line)
        }, None => false
    };

    let val: TokenTree = match kv.value.parse() {
        Ok(ok) => TokenTree::Literal(Literal::f32_unsuffixed(ok)),
        Err(_) => panic!("Invalid value for '{}'. Expected either an 'int' or a 'float' on line {}.",
            src_field.as_ref(),
            kv.line)
    };

    value_trees.push(Ident::new("PxOrPct", Span::call_site()).into());
    value_trees.push(Punct::new(':', Spacing::Joint).into());
    value_trees.push(Punct::new(':', Spacing::Alone).into());

    match is_pct {
        true => value_trees.push(Ident::new("Pct", Span::call_site()).into()),
        false => value_trees.push(Ident::new("Px", Span::call_site()).into()),
    }

    value_trees.push(Group::new(
        Delimiter::Parenthesis,
        proc_macro2::TokenStream::from_iter(once(val))
    ).into());

    kv_trees(dst_field.as_ref(), value_trees)
}

#[proc_macro]
pub fn style(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let key_values = parse_group(input, Vec::new());

    for kv in &key_values {
        println!("Line: {} | Groups: {:?} | Key: {} | Value: {} | Unit: {:?}",
            kv.line,
            kv.groups,
            kv.key,
            kv.value,
            kv.unit
        );
    }

    let mut trees: Vec<TokenTree> = Vec::new();
    
    for kv in key_values {
        if kv.groups.is_empty() {
            match kv.key.as_str() {
                "position" => {
                    let varient = match kv.value.as_str() {
                        "parent" => "Parent",
                        "window" => "Window",
                        "floating" => "Floating",
                        _ => panic!("Unknown position type '{}' on line {}. Possible types 'window', 'parent', & 'floating'.", kv.value, kv.line)
                    };

                    trees.append(&mut kv_trees("position", enum_trees("Position", varient)));
                },
                "pos_from_t" => trees.append(&mut px_or_pct_trees("pos_from_t", "pos_from_t", kv)),
                "pos_from_b" => trees.append(&mut px_or_pct_trees("pos_from_b", "pos_from_b", kv)),
                "pos_from_l" => trees.append(&mut px_or_pct_trees("pos_from_l", "pos_from_l", kv)),
                "pos_from_r" => trees.append(&mut px_or_pct_trees("pos_from_r", "pos_from_r", kv)),
                "vert_offset" => trees.append(&mut px_or_pct_trees("vert_offset", "vert_offset", kv)),
                "hori_offset" => trees.append(&mut px_or_pct_trees("hori_offset", "hori_offset", kv)),
                "width" => trees.append(&mut px_or_pct_trees("width", "width", kv)),
                "height" => trees.append(&mut px_or_pct_trees("height", "height", kv)),
                "height_offset" => trees.append(&mut px_or_pct_trees("height_offset", "height_offset", kv)),
                "width_offset" => trees.append(&mut px_or_pct_trees("width_offset", "width_offset", kv)),
                "hidden" => todo!(),
                "opacity_pct" => todo!(),
                "custom_verts" => todo!(),
                "pass_input" => todo!(),
                "margin_t" => trees.append(&mut px_or_pct_trees("margin_t", "margin_t", kv)),
                "margin_b" => trees.append(&mut px_or_pct_trees("margin_b", "margin_b", kv)),
                "margin_l" => trees.append(&mut px_or_pct_trees("margin_l", "margin_l", kv)),
                "margin_r" => trees.append(&mut px_or_pct_trees("margin_r", "margin_r", kv)),
                "padding_t" => trees.append(&mut px_or_pct_trees("padding_t", "padding_t", kv)),
                "padding_b" => trees.append(&mut px_or_pct_trees("padding_b", "padding_b", kv)),
                "padding_l" => trees.append(&mut px_or_pct_trees("padding_l", "padding_l", kv)),
                "padding_r" => trees.append(&mut px_or_pct_trees("padding_r", "padding_r", kv)),
                "overflow_hori" => todo!(),
                "overflow_vert" => todo!(),
                "scroll_vert" => trees.append(&mut px_or_pct_trees("scroll_vert", "scroll_vert", kv)),
                "scroll_hori" => trees.append(&mut px_or_pct_trees("scroll_hori", "scroll_hori", kv)),
                "text" => todo!(),
                "text_secret" => todo!(),
                "text_color" => todo!(),
                "text_height" => trees.append(&mut px_or_pct_trees("text_height", "text_height", kv)),
                "text_wrap" => todo!(),
                "text_vert_align" => todo!(),
                "text_hori_align" => todo!(),
                "line_spacing" => trees.append(&mut px_or_pct_trees("line_spacing", "line_spacing", kv)),
                "line_limit" => todo!(),
                "border_size_t" => todo!(),
                "border_size_b" => todo!(),
                "border_size_l" => todo!(),
                "border_size_r" => todo!(),
                "border_color_t" => todo!(),
                "border_color_b" => todo!(),
                "border_color_l" => todo!(),
                "border_color_r" => todo!(),
                "border_radius_tl" => todo!(),
                "border_radius_tr" => todo!(),
                "border_radius_bl" => todo!(),
                "border_radius_br" => todo!(),
                _ => panic!("Unknown key '{}' at line {}", kv.key, kv.line)
            }
        }
    }

    trees.push(Punct::new('.', Spacing::Joint).into());
    trees.push(Punct::new('.', Spacing::Alone).into());
    trees.push(Ident::new("Style", Span::call_site()).into());
    trees.push(Punct::new(':', Spacing::Joint).into());
    trees.push(Punct::new(':', Spacing::Alone).into());
    trees.push(Ident::new("default", Span::call_site()).into());
    trees.push(Group::new(Delimiter::Parenthesis, proc_macro2::TokenStream::new()).into());

    let out_trees: Vec<TokenTree> = vec![
        Ident::new("Style", Span::call_site()).into(),
        Group::new(Delimiter::Brace, proc_macro2::TokenStream::from_iter(trees.into_iter())).into()
    ];

    proc_macro::TokenStream::from(proc_macro2::TokenStream::from_iter(out_trees.into_iter()))
}
