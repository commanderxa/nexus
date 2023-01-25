// KEYSPACE
pub static CREATE_KEYSPACE_QUERY: &str = r#"
  CREATE KEYSPACE IF NOT EXISTS litera
    WITH REPLICATION = {
      'class': 'SimpleStrategy',
      'replication_factor': 1
    };
"#;

// USERS
pub static CREATE_USER_TABLE_QUERY: &str = r#"
  CREATE TABLE IF NOT EXISTS litera.users (
    uuid UUID,
    username text,
    password text,
    role Tinyint,
    public_key text,
    created_at timestamp,
    PRIMARY KEY(uuid, username)
  );
"#;

// CHAT KEYS
pub static CREATE_CHAT_KEYS_TABLE_QUERY: &str = r#"
  CREATE TABLE IF NOT EXISTS litera.chat_keys (
    user UUID,
    private_key text,
    PRIMARY KEY(user)
  );
"#;

// SESSION
pub static CREATE_SESSION_TABLE_QUERY: &str = r#"
  CREATE TABLE IF NOT EXISTS litera.sessions (
    jwt text,
    user UUID,
    location text,
    device_name text,
    device_type text,
    device_os text,
    created_at timestamp,
    PRIMARY KEY(jwt, user)
  );
"#;

// MESSAGES
pub static CREATE_MESSAGE_TABLE_QUERY: &str = r#"
  CREATE TABLE IF NOT EXISTS litera.messages (
    uuid UUID,
    text text,
    nonce text,
    filename text,
    filepath text,
    sender UUID,
    receiver UUID,
    sent Boolean,
    read Boolean,
    edited Boolean,
    msg_type Tinyint,
    created_at timestamp,
    PRIMARY KEY(created_at, sender, uuid)
  );
"#;
