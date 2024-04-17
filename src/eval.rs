use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::str::from_utf8;

use thiserror::Error;

use crate::eval::EvalError::{InvalidArguments, UnableToEvalFunction};
use crate::tokenize::{AstNode, Value};
use crate::tokenize::AstNode::{List, Num, Str, Sym};

struct LispFn(Box<dyn Fn(&[Value]) -> Result<Value, EvalError>>);

impl LispFn {
    fn call(&self, arguments: &[Value]) -> Result<Value, EvalError> {
        self.0(arguments)
    }
}

impl<F> From<F> for LispFn
    where F: Fn(&[Value]) -> Result<Value, EvalError> + 'static {
    fn from(value: F) -> Self {
        Self {
            0: Box::new(value)
        }
    }
}

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("there are no available functions with name: {0}")]
    UnableToEvalFunction(String),
    #[error("cannot evaluate empty list")]
    CannotEvaluateEmptyList,
    #[error("cannot evaluate non-symbol")]
    CannotEvaluateNonSymbol,
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
}


fn lisp_plus(arguments: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::Num(arguments.iter().map(Value::num).collect::<Option<Vec<isize>>>().ok_or_else(|| EvalError::InvalidArguments("Non-number in sum operation".into()))?.iter().sum()))
}

pub struct GlobalNamespace {
    functions: RefCell<HashMap<Box<[u8]>, LispFn>>,
}

impl Default for GlobalNamespace {
    fn default() -> Self {
        // Create a global namespace with all the primitives
        let mut namespace = GlobalNamespace::empty();
        namespace.defn(b"+", lisp_plus.into());
        namespace
    }
}

impl GlobalNamespace {
    pub fn empty() -> GlobalNamespace {
        GlobalNamespace {
            functions: RefCell::new(HashMap::new())
        }
    }
    pub fn new() -> GlobalNamespace {
        Self::default()
    }

    pub fn defn(&mut self, key: &[u8], function: LispFn) {
        let new_key = key.iter().cloned().collect();
        self.functions.borrow_mut().insert(new_key, function);
    }

    pub fn eval(&mut self, key: &[u8], arguments: Vec<Value>) -> Result<Value, EvalError> {
        let map_ref = self.functions.borrow();
        let function = map_ref.get(key)
            .ok_or_else(|| UnableToEvalFunction(from_utf8(key).unwrap().to_string()))?;
        function.call(&arguments)
    }
}


pub fn eval(node: &AstNode, global_namespace: &mut GlobalNamespace) -> Result<Value, EvalError> {
    // We can only eval lists
    let the_list = match node {
        List(the_list) => { the_list }
        Num(the_num) => { return Ok(Value::Num(*the_num)); }
        Sym(the_sym) => { return Err(InvalidArguments("Cannot eval a plain symbol since vars are not supported yet".into())); }
        Str(the_str) => { return Ok(Value::Str(the_str.clone())); }
    };
    let mut list_iter = the_list.into_iter();
    let Some(Sym(symbol_name)) = list_iter.next() else {
        return Err(InvalidArguments("The evaluated value must exist and be a symbol".into()));
    };

    let evaluated_arguments: Vec<Value> = list_iter
        .map(|node| eval(node, global_namespace)).collect::<Result<Vec<Value>, EvalError>>()?;
    return global_namespace.eval(symbol_name, evaluated_arguments);
}
