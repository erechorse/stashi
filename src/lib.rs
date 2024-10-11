pub mod api;
pub mod config;
pub mod tool;

#[cfg(test)]
pub mod test_utils {
    use mockito::ServerGuard;

    pub struct TestServer {
        server: ServerGuard,
    }

    impl TestServer {
        pub fn new(server: ServerGuard) -> Self {
            Self { server }
        }
        pub fn create_mock(&mut self, method: &str, path: &str, body: &str) {
            self.server
                .mock(method, path)
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(body)
                .create();
        }
        pub fn url(&self) -> String {
            self.server.url()
        }
    }
}
