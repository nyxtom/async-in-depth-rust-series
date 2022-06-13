use crate::router::{Method, Router};

pub fn configure(router: &mut Router) {
    router.insert(Method::GET, "/", || {
        (200, String::from("static/index.html"))
    });
    router.insert(Method::GET, "/todo", || {
        (200, String::from("static/todo.html"))
    });
    router.insert(Method::GET, "/static/styles.css", || {
        (200, String::from("static/styles.css"))
    });
    router.insert(Method::GET, "/favicon.ico", || {
        (200, String::from("static/favicon.ico"))
    });
}
