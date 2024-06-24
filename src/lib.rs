/*
 * Copyright (c) 2020-2023, Stalwart Labs Ltd.
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

//! # mail-auth
//!
//! [![crates.io](https://img.shields.io/crates/v/mail-auth)](https://crates.io/crates/mail-auth)
//! [![build](https://github.com/stalwartlabs/mail-auth/actions/workflows/rust.yml/badge.svg)](https://github.com/stalwartlabs/mail-auth/actions/workflows/rust.yml)
//! [![docs.rs](https://img.shields.io/docsrs/mail-auth)](https://docs.rs/mail-auth)
//! [![crates.io](https://img.shields.io/crates/l/mail-auth)](http://www.apache.org/licenses/LICENSE-2.0)
//!
//! _mail-auth_ is an e-mail authentication and reporting library written in Rust that supports the **DKIM**, **ARC**, **SPF** and **DMARC**
//! protocols. The library aims to be fast, safe and correct while supporting all major [message authentication and reporting RFCs](#conformed-rfcs).
//!
//! Features:
//!
//! - **DomainKeys Identified Mail (DKIM)**:
//!   - ED25519-SHA256 (Edwards-Curve Digital Signature Algorithm), RSA-SHA256 and RSA-SHA1 signing and verification.
//!   - DKIM Authorized Third-Party Signatures.
//!   - DKIM failure reporting using the Abuse Reporting Format.
//! - **Authenticated Received Chain (ARC)**:
//!   - ED25519-SHA256 (Edwards-Curve Digital Signature Algorithm), RSA-SHA256 and RSA-SHA1 chain verification.
//!   - ARC sealing.
//! - **Sender Policy Framework (SPF)**:
//!   - Policy evaluation.
//!   - SPF failure reporting using the Abuse Reporting Format.
//! - **Domain-based Message Authentication, Reporting, and Conformance (DMARC)**:
//!   - Policy evaluation.
//!   - DMARC aggregate report parsing and generation.
//! - **Abuse Reporting Format (ARF)**:
//!   - Abuse and Authentication failure reporting.
//!   - Feedback report parsing and generation.
//! - **SMTP TLS Reporting**:
//!   - Report parsing and generation.
//!
//! ## Usage examples
//!
//! ### DKIM Signature Verification
//!
//! ```rust
//!     // Create a resolver using Cloudflare DNS
//!     let resolver = Resolver::new_cloudflare_tls().unwrap();
//!
//!     // Parse message
//!     let authenticated_message = AuthenticatedMessage::parse(RFC5322_MESSAGE.as_bytes()).unwrap();
//!
//!     // Validate signature
//!     let result = resolver.verify_dkim(&authenticated_message).await;
//!
//!     // Make sure all signatures passed verification
//!     assert!(result.iter().all(|s| s.result() == &DkimResult::Pass));
//! ```
//!
//! ### DKIM Signing
//!
//! ```rust
//!     // Sign an e-mail message using RSA-SHA256
//!     let pk_rsa = RsaKey::<Sha256>::from_pkcs1_pem(RSA_PRIVATE_KEY).unwrap();
//!     let signature_rsa = DkimSigner::from_key(pk_rsa)
//!         .domain("example.com")
//!         .selector("default")
//!         .headers(["From", "To", "Subject"])
//!         .sign(RFC5322_MESSAGE.as_bytes())
//!         .unwrap();
//!
//!     // Sign an e-mail message using ED25519-SHA256
//!     let pk_ed = Ed25519Key::from_bytes(
//!         &base64_decode(ED25519_PUBLIC_KEY.as_bytes()).unwrap(),
//!         &base64_decode(ED25519_PRIVATE_KEY.as_bytes()).unwrap(),
//!     )
//!     .unwrap();
//!
//!     let signature_ed = DkimSigner::from_key(pk_ed)
//!         .domain("example.com")
//!         .selector("default-ed")
//!         .headers(["From", "To", "Subject"])
//!         .sign(RFC5322_MESSAGE.as_bytes())
//!         .unwrap();
//!
//!     // Print the message including both signatures to stdout
//!     println!(
//!         "{}{}{}",
//!         signature_rsa.to_header(),
//!         signature_ed.to_header(),
//!         RFC5322_MESSAGE
//!     );
//! ```
//!
//! ### ARC Chain Verification
//!
//! ```rust
//!     // Create a resolver using Cloudflare DNS
//!     let resolver = Resolver::new_cloudflare_tls().unwrap();
//!
//!     // Parse message
//!     let authenticated_message = AuthenticatedMessage::parse(RFC5322_MESSAGE.as_bytes()).unwrap();
//!
//!     // Validate ARC chain
//!     let result = resolver.verify_arc(&authenticated_message).await;
//!
//!     // Make sure ARC passed verification
//!     assert_eq!(result.result(), &DkimResult::Pass);
//! ```
//!
//! ### ARC Chain Sealing
//!
//! ```rust
//!     // Create a resolver using Cloudflare DNS
//!     let resolver = Resolver::new_cloudflare_tls().unwrap();
//!
//!     // Parse message to be sealed
//!     let authenticated_message = AuthenticatedMessage::parse(RFC5322_MESSAGE.as_bytes()).unwrap();
//!
//!     // Verify ARC and DKIM signatures
//!     let arc_result = resolver.verify_arc(&authenticated_message).await;
//!     let dkim_result = resolver.verify_dkim(&authenticated_message).await;
//!
//!     // Build Authenticated-Results header
//!     let auth_results = AuthenticationResults::new("mx.mydomain.org")
//!         .with_dkim_result(&dkim_result, "sender@example.org")
//!         .with_arc_result(&arc_result, "127.0.0.1".parse().unwrap());
//!
//!     // Seal message
//!     if arc_result.can_be_sealed() {
//!         // Seal the e-mail message using RSA-SHA256
//!         let pk_rsa = RsaKey::<Sha256>::from_pkcs1_pem(RSA_PRIVATE_KEY).unwrap();
//!         let arc_set = ArcSealer::from_key(pk_rsa)
//!             .domain("example.org")
//!             .selector("default")
//!             .headers(["From", "To", "Subject", "DKIM-Signature"])
//!             .seal(&authenticated_message, &auth_results, &arc_result)
//!             .unwrap();
//!
//!         // Print the sealed message to stdout
//!         println!("{}{}", arc_set.to_header(), RFC5322_MESSAGE)
//!     } else {
//!         eprintln!("The message could not be sealed, probably an ARC chain with cv=fail was found.")
//!     }
//! ```
//!
//! ### SPF Policy Evaluation
//!
//! ```rust
//!     // Create a resolver using Cloudflare DNS
//!     let resolver = Resolver::new_cloudflare_tls().unwrap();
//!
//!     // Verify HELO identity
//!     let result = resolver
//!         .verify_spf_helo("127.0.0.1".parse().unwrap(), "gmail.com", "my-local-domain.org")
//!         .await;
//!     assert_eq!(result.result(), SpfResult::Fail);
//!
//!     // Verify MAIL-FROM identity
//!     let result = resolver
//!         .verify_spf_sender("::1".parse().unwrap(), "gmail.com", "my-local-domain.org", "sender@gmail.com")
//!         .await;
//!     assert_eq!(result.result(), SpfResult::Fail);
//! ```
//!
//! ### DMARC Policy Evaluation
//!
//! ```rust
//!     // Create a resolver using Cloudflare DNS
//!     let resolver = Resolver::new_cloudflare_tls().unwrap();
//!
//!     // Verify DKIM signatures
//!     let authenticated_message = AuthenticatedMessage::parse(RFC5322_MESSAGE.as_bytes()).unwrap();
//!     let dkim_result = resolver.verify_dkim(&authenticated_message).await;
//!
//!     // Verify SPF MAIL-FROM identity
//!     let spf_result = resolver
//!         .verify_spf_sender("::1".parse().unwrap(), "example.org", "my-local-domain.org", "sender@example.org")
//!         .await;
//!
//!     // Verify DMARC
//!     let dmarc_result = resolver
//!         .verify_dmarc(
//!             &authenticated_message,
//!             &dkim_result,
//!             "example.org",
//!             &spf_result,
//!         )
//!         .await;
//!     assert_eq!(dmarc_result.dkim_result(), &DmarcResult::Pass);
//!     assert_eq!(dmarc_result.spf_result(), &DmarcResult::Pass);
//! ```
//!
//! More examples available under the [examples](examples) directory.
//!
//! ## Testing & Fuzzing
//!
//! To run the testsuite:
//!
//! ```bash
//!  $ cargo test --features test
//! ```
//!
//! To fuzz the library with `cargo-fuzz`:
//!
//! ```bash
//!  $ cargo +nightly fuzz run mail_auth
//! ```
//!
//! ## Conformed RFCs
//!
//! ### DKIM
//!
//! - [RFC 6376 - DomainKeys Identified Mail (DKIM) Signatures](https://datatracker.ietf.org/doc/html/rfc6376)
//! - [RFC 6541 - DomainKeys Identified Mail (DKIM) Authorized Third-Party Signatures](https://datatracker.ietf.org/doc/html/rfc6541)
//! - [RFC 6651 - Extensions to DomainKeys Identified Mail (DKIM) for Failure Reporting](https://datatracker.ietf.org/doc/html/rfc6651)
//! - [RFC 8032 - Edwards-Curve Digital Signature Algorithm (EdDSA)](https://datatracker.ietf.org/doc/html/rfc8032)
//! - [RFC 4686 - Analysis of Threats Motivating DomainKeys Identified Mail (DKIM)](https://datatracker.ietf.org/doc/html/rfc4686)
//! - [RFC 5016 - Requirements for a DomainKeys Identified Mail (DKIM) Signing Practices Protocol](https://datatracker.ietf.org/doc/html/rfc5016)
//! - [RFC 5585 - DomainKeys Identified Mail (DKIM) Service Overview](https://datatracker.ietf.org/doc/html/rfc5585)
//! - [RFC 5672 - DomainKeys Identified Mail (DKIM) Signatures -- Update](https://datatracker.ietf.org/doc/html/rfc5672)
//! - [RFC 5863 - DomainKeys Identified Mail (DKIM) Development, Deployment, and Operations](https://datatracker.ietf.org/doc/html/rfc5863)
//! - [RFC 6377 - DomainKeys Identified Mail (DKIM) and Mailing Lists](https://datatracker.ietf.org/doc/html/rfc6377)
//!
//! ### SPF
//! - [RFC 7208 - Sender Policy Framework (SPF)](https://datatracker.ietf.org/doc/html/rfc7208)
//! - [RFC 6652 - Sender Policy Framework (SPF) Authentication Failure Reporting Using the Abuse Reporting Format](https://datatracker.ietf.org/doc/html/rfc6652)
//!
//! ### DMARC
//! - [RFC 7489 - Domain-based Message Authentication, Reporting, and Conformance (DMARC)](https://datatracker.ietf.org/doc/html/rfc7489)
//! - [RFC 8617 - The Authenticated Received Chain (ARC) Protocol](https://datatracker.ietf.org/doc/html/rfc8617)
//! - [RFC 8601 - Message Header Field for Indicating Message Authentication Status](https://datatracker.ietf.org/doc/html/rfc8601)
//! - [RFC 8616 - Email Authentication for Internationalized Mail](https://datatracker.ietf.org/doc/html/rfc8616)
//! - [RFC 7960 - Interoperability Issues between Domain-based Message Authentication, Reporting, and Conformance (DMARC) and Indirect Email Flows](https://datatracker.ietf.org/doc/html/rfc7960)
//!
//! ### ARF
//! - [RFC 5965 - An Extensible Format for Email Feedback Reports](https://datatracker.ietf.org/doc/html/rfc5965)
//! - [RFC 6430 - Email Feedback Report Type Value: not-spam](https://datatracker.ietf.org/doc/html/rfc6430)
//! - [RFC 6590 - Redaction of Potentially Sensitive Data from Mail Abuse Reports](https://datatracker.ietf.org/doc/html/rfc6590)
//! - [RFC 6591 - Authentication Failure Reporting Using the Abuse Reporting Format](https://datatracker.ietf.org/doc/html/rfc6591)
//! - [RFC 6650 - Creation and Use of Email Feedback Reports: An Applicability Statement for the Abuse Reporting Format (ARF)](https://datatracker.ietf.org/doc/html/rfc6650)
//!
//! ### SMTP TLS Reporting
//! - [RFC 8460 - SMTP TLS Reporting](https://datatracker.ietf.org/doc/html/rfc8460)
//!
//! ## License
//!
//! Licensed under either of
//!
//!  * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
//!  * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
//!
//! at your option.
//!
//! ## Copyright
//!
//! Copyright (C) 2020-2023, Stalwart Labs Ltd.
//!

use std::{
    cell::Cell,
    fmt::Display,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::Arc,
    time::SystemTime,
};

use arc::Set;
use common::{crypto::HashAlgorithm, headers::Header, lru::LruCache, verify::DomainKey};
use dkim::{Atps, Canonicalization, DomainKeyReport};
use dmarc::Dmarc;
use hickory_resolver::{
    proto::{error::ProtoError, op::ResponseCode},
    TokioAsyncResolver,
};
use mta_sts::{MtaSts, TlsRpt};
use parking_lot::Mutex;
use spf::{Macro, Spf};

pub mod arc;
pub mod common;
pub mod dkim;
pub mod dmarc;
pub mod mta_sts;
pub mod report;
pub mod spf;

pub use flate2;
pub use hickory_resolver;
pub use zip;

pub struct Resolver {
    pub(crate) resolver: TokioAsyncResolver,
    pub(crate) cache_txt: LruCache<String, Txt>,
    pub(crate) cache_mx: LruCache<String, Arc<Vec<MX>>>,
    pub(crate) cache_ipv4: LruCache<String, Arc<Vec<Ipv4Addr>>>,
    pub(crate) cache_ipv6: LruCache<String, Arc<Vec<Ipv6Addr>>>,
    pub(crate) cache_ptr: LruCache<IpAddr, Arc<Vec<String>>>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum IpLookupStrategy {
    /// Only query for A (Ipv4) records
    Ipv4Only,
    /// Only query for AAAA (Ipv6) records
    Ipv6Only,
    /// Query for A and AAAA in parallel
    //Ipv4AndIpv6,
    /// Query for Ipv6 if that fails, query for Ipv4
    Ipv6thenIpv4,
    /// Query for Ipv4 if that fails, query for Ipv6 (default)
    #[default]
    Ipv4thenIpv6,
}

#[derive(Clone)]
pub enum Txt {
    Spf(Arc<Spf>),
    SpfMacro(Arc<Macro>),
    DomainKey(Arc<DomainKey>),
    DomainKeyReport(Arc<DomainKeyReport>),
    Dmarc(Arc<Dmarc>),
    Atps(Arc<Atps>),
    MtaSts(Arc<MtaSts>),
    TlsRpt(Arc<TlsRpt>),
    Error(Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MX {
    pub exchanges: Vec<String>,
    pub preference: u16,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedMessage<'x> {
    pub headers: Vec<(&'x [u8], &'x [u8])>,
    pub from: Vec<String>,
    pub raw_message: &'x [u8],
    pub body_offset: usize,
    pub body_hashes: Vec<(Canonicalization, HashAlgorithm, u64, Vec<u8>)>,
    pub dkim_headers: Vec<Header<'x, crate::Result<dkim::Signature>>>,
    pub ams_headers: Vec<Header<'x, crate::Result<arc::Signature>>>,
    pub as_headers: Vec<Header<'x, crate::Result<arc::Seal>>>,
    pub aar_headers: Vec<Header<'x, crate::Result<arc::Results>>>,
    pub received_headers_count: usize,
    pub date_header_present: bool,
    pub message_id_header_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
// Authentication-Results header
pub struct AuthenticationResults<'x> {
    pub(crate) hostname: &'x str,
    pub(crate) auth_results: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
// Received-SPF header
pub struct ReceivedSpf {
    pub(crate) received_spf: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DkimResult {
    Pass,
    Neutral(crate::Error),
    Fail(crate::Error),
    PermError(crate::Error),
    TempError(crate::Error),
    None,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DkimOutput<'x> {
    result: DkimResult,
    signature: Option<&'x dkim::Signature>,
    report: Option<String>,
    is_atps: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ArcOutput<'x> {
    result: DkimResult,
    set: Vec<Set<'x>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SpfResult {
    Pass,
    Fail,
    SoftFail,
    Neutral,
    TempError,
    PermError,
    None,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SpfOutput {
    result: SpfResult,
    domain: String,
    report: Option<String>,
    explanation: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DmarcOutput {
    spf_result: DmarcResult,
    dkim_result: DmarcResult,
    domain: String,
    policy: dmarc::Policy,
    record: Option<Arc<Dmarc>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DmarcResult {
    Pass,
    Fail(crate::Error),
    TempError(crate::Error),
    PermError(crate::Error),
    None,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IprevOutput {
    pub result: IprevResult,
    pub ptr: Option<Arc<Vec<String>>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IprevResult {
    Pass,
    Fail(crate::Error),
    TempError(crate::Error),
    PermError(crate::Error),
    None,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Version {
    V1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    ParseError,
    MissingParameters,
    NoHeadersFound,
    CryptoError(String),
    Io(String),
    Base64,
    UnsupportedVersion,
    UnsupportedAlgorithm,
    UnsupportedCanonicalization,
    UnsupportedKeyType,
    FailedBodyHashMatch,
    FailedVerification,
    FailedAuidMatch,
    RevokedPublicKey,
    IncompatibleAlgorithms,
    SignatureExpired,
    SignatureLength,
    DnsError(String),
    DnsRecordNotFound(ResponseCode),
    ArcChainTooLong,
    ArcInvalidInstance(u32),
    ArcInvalidCV,
    ArcHasHeaderTag,
    ArcBrokenChain,
    NotAligned,
    InvalidRecordType,
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseError => write!(f, "Parse error"),
            Error::MissingParameters => write!(f, "Missing parameters"),
            Error::NoHeadersFound => write!(f, "No headers found"),
            Error::CryptoError(err) => write!(f, "Cryptography layer error: {err}"),
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Base64 => write!(f, "Base64 encode or decode error."),
            Error::UnsupportedVersion => write!(f, "Unsupported version in DKIM Signature"),
            Error::UnsupportedAlgorithm => write!(f, "Unsupported algorithm in DKIM Signature"),
            Error::UnsupportedCanonicalization => {
                write!(f, "Unsupported canonicalization method in DKIM Signature")
            }
            Error::UnsupportedKeyType => {
                write!(f, "Unsupported key type in DKIM DNS record")
            }
            Error::FailedBodyHashMatch => {
                write!(f, "Calculated body hash does not match signature hash")
            }
            Error::RevokedPublicKey => write!(f, "Public key for this signature has been revoked"),
            Error::IncompatibleAlgorithms => write!(
                f,
                "Incompatible algorithms used in signature and DKIM DNS record"
            ),
            Error::FailedVerification => write!(f, "Signature verification failed"),
            Error::SignatureExpired => write!(f, "Signature expired"),
            Error::SignatureLength => write!(f, "Insecure 'l=' tag found in Signature"),
            Error::FailedAuidMatch => write!(f, "AUID does not match domain name"),
            Error::ArcInvalidInstance(i) => {
                write!(f, "Invalid 'i={i}' value found in ARC header")
            }
            Error::ArcInvalidCV => write!(f, "Invalid 'cv=' value found in ARC header"),
            Error::ArcHasHeaderTag => write!(f, "Invalid 'h=' tag present in ARC-Seal"),
            Error::ArcBrokenChain => write!(f, "Broken or missing ARC chain"),
            Error::ArcChainTooLong => write!(f, "Too many ARC headers"),
            Error::InvalidRecordType => write!(f, "Invalid record"),
            Error::DnsError(err) => write!(f, "DNS resolution error: {err}"),
            Error::DnsRecordNotFound(code) => write!(f, "DNS record not found: {code}"),
            Error::NotAligned => write!(f, "Policy not aligned"),
        }
    }
}

impl Display for SpfResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SpfResult::Pass => "Pass",
            SpfResult::Fail => "Fail",
            SpfResult::SoftFail => "SoftFail",
            SpfResult::Neutral => "Neutral",
            SpfResult::TempError => "TempError",
            SpfResult::PermError => "PermError",
            SpfResult::None => "None",
        })
    }
}

impl Display for IprevResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IprevResult::Pass => f.write_str("pass"),
            IprevResult::Fail(err) => write!(f, "fail; {err}"),
            IprevResult::TempError(err) => write!(f, "temp error; {err}"),
            IprevResult::PermError(err) => write!(f, "perm error; {err}"),
            IprevResult::None => f.write_str("none"),
        }
    }
}

impl Display for DkimResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DkimResult::Pass => f.write_str("pass"),
            DkimResult::Fail(err) => write!(f, "fail; {err}"),
            DkimResult::Neutral(err) => write!(f, "neutral; {err}"),
            DkimResult::TempError(err) => write!(f, "temp error; {err}"),
            DkimResult::PermError(err) => write!(f, "perm error; {err}"),
            DkimResult::None => f.write_str("none"),
        }
    }
}

impl Display for DmarcResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DmarcResult::Pass => f.write_str("pass"),
            DmarcResult::Fail(err) => write!(f, "fail; {err}"),
            DmarcResult::TempError(err) => write!(f, "temp error; {err}"),
            DmarcResult::PermError(err) => write!(f, "perm error; {err}"),
            DmarcResult::None => f.write_str("none"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

impl From<ProtoError> for Error {
    fn from(err: ProtoError) -> Self {
        Error::DnsError(err.to_string())
    }
}

#[cfg(feature = "rsa")]
impl From<rsa::errors::Error> for Error {
    fn from(err: rsa::errors::Error) -> Self {
        Error::CryptoError(err.to_string())
    }
}

#[cfg(feature = "ed25519-dalek")]
impl From<ed25519_dalek::ed25519::Error> for Error {
    fn from(err: ed25519_dalek::ed25519::Error) -> Self {
        Error::CryptoError(err.to_string())
    }
}

impl Default for SpfOutput {
    fn default() -> Self {
        Self {
            result: SpfResult::None,
            domain: Default::default(),
            report: Default::default(),
            explanation: Default::default(),
        }
    }
}

thread_local!(static COUNTER: Cell<u64>  = const { Cell::new(0) });

/// Generates a random value between 0 and 100.
/// Returns true if the generated value is within the requested
/// sampling percentage specified in a SPF, DKIM or DMARC policy.
pub(crate) fn is_within_pct(pct: u8) -> bool {
    pct == 100
        || COUNTER.with(|c| {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
                .wrapping_add(c.replace(c.get() + 1))
                .wrapping_mul(11400714819323198485u64)
        }) % 100
            < pct as u64
}

impl Clone for Resolver {
    fn clone(&self) -> Self {
        Self {
            resolver: self.resolver.clone(),
            cache_txt: Mutex::new(self.cache_txt.lock().clone()),
            cache_mx: Mutex::new(self.cache_mx.lock().clone()),
            cache_ipv4: Mutex::new(self.cache_ipv4.lock().clone()),
            cache_ipv6: Mutex::new(self.cache_ipv6.lock().clone()),
            cache_ptr: Mutex::new(self.cache_ptr.lock().clone()),
        }
    }
}
