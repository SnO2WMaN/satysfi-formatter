mod comment;
#[cfg(test)]
mod tests;
mod visualize;

use comment::*;
use satysfi_parser::{grammar, Cst, CstText};
pub use visualize::*;

type ReservedText = &'static str;

#[allow(dead_code)]
struct ReservedWord {
    let_block: ReservedText,
    let_math: ReservedText,
    let_mutable: ReservedText,
    type_stmt: ReservedText,
    let_inline: ReservedText,
    constraint: ReservedText,
    inline_command: ReservedText,
    block_command: ReservedText,
    math_command: ReservedText,
    let_rec: ReservedText,
    controls: ReservedText,
    command: ReservedText,
    before: ReservedText,
    module: ReservedText,
    direct: ReservedText,
    struct_stmt: ReservedText,
    cycle: ReservedText,
    match_stmt: ReservedText,
    while_stmt: ReservedText,
    if_stmt: ReservedText,
    else_stmt: ReservedText,
    true_stmt: ReservedText,
    false_stmt: ReservedText,
    open: ReservedText,
    then: ReservedText,
    when: ReservedText,
    with: ReservedText,
    and: ReservedText,
    end: ReservedText,
    fun: ReservedText,
    let_stmt: ReservedText,
    mod_stmt: ReservedText,
    not: ReservedText,
    sig: ReservedText,
    val: ReservedText,
    as_stmt: ReservedText,
    do_stmt: ReservedText,
    in_stmt: ReservedText,
    of: ReservedText,
}

const RESERVED_WORD: ReservedWord = ReservedWord {
    constraint: "constraint",
    inline_command: "inline-cmd",
    block_command: "block-cmd",
    math_command: "math-cmd",
    let_mutable: "let-mutable",
    let_inline: "let-inline",
    let_block: "let-block",
    let_math: "let-math",
    type_stmt: "type",
    let_rec: "let-rec",
    controls: "controls",
    command: "command",
    before: "before",
    module: "module",
    direct: "direct",
    struct_stmt: "struct",
    cycle: "cycle",
    match_stmt: "match",
    while_stmt: "while",
    if_stmt: "if",
    else_stmt: "else",
    true_stmt: "true",
    false_stmt: "false",
    open: "open",
    then: "then",
    when: "when",
    with: "with",
    and: "and",
    end: "end",
    fun: "fun",
    let_stmt: "let",
    mod_stmt: "mod",
    not: "not",
    sig: "sig",
    val: "val",
    as_stmt: "as",
    do_stmt: "do",
    in_stmt: "in",
    of: "of",
};

pub struct OptionData {
    row_length: usize,
    indent_space: usize,
}

#[allow(non_upper_case_globals)]
// format 設定のオプション
static default_option: OptionData = OptionData {
    row_length: 80,
    indent_space: 4,
};

/// satysfi の文字列を渡すと format したものを返す
/// * `input` - satysfi のコード  
/// * `output` - format された文字列
pub fn format(input: &str) -> String {
    /*
    CstText {
        text: string,
        lines: Vec<usize>, // start
        cst: Cst,
    }
    */
    let csttext = CstText::parse(input, grammar::program).expect("parse error");
    let csttext = csttext_insert_comments(csttext);

    #[cfg(debug_assertions)]
    visualize_csttext_tree(&csttext);
    // #[cfg(debug_assertions)]
    // dbg!(&csttext);

    let depth = 0;
    let mut output = to_string_cst_inner(input, &csttext.cst, depth);

    // 末尾に改行がない場合、改行を挿入して終了
    if !output.ends_with("\n") {
        output += "\n";
    }

    output
}

/// cst の inner の要素を結合して文字列に変換する関数
fn to_string_cst_inner(text: &str, cst: &Cst, depth: usize) -> String {
    /*
    Cst {
        rule: Rule,
        span: Span { start: number, end: number },
        inner: [Cst]
    }
    */
    use satysfi_parser::Rule;
    let csts = cst.inner.clone();
    // 関数ないで改行するときはこれを使用する
    let indent = indent_space(depth);
    let newline = format!("\n{indent}");
    let sep = &match cst.rule {
        Rule::block_cmd | Rule::inline_cmd => " ".to_string(),
        Rule::type_prod | Rule::type_application => " ".to_string(),
        Rule::dyadic_expr | Rule::match_expr | Rule::unary_operator_expr => " ".to_string(),
        Rule::vertical | Rule::horizontal_bullet_list => newline.clone(),
        Rule::record | Rule::type_record | Rule::list => format!(";{newline}"),
        Rule::type_block_cmd | Rule::type_inline_cmd | Rule::type_math_cmd => format!(";{newline}"),
        Rule::horizontal_single => "".to_string(),
        _ => "".to_string(),
    };

    let output = match cst.rule {
        Rule::let_block_stmt_ctx
        | Rule::let_block_stmt_noctx
        | Rule::let_inline_stmt_ctx
        | Rule::let_inline_stmt_noctx
        | Rule::let_stmt
        | Rule::let_rec_stmt
        | Rule::sig_val_stmt
        | Rule::sig_direct_stmt
        | Rule::type_stmt
        | Rule::let_math_stmt => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            match now_cst.rule {
                Rule::var => current + " " + &s,
                Rule::block_cmd_name => current + " " + &s,
                Rule::type_expr => current + ": " + &s,
                Rule::expr => {
                    // ブロック定義は例外
                    if !s.starts_with("'<") && s.contains("\n") {
                        // 1つインデントを深くする
                        let s = to_string_cst(text, now_cst, depth + 1);
                        current + " =" + &newline + &indent_space(1) + s.trim_start()
                    } else {
                        current + " = " + &s
                    }
                }
                Rule::comments => current + &s,
                _ => current + " " + &s,
            }
        }),
        Rule::unary => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            match now_cst.rule {
                Rule::unary_operator | Rule::expr => current + &format!("({s})", s = s),
                _ => current + &s,
            }
        }),
        Rule::lambda => csts
            .iter()
            .fold(RESERVED_WORD.fun.to_string(), |current, now_cst| {
                let s = to_string_cst(text, now_cst, depth);
                if current.is_empty() {
                    s
                } else {
                    match now_cst.rule {
                        Rule::pattern => current + " " + &s,
                        Rule::comments => current + &s,
                        _ => current + " -> " + &s,
                    }
                }
            }),
        Rule::record_unit => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            if current.is_empty() {
                s
            } else {
                match now_cst.rule {
                    Rule::var_ptn => current + " " + &s,
                    Rule::expr => current + " = " + &s,
                    Rule::comments => current + &s,
                    _ => unreachable!(),
                }
            }
        }),
        Rule::type_record_unit => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            if current.is_empty() {
                s
            } else {
                match now_cst.rule {
                    Rule::var => current + " " + &s,
                    Rule::type_expr => current + ": " + &s,
                    Rule::comments => current + &s,
                    _ => unreachable!(),
                }
            }
        }),
        Rule::type_application => {
            let mut output = String::new();
            for cst in &csts {
                let s = to_string_cst(text, cst, depth);
                if !output.is_empty() {
                    output += &sep;
                }
                if cst.rule == Rule::type_expr {
                    output += &format!("({s})");
                } else {
                    output += &s;
                }
            }
            // println!("match type_application {output}");
            output
        }
        Rule::application => {
            if csts.is_empty() {
                "".to_string()
            } else {
                let first_text = to_string_cst(text, &csts[0], depth);
                let insert_space = first_text != "document";
                let mut output = first_text;
                for cst in csts.iter().skip(1) {
                    let s = to_string_cst(text, cst, depth);
                    if insert_space {
                        output += " ";
                    }
                    output += &s;
                }

                output
            }
        }
        Rule::bind_stmt => {
            // let* ~ in のとき用
            let output = csts.iter().fold(String::new(), |current, now_cst| {
                let s = to_string_cst(text, now_cst, depth);
                if current.is_empty() {
                    s
                } else {
                    match now_cst.rule {
                        Rule::expr => {
                            if s.contains("\n") {
                                current.trim_end().to_string() + " in" + &newline + s.trim_start()
                            } else {
                                current + " in " + &s
                            }
                        }
                        Rule::comments => current + &s,
                        _ => current + &s,
                    }
                }
            });
            output
        }
        Rule::type_expr => csts.iter().fold(String::new(), |current, now_cst| {
            // 型定義
            let s = to_string_cst(text, now_cst, depth);
            if current.is_empty() {
                s
            } else {
                match now_cst.rule {
                    Rule::type_optional => current + " ?-> " + &s,
                    Rule::type_prod => current + " -> " + &s,
                    Rule::comments => current + &s,
                    _ => current + " " + &s,
                }
            }
        }),
        Rule::module_stmt => {
            let output = csts.iter().fold(String::new(), |current, now_cst| {
                let s = to_string_cst(text, now_cst, depth);
                match now_cst.rule {
                    Rule::module_name => current + " " + &s,
                    Rule::sig_stmt => current + ": " + &s,
                    Rule::struct_stmt => current + " = " + &s,
                    Rule::comments => (current + &s),
                    _ => current + &s,
                }
            });
            RESERVED_WORD.module.to_string() + " " + output.trim()
        }
        Rule::struct_stmt => {
            let output = csts.iter().fold(String::new(), |current, now_cst| {
                let s = to_string_cst(text, now_cst, depth);
                match now_cst.rule {
                    Rule::comments => current + &s,
                    _ => {
                        if current.is_empty() || s.ends_with(RESERVED_WORD.in_stmt) {
                            current + &s
                        } else {
                            // 基本的に改行する
                            current + "\n" + &s
                        }
                    }
                }
            });
            output.trim_end().to_string()
        }
        Rule::sig_stmt => {
            let output = csts.iter().fold(String::new(), |current, now_cst| {
                let s = to_string_cst(text, now_cst, depth);
                match now_cst.rule {
                    Rule::module_name => current + " " + &s,
                    Rule::struct_stmt => current + "= " + RESERVED_WORD.struct_stmt + &s,
                    Rule::comments => current + &s,
                    _ => current + &s,
                }
            });
            output
        }
        Rule::horizontal_single => {
            let output =
                csts.iter()
                    .enumerate()
                    .fold(String::new(), |current, (index, now_cst)| {
                        let s = to_string_cst(text, &now_cst, depth);
                        let start_newline =
                            text[now_cst.span.start..now_cst.span.end].starts_with("\n");
                        if current.is_empty() {
                            s
                        } else if index > 0
                            && (csts[index - 1].rule == Rule::regular_text
                                || csts[index - 1].rule == Rule::inline_cmd)
                            && now_cst.rule == Rule::inline_cmd
                        {
                            // regular_text と inline_cmd の間にスペースがある場合、フォーマットの例外
                            let last_span = csts[index - 1].span;
                            let end_space =
                                text[last_span.start..last_span.end].ends_with(char::is_whitespace);
                            // 改行している
                            let end_newline = text[last_span.start..last_span.end]
                                .lines()
                                .last()
                                .unwrap_or("")
                                .trim()
                                .is_empty();

                            if end_space {
                                if end_newline || s.len() > default_option.row_length {
                                    format!("{}{newline}{s}", current.trim_end())
                                } else {
                                    current + " " + &s
                                }
                            } else {
                                current + &s
                            }
                        } else if index + 1 < csts.len()
                            && (now_cst.rule == Rule::regular_text
                                || now_cst.rule == Rule::inline_cmd)
                            && (csts[index + 1].rule != Rule::regular_text
                                && csts[index + 1].rule != Rule::inline_cmd)
                            && s.trim().is_empty()
                        {
                            // 空文字の処理例外
                            // ex: `const-str` `const-str` のような場合
                            // 改行が含まれる場合改行
                            let include_kaigyo =
                                text[now_cst.span.start..now_cst.span.end].contains("\n");
                            if include_kaigyo {
                                format!("{}{newline}{s}", current.trim_end())
                            } else {
                                // 改行がない場合スペースを1つ
                                format!("{} {s}", current)
                            }
                        } else if s.trim().is_empty() {
                            // 空行の処理
                            // 文字がない場合は何もしない
                            current
                        } else if now_cst.rule == Rule::comments {
                            format!("{}{newline}{s}", current.trim_end())
                        } else if start_newline {
                            format!("{}{newline}{s}", current.trim_end())
                        } else {
                            current + &s
                        }
                    });

            // コメントが末尾にあるとき余計な改行が残ってしまうので削除
            output.trim().to_string()
        }
        Rule::preamble => csts.iter().fold(String::new(), |current, now_cst| {
            // 例外処理
            let s = to_string_cst(text, &now_cst, depth).trim().to_string();
            if current.is_empty() {
                s
            } else if s.is_empty() {
                current
            } else {
                current + "\n" + &s
            }
        }),
        _ => {
            let output = csts
                .iter()
                .fold((String::new(), false), |current, now_cst| {
                    let s = to_string_cst(text, &now_cst, depth);
                    let flag = now_cst.rule == Rule::comments;
                    let output = if current.1 {
                        (current.0.clone() + &s, flag)
                    } else if current.0.is_empty() {
                        (s, flag)
                    } else if s.is_empty() {
                        current
                    } else {
                        (current.0 + sep + &s, flag)
                    };

                    if cst.rule == Rule::program_saty && now_cst.rule == Rule::preamble {
                        // program saty だった場合、in を入れる
                        (output.0 + "\n" + RESERVED_WORD.in_stmt + "\n\n", output.1)
                    } else {
                        output
                    }
                })
                .0;

            output
        }
    };

    output
}

/// cst を文字列にするための関数
fn to_string_cst(text: &str, cst: &Cst, depth: usize) -> String {
    // インデントを制御するための変数
    let new_depth = match cst.rule {
        Rule::block_text | Rule::cmd_text_arg | Rule::record | Rule::type_record | Rule::list => {
            depth + 1
        }
        Rule::type_block_cmd | Rule::type_inline_cmd | Rule::math_cmd => depth + 1,
        Rule::sig_stmt | Rule::struct_stmt => depth + 1,
        _ => depth,
    };
    let start_indent = "\n".to_string() + &indent_space(new_depth);
    let end_indent = "\n".to_string() + &indent_space(depth);

    let output = to_string_cst_inner(text, cst, new_depth);
    let self_text = text.get(cst.span.start..cst.span.end).unwrap().to_string();

    use satysfi_parser::Rule;
    // 中身をそのまま返すものは output をそのまま返す
    match cst.rule {
        Rule::comments => to_comment_string(self_text) + &end_indent,
        // header
        Rule::stage => "@stage: ".to_string() + self_text.trim() + "\n\n",
        Rule::headers => {
            if output.len() > 0 {
                output + "\n"
            } else {
                output
            }
        }
        Rule::header_require => "@require: ".to_string() + &output + "\n",
        Rule::header_import => "@import: ".to_string() + &output + "\n",
        Rule::pkgname => self_text.trim().to_string(), // 末尾のスペースなどは削除 (スペースで終わるpkgnameが導入されるとバグるけれど無いでしょう……)

        // statement
        Rule::let_stmt => format!("{start_indent}{}{}", RESERVED_WORD.let_stmt, output),
        Rule::let_rec_stmt => format!("{start_indent}{}{}", RESERVED_WORD.let_rec, output),
        Rule::let_rec_inner => output,
        Rule::let_rec_matcharm => output,
        Rule::let_inline_stmt_ctx => {
            format!("{start_indent}{}{}", RESERVED_WORD.let_inline, output)
        }
        Rule::let_inline_stmt_noctx => {
            format!("{start_indent}{}{}", RESERVED_WORD.let_inline, output)
        }
        Rule::let_block_stmt_ctx => format!("{start_indent}{}{}", RESERVED_WORD.let_block, output),

        Rule::let_block_stmt_noctx => {
            format!("{start_indent}{}{}", RESERVED_WORD.let_block, output)
        }
        Rule::let_math_stmt => format!("{start_indent}{}{}", RESERVED_WORD.let_math, output),
        Rule::let_mutable_stmt => format!("{start_indent}{}{}", RESERVED_WORD.let_mutable, output),
        Rule::type_stmt => format!("{start_indent}{}{}", RESERVED_WORD.type_stmt, output),
        Rule::type_inner => output,
        Rule::type_variant => output,
        Rule::module_stmt => output,
        Rule::open_stmt => output,
        Rule::arg => output,

        // struct
        Rule::sig_stmt => format!(
            "{}{}{end_indent}{}",
            RESERVED_WORD.sig, output, RESERVED_WORD.end
        ), // TODO
        Rule::struct_stmt => format!(
            "{}{}{end_indent}{}",
            RESERVED_WORD.struct_stmt, output, RESERVED_WORD.end
        ), // TODO
        Rule::sig_type_stmt => format!("{start_indent}{}{}", RESERVED_WORD.type_stmt, output),
        Rule::sig_val_stmt => format!("{start_indent}{}{}", RESERVED_WORD.val, output),
        Rule::sig_direct_stmt => format!("{start_indent}{}{}", RESERVED_WORD.direct, output),

        // types
        Rule::type_expr => output,
        Rule::type_optional => output,
        Rule::type_prod => output,
        Rule::type_inline_cmd => {
            if output.contains("\n") {
                format!(
                    "[{start_indent}{output};{end_indent}] {}",
                    RESERVED_WORD.inline_command
                )
            } else {
                format!("[{output}] {}", RESERVED_WORD.inline_command)
            }
        }
        Rule::type_block_cmd => {
            if output.contains("\n") {
                format!(
                    "[{start_indent}{output};{end_indent}] {}",
                    RESERVED_WORD.block_command
                )
            } else {
                format!("[{output}] {}", RESERVED_WORD.block_command)
            }
        }
        Rule::type_math_cmd => {
            if output.contains("\n") {
                format!(
                    "[{start_indent}{output};{end_indent}] {}",
                    RESERVED_WORD.math_command
                )
            } else {
                format!("[{output}] {}", RESERVED_WORD.math_command)
            }
        }
        Rule::type_list_unit_optional => output,
        Rule::type_application => output,
        Rule::type_name => self_text,
        // Rule::type_record => output,
        Rule::type_record_unit => output,
        Rule::type_param => output,
        Rule::constraint => {
            format!("{}{}", RESERVED_WORD.constraint, output)
        }

        // unary
        Rule::unary => output,
        Rule::unary_prefix => output,
        Rule::block_text => {
            if self_text.starts_with("'") {
                if output.len() > 0 {
                    format!("'<{start_indent}{output}{end_indent}>")
                } else {
                    format!("'<{output}>")
                }
            } else {
                if output.len() > 0 {
                    format!("<{start_indent}{output}{end_indent}>")
                } else {
                    format!("<{output}>")
                }
            }
        }
        Rule::horizontal_text => self_text,
        Rule::math_text => self_text,
        Rule::list => {
            let output = if output.len() > 0 {
                // TODO かなり無理な方法で末尾のセミコロンをつけているので、どうにかする
                // match self_text.split(";").last() {
                //     Some(text) if text.trim() == "];" || text.trim() == "]" => format!("[{start_indent}{output};{end_indent}]"),
                //     _ => format!("[{start_indent}{output}{end_indent}]"),
                // }
                if output.ends_with("}") {
                    format!("[{start_indent}{output}{end_indent}]")
                } else {
                    format!("[{start_indent}{output};{end_indent}]")
                }
            } else {
                format!("[]")
            };
            if self_text.ends_with(";") {
                output + ";"
            } else {
                output
            }
        }
        Rule::record | Rule::type_record => {
            // TODO: consider
            if cst.inner.len() > 1 {
                // 2 つ以上のときは改行
                format!("(|{start_indent}{output};{end_indent}|)")
            } else {
                // 1つだけの時は、改行しない
                format!("(|{output}|)")
            }
        }
        Rule::record_unit => output,
        Rule::tuple => self_text,
        Rule::bin_operator => self_text,
        Rule::expr_with_mod => self_text,
        Rule::var => self_text,
        Rule::var_ptn => self_text,
        Rule::modvar => self_text,
        Rule::mod_cmd_name => self_text,
        Rule::module_name => self_text,
        Rule::variant_name => self_text,

        // command
        Rule::cmd_name_ptn => self_text,
        Rule::cmd_expr_arg => {
            if self_text.starts_with("(") {
                format!("({output})",)
            } else {
                output
            }
        }
        Rule::cmd_expr_option => self_text,
        Rule::cmd_text_arg => {
            // 括弧の種類を取得
            let start_arg = self_text.chars().nth(0).unwrap();
            let end_arg = self_text.chars().nth_back(0).unwrap();
            // コメントで開始 or 改行を含んでいたら、改行を入れる
            let include_comment = output.starts_with("%");
            let include_kaigyou = output.find("\n") != None || start_arg == '<' || include_comment;
            match output.trim().len() {
                0 => format!("{start_arg}{end_arg}"),
                num if include_kaigyou || num > default_option.row_length => {
                    format!("{start_arg}{start_indent}{output}{end_indent}{end_arg}")
                }
                _ => format!("{start_arg} {output} {end_arg}"),
            }
        }
        Rule::inline_cmd => {
            if self_text.ends_with(";") {
                output + ";"
            } else {
                output
            }
        }
        Rule::inline_cmd_name => self_text,
        Rule::block_cmd => {
            if self_text.ends_with(";") {
                output + ";"
            } else {
                output
            }
        }

        Rule::block_cmd_name => self_text,
        Rule::math_cmd => self_text,
        Rule::math_cmd_name => self_text,
        Rule::math_cmd_expr_arg => self_text,
        Rule::math_cmd_expr_option => self_text,

        // pattern
        Rule::pat_as => output,
        Rule::pat_cons => output,
        Rule::pattern => self_text, // TODO どのパターンでも中身をそのまま出力
        Rule::pat_variant => output,
        Rule::pat_list => output,
        Rule::pat_tuple => output,

        // expr
        Rule::expr => {
            if self_text.ends_with(";") {
                output + ";"
            } else {
                output
            }
        }
        Rule::match_expr => self_text,       // TODO
        Rule::match_arm => output,           // TODO
        Rule::match_guard => output,         // TODO
        Rule::bind_stmt => output,           // TODO
        Rule::ctrl_while => output,          // TODO
        Rule::ctrl_if => output,             // TODO
        Rule::lambda => output,              // TODO
        Rule::assignment => output,          // TODO
        Rule::dyadic_expr => output,         // TODO
        Rule::unary_operator_expr => output, // TODO
        Rule::unary_operator => output,      // TODO
        // application
        Rule::application => output,
        Rule::application_args_normal => output,
        Rule::application_args_optional => output,
        Rule::command_application => output,
        Rule::variant_constructor => output,

        // horizontal
        Rule::horizontal_single => output,
        Rule::horizontal_list => output,                  // TODO
        Rule::horizontal_bullet_list => output,           // TODO
        Rule::horizontal_bullet => output,                // TODO
        Rule::horizontal_bullet_star => "* ".to_string(), // TODO
        Rule::regular_text => {
            let sep = format!("\n{}", indent_space(depth));
            let output = self_text
                .split("\n")
                .into_iter()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<String>>()
                .join(&sep);
            output
        }
        Rule::horizontal_escaped_char => self_text, // TODO
        Rule::inline_text_embedding => output,      // TODO

        // vertical
        Rule::vertical => output,             // TODO
        Rule::block_text_embedding => output, // TODO

        // constants
        Rule::const_unit => self_text,
        Rule::const_bool => self_text,
        Rule::const_int => self_text,
        Rule::const_float => self_text,
        Rule::const_length => self_text,
        Rule::const_string => self_text,

        // TODO other things
        Rule::misc => " ".to_string(),
        Rule::program_saty => output.trim_start().to_string(),
        Rule::program_satyh => output.trim_start().to_string(),
        Rule::preamble => output.trim_start().to_string(),
        // TODO
        // _ => self_text,
        _ => "".to_string(),
    }
}

#[inline]
fn indent_space(depth: usize) -> String {
    let mut output = String::new();
    for _ in 0..default_option.indent_space * depth {
        output += " "
    }
    output
}
