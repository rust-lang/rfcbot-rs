use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::mac::MacResult;
use crypto::sha1::Sha1;
use hex::FromHex;
use super::Delivery;

/// Handles webhook deliveries
pub trait Hook: Send + Sync {
    /// Implementations are expected to deliveries here
    fn handle(&self, delivery: &Delivery);
}

/// A delivery authenticator for hooks
pub struct AuthenticateHook<H: Hook + 'static> {
    secret: String,
    hook: H,
}

impl<H: Hook + 'static> AuthenticateHook<H> {
    pub fn new<S>(secret: S, hook: H) -> AuthenticateHook<H>
        where S: Into<String>
    {
        AuthenticateHook {
            secret: secret.into(),
            hook: hook,
        }
    }

    fn authenticate(&self, payload: &str, signature: &str) -> bool {
        // https://developer.github.com/webhooks/securing/#validating-payloads-from-github
        let sans_prefix = signature[5..signature.len()].as_bytes();
        match Vec::from_hex(sans_prefix) {
            Ok(sigbytes) => {
                let sbytes = self.secret.as_bytes();
                let mut mac = Hmac::new(Sha1::new(), &sbytes);
                let pbytes = payload.as_bytes();
                mac.input(&pbytes);
                // constant time comparison
                mac.result() == MacResult::new(&sigbytes)
            }
            Err(_) => false,
        }
    }
}

impl<H: Hook + 'static> Hook for AuthenticateHook<H> {
    fn handle(&self, delivery: &Delivery) {
        if let Some(sig) = delivery.signature {
            if self.authenticate(delivery.unparsed_payload, sig) {
                self.hook.handle(delivery)
            }
        }
    }
}

impl<F> Hook for F
    where F: Fn(&Delivery),
          F: Sync + Send
{
    fn handle(&self, delivery: &Delivery) {
        self(delivery)
    }
}

#[cfg(test)]
mod tests {
    use crypto::hmac::Hmac;
    use crypto::mac::Mac;
    use crypto::sha1::Sha1;
    use hex::ToHex;
    use super::*;
    use super::super::Delivery;

    #[test]
    fn authenticate_signatures() {
        let authenticated = AuthenticateHook::new("secret", |_: &Delivery| {
        });
        let payload = r#"{"zen": "Approachable is better than simple."}"#;
        let secret = "secret";
        let sbytes = secret.as_bytes();
        let pbytes = payload.as_bytes();
        let mut mac = Hmac::new(Sha1::new(), &sbytes);
        mac.input(&pbytes);
        let signature = mac.result().code().to_hex();
        assert!(authenticated.authenticate(payload, format!("sha1={}", signature).as_ref()))
    }
}
