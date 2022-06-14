use crate::router::{Method, Router};

pub fn configure(router: &mut Router) {
    router.insert(Method::GET, "/", |res| {
        res.sendfile(200, "static/index.html")
    });
    router.insert(Method::GET, "/todo", |res| {
        res.sendfile(200, "static/todo.html")
    });
    router.insert(Method::GET, "/static/styles.css", |res| {
        res.sendfile(200, "static/styles.css")
    });
    router.insert(Method::GET, "/favicon.ico", |res| {
        res.sendfile(200, "static/favicon.ico")
    });
}
