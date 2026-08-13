//! Shared rstest fixtures for the e2e integration tests (feature `native-e2e`).
#![cfg(feature = "native-e2e")]

use rstest::fixture;
use xray_tui_native::e2e::{
    Certs, CoreKind, CoreUnderTest, EchoServer, SINGBOX_VERSION, TlsEchoServer, XRAY_VERSION,
    generate_certs, spawn_echo, spawn_tls_echo,
};

/// Resolved core binaries, once per test binary (hard-fails when
/// `XRAY_TUI_CORE_BIN_DIR` is missing/unversioned).
#[fixture]
#[once]
pub fn cores() -> (CoreUnderTest, CoreUnderTest) {
    let xray = CoreUnderTest::resolve(CoreKind::Xray, XRAY_VERSION).expect("resolve xray-core");
    let singbox =
        CoreUnderTest::resolve(CoreKind::SingBox, SINGBOX_VERSION).expect("resolve sing-box");
    (xray, singbox)
}

/// Harness CA (cert/key), generated once per test binary.
#[fixture]
#[once]
pub fn certs() -> Certs {
    generate_certs()
}

/// Plain HTTP echo; `per-test` (`tiny_http` owns a thread; Drop unblocks it).
#[fixture]
pub fn echo() -> EchoServer {
    spawn_echo()
}

/// TLS HTTP echo with the harness CA; per-test — `reset_recording`/
/// `saw_h2_preface` is mutable fallback-case state, so sharing would race.
#[fixture]
pub fn tls_echo(certs: &Certs) -> TlsEchoServer {
    spawn_tls_echo(certs)
}

/// Select the resolved core for a `CoreKind` case value.
#[must_use]
pub const fn pick(cores: &(CoreUnderTest, CoreUnderTest), kind: CoreKind) -> &CoreUnderTest {
    match kind {
        CoreKind::Xray => &cores.0,
        CoreKind::SingBox => &cores.1,
    }
}
