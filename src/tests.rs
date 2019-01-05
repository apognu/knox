use std::env;

use gpgme::{Context, Data, Protocol};
use tempfile::TempDir;

use crate::pb;

pub(crate) const GPG_IDENTITY: &str = "vault@apognu.github.com";
const GPG_KEY: &str = "-----BEGIN PGP PRIVATE KEY BLOCK-----
lQHYBFwvrOYBBADW27WG8CY/pDPnyYxGUz9NO4LmUUjs0IBEjHO39j/swur1sEx3
RX3M7vToe0NympmZBIT+kdE2xz2PH8Ssj9ZHpvnOLdsw2cgNoIghpp4SI+LM5Say
uQFpEZaDW2I4w4wbnKyvwxkYGe7cKJ+syctlnX85qOCDkKDuzakgByFZJQARAQAB
AAP8DumewesrvgtC3pzpED2zIF4RWeec/nICrRjrtGiyWCSeBtTbm2pzbrfMWZ4Q
knhAUYdBE3A1S3sgGGDil2PzKDmTredBxgSx9DgSFdVbaWzoZbK+Jlq/eqfZwHl7
TIEb3JjKyBG8CkACSxilIwTV5bVYR+l2QBcubsSkVTc+d2MCAN2NvpbERYAoGQO7
LLp0aaWkTeA0IpClGHvoe5GWOWMAPz4ZBm3Jjwy6EnKlpqJ+p9wEMiUI3DjzA6af
O7Mau8MCAPhDeRcdvHiEtDuN8hP3hm0FbwiaXf7ehgguQFYanCk4NMgxVlBhSuHU
M0F/grJZcl3ZWeOtg5d+ZhMdk0leEPcB/Alf1ipW0PmxGlfZayvtxyp1DXRDZXKG
oc9kMIZNykTZh5yWeKmHRxv7urspgyQFBh+LvB0ebDM2Vg3BrfLmDqOT1rQlVmF1
bHQgVGVzdHMgPHZhdWx0QGFwb2dudS5naXRodWIuY29tPojOBBMBCAA4FiEEr9Z1
cKGnE0+R2Q6nOByJ+6Lg2SAFAlwvrOYCGwMFCwkIBwMFFQoJCAsFFgMCAQACHgEC
F4AACgkQOByJ+6Lg2SChgwP+KpQBGdAgXRzx1J0DEZT0PuvgUPUmFv7/ACOyh6MH
NcgHOvAYvgvb8E9I+Zk36f10Pf9fDFOA3Di3BNsFF/hrL/q0dQkaewjxOgAY1nvw
v/5D4h5iMAWxLRPCTzLFuUttMtAwDnYYaNR4Hh2jGBaI1NcIaHqUHJnFVy6Ads72
3J+dAdgEXC+s5gEEALH8soBR/aPCNFfoZP6ggHTr0oAJ1121xeF4EnrooWNNXoVy
YqipmYULYgye4Xy1h50NsR1nz42OtMD4l8416YnacePgi9a5BWwIy6ZexsIltr4i
+JNDzQxJlN/p80HvjdaS4NnwuVkOqC1sVh75ybrkPZv+5uEMJRq2BjpaD1tRABEB
AAEAA/4ypnC9p5eAdJGkupOTCmXD4CAlI6fQGRxYz2yi4XSb57aQTz7YNHtlqxmZ
8dTFQnt3LCBM9+/Ont+9UoEQw7LTeGKDkT9t4RBFZutx5HdJc+KzeWiqHfE/GKa1
nNSUc7l3UyB+K4mqIx5Yu0J0C920YOFP/7zde7hD1XbA8nOpSQIAy+38I5R5ApFu
9uW1zvRHpIkA64zNWpgs70RKm7ZKiCnLKPABRwPWtisHPMV9uNtlACVyBaypupQ9
ev81zei2awIA3271jOHa1kvtmPSrvPdgaHgCLVgDbMxDEFTVt4Sg+jYE0SOy+CwV
X6XBG+RtHag+k0bZfoARQrebSr/9sfIMMwIAlk9uT8+FN9Z6kOKopWzyXfFZE9i7
+R/NHS44YwYBEmhB2DhfDAcdR3vUzmifk3SK7Y9z2XblCJLmdTu9YvaPXKYtiLYE
GAEIACAWIQSv1nVwoacTT5HZDqc4HIn7ouDZIAUCXC+s5gIbDAAKCRA4HIn7ouDZ
ILABA/9UbnZnNeJR37JVy6HL2b0FKyH+wJIwhCRxGntg9c788fNFPEra9Ul+SNRs
XY/Dt+FfsBvYHsRBMmYxc+a3TFEKQlwUU4T6uovs8Bpz7qKMWkRlXUHiPug27v80
yd/pB+qiCOIZbtHc9OfGKl9nhLFWNnATXGAHwP+vq/KNn8wI8g==
=FOJS
-----END PGP PRIVATE KEY BLOCK-----";

pub(crate) fn setup() -> TempDir {
  let tmp = tempfile::tempdir().expect("could not create temporary directory");

  let mut context =
    Context::from_protocol(Protocol::OpenPgp).expect("could not create GPG context");

  context.set_armor(true);
  context
    .import(Data::from_bytes(&GPG_KEY).expect("could not read GPG key"))
    .expect("could not import GPG key");

  env::set_var("VAULT_PATH", tmp.path());

  tmp
}

pub(crate) fn get_test_vault() -> pb::Vault {
  pb::Vault {
    identity: crate::tests::GPG_IDENTITY.to_string(),
    ..pb::Vault::default()
  }
}
