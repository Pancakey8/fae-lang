use std::{collections::HashMap, process::exit};

use crate::{lexer::Span, runtime::*};

// I/O
pub fn println(scope: &mut Scope, at: Span) {
    scope.ensure_params(&["string"], at);
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
        scope.ensure_params(&["any"], at);
        unreachable!("params will fail");
    };
    scope.stack.push(v.clone());
}

pub fn pop(scope: &mut Scope, at: Span) {
    if scope.stack.pop().is_none() {
        scope.ensure_params(&["any"], at);
    }
}

pub fn rot2(scope: &mut Scope, at: Span) {
    let Some(top) = scope.stack.pop() else {
        scope.ensure_params(&["any", "any"], at);
        unreachable!("params will fail");
    };
    let Some(bot) = scope.stack.pop() else {
        scope.ensure_params(&["any", "any"], at);
        unreachable!("params will fail");
    };
    scope.stack.push(top);
    scope.stack.push(bot);
}

pub fn rot3(scope: &mut Scope, at: Span) {
    let Some(top) = scope.stack.pop() else {
        scope.ensure_params(&["any", "any", "any"], at);
        unreachable!("params will fail");
    };
    let Some(mid) = scope.stack.pop() else {
        scope.ensure_params(&["any", "any", "any"], at);
        unreachable!("params will fail");
    };
    let Some(bot) = scope.stack.pop() else {
        scope.ensure_params(&["any", "any", "any"], at);
        unreachable!("params will fail");
    };
    scope.stack.push(mid);
    scope.stack.push(top);
    scope.stack.push(bot);
}

pub fn rot3_rev(scope: &mut Scope, at: Span) {
    let Some(top) = scope.stack.pop() else {
        scope.ensure_params(&["any", "any", "any"], at);
        unreachable!("params will fail");
    };
    let Some(mid) = scope.stack.pop() else {
        scope.ensure_params(&["any", "any", "any"], at);
        unreachable!("params will fail");
    };
    let Some(bot) = scope.stack.pop() else {
        scope.ensure_params(&["any", "any", "any"], at);
        unreachable!("params will fail");
    };
    scope.stack.push(top);
    scope.stack.push(bot);
    scope.stack.push(mid);
}

pub fn depth(scope: &mut Scope, at: Span) {
    let depth = scope.stack.depth();
    scope.stack.push(Value::Int(depth as i32));
}

// ARITHMETIC
fn get_integral2(scope: &mut Scope, at: Span) -> (f32, f32) {
    let Some(Some(right)) = scope.stack.pop().map(|c| c.as_integral()) else {
        scope.ensure_params(&["Integral", "Integral"], at);
        unreachable!("params will fail");
    };
    let Some(Some(left)) = scope.stack.pop().map(|c| c.as_integral()) else {
        scope.ensure_params(&["Integral", "Integral"], at);
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
        scope.ensure_params(&["any", "any"], at);
        unreachable!("params will fail");
    };
    let Some(left) = scope.stack.pop() else {
        scope.ensure_params(&["any", "any"], at);
        unreachable!("params will fail");
    };

    scope.stack.push(Value::Bool(match (left, right) {
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        _ => false,
    }));
}

// CONVERSION
pub fn stringify(scope: &mut Scope, at: Span) {
    let Some(Some(top)) = scope.stack.pop().map(|c| c.as_string()) else {
        return scope.ensure_params(&["Stringable"], at);
    };
    scope.stack.push(Value::String(top));
}

pub fn intify(scope: &mut Scope, at: Span) {
    let Some(Some(top)) = scope.stack.pop().map(|c| c.as_int()) else {
        return scope.ensure_params(&["Intable"], at);
    };
    scope.stack.push(Value::Int(top));
}

pub fn std_get_fns() -> HashMap<String, Function> {
    HashMap::from(
        [
            ("println", println as InternalFn),
            ("dump", dump as InternalFn),
            ("dup", dup as InternalFn),
            ("pop", pop as InternalFn),
            ("swap", rot2 as InternalFn),
            ("rot", rot3 as InternalFn),
            ("rot-", rot3_rev as InternalFn),
            ("depth", depth as InternalFn),
            ("+", add as InternalFn),
            ("-", sub as InternalFn),
            ("*", mul as InternalFn),
            ("/", div as InternalFn),
            ("**", exp as InternalFn),
            ("str", stringify as InternalFn),
            ("int", intify as InternalFn),
            (">", greater as InternalFn),
            (">=", greatereq as InternalFn),
            ("<", less as InternalFn),
            ("<=", lesseq as InternalFn),
            ("=", equ as InternalFn),
        ]
        .map(|(s, f)| (s.to_string(), Function::Internal(f))),
    )
}
