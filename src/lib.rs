pub mod auth;
pub mod oauth2;

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
