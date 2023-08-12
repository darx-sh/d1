use nanoid::nanoid;

pub fn new_nano_id() -> String {
  let alphabet = "0123456789abcdefghijklmnopqrstuvwxyz";
  let chars = alphabet.chars().collect::<Vec<_>>();
  nanoid!(12, &chars)
}

pub fn test_control_db_url() -> &'static str {
  "mysql://root:12345678@localhost:3306/darx_control"
}
