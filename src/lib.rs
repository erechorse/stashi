pub mod config;
pub mod api;
pub mod tool;

#[cfg(test)]
pub mod test_utils {
    use mockito::ServerGuard;

    pub fn create_mock(mut server: ServerGuard, method: String, path: String, body: String) -> ServerGuard {
        server.mock(&method, &*path)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();
        server
    }
}