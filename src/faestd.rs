use std::{collections::HashMap, fs, process::exit};

use crate::{lexer::Span, runtime::*};

// I/O
pub fn println(scope: &mut Scope, at: Span) {
    let Some(Value::String(s)) = scope.stack.pop() else {
        unreachable!("params assertion");
    };
    println!("{s}");
}

pub fn dump(scope: &mut Scope, at: Span) {
    println!("STACK:");
    for v in scope.stack.values.iter().rev() {
        println!("{:?}", v);
    }
    println!("---");
}

// STACK
pub fn dup(scope: &mut Scope, at: Span) {
    let Some(v) = scope.stack.peek() else {
        unreachable!("params will fail");
    };
    scope.stack.push(v.clone());
}

pub fn pop(scope: &mut Scope, at: Span) {
    scope.stack.pop();
}

pub fn rot2(scope: &mut Scope, at: Span) {
    let Some(top) = scope.stack.pop() else {
        unreachable!("params will fail");
    };
    let Some(bot) = scope.stack.pop() else {
        unreachable!("params will fail");
    };
    scope.stack.push(top);
    scope.stack.push(bot);
}

pub fn rot3(scope: &mut Scope, at: Span) {
    let Some(top) = scope.stack.pop() else {
        unreachable!("params will fail");
    };
    let Some(mid) = scope.stack.pop() else {
        unreachable!("params will fail");
    };
    let Some(bot) = scope.stack.pop() else {
        unreachable!("params will fail");
    };
    scope.stack.push(mid);
    scope.stack.push(top);
    scope.stack.push(bot);
}

pub fn rot3_rev(scope: &mut Scope, at: Span) {
    let Some(top) = scope.stack.pop() else {
        unreachable!("params will fail");
    };
    let Some(mid) = scope.stack.pop() else {
        unreachable!("params will fail");
    };
    let Some(bot) = scope.stack.pop() else {
        unreachable!("params will fail");
    };
    scope.stack.push(top);
    scope.stack.push(bot);
    scope.stack.push(mid);
}

pub fn flip(scope: &mut Scope, at: Span) {
    scope.stack.values.reverse();
}

pub fn depth(scope: &mut Scope, at: Span) {
    let depth = scope.stack.depth();
    scope.stack.push(Value::Int(depth as i32));
}

pub fn expand(scope: &mut Scope, at: Span) {
    let Some(Value::Stack(Stack { mut values })) = scope.stack.pop() else {
        unreachable!("params will fail")
    };
    scope.stack.values.append(&mut values);
}

pub fn shove(scope: &mut Scope, at: Span) {
    let Some(v) = scope.stack.pop() else {
        unreachable!("params will fail")
    };
    scope.stack.values.insert(0, v);
}

pub fn push(scope: &mut Scope, at: Span) {
    let Some(v) = scope.stack.pop() else {
        unreachable!("params")
    };
    let Some(Value::Stack(mut s)) = scope.stack.pop() else {
        unreachable!("params")
    };
    s.push(v);
    scope.stack.push(Value::Stack(s));
}

// ARITHMETIC
fn get_integral2(scope: &mut Scope, at: Span) -> (f32, f32) {
    let Some(Some(right)) = scope.stack.pop().map(|c| c.as_integral()) else {
        unreachable!("params will fail");
    };
    let Some(Some(left)) = scope.stack.pop().map(|c| c.as_integral()) else {
        unreachable!("params will fail");
    };

    (left, right)
}

pub fn add(scope: &mut Scope, at: Span) {
    let (left, right) = get_integral2(scope, at);
    scope.stack.push(Value::Number(left + right));
}

pub fn sub(scope: &mut Scope, at: Span) {
    let (left, right) = get_integral2(scope, at);
    scope.stack.push(Value::Number(left - right));
}

pub fn mul(scope: &mut Scope, at: Span) {
    let (left, right) = get_integral2(scope, at);
    scope.stack.push(Value::Number(left * right));
}

pub fn div(scope: &mut Scope, at: Span) {
    let (left, right) = get_integral2(scope, at.clone());
    if right == 0.0 {
        scope.errs.print_error(&at, format!("Division by zero"));
        exit(1);
    }
    scope.stack.push(Value::Number(left / right));
}

pub fn exp(scope: &mut Scope, at: Span) {
    let (left, right) = get_integral2(scope, at.clone());
    scope.stack.push(Value::Number(left.powf(right)));
}

pub fn not(scope: &mut Scope, at: Span) {
    let Some(Value::Bool(b)) = scope.stack.pop() else {
        unreachable!("params");
    };
    scope.stack.push(Value::Bool(!b));
}

// COMPARISON
pub fn greater(scope: &mut Scope, at: Span) {
    let (left, right) = get_integral2(scope, at);
    scope.stack.push(Value::Bool(left > right));
}

pub fn greatereq(scope: &mut Scope, at: Span) {
    let (left, right) = get_integral2(scope, at);
    scope.stack.push(Value::Bool(left >= right));
}

pub fn less(scope: &mut Scope, at: Span) {
    let (left, right) = get_integral2(scope, at);
    scope.stack.push(Value::Bool(left < right));
}

pub fn lesseq(scope: &mut Scope, at: Span) {
    let (left, right) = get_integral2(scope, at);
    scope.stack.push(Value::Bool(left <= right));
}

pub fn equ(scope: &mut Scope, at: Span) {
    let Some(right) = scope.stack.pop() else {
        unreachable!("params will fail");
    };
    let Some(left) = scope.stack.pop() else {
        unreachable!("params will fail");
    };

    scope.stack.push(Value::Bool(match (&left, &right) {
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Char(a), Value::Char(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        _ => {
            scope.errs.print_error(
                &at,
                format!(
                    "Equality between {} and {} not allowed",
                    left.type_name().into_str(),
                    right.type_name().into_str()
                ),
            );
            exit(1);
        }
    }));
}

pub fn nequ(scope: &mut Scope, at: Span) {
    let Some(right) = scope.stack.pop() else {
        unreachable!("params will fail");
    };
    let Some(left) = scope.stack.pop() else {
        unreachable!("params will fail");
    };

    scope.stack.push(Value::Bool(match (&left, &right) {
        (Value::String(a), Value::String(b)) => a != b,
        (Value::Char(a), Value::Char(b)) => a != b,
        (Value::Int(a), Value::Int(b)) => a != b,
        (Value::Number(a), Value::Number(b)) => a != b,
        (Value::Bool(a), Value::Bool(b)) => a != b,
        _ => {
            scope.errs.print_error(
                &at,
                format!(
                    "Equality between {} and {} not allowed",
                    left.type_name().into_str(),
                    right.type_name().into_str()
                ),
            );
            exit(1);
        }
    }));
}

// CONVERSION
pub fn stringify(scope: &mut Scope, at: Span) {
    let Some(top) = scope.stack.pop().map(|c| c.as_string()) else {
        scope
            .errs
            .print_error(&at, "Failed to cast to string".to_string());
        exit(1);
    };
    scope.stack.push(Value::String(top));
}

pub fn intify(scope: &mut Scope, at: Span) {
    let Some(Some(top)) = scope.stack.pop().map(|c| c.as_int()) else {
        scope
            .errs
            .print_error(&at, "Failed to cast to int".to_string());
        exit(1);
    };
    scope.stack.push(Value::Int(top));
}

// STRING
pub fn chars_expand(scope: &mut Scope, at: Span) {
    let Some(Value::String(s)) = scope.stack.pop() else {
        unreachable!("params");
    };
    let chars = s.chars().map(|c| Value::Char(c)).collect::<Vec<_>>();
    scope.stack.push(Value::Stack(Stack { values: chars }));
}

pub fn chars_group(scope: &mut Scope, at: Span) {
    let Some(Value::Stack(Stack { values })) = scope.stack.pop() else {
        unreachable!("params")
    };
    let mut chars = String::new();
    for val in values {
        match val {
            Value::Char(c) => chars.push(c),
            _ => {
                scope
                    .errs
                    .print_error(&at, format!("Non-character found in stack"));
                exit(1);
            }
        }
    }
    scope.stack.push(Value::String(chars));
}

pub fn concat_str(scope: &mut Scope, at: Span) {
    let Some(Value::String(rhs)) = scope.stack.pop() else {
        unreachable!("params");
    };

    let Some(Value::String(lhs)) = scope.stack.pop() else {
        unreachable!("params");
    };

    scope.stack.push(Value::String(lhs + &rhs));
}

// OS
pub fn fae_exit(scope: &mut Scope, at: Span) {
    exit(0);
}

// FILES
pub fn fs_write(scope: &mut Scope, at: Span) {
    let Some(Value::String(contents)) = scope.stack.pop() else {
        unreachable!("params");
    };
    let Some(Value::String(filepath)) = scope.stack.pop() else {
        unreachable!("params");
    };
    match fs::write(filepath, contents) {
        Ok(_) => scope.stack.push(Value::Bool(true)),
        Err(_) => scope.stack.push(Value::Bool(false)),
    }
}

pub fn fs_read(scope: &mut Scope, at: Span) {
    let Some(Value::String(filepath)) = scope.stack.pop() else {
        unreachable!("params");
    };
    match fs::read_to_string(filepath) {
        Ok(contents) => {
            scope.stack.push(Value::String(contents));
            scope.stack.push(Value::Bool(true));
        }
        Err(_) => scope.stack.push(Value::Bool(false)),
    }
}

pub fn std_get_fns() -> HashMap<String, Function> {
    HashMap::from(
        [
            ("println", vec![ValueType::String], println as InternalFn),
            ("dump", vec![], dump as InternalFn),
            ("dup", vec![ValueType::Any], dup as InternalFn),
            ("pop", vec![ValueType::Any], pop as InternalFn),
            (
                "swap",
                vec![ValueType::Any, ValueType::Any],
                rot2 as InternalFn,
            ),
            (
                "rot",
                vec![ValueType::Any, ValueType::Any, ValueType::Any],
                rot3 as InternalFn,
            ),
            (
                "rot-",
                vec![ValueType::Any, ValueType::Any, ValueType::Any],
                rot3_rev as InternalFn,
            ),
            ("flip", vec![], flip as InternalFn),
            ("depth", vec![], depth as InternalFn),
            ("@", vec![ValueType::Stack], expand as InternalFn),
            ("shove", vec![ValueType::Any], shove as InternalFn),
            (
                "push",
                vec![ValueType::Stack, ValueType::Any],
                push as InternalFn,
            ),
            (
                "+",
                vec![ValueType::Integral, ValueType::Integral],
                add as InternalFn,
            ),
            (
                "-",
                vec![ValueType::Integral, ValueType::Integral],
                sub as InternalFn,
            ),
            (
                "*",
                vec![ValueType::Integral, ValueType::Integral],
                mul as InternalFn,
            ),
            (
                "/",
                vec![ValueType::Integral, ValueType::Integral],
                div as InternalFn,
            ),
            (
                "**",
                vec![ValueType::Integral, ValueType::Integral],
                exp as InternalFn,
            ),
            ("not", vec![ValueType::Bool], not as InternalFn),
            ("str", vec![ValueType::Any], stringify as InternalFn),
            ("int", vec![ValueType::Any], intify as InternalFn),
            (
                "chars@",
                vec![ValueType::String],
                chars_expand as InternalFn,
            ),
            ("chars&", vec![ValueType::Stack], chars_group as InternalFn),
            (
                "str&",
                vec![ValueType::String, ValueType::String],
                concat_str as InternalFn,
            ),
            (
                ">",
                vec![ValueType::Integral, ValueType::Integral],
                greater as InternalFn,
            ),
            (
                ">=",
                vec![ValueType::Integral, ValueType::Integral],
                greatereq as InternalFn,
            ),
            (
                "<",
                vec![ValueType::Integral, ValueType::Integral],
                less as InternalFn,
            ),
            (
                "<=",
                vec![ValueType::Integral, ValueType::Integral],
                lesseq as InternalFn,
            ),
            ("=", vec![ValueType::Any, ValueType::Any], equ as InternalFn),
            (
                "/=",
                vec![ValueType::Any, ValueType::Any],
                nequ as InternalFn,
            ),
            ("exit", vec![], fae_exit as InternalFn),
            (
                "fs/write",
                vec![ValueType::String, ValueType::String],
                fs_write as InternalFn,
            ),
            ("fs/read", vec![ValueType::String], fs_read as InternalFn),
        ]
        .map(|(s, p, f)| {
            (
                s.to_string(),
                Function::Internal {
                    params: p.to_vec(),
                    body: f,
                },
            )
        }),
    )
}
