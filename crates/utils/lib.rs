use darx_db::mysql::MySqlPool;

pub fn test_mysql_db_pool() -> MySqlPool {
    MySqlPool::new("mysql://root:12345678@localhost:3306/test")
}
