use crate::util::{contains_this_expr, ExprFactory};
use ast::*;
use swc_common::{Fold, FoldWith, DUMMY_SP};

#[cfg(test)]
mod tests;

/// Compile ES2015 arrow functions to ES5
///
///# Example
///
///## In
/// ```js
/// var a = () => {};
/// var a = (b) => b;
///
/// const double = [1,2,3].map((num) => num * 2);
/// console.log(double); // [2,4,6]
///
/// var bob = {
///   _name: "Bob",
///   _friends: ["Sally", "Tom"],
///   printFriends() {
///     this._friends.forEach(f =>
///       console.log(this._name + " knows " + f));
///   }
/// };
/// console.log(bob.printFriends());
/// ```
///
///## Out
///```js
/// var a = function () {};
/// var a = function (b) {
///   return b;
/// };
///
/// const double = [1, 2, 3].map(function (num) {
///   return num * 2;
/// });
/// console.log(double); // [2,4,6]
///
/// var bob = {
///   _name: "Bob",
///   _friends: ["Sally", "Tom"],
///   printFriends() {
///     var _this = this;
///
///     this._friends.forEach(function (f) {
///       return console.log(_this._name + " knows " + f);
///     });
///   }
/// };
/// console.log(bob.printFriends());
/// ```
pub fn arrow() -> impl Fold<Expr> {
    Arrow
}

#[derive(Debug, Clone, Copy)]
struct Arrow;

impl Fold<Expr> for Arrow {
    fn fold(&mut self, e: Expr) -> Expr {
        let e = e.fold_children(self);

        match e {
            Expr::Arrow(ArrowExpr {
                span,
                params,
                body,
                is_async,
                is_generator,
            }) => {
                let used_this = contains_this_expr(&body);

                let fn_expr = Expr::Fn(FnExpr {
                    ident: None,
                    function: Function {
                        span,
                        params,
                        is_async,
                        is_generator,
                        body: match body {
                            BlockStmtOrExpr::BlockStmt(block) => block,
                            BlockStmtOrExpr::Expr(expr) => BlockStmt {
                                span: DUMMY_SP,
                                stmts: vec![Stmt::Return(ReturnStmt {
                                    span: DUMMY_SP,
                                    arg: Some(expr),
                                })],
                            },
                        },
                    },
                });

                if !used_this {
                    return fn_expr;
                }

                Expr::Call(CallExpr {
                    span,
                    callee: Expr::Member(MemberExpr {
                        span,
                        obj: ExprOrSuper::Expr(box fn_expr),
                        prop: box quote_ident!("bind").into(),
                        computed: false,
                    })
                    .as_callee(),
                    args: vec![ThisExpr { span: DUMMY_SP }.as_arg()],
                })
            }
            _ => e,
        }
    }
}
