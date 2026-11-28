use crate::db::UserId;
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use rand::{Rng, SeedableRng, rngs::SysRng};
use rand_chacha::ChaCha20Rng;
use std::{
    collections::{BTreeMap, HashMap},
    ops::DerefMut,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::sleep;

const TOKEN_LIFETIME: Duration = Duration::from_hours(5);

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub struct Token([u8; 256]);

impl Token {
    pub fn base64(&self) -> String {
        BASE64_STANDARD_NO_PAD.encode(self.0)
    }

    pub fn try_from_base64(base64: &str) -> Option<Token> {
        let mut token = Token([0; 256]);
        BASE64_STANDARD_NO_PAD
            .decode_slice(base64, &mut token.0)
            .ok()?;
        Some(token)
    }

    fn generate(rng: &mut impl Rng) -> Token {
        let mut token = Token([0; 256]);
        rng.fill_bytes(&mut token.0);
        token
    }
}

struct LoginInner {
    random: ChaCha20Rng,
    tokens_users: HashMap<Token, (UserId, Instant)>,
    users_tokens: BTreeMap<UserId, Token>,
}

pub struct Login(Mutex<LoginInner>);

impl Login {
    pub fn new() -> Self {
        Login(
            LoginInner {
                random: ChaCha20Rng::try_from_rng(&mut SysRng).unwrap(),
                tokens_users: HashMap::default(),
                users_tokens: BTreeMap::default(),
            }
            .into(),
        )
    }

    pub fn new_token(&self, user: UserId) -> Token {
        let mut this = self.0.lock().unwrap();
        let token = Token::generate(&mut this.random);
        let now = Instant::now();

        this.users_tokens.insert(user, token);
        this.tokens_users.insert(token, (user, now));
        token
    }

    pub fn check_token(&self, token: &Token) -> Option<UserId> {
        let this = self.0.lock().unwrap();
        this.tokens_users.get(token).map(|x| x.0)
    }

    pub fn revoke_token(&self, user_id: UserId) {
        let mut this = self.0.lock().unwrap();
        if let Some(token) = this.users_tokens.remove(&user_id) {
            this.tokens_users.remove(&token);
        }
    }

    fn cleanup(&self) {
        let mut this = self.0.lock().unwrap();
        let this = this.deref_mut();

        this.tokens_users.retain(|_, (user, instant)| {
            let alive = instant.elapsed() < TOKEN_LIFETIME;

            if !alive {
                this.users_tokens.remove(user);
            }

            alive
        });
    }

    pub async fn worker(self: Arc<Self>) {
        sleep(TOKEN_LIFETIME).await;
        self.cleanup();
    }
}
