use crate::{
    expressions::{BinaryExpr, UnaryExpr},
    object::Object,
    tokens::TokenType,
    Expr, LoxError,
};

pub struct Interpreter;

impl Interpreter {
    pub fn interpret(expr: Expr) -> Result<Object, LoxError> {
        Self::eval_expr(expr)
    }

    fn eval_expr(expr: Expr) -> Result<Object, LoxError> {
        match expr {
            Expr::Literal(e) => Ok(e.value),
            Expr::Grouping(e) => Self::eval_expr(*e.expression),
            Expr::Unary(e) => Self::unary_expr(e),
            Expr::Binary(e) => Self::binary_expr(e),

            _ => unimplemented!(),
        }
    }

    fn unary_expr(e: UnaryExpr) -> Result<Object, LoxError> {
        let right = Self::eval_expr(*e.right)?;

        if e.operator.token_type == TokenType::MINUS {
            if let Object::Number(n) = right {
                return Ok(Object::Number(n * -1.0));
            }
        }

        unreachable!();
    }

    fn binary_expr(e: BinaryExpr) -> Result<Object, LoxError> {
        let left = Self::eval_expr(*e.left)?;
        let right = Self::eval_expr(*e.right)?;

        let get_num = |x| Self::get_v_num(x);
        let get_str = |x| Self::get_v_string(x);

        let is_num = |x: &Object| get_num(x.clone()).is_ok();

        let to_num = |n| Object::Number(n);
        let to_str = |s| Object::String(s);
        let to_bool = |b| Object::Boolean(b);

        let obj = {
            match e.operator.token_type {
                TokenType::MINUS => to_num(get_num(left)? - get_num(right)?),
                TokenType::SLASH => to_num(get_num(left)? / get_num(right)?),
                TokenType::STAR => to_num(get_num(left)? * get_num(right)?),
                TokenType::PLUS => {
                    if is_num(&left) && is_num(&right) {
                        to_num(get_num(left)? + get_num(right)?)
                    } else {
                        to_str(get_str(left)? + &get_str(right)?)
                    }
                }
                TokenType::GREATER => to_bool(get_num(left)? > get_num(right)?),
                TokenType::GREATER_EQUAL => to_bool(get_num(left)? >= get_num(right)?),
                TokenType::LESS => to_bool(get_num(left)? < get_num(right)?),
                TokenType::LESS_EQUAL => to_bool(get_num(left)? <= get_num(right)?),
                TokenType::BANG_EQUAL => to_bool(!Self::is_equal(left, right)),
                TokenType::EQUAL_EQUAL => to_bool(Self::is_equal(left, right)),
                _ => unreachable!(),
            }
        };
        return Ok(obj);
    }

    fn get_v_num(obj: Object) -> Result<f64, LoxError> {
        if let Object::Number(n) = obj {
            return Ok(n);
        } else {
            return Err(LoxError::Error(format!("'{:?}' must be a number.", obj)));
        }
    }

    fn get_v_string(obj: Object) -> Result<String, LoxError> {
        if let Object::String(s) = obj {
            return Ok(s);
        } else {
            return Err(LoxError::Error(format!("'{:?}' must be a string.", obj)));
        }
    }

    fn is_truthy(obj: Object) -> bool {
        if obj == Object::Nil {
            return false;
        }
        if let Object::Boolean(b) = obj {
            return b;
        };

        return true;
    }

    fn is_equal(a: Object, b: Object) -> bool {
        match (a, b) {
            (Object::Nil, Object::Nil) => return true,
            (Object::Boolean(a), Object::Boolean(b)) => return a == b,
            (Object::Number(a), Object::Number(b)) => return a == b,
            (Object::String(a), Object::String(b)) => return a == b,
            _ => return false,
        }
    }
}
