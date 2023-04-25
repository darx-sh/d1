pub fn create_db_pool() -> mysql_async::Pool {
    mysql_async::Pool::new("mysql://root:12345678@localhost:3306/test")
}
