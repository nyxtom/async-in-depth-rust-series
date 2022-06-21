use super::{
    response::Response,
    router::{Method, Router},
};

pub fn configure(router: &mut Router) {
    router.insert(Method::GET, "/", |client| async {
        let mut res = Response::new(client);
        res.send_file(200, "static/index.html").await
    });
    router.insert(Method::GET, "/todo", |client| async {
        let mut res = Response::new(client);
        res.send_file(200, "static/todo.html").await
    });
    router.insert(Method::GET, "/static/styles.css", |client| async {
        let mut res = Response::new(client);
        res.send_file(200, "static/styles.css").await
    });
    router.insert(Method::GET, "/favicon.ico", |client| async {
        let mut res = Response::new(client);
        res.send_file(200, "static/favicon.ico").await
    });
}
